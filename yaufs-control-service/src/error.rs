use crate::prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AccountServiceError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Bad request")]
    BadRequest,
}

pub type Result<T> = std::result::Result<T, AccountServiceError>;
