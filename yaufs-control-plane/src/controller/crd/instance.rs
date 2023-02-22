/*
 *    Copyright  2023.  Fritz Ochsmann
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

use crate::controller::crd::{apply_finalizer, remove_finalizer, ActionDeterminable, CRDAction};
use crate::controller::{default_error_policy, ControlPlaneError, ControllerContext, INSTANCE};
use futures::StreamExt;
use k8s_openapi::api::apps::v1::Deployment;
use kube::api::{DeleteParams, ListParams, PostParams};
use kube::runtime::controller::Action;
use kube::runtime::Controller;
use kube::{Api, Client};
use std::sync::Arc;
use tonic::{Request, Response};
use yaufs_common::error::YaufsError;
use yaufs_common::yaufs_proto::fluvio::{InstanceDeployed, InstanceStopped, YaufsEvent};
use yaufs_common::yaufs_proto::template_service_v1::{Template, TemplateId};

#[derive(Serialize, Deserialize, CustomResource, Debug, Clone, JsonSchema)]
#[kube(group = "yaufs.io", version = "v1alpha1", kind = "Instance")]
#[kube(namespaced)]
pub struct InstanceSpec {
    template: String,
    replicas: u8,
}

pub async fn init(context: Arc<ControllerContext>) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Starting Controller for Instances-CRD");
    let kube_client = &context.kube_client;
    // load the crd
    let instances = Api::<Instance>::all(kube_client.clone());

    // run the controller
    tokio::spawn(async move {
        Controller::new(instances, ListParams::default())
            .shutdown_on_signal()
            .run(reconcile, default_error_policy, context)
            .for_each(|response| async move {
                match response {
                    Ok(data) => info!("reconciled {:?}", data),
                    Err(error) => warn!("reconcile failed: {}", error),
                }
            })
            .await;
    });

    Ok(())
}

async fn reconcile(
    instance: Arc<Instance>,
    context: Arc<ControllerContext>,
) -> Result<Action, ControlPlaneError> {
    let client = &context.kube_client;

    let id = instance.metadata.name.as_ref().expect("name on metadata");
    let namespace = instance
        .metadata
        .namespace
        .as_ref()
        .expect("namespace on metadata");

    // match the event type
    match instance.determine_action() {
        CRDAction::Create => {
            create_deployment(id.as_str(), &instance, context.clone()).await?;
            // finalize the crd
            apply_finalizer::<Instance>(id.as_str(), namespace.as_str(), client.clone()).await?;
        }
        CRDAction::Delete => {
            delete_deployment(id, context.clone()).await?;
            // delete the finalizer
            remove_finalizer::<Instance>(id.as_str(), namespace.as_str(), client.clone()).await?;
        }
        CRDAction::Update => {
            debug!("Redeploying instance {}", id);
            // delete the deployment
            delete_deployment(id, context.clone()).await?;
            // deploy a new instance
            create_deployment(id.as_str(), &instance, context.clone()).await?;
        }
    }

    Ok(Action::await_change())
}

/// Create a new deployment based on the given instance crd. The function ensures that the deployments
/// starts and emits `YaufsEvent::INSTANCE_DEPLOYED` therefor. The integrity of the deployment as such
/// is not given here, because this is handled by the specific controller for the `Deployment`.
async fn create_deployment(
    id: &str,
    instance: &Instance,
    context: Arc<ControllerContext>,
) -> Result<(), ControlPlaneError> {
    // fetch the template
    let request = Request::new(TemplateId {
        id: instance.spec.template.clone(),
    });
    let request = context.authorize_request(request).await?;
    let mut template_client = context.template_client.lock().await;
    let response: Response<Template> = template_client.get_template(request).await?;
    let template = response.into_inner();

    // setup the api
    let deployments = Api::<Deployment>::namespaced(context.kube_client.clone(), INSTANCE);
    // build the yaml configuration
    let deployment: Deployment = serde_json::from_value(serde_json::json!({
        "apiVersion": "v1",
        "kind": "Deployment",
        "metadata": {
            "name": id,
            "namespace": INSTANCE,
            "annotations": {
                "linkerd.io/inject": "enabled",
            },
        },
        "spec": {
            "replicas": &instance.spec.replicas,
            "template": {
                "spec": {
                    "containers": [
                        {
                            "name": "instance",
                            "image": template.image.as_str(),
                        }
                    ]
                }
            }
        }
    }))?;
    // apply the configuration
    deployments
        .create(&PostParams::default(), &deployment)
        .await?;

    // emit the instance deployed event
    context
        .producer
        .send(
            YaufsEvent::INSTANCE_DEPLOYED,
            InstanceDeployed {
                id: id.to_string(),
                // TODO
                issuer: None,
            },
        )
        .await?;

    Ok(())
}

/// Delete a deployment identified by the given instance.
async fn delete_deployment(
    id: &str,
    context: Arc<ControllerContext>,
) -> Result<(), ControlPlaneError> {
    // setup the api
    let deployments = Api::<Deployment>::namespaced(context.kube_client.clone(), INSTANCE);
    // delete the deployment
    deployments.delete(id, &DeleteParams::default()).await?;
    debug!("Starting termination of instance {}", id);

    // emit the instance stopped event
    context
        .producer
        .send(
            YaufsEvent::INSTANCE_STOPPED,
            InstanceStopped {
                id: id.to_string(),
                // TODO
                issuer: None,
            },
        )
        .await?;

    Ok(())
}

/// Create a new instance crd to interact with the controller. This function is used by the grpc
/// endpoint of this very specific control plane. The creation will trigger a reconciliation cycle
/// and issue the deployment.
#[instrument(skip(client))]
pub async fn create_crd(
    instance: &crate::prelude::Instance,
    client: Client,
) -> yaufs_common::error::Result<()> {
    // build the custom crd
    let crd = serde_json::from_value::<Instance>(serde_json::json!({
        "apiVersion": "yaufs.io/v1alpha1",
        "kind": "Instance",
        "metadata": {
            "name": instance.id.as_str(),
        },
        "spec": {
            "template": instance.template_id.as_str(),
            "replicas": 1,
        }
    }))?;
    // post the crd to the cluster
    Api::<Instance>::all(client)
        .create(&PostParams::default(), &crd)
        .await
        .map_err(|error| YaufsError::InternalServerError(error.to_string()))?;

    Ok(())
}
