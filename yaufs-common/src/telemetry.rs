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

#[cfg(not(test))]
use opentelemetry::global;
#[cfg(not(test))]
use opentelemetry::sdk::propagation::TraceContextPropagator;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;

/// This function has to be called at the top of every main function belonging
/// to the yaufs-stack. Here we setup the most important functionalities for the monitoring
/// (observability and metrics).
#[allow(dead_code)]
pub fn init_tracing(service_name: &'static str) {
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env());

    cfg_if::cfg_if! {
        if #[cfg(not(test))] {
            global::set_text_map_propagator(TraceContextPropagator::new());
            let tracer = opentelemetry_jaeger::new_agent_pipeline()
                .with_service_name(service_name)
                .install_batch(opentelemetry::runtime::Tokio)
                .unwrap();
            let layer = tracing_opentelemetry::layer().with_tracer(tracer);
            let subscriber = subscriber.with(layer);
        }
    }

    tracing::subscriber::set_global_default(subscriber).unwrap();
}

#[macro_export]
macro_rules! init_telemetry {
    () => {
        $crate::telemetry::init_tracing(env!("CARGO_PKG_NAME"))
    };
}
