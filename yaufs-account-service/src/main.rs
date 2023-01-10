extern crate aide;
extern crate axum;
extern crate thiserror;
extern crate tokio;
#[macro_use]
extern crate tracing;
extern crate yaufs_axum;
#[macro_use]
extern crate schemars;
#[macro_use]
extern crate serde;

mod docs;
mod error;
mod routes;

use aide::axum::ApiRouter;
use aide::openapi::OpenApi;
use axum::{Extension, Router};
use std::net::SocketAddr;
use std::sync::Arc;

const ADDRESS: &'static str = "0.0.0.0:8000";

#[tokio::main]
async fn main() {
    // init the monitoring services
    yaufs_monitoring::init!();

    let mut api = OpenApi::default();
    let router: Router<()> = ApiRouter::<()>::new()
        .nest_api_service("/docs", yaufs_axum::router::redoc(()))
        .api_route("/health", yaufs_axum::router::health_endpoint())
        .finish_api_with(&mut api, docs::docs)
        .layer(Extension(Arc::new(api)))
        .layer(yaufs_monitoring::axum_tracing_opentelemetry::opentelemetry_tracing_layer());

    info!("Starting axum on {}", ADDRESS);

    axum::Server::bind(&ADDRESS.parse().unwrap())
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

mod prelude {
    pub use crate::error::{AccountServiceError, Result};
    pub use yaufs_axum::prelude::*;
}
