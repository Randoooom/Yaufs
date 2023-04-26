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

use crate::error::{Result, YaufsError};
use cached::lazy_static::lazy_static;
use fluvio::config::{TlsConfig, TlsPaths, TlsPolicy};
use fluvio::{Fluvio, FluvioConfig, PartitionConsumer, TopicProducer};
use std::path::PathBuf;

const FLUVIO_ENDPOINT: &str = "FLUVIO_ENDPOINT";
const FLUVIO_TLS_DOMAIN: &str = "FLUVIO_TLS_DOMAIN";
const FLUVIO_TLS_KEY_PATH: &str = "FLUVIO_TLS_KEY_PATH";
const FLUVIO_TLS_CRT_PATH: &str = "FLUVIO_TLS_CRT_PATH";
const FLUVIO_TLS_CA_PATH: &str = "FLUVIO_TLS_CA_PATH";

lazy_static! {
    pub static ref FLUVIO_CONFIG: FluvioConfig = FluvioConfig::new(
        std::env::var(FLUVIO_ENDPOINT)
            .unwrap_or_else(|_| panic!("Missing env var {FLUVIO_ENDPOINT}")),
    )
    .with_tls(TlsPolicy::Verified(TlsConfig::Files(TlsPaths {
        domain: std::env::var(FLUVIO_TLS_DOMAIN)
            .unwrap_or_else(|_| panic!("Missing env var {FLUVIO_TLS_DOMAIN}")),
        key: PathBuf::from(
            std::env::var(FLUVIO_TLS_KEY_PATH)
                .unwrap_or_else(|_| panic!("Missing env var {FLUVIO_TLS_KEY_PATH}")),
        ),
        cert: PathBuf::from(
            std::env::var(FLUVIO_TLS_CRT_PATH)
                .unwrap_or_else(|_| panic!("Missing env var {FLUVIO_TLS_CRT_PATH}")),
        ),
        ca_cert: PathBuf::from(
            std::env::var(FLUVIO_TLS_CA_PATH)
                .unwrap_or_else(|_| panic!("Missing env var {FLUVIO_TLS_CA_PATH}")),
        ),
    })));
}

pub async fn producer() -> Result<TopicProducer> {
    // connect to the fluvio spu group
    let fluvio = Fluvio::connect_with_config(&FLUVIO_CONFIG)
        .await
        .map_err(|error| YaufsError::FluvioError(error.to_string()))?;
    // start the producer
    let producer = fluvio
        .topic_producer("events")
        .await
        .map_err(|error| YaufsError::FluvioError(error.to_string()))?;

    Ok(producer)
}

pub async fn consumer() -> Result<PartitionConsumer> {
    // connect to the fluvio spu group
    let fluvio = Fluvio::connect_with_config(&FLUVIO_CONFIG)
        .await
        .map_err(|error| YaufsError::FluvioError(error.to_string()))?;
    // start the consumer
    let consumer = fluvio
        .partition_consumer("events", 0)
        .await
        .map_err(|error| YaufsError::FluvioError(error.to_string()))?;

    Ok(consumer)
}

#[macro_export]
macro_rules! fluvio_err {
    ($expr:expr) => {
        $expr.map_err(|error| YaufsError::FluvioError(error.to_string()))
    };
}
