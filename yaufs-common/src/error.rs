/*
 *    Copyright  2023.  Fritz Ochsmann
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

use openidconnect::core::CoreErrorResponseType;
use openidconnect::{RequestTokenError, StandardErrorResponse, UserInfoError};
use tonic::Status;

#[derive(thiserror::Error, Debug)]
pub enum YaufsError {
    #[error("{0}")]
    NotFound(&'static str),
    #[error("Unauthorized")]
    Unauthorized,
    #[cfg(feature = "surrealdb")]
    #[error(transparent)]
    SurrealdbError(#[from] surrealdb::Error),
    #[cfg(feature = "skytable")]
    #[error(transparent)]
    SkytableError(#[from] skytable::error::Error),
    #[cfg(feature = "skytable")]
    #[error(transparent)]
    SkytablePoolError(#[from] skytable::pool::bb8Error<skytable::error::Error>),
    #[cfg(feature = "fluvio")]
    #[error(transparent)]
    FluvioError(#[from] fluvio::FluvioError),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error("{0}")]
    InternalServerError(String),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, YaufsError>;

impl From<YaufsError> for Status {
    fn from(value: YaufsError) -> Self {
        tracing::error!("Error occurred: {value}");

        match value {
            YaufsError::NotFound(message) => Status::not_found(message),
            YaufsError::Unauthorized => Status::unauthenticated(value.to_string()),
            _ => Status::internal("Error occurred while processing the request"),
        }
    }
}

macro_rules! from_openid_error {
    ($error:path) => {
        impl<T> From<$error> for YaufsError
        where
            T: std::fmt::Debug + std::error::Error,
        {
            fn from(error: $error) -> Self {
                tracing::error!("Error occurred: {:?}", error);

                Self::Unauthorized
            }
        }
    };
}

from_openid_error!(UserInfoError<T>);
from_openid_error!(RequestTokenError<T, StandardErrorResponse<CoreErrorResponseType>>);

impl From<openidconnect::ConfigurationError> for YaufsError {
    fn from(error: openidconnect::ConfigurationError) -> Self {
        Self::InternalServerError(error.to_string())
    }
}
