use crate::extractor::Json;
use aide::axum::routing::{get, get_with};
use aide::axum::{ApiRouter, IntoApiResponse};
use aide::openapi::OpenApi;
use aide::redoc::Redoc;
use axum::response::IntoResponse;
use std::ops::Deref;

pub fn redoc<S: 'static + Send + Sync + Clone>(state: S) -> ApiRouter<S> {
    aide::gen::infer_responses(true);

    let router = ApiRouter::new()
        .api_route_with(
            "/",
            get_with(
                Redoc::new("/docs/private/api.json")
                    .with_title("Api-Documentation")
                    .axum_handler(),
                |op| op.description("This page"),
            ),
            |p| p.security_requirement("OpenId"),
        )
        .route("/private/api.json", get(serve_specification))
        .with_state(state);

    aide::gen::infer_responses(false);
    router
}

/// Serve the openapi specification
async fn serve_specification(
    axum::Extension(api): axum::Extension<std::sync::Arc<OpenApi>>,
) -> impl IntoApiResponse {
    Json(api.deref()).into_response()
}
