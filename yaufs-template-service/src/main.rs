extern crate serde;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate tracing;

use crate::prelude::*;
use crate::proto::template_service_server::{TemplateService, TemplateServiceServer};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tonic::transport::Server;
use tower_http::trace::TraceLayer;

mod database;
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
        _request: Request<TemplateId>,
    ) -> Result<Response<Template>, Status> {
        todo!()
    }

    async fn list_templates(
        &self,
        request: Request<ListTemplatesMessage>,
    ) -> Result<Response<TemplateList>, Status> {
        Ok(Response::new(TemplateList { templates: vec![] }))
    }

    async fn delete_template(
        &self,
        _request: Request<TemplateId>,
    ) -> Result<Response<Template>, Status> {
        todo!()
    }

    async fn create_template(
        &self,
        _request: Request<CreateTemplateMessage>,
    ) -> Result<Response<Template>, Status> {
        todo!()
    }
}

const ADDRESS: &str = "0.0.0.0:8000";
pub static SURREALDB: Surreal<Client> = Surreal::init();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // init opentelemetry
    yaufs_common::init_telemetry!();

    // connect to the surrealdb instance
    database::connect().await?;

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
        .add_service(
            yaufs_common::tonic::init_health::<TemplateServiceServer<TemplateServiceContext>>()
                .await,
        )
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
