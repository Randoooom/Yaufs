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
use kube::runtime::controller::Action;
use kube::Client;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::Mutex;
use tonic::codegen::http::header::AUTHORIZATION;
use tonic::transport::Channel;
use tonic::{Request, Status};
use yaufs_common::error::YaufsError;
use yaufs_common::oidc::OIDCClient;
use yaufs_common::tonic::inject_tracing_context;
use yaufs_common::yaufs_proto::template_service_v1::template_service_v1_client::TemplateServiceV1Client;

const INSTANCE: &str = "instance";
const TEMPLATE_SERVICE_ENDPOINT: &str = "TEMPLATE_SERVICE_ENDPOINT";

pub mod crd;

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

pub struct ControllerContext {
    kube_client: Client,
    producer: TopicProducer,
    template_client: Arc<Mutex<TemplateServiceV1Client<Channel>>>,
    oidc_client: OIDCClient,
}

impl ControllerContext {
    pub async fn authorize_request<T>(
        &self,
        mut request: Request<T>,
    ) -> Result<Request<T>, ControlPlaneError> {
        let access_token = self.oidc_client.obtain_access_token().await?;
        request
            .metadata_mut()
            .insert(AUTHORIZATION.as_str(), access_token.parse().unwrap());

        Ok(inject_tracing_context(request))
    }
}

pub fn default_error_policy<T>(
    _object: Arc<T>,
    _error: &ControlPlaneError,
    _context: Arc<ControllerContext>,
) -> Action {
    Action::requeue(Duration::from_secs(30))
}

pub async fn init(client: Client) -> Result<(), Box<dyn std::error::Error>> {
    // connect to the template service
    let template_client = TemplateServiceV1Client::connect(
        std::env::var(TEMPLATE_SERVICE_ENDPOINT)
            .unwrap_or_else(|_| panic!("missing env var {TEMPLATE_SERVICE_ENDPOINT}")),
    )
    .await
    .map_err(|error| YaufsError::InternalServerError(error.to_string()))?;

    let context = Arc::new(ControllerContext {
        kube_client: client,
        // establish connection to the event streaming spu gorup
        producer: yaufs_common::fluvio_util::producer().await?,
        template_client: Arc::new(Mutex::new(template_client)),
        oidc_client: OIDCClient::new_from_env(vec![
            String::from("templating"),
            String::from("control-plane"),
        ])
        .await?,
    });

    // start the controllers
    crd::instance::init(context.clone()).await?;
    crd::template::init(context).await?;

    Ok(())
}
