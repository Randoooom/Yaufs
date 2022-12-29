use crate::error::ServiceRejection;
use axum::response::IntoResponse;

#[derive(FromRequest, OperationIo)]
#[from_request(via(axum_jsonschema::Json), rejection(ServiceRejection))]
#[aide(
    input_with = "axum_jsonschema::Json<T>",
    output_with = "axum_jsonschema::Json<T>",
    json_schema
)]
pub struct Json<T>(pub T);

impl<T> IntoResponse for Json<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> axum::response::Response {
        axum::Json(self.0).into_response()
    }
}
