extern crate aide;
extern crate thiserror;
extern crate tokio;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate serde;

mod error;

const ADDRESS: &'static str = "0.0.0.0:8000";

#[tokio::main]
async fn main() {
    // init the monitoring services
    yaufs_monitoring::init!();
}

mod prelude {
    pub use crate::error::{AccountServiceError, Result};
}
