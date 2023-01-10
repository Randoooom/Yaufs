use axum::http::StatusCode;
use axum::response::IntoResponse;

#[derive(Debug, Serialize, JsonSchema)]
pub struct ServiceRejection {
    /// The status code to apply to the error response
    #[serde(skip)]
    status_code: StatusCode,
    /// the id of the processed trace / request for identifiyng issues
    trace_id: Option<String>,
    /// the error message
    error: String,
}

impl ServiceRejection {
    /// Create a new rejection based on the given parameters
    pub fn new(status_code: StatusCode, error: String) -> Self {
        Self {
            status_code,
            error,
            trace_id: axum_tracing_opentelemetry::find_current_trace_id(),
        }
    }
}

impl IntoResponse for ServiceRejection {
    fn into_response(self) -> axum::response::Response {
        let status_code = self.status_code;
        let mut response = axum::Json(self).into_response();
        *response.status_mut() = status_code;
        response
    }
}
