use crate::extractor::Json;

use aide::axum::routing::ApiMethodRouter;
use axum::body::HttpBody;

#[derive(Debug, Serialize, JsonSchema)]
pub struct HealthResponse {
    healthy: bool,
}

/// This is the basic endpoint for the service health.
/// All axum based services can use this for kubernetes / consul / apisix health checks.
/// Such a feature is required for an advanced implementation of circuit breakers.
pub fn health_endpoint<S, B>() -> ApiMethodRouter<S, B>
where
    S: 'static + Clone + Send + Sync,
    B: 'static + Send + Sync + HttpBody,
{
    aide::axum::routing::get_with(
        || async { Json(HealthResponse { healthy: true }) },
        |transform| {
            transform
                .description("Check the health status of the service")
                .response_with::<200, Json<HealthResponse>, _>(|response| {
                    response
                        .description("The service is up and healthy")
                        .example(HealthResponse { healthy: true })
                })
        },
    )
}
