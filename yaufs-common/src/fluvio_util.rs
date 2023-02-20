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

use crate::error::Result;
use fluvio::{Fluvio, FluvioConfig, PartitionConsumer, TopicProducer};

const FLUVIO_ENDPOINT: &str = "FLUVIO_ENDPOINT";

pub async fn producer() -> Result<TopicProducer> {
    // connect to the fluvio spu group
    let config = FluvioConfig::new(
        std::env::var(FLUVIO_ENDPOINT)
            .unwrap_or_else(|_| panic!("Missing env var {FLUVIO_ENDPOINT}")),
    );
    let fluvio = Fluvio::connect_with_config(&config).await?;
    // start the producer
    let producer = fluvio.topic_producer("events").await?;

    Ok(producer)
}

pub async fn consumer() -> Result<PartitionConsumer> {
    // connect to the fluvio spu group
    let config = FluvioConfig::new(
        std::env::var(FLUVIO_ENDPOINT)
            .unwrap_or_else(|_| panic!("Missing env var {FLUVIO_ENDPOINT}")),
    );
    let fluvio = Fluvio::connect_with_config(&config).await?;
    // start the consumer
    let consumer = fluvio.partition_consumer("events", 0).await?;

    Ok(consumer)
}
