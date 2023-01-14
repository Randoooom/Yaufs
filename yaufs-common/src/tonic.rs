use opentelemetry::trace::TraceId;
use opentelemetry::Context;
use std::time::Duration;
use tonic::codegen::http::Request;
use tonic::server::NamedService;
use tonic_health::proto::health_server::{Health, HealthServer};
use tonic_health::server::HealthReporter;
use tower_http::classify::{GrpcErrorsAsFailures, SharedClassifier};
use tower_http::trace::{MakeSpan, TraceLayer};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

const TRACE_ALPHABET: [char; 16] = [
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
];

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
        let headers = request.headers();
        let method = request.method().as_str();
        let uri = request.uri();

        // try to access the x-request-id header otherwise generate a new traceid
        let trace_id = headers
            .get("x-request-id")
            .and_then(|value| value.to_str().ok().and_then(|v| Some(v.to_string())))
            .unwrap_or_else(|| nanoid::nanoid!(32, &TRACE_ALPHABET));

        // make the span
        let span = tracing::info_span!(
            "GRPC Request",
            method = method,
            rpc.system = "grpc",
            uri = %uri,
            version = ?request.version(),
            headers = ?request.headers(),
            trace_id = trace_id,
        );
        println!("{:?}", Context::current());

        span
    }
}

pub fn trace_layer() -> TraceLayer<SharedClassifier<GrpcErrorsAsFailures>, MakeYaufsTonicSpan> {
    TraceLayer::new_for_grpc().make_span_with(MakeYaufsTonicSpan)
}
