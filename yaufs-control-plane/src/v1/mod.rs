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

use crate::prelude::*;
use control_plane_v1_server::{ControlPlaneV1, ControlPlaneV1Server};
use fluvio::dataplane::record::ConsumerRecord;
use fluvio::Offset;
use futures::stream::StreamExt;
use kube::Client;
use yaufs_common::error::YaufsError;
use yaufs_common::fluvio_err;
use yaufs_common::skytable::ddl::{AsyncDdl, Keymap, KeymapType};
use yaufs_common::skytable::pool::AsyncPool;
use yaufs_common::yaufs_proto::fluvio::{TemplateCreated, TemplateDeleted, YaufsEvent};

mod handler;

pub struct ControlPlaneV1Context {
    pub skytable: AsyncPool,
    pub kube_client: Client,
}

pub type Server = ControlPlaneV1Server<ControlPlaneV1Context>;

pub async fn new(skytable: AsyncPool, kube_client: Client) -> yaufs_common::error::Result<Server> {
    #[cfg(not(test))]
    {
        // handle the stream in a new tokio process
        let pool = skytable.clone();
        tokio::spawn(async move {
            // start the event consumer
            let consumer = yaufs_common::fluvio_util::consumer().await?;
            // access the fluvio stream
            let mut stream = fluvio_err!(consumer.stream(Offset::end()).await)?;

            while let Some(Ok(record)) = stream.next().await {
                let record: ConsumerRecord = record;
                let key = record.key().ok_or(YaufsError::InternalServerError(
                    "Invalid fluvio message received: missing key".to_owned(),
                ))?;
                // build the event
                let event = String::from_utf8_lossy(key);

                match event.as_ref() {
                    YaufsEvent::TEMPLATE_CREATED => {
                        info!("Received TEMPLATE_CREATED");
                        // parse the data
                        let data = serde_json::from_slice::<TemplateCreated>(record.value())?;

                        // create a new ddl table in the kv server for it
                        let keymap = Keymap::new(format!("instances:{}", data.template_id))
                            .set_ktype(KeymapType::Str)
                            .set_vtype(KeymapType::Binstr);
                        pool.get().await?.create_table(keymap).await?;
                    }
                    YaufsEvent::TEMPLATE_DELETED => {
                        info!("Received TEMPLATE_DELETED");
                        // parse the data
                        let data = serde_json::from_slice::<TemplateDeleted>(record.value())?;

                        // delete the ddl table
                        let mut connection = pool.get().await?;
                        connection.switch("default").await?;
                        connection
                            .drop_table(format!("instances:{}", data.template_id))
                            .await?;
                    }
                    // we do not listen for any other events here
                    _ => {}
                }
            }

            Ok::<(), YaufsError>(())
        })
    };

    Ok(ControlPlaneV1Server::new(ControlPlaneV1Context {
        skytable,
        kube_client,
    }))
}

#[async_trait]
impl ControlPlaneV1 for ControlPlaneV1Context {
    #[instrument(skip_all)]
    async fn start_instance(
        &self,
        request: Request<StartInstanceRequest>,
    ) -> Result<Response<StartInstanceResponse>, Status> {
        let response = handler::start_instance(self, request).await?;

        Ok(response)
    }

    #[instrument(skip_all)]
    async fn list_instances(
        &self,
        request: Request<ListInstancesRequest>,
    ) -> Result<Response<ListInstancesResponse>, Status> {
        let response = handler::list_instances(self, request).await?;

        Ok(response)
    }

    #[instrument(skip_all)]
    async fn get_instance(
        &self,
        request: Request<InstanceId>,
    ) -> Result<Response<Instance>, Status> {
        let response = handler::get_instance(self, request).await?;

        Ok(response)
    }

    #[instrument(skip_all)]
    async fn stop_instance(&self, request: Request<InstanceId>) -> Result<Response<Empty>, Status> {
        let response = handler::stop_instance(self, request).await?;

        Ok(response)
    }
}
