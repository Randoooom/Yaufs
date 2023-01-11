#[macro_use]
extern crate serde;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate tracing;

use crate::prelude::*;
use crate::proto::template_service_server::{TemplateService, TemplateServiceServer};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tonic::transport::Server;
use tower_http::trace::TraceLayer;

mod methods;

pub mod proto {
    tonic::include_proto!("template_service");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("template_service_descriptor");
}

pub struct TemplateServiceContext;

#[tonic::async_trait]
impl TemplateService for TemplateServiceContext {
    async fn get_template(
        &self,
        request: Request<TemplateId>,
    ) -> Result<Response<Template>, Status> {
        todo!()
    }

    async fn list_templates(
        &self,
        request: Request<ListTemplatesMessage>,
    ) -> Result<Response<TemplateList>, Status> {
        todo!()
    }

    async fn delete_template(
        &self,
        request: Request<TemplateId>,
    ) -> Result<Response<Template>, Status> {
        todo!()
    }

    async fn create_template(
        &self,
        request: Request<CreateTemplateMessage>,
    ) -> Result<Response<Template>, Status> {
        todo!()
    }
}

const ADDRESS: &'static str = "0.0.0.0:8000";
pub static SURREALDB: Surreal<Client> = Surreal::init();
const SURREALDB_ENDPOINT: &'static str = "SURREALDB_ENDPOINT";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initiate the monitoring hooks
    yaufs_monitoring::init!();

    // connect to the surrealdb instance
    SURREALDB
        .connect::<Ws>(
            std::env::var(SURREALDB_ENDPOINT).expect("Missing SURREALDB_ENDPOINT env variable"),
        )
        .await?;
    // TODO: integrate vault storage
    SURREALDB
        .signin(Root {
            username: "root",
            password: "root",
        })
        .await?;

    // start tonic serve on specified address
    info!("Starting grpc server on {}", ADDRESS);
    let tower_layer = tower::ServiceBuilder::new()
        .layer(TraceLayer::new_for_grpc())
        .into_inner();
    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();
    Server::builder()
        .layer(tower_layer)
        .add_service(reflection)
        .add_service(TemplateServiceServer::new(TemplateServiceContext))
        .serve(ADDRESS.parse().unwrap())
        .await?;

    Ok(())
}

pub mod prelude {
    pub use crate::proto::*;
    pub use tonic::{Request, Response, Status};
}
