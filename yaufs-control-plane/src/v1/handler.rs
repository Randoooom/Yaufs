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
use chrono::Utc;
use futures::{StreamExt, TryStreamExt};
use kube::Client;
use yaufs_common::error::Result;
use yaufs_common::skytable::actions::AsyncActions;
use yaufs_common::skytable::ddl::AsyncDdl;
use yaufs_common::skytable::pool::AsyncPool;
use yaufs_common::skytable::types::FromSkyhashBytes;
use yaufs_common::skytable::{query, Pipeline};

pub async fn start_instance(
    skytable: AsyncPool,
    client: Client,
    request: Request<StartInstanceRequest>,
) -> Result<Response<StartInstanceResponse>> {
    let mut connection = skytable.get().await?;
    let data = request.into_inner();
    let count = data.count;

    // create the instances
    let mut instances: Vec<Instance> = Vec::new();
    let mut keys: Vec<&str> = Vec::new();
    for _ in 0..count {
        let id = nanoid::nanoid!();

        keys.push(id.as_str());
        instances.push(Instance {
            id,
            template_id: data.template_id.clone(),
            created_at: Utc::now().to_rfc3339(),
        });
    }
    // save them into the skytable
    kv_span!(
        connection
            .switch(format!("instances:{}", data.template_id))
            .await,
        "switch"
    )?;
    kv_span!(
        connection.mset(keys, instances.clone()).await,
        "write instances"
    )?;

    futures::stream::iter(instances)
        .then(|instance| {
            // fetch the required template
            // TODO

            let instance = serde_json::json!({
                "apiVersion": "yaufs.io/v1alpha1"
                "kind": "Instance",
                "metadata": {
                    "name": &instance.id,
                    "namespace": "instance"
                },
                "spec": {
                    "image"
                }
            });

            Ok(())
        })
        .await?;

    todo!()
}

pub async fn list_instances(
    skytable: AsyncPool,
    request: Request<ListInstancesRequest>,
) -> Result<Response<ListInstancesResponse>> {
    let mut connection = skytable.get().await?;
    kv_span!(
        connection
            .switch(format!("instances:{}", request.into_inner().template_id))
            .await,
        "switch"
    )?;

    // collect all existing instance keys
    let mut pipeline = Pipeline::new();
    kv_span!(connection.lskeys::<Vec<String>>(100).await, "fetch keys")?
        .into_iter()
        .for_each(|key| pipeline.push(query!("MGET", key)));

    // fetch all instances
    let instances = kv_span!(connection.run_pipeline(pipeline).await, "fetch values")?
        .into_iter()
        // parse from strbin
        .map(Instance::from_element)
        .try_collect::<Vec<Instance>>()?;

    Ok(Response::new(ListInstancesResponse { instances }))
}

pub async fn get_instance(
    skytable: AsyncPool,
    request: Request<InstanceId>,
) -> Result<Response<Instance>> {
    let data = request.into_inner();
    let mut connection = skytable.get().await?;

    kv_span!(
        connection
            .switch(format!("instances:{}", data.template_id))
            .await,
        "switch"
    )?;
    let instance = kv_span!(connection.get::<Instance>(data.id).await)?;

    Ok(Response::new(instance))
}

pub async fn stop_instance(
    skytable: AsyncPool,
    client: Client,
    request: Request<InstanceId>,
) -> Result<Response<StopInstanceResponse>> {
    todo!()
}
