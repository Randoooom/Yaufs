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

use fluvio::TopicProducer;
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::apps::v1::Deployment;
use kube::api::{ListParams, PostParams, WatchEvent};
use kube::runtime::controller::Action;
use kube::runtime::Controller;
use kube::{Api, Client};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use yaufs_common::yaufs_proto::fluvio::{InstanceDeployed, YaufsEvent};

const INSTANCE: &str = "instance";

#[derive(Error, Debug)]
pub enum ControlPlaneError {
    #[error(transparent)]
    KubeError(#[from] kube::Error),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    FluvioError(#[from] fluvio::FluvioError),
}

#[derive(Serialize, Deserialize, CustomResource, Debug, Clone, JsonSchema)]
#[kube(group = "yaufs.io", version = "v1alpha1", kind = "Instance")]
#[kube(namespaced)]
pub struct InstanceSpec {
    image: String,
    image_pull_secret: Option<String>,
    replicas: u8,
    id: String,
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

    let id = instance.spec.id.as_str();
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
                            "image": instance.spec.image.as_str(),
                            "imagePullSecrets": [ instance.spec.image_pull_secret.clone() ],
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
        .fields(format!("metadata.name={},metadata.namespace={}", id, INSTANCE).as_str())
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
                    .into_iter()
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
