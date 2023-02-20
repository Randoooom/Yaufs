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
use yaufs_common::skytable::pool::AsyncPool;

mod handler;

pub struct ControlPlaneV1Context {
    skytable: AsyncPool,
}

pub type Server = ControlPlaneV1Server<ControlPlaneV1Context>;

pub fn new(skytable: AsyncPool) -> Server {
    ControlPlaneV1Server::new(ControlPlaneV1Context { skytable })
}

#[async_trait]
impl ControlPlaneV1 for ControlPlaneV1Context {
    #[instrument(skip_all)]
    async fn start_instance(
        &self,
        request: Request<StartInstanceRequest>,
    ) -> Result<Response<StartInstanceResponse>, Status> {
        let result = handler::start_instance(self.skytable.clone(), request).await?;
        Ok(result)
    }

    #[instrument(skip_all)]
    async fn list_instances(
        &self,
        request: Request<ListInstancesRequest>,
    ) -> Result<Response<ListInstancesResponse>, Status> {
        let result = handler::list_instances(self.skytable.clone(), request).await?;
        Ok(result)
    }

    #[instrument(skip_all)]
    async fn list_active_instances(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<ListInstancesResponse>, Status> {
        let result = handler::list_active_instances(self.skytable.clone(), request).await?;
        Ok(result)
    }

    #[instrument(skip_all)]
    async fn get_instance(
        &self,
        request: Request<InstanceId>,
    ) -> Result<Response<Instance>, Status> {
        let result = handler::get_instance(self.skytable.clone(), request).await?;
        Ok(result)
    }

    #[instrument(skip_all)]
    async fn stop_instance(
        &self,
        request: Request<InstanceId>,
    ) -> Result<Response<StopInstanceResponse>, Status> {
        let result = handler::stop_instance(self.skytable.clone(), request).await?;
        Ok(result)
    }
}
