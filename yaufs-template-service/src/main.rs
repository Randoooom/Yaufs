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

use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tonic::transport::server::TcpIncoming;
use tonic::transport::Server;

mod v1;

cfg_if::cfg_if! {
    if #[cfg(test)] {
        const ADDRESS: &str = "127.0.0.1:0";
    } else {
        use yaufs_common::tower::auth::AuthenticationLayer;
        use yaufs_common::oidc::OIDCClient;
        use std::str::FromStr;
        use std::net::SocketAddr;

        const ADDRESS: &str = "0.0.0.0:8000";
        const HEALTH: &str = "0.0.0.0:8001";
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_, _, join) = init().await?;
    join.await?;

    Ok(())
}

async fn init() -> Result<(String, Surreal<Client>, JoinHandle<()>), Box<dyn std::error::Error>> {
    yaufs_common::init_telemetry!();
    // connect to the surrealdb instance
    let surreal =
        yaufs_common::database::surrealdb::connect(include_str!("./surrealql/up.surrealql"))
            .await?;
    let client = surreal.clone();
    // execute possible migrations
    yaufs_common::database::surrealdb::migrate(&surreal, env!("CARGO_PKG_VERSION"), vec![]).await?;

    // start tonic serve on specified address
    info!("Starting grpc server on {ADDRESS}");
    let listener = TcpListener::bind(ADDRESS).await?;
    let local_addr = listener.local_addr().unwrap();
    info!("Listening on {}", local_addr);
    let incoming = TcpIncoming::from_listener(listener, true, None).unwrap();

    cfg_if::cfg_if! {
        if #[cfg(test)] {
            let tower_layer = tower::ServiceBuilder::new()
                .layer(yaufs_common::tonic::trace_layer())
                .into_inner();
        } else {
            let tower_layer = tower::ServiceBuilder::new()
                .layer(yaufs_common::tonic::trace_layer())
                .layer(AuthenticationLayer::from(OIDCClient::new_from_env(Vec::new()).await?))
                .into_inner();
        }
    }

    let join = tokio::spawn(async move {
        // expose the health check and in future version the metrics
        #[cfg(not(test))]
        tokio::spawn(async move {
            Server::builder()
                .add_service(yaufs_common::tonic::init_health::<v1::Server>().await)
                .serve(SocketAddr::from_str(HEALTH).unwrap())
                .await
                .unwrap()
        });

        Server::builder()
            .layer(tower_layer)
            .add_service(v1::new(surreal))
            .serve_with_incoming(incoming)
            .await
            .unwrap()
    });

    Ok((format!("ws://{local_addr}"), client, join))
}

pub mod prelude {
    pub use tonic::{Request, Response, Status};
    pub use yaufs_common::proto::template_service_v1::*;
    pub use yaufs_common::sql_span;
}
