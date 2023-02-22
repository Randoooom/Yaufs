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
use crate::controller::{default_error_policy, ControlPlaneError, ControllerContext};
use futures::StreamExt;
use kube::api::{ListParams, Patch, PatchParams};
use kube::runtime::controller::Action;
use kube::runtime::Controller;
use kube::Api;
use std::sync::Arc;
use tonic::{Request, Response};
use yaufs_common::yaufs_proto::template_service_v1::{CreateTemplateRequest, TemplateId};

#[derive(Serialize, Deserialize, CustomResource, Debug, Clone, JsonSchema)]
#[kube(group = "yaufs.io", version = "v1alpha1", kind = "Template")]
#[kube(namespaced)]
pub struct TemplateSpec {
    image: String,
    id: Option<String>,
}

pub async fn init(context: Arc<ControllerContext>) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Starting Controller for Instances-CRD");
    let kube_client = &context.kube_client;
    // load the crd
    let instances = Api::<Template>::all(kube_client.clone());

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

pub async fn reconcile(
    template: Arc<Template>,
    context: Arc<ControllerContext>,
) -> Result<Action, ControlPlaneError> {
    let client = &context.kube_client;

    let name = template.metadata.name.as_ref().expect("name on metadata");
    let namespace = template
        .metadata
        .namespace
        .as_ref()
        .expect("namespace on metadata");

    match template.determine_action() {
        CRDAction::Create => {
            // create the template
            let request = Request::new(CreateTemplateRequest {
                name: name.to_string(),
                image: template.spec.image.clone(),
            });
            // append authorization header
            let request = context.authorize_request(request).await?;
            let mut template_client = context.template_client.lock().await;
            let response: Response<yaufs_common::yaufs_proto::template_service_v1::Template> =
                template_client.create_template(request).await?;
            let template = response.into_inner();

            // write the returned id into the crd
            let api = Api::<Template>::namespaced(client.clone(), namespace);
            let data = serde_json::json!({
                "spec": {
                    "id": template.id.as_str(),
                }
            });
            let patch = Patch::Merge(&data);
            // patch the crd
            api.patch(name.as_str(), &PatchParams::default(), &patch)
                .await?;

            // finalize the crd
            apply_finalizer::<Template>(name.as_str(), namespace.as_str(), client.clone()).await?;
        }
        CRDAction::Delete => {
            let template_id = template.spec.id.as_ref().expect("id on spec");
            // delete the template
            let request = Request::new(TemplateId {
                id: template_id.clone(),
            });
            // append authorization header
            let request = context.authorize_request(request).await?;
            let mut template_client = context.template_client.lock().await;
            template_client.delete_template(request).await?;

            // remove the finalizer from the crd
            remove_finalizer::<Template>(name.as_str(), namespace.as_str(), client.clone()).await?;
        }
        CRDAction::Update => {}
    }

    Ok(Action::await_change())
}
