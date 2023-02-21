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

use async_once::AsyncOnce;
use fluvio::TopicProducer;
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::apps::v1::Deployment;
use kube::api::{DeleteParams, ListParams, PostParams, WatchEvent};
use kube::runtime::controller::Action;
use kube::runtime::watcher::Event;
use kube::runtime::{watcher, Controller};
use kube::{Api, Client};
use lazy_static::lazy_static;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::Mutex;
use tonic::codegen::http::header::AUTHORIZATION;
use tonic::{Request, Response, Status};
use yaufs_common::error::YaufsError;
use yaufs_common::oidc::OIDCClient;
use yaufs_common::yaufs_proto::fluvio::{InstanceDeployed, YaufsEvent};
use yaufs_common::yaufs_proto::template_service_v1::template_service_v1_client::TemplateServiceV1Client;
use yaufs_common::yaufs_proto::template_service_v1::{Template, TemplateId};

const INSTANCE: &str = "instance";
const TEMPLATE_SERVICE_ENDPOINT: &str = "TEMPLATE_SERVICE_ENDPOINT";

lazy_static! {
    pub static ref OPENID_CLIENT: AsyncOnce<Arc<Mutex<OIDCClient>>> = AsyncOnce::new(async move {
        Arc::new(Mutex::new(
            OIDCClient::new_from_env(vec!["templating".to_owned()])
                .await
                .unwrap(),
        ))
    });
}

#[derive(Error, Debug)]
pub enum ControlPlaneError {
    #[error(transparent)]
    Kube(#[from] kube::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Fluvio(#[from] fluvio::FluvioError),
    #[error(transparent)]
    Yaufs(#[from] YaufsError),
    #[error(transparent)]
    Watch(#[from] kube::runtime::watcher::Error),
}

impl From<Status> for ControlPlaneError {
    fn from(status: Status) -> Self {
        Self::Yaufs(YaufsError::from(status))
    }
}

#[derive(Serialize, Deserialize, CustomResource, Debug, Clone, JsonSchema)]
#[kube(group = "yaufs.io", version = "v1alpha1", kind = "Instance")]
#[kube(namespaced)]
pub struct InstanceSpec {
    template: String,
    replicas: u8,
}

pub struct ControllerContext {
    kube_client: Client,
    producer: TopicProducer,
}

async fn reconcile(
    instance: Arc<Instance>,
    context: Arc<ControllerContext>,
) -> Result<Action, ControlPlaneError> {
    let client = &context.kube_client;
    let producer = &context.producer;

    let mut template_client = TemplateServiceV1Client::connect(
        std::env::var(TEMPLATE_SERVICE_ENDPOINT)
            .unwrap_or_else(|_| panic!("missing env var {TEMPLATE_SERVICE_ENDPOINT}")),
    )
    .await
    .map_err(|error| YaufsError::InternalServerError(error.to_string()))?;
    // fetch the template from the service
    let mut request = Request::new(TemplateId {
        id: instance.spec.template.clone(),
    });
    let oidc_client = OPENID_CLIENT.get().await.lock().await;
    let access_token = oidc_client.obtain_access_token().await?;
    request
        .metadata_mut()
        .insert(AUTHORIZATION.as_str(), access_token.parse().unwrap());
    let response: Response<Template> = template_client.get_template(request).await?;
    let template = response.into_inner();

    let id = instance
        .metadata
        .name
        .as_ref()
        .expect("metadata contains name");
    debug!("Preparing deployment for instance {}", id);
    // check if the deployment already exists
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
    let deployments = Api::<Deployment>::namespaced(client.clone(), INSTANCE);
    debug!("Starting pod for instance {}", id);
    deployments
        .create(&PostParams::default(), &deployment)
        .await?;

    // wait until the pod has started
    let list_params = ListParams::default()
        .fields(format!("metadata.name={id}").as_str())
        .timeout(10);
    let mut stream = deployments.watch(&list_params, "0").await?.boxed();
    while let Some(status) = stream.try_next().await? {
        match status {
            WatchEvent::Added(_) => {
                info!("Added instance {}", id);
            }
            WatchEvent::Modified(pod) => {
                let status = pod.status.as_ref().expect("status on deployment");
                if status
                    .conditions
                    .as_ref()
                    .expect("conditions on deploymentStatus")
                    .iter()
                    .all(|condition| condition.status.eq("Running"))
                {
                    info!("Attached instance {}", id);
                    break;
                }
            }
            _ => {}
        }
    }

    producer
        .send(
            YaufsEvent::INSTANCE_DEPLOYED,
            InstanceDeployed {
                id: id.to_owned(),
                // TODO
                issuer: None,
            },
        )
        .await?;

    Ok(Action::await_change())
}

fn error_policy(
    _object: Arc<Instance>,
    _error: &ControlPlaneError,
    _context: Arc<ControllerContext>,
) -> Action {
    Action::requeue(Duration::from_secs(30))
}

pub async fn init(client: Client) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting controller");
    let instances = Api::<Instance>::all(client.clone());
    debug!("Fetched kind 'Instance'");

    // watch for the crd to be deleted
    let mut watcher_stream = watcher(instances.clone(), ListParams::default()).boxed();
    let watcher_client = client.clone();
    // spawn new process
    tokio::spawn(async move {
        while let Some(event) = watcher_stream.try_next().await? {
            if let Event::Deleted(instance) = event {
                handle_instance_crd_deleted(instance, watcher_client.clone()).await?;
            }
        }

        Ok::<(), ControlPlaneError>(())
    });

    let context = ControllerContext {
        kube_client: client,
        // establish connection to the event streaming spu gorup
        producer: yaufs_common::fluvio_util::producer().await?,
    };

    // start the controller
    Controller::new(instances, ListParams::default())
        .shutdown_on_signal()
        .run(reconcile, error_policy, Arc::new(context))
        .for_each(|response| async move {
            match response {
                Ok(data) => info!("reconciled {:?}", data),
                Err(error) => warn!("reconcile failed: {}", error),
            }
        })
        .await;

    Ok(())
}

async fn handle_instance_crd_deleted(
    instance: Instance,
    client: Client,
) -> Result<(), ControlPlaneError> {
    let name = instance.metadata.name.as_ref().expect("name on metadata");

    // fetch the deployments
    let deployments = Api::<Deployment>::namespaced(client.clone(), INSTANCE);
    // delete the given instance
    deployments
        .delete(name.as_str(), &DeleteParams::default())
        .await?;
    debug!("Terminating deployment {}", name);

    Ok(())
}

#[instrument(skip(client))]
pub async fn create_instance_crd(
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
