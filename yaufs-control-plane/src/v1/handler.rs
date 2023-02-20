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
use yaufs_common::error::{Result, YaufsError};
use yaufs_common::skytable::actions::AsyncActions;
use yaufs_common::skytable::pool::AsyncPool;
use yaufs_common::skytable::types::{Array, FlatElement};
use yaufs_common::skytable::{query, Element, Pipeline};

pub async fn start_instance(
    skytable: AsyncPool,
    request: Request<StartInstanceRequest>,
) -> Result<Response<StartInstanceResponse>> {
    todo!()
}

pub async fn list_instances(
    skytable: AsyncPool,
    request: Request<ListInstancesRequest>,
) -> Result<Response<ListInstancesResponse>> {
    let data = request.into_inner();
    let mut connection = skytable.get().await?;

    let query = query!("LGET", data.template_id);
    let result = kv_span!(connection.run_query_raw(query).await)?;

    Ok(Response::new(ListInstancesResponse { instances: result }))
}

pub async fn list_active_instances(
    skytable: AsyncPool,
    _request: Request<Empty>,
) -> Result<Response<ListInstancesResponse>> {
    let mut connection = skytable.get().await?;

    let mut pipeline = Pipeline::new();
    kv_span!(connection.lskeys::<Vec<String>>(100).await)?
        .into_iter()
        .for_each(|key| {
            pipeline.push(query!("LGET", key));
        });

    connection
        .run_pipeline(pipeline)
        .await?
        .into_iter()
        .for_each(|list| println!("{:?}", list));

    Ok(Response::new(ListInstancesResponse { instances: vec![] }))
}

pub async fn get_instance(
    skytable: AsyncPool,
    request: Request<InstanceId>,
) -> Result<Response<Instance>> {
    todo!()
}

pub async fn stop_instance(
    skytable: AsyncPool,
    request: Request<InstanceId>,
) -> Result<Response<StopInstanceResponse>> {
    todo!()
}
