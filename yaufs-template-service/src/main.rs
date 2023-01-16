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

extern crate serde;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate tracing;

use crate::prelude::*;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tonic::transport::Server;

mod v1;

const ADDRESS: &str = "0.0.0.0:8000";
pub static SURREALDB: Surreal<Client> = Surreal::init();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // init opentelemetry
    yaufs_common::init_telemetry!();

    // connect to the surrealdb instance
    yaufs_common::database::surrealdb::connect(
        &SURREALDB,
        include_str!("./surrealql/up.surrealql"),
    )
    .await?;
    // execute possible migrations
    yaufs_common::database::surrealdb::migrate(&SURREALDB, env!("CARGO_PKG_VERSION"), vec![])
        .await?;

    // start tonic serve on specified address
    info!("Starting grpc server on {ADDRESS}");
    let tower_layer = tower::ServiceBuilder::new()
        .layer(yaufs_common::tonic::trace_layer())
        .into_inner();
    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();
    Server::builder()
        .layer(tower_layer)
        .add_service(yaufs_common::tonic::init_health::<v1::Server>().await)
        .add_service(reflection)
        .add_service(v1::new())
        .serve(ADDRESS.parse().unwrap())
        .await?;

    Ok(())
}

pub mod prelude {
    pub use tonic::{Request, Response, Status};
    pub use yaufs_common::proto::template_service_v1::*;
}
