use crate::prelude::*;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AccountServiceError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Bad request")]
    BadRequest,
}

pub type Result<T> = std::result::Result<T, AccountServiceError>;

impl IntoResponse for AccountServiceError {
    fn into_response(self) -> Response {
        match self {
            AccountServiceError::BadRequest => {
                ServiceRejection::new(StatusCode::BAD_REQUEST, self.to_string())
            }
            AccountServiceError::Unauthorized => {
                ServiceRejection::new(StatusCode::UNAUTHORIZED, self.to_string())
            }
        }
        .into_response()
    }
}
