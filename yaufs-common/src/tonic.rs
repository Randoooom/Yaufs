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
            headers = ?request.headers(),
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
