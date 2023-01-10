#[macro_use]
extern crate serde;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate tracing;

use crate::prelude::*;
use crate::template_service::template_service_server::{TemplateService, TemplateServiceServer};
use std::sync::Arc;
use surrealdb::engine::remote::ws::{Ws, Wss};
use surrealdb::Surreal;
use tonic::transport::Server;
use tower_http::trace::TraceLayer;

mod methods;

pub mod template_service {
    tonic::include_proto!("template_service");
}

pub struct TemplateServiceContext {
    surreal: Arc<Surreal<Wss>>,
}

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
const SURREALDB: &'static str = "SURREALDB";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initiate the monitoring hooks
    yaufs_monitoring::init!();

    // connect to the surrealdb instance
    let surreal =
        Surreal::new::<Wss>(std::env::var(SURREALDB).expect("Missing SURREALDB env variable"))
            .await?;

    // start tonic serve on specified address
    info!("Starting grpc server on {}", ADDRESS);
    Server::builder()
        .layer(TraceLayer::new_for_grpc())
        .add_service(TemplateServiceServer::new(TemplateServiceContext {
            surreal: Arc::new(surreal),
        }))
        .serve(ADDRESS.parse().unwrap())
        .await?;

    Ok(())
}

pub mod prelude {
    pub use crate::template_service::*;
    pub use tonic::{Request, Response, Status};
}
