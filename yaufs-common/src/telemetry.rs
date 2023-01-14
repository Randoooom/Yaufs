use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;

/// This function has to be called at the top of every main function belonging
/// to the yaufs-stack. Here we setup the most important functionalities for the monitoring
/// (observability and metrics).
#[allow(dead_code)]
pub fn init_tracing(service_name: &'static str) {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name(service_name)
        .install_batch(opentelemetry::runtime::Tokio)
        .unwrap();

    let layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env())
        .with(layer);

    tracing::subscriber::set_global_default(subscriber).unwrap();
}

#[macro_export]
macro_rules! init_telemetry {
    () => {
        $crate::telemetry::init_tracing(env!("CARGO_PKG_NAME"))
    };
}
