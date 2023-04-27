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

use opentelemetry::global;
use opentelemetry::propagation::{Extractor, Injector};
use opentelemetry_http::HeaderExtractor;
use std::time::Duration;
use tonic::codegen::http::Request;
use tonic::server::NamedService;
use tonic_health::proto::health_server::{Health, HealthServer};
use tonic_health::server::HealthReporter;
use tower_http::classify::{GrpcErrorsAsFailures, SharedClassifier};
use tower_http::trace::{MakeSpan, TraceLayer};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

fn twiddle_service_status<S>(mut reporter: HealthReporter)
where
    S: NamedService,
{
    tokio::spawn(async move {
        let mut iter = 0u64;
        loop {
            iter += 1;
            tokio::time::sleep(Duration::from_secs(1)).await;

            if iter % 2 == 0 {
                reporter.set_serving::<S>().await;
            } else {
                reporter.set_not_serving::<S>().await;
            };
        }
    });
}

pub async fn init_health<S>() -> HealthServer<impl Health + Sized>
where
    S: NamedService,
{
    let (mut reporter, service) = tonic_health::server::health_reporter();
    // start serving
    reporter.set_not_serving::<S>().await;
    twiddle_service_status::<S>(reporter);

    service
}

#[derive(Clone)]
pub struct MakeYaufsTonicSpan;
impl<B> MakeSpan<B> for MakeYaufsTonicSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        // make the span
        let span = tracing::info_span!(
            "GRPC Request",
            method = request.method().as_str(),
            rpc.system = "grpc",
            uri = %request.uri(),
            version = ?request.version(),
        );
        let context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderExtractor(request.headers()))
        });
        span.set_parent(context);

        span
    }
}

pub fn trace_layer() -> TraceLayer<SharedClassifier<GrpcErrorsAsFailures>, MakeYaufsTonicSpan> {
    TraceLayer::new_for_grpc().make_span_with(MakeYaufsTonicSpan)
}

/// Injects the tracing context for a new request
pub fn inject_tracing_context<T>(mut request: tonic::Request<T>) -> tonic::Request<T> {
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(
            &Span::current().context(),
            &mut MutMetadataMap(request.metadata_mut()),
        )
    });

    request
}

struct MetadataMap<'a>(pub &'a tonic::metadata::MetadataMap);
struct MutMetadataMap<'a>(pub &'a mut tonic::metadata::MetadataMap);

impl<'a> Injector for MutMetadataMap<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes()) {
            if let Ok(val) = tonic::metadata::MetadataValue::try_from(&value) {
                self.0.insert(key, val);
            }
        }
    }
}

impl<'a> Extractor for MetadataMap<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|metadata| metadata.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .map(|key| match key {
                tonic::metadata::KeyRef::Ascii(v) => v.as_str(),
                tonic::metadata::KeyRef::Binary(v) => v.as_str(),
            })
            .collect::<Vec<_>>()
    }
}
