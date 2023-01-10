extern crate axum;
#[macro_use]
extern crate schemars;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate axum_macros;
#[macro_use]
extern crate aide;

mod error;
mod extractor;
pub mod router;

pub mod prelude {
    pub use crate::error::ServiceRejection;
    pub use crate::extractor::Json;

    pub use axum::extract::Query;
    pub use axum::http::StatusCode;
}
