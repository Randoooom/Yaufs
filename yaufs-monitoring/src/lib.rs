pub extern crate axum_tracing_opentelemetry;

use axum_tracing_opentelemetry::make_resource;
use tracing_subscriber::layer::SubscriberExt;

/// This function has to be called at the top of every main function belonging
/// to the yaufs-stack. Here we setup the most important functionalities for the monitoring
/// (observability and metrics).
#[allow(dead_code)]
pub fn init_tracing(service_name: &'static str, service_version: &'static str) {
    let resource = make_resource(service_name, service_version);
    let tracer = axum_tracing_opentelemetry::jaeger::init_tracer(
        resource,
        axum_tracing_opentelemetry::jaeger::identity,
    )
    .unwrap();

    let layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(layer);

    tracing::subscriber::set_global_default(subscriber).unwrap();
}

#[macro_export]
macro_rules! init {
    () => {
        $crate::init_tracing(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    };
}
