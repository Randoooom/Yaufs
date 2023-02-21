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
use crate::v1::ControlPlaneV1Context;
use chrono::Utc;
use futures::{StreamExt, TryStreamExt};
use kube::api::DeleteParams;
use kube::Api;
use tonic::{Request, Response};
use yaufs_common::error::{Result, YaufsError};
use yaufs_common::skytable::actions::AsyncActions;
use yaufs_common::skytable::ddl::AsyncDdl;
use yaufs_common::skytable::types::FromSkyhashBytes;
use yaufs_common::skytable::{query, Pipeline};
use yaufs_common::{kv_span, map_internal_error};

pub async fn start_instance(
    context: &ControlPlaneV1Context,
    request: Request<StartInstanceRequest>,
) -> Result<Response<StartInstanceResponse>> {
    // convert the request into the inner data
    let data = request.into_inner();
    let mut connection = map_internal_error!(
        context.skytable.get().await,
        "failed to access skytable pool"
    )?;
    let count = data.count;

    // create the instances
    let mut instances: Vec<Instance> = Vec::new();
    let mut keys: Vec<String> = Vec::new();
    for _ in 0..count {
        let id = nanoid::nanoid!();

        keys.push(id.clone());
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
        connection
            .mset(keys, instances.iter().collect::<Vec<&Instance>>())
            .await,
        "write instances"
    )?;

    // start all the instances
    futures::stream::iter(instances.iter())
        .then(|instance| async {
            // apply a custom crd for the controller to manage
            crate::controller::create_instance_crd(instance, context.kube_client.clone()).await
        })
        .try_collect::<Vec<()>>()
        .await?;

    Ok(Response::new(StartInstanceResponse { instances }))
}

pub async fn list_instances(
    context: &ControlPlaneV1Context,
    request: Request<ListInstancesRequest>,
) -> Result<Response<ListInstancesResponse>> {
    let data = request.into_inner();
    let mut connection = map_internal_error!(
        context.skytable.get().await,
        "failed to access skytable pool"
    )?;

    // change the table
    kv_span!(
        connection
            .switch(format!("instances:{}", data.template_id))
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
    context: &ControlPlaneV1Context,
    request: Request<InstanceId>,
) -> Result<Response<Instance>> {
    let data = request.into_inner();
    let mut connection = map_internal_error!(
        context.skytable.get().await,
        "failed to access skytable pool"
    )?;

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
    context: &ControlPlaneV1Context,
    request: Request<InstanceId>,
) -> Result<Response<Empty>> {
    let data = request.into_inner();
    let mut connection = map_internal_error!(
        context.skytable.get().await,
        "failed to access skytable pool"
    )?;

    // delete the crd
    map_internal_error!(
        Api::<crate::controller::Instance>::all(context.kube_client.clone())
            .delete(data.id.as_str(), &DeleteParams::default())
            .await,
        "Error while deleting crd"
    )?;
    debug!("Deleted instance crd {}", data.id.as_str());

    // remove the instance from the kv
    kv_span!(
        connection
            .switch(format!("instances:{}", data.template_id))
            .await,
        "switch"
    )?;
    kv_span!(connection.del(data.id.as_str()).await, "delete")?;
    info!("Deleted instance {}", data.id.as_str());

    Ok(Response::new(Empty {}))
}
