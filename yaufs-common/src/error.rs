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

use tonic::Status;

#[derive(thiserror::Error, Debug)]
pub enum YaufsError {
    #[error("{0}")]
    NotFound(&'static str),
    #[error("Unauthorized")]
    Unauthorized,
    #[error(transparent)]
    SurrealdbError(#[from] surrealdb::Error),
}

pub type Result<T> = std::result::Result<T, YaufsError>;

impl From<YaufsError> for Status {
    fn from(value: YaufsError) -> Self {
        tracing::error!("Error occurred: {value}");

        match value {
            YaufsError::NotFound(message) => Status::not_found(message),
            YaufsError::Unauthorized => Status::unauthenticated(value.to_string()),
            YaufsError::SurrealdbError(_) => {
                Status::internal("Error occurred while calling database")
            }
        }
    }
}