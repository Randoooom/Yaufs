use opentelemetry::sdk::trace::Tracer;
use opentelemetry::sdk::Resource;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use tonic::metadata::{MetadataKey, MetadataMap};
use tonic::transport::ClientTlsConfig;
use url::Url;

const ENDPOINT: &str = "OTLP_TONIC_ENDPOINT";
const HONEYCOMB_SERVICE: &str = "OTLP_TONIC_SERVICE_NAME";
const HONEYCOMB_TEAM: &str = "OTLP_TONIC_X_HONEYCOMB_TEAM";
const X_HONEYCOMB_TEAM: &str = "X-HONEYCOMB-TEAM";

pub fn install_tracer() -> Tracer {
    // build the metadata
    let mut metadata = MetadataMap::new();
    // add the honeycomb header used for the authentication
    metadata.insert(
        MetadataKey::from_static(X_HONEYCOMB_TEAM),
        std::env::var(HONEYCOMB_SERVICE)
            .expect("missing OTLP_TONIC_X_HONEYCOMB_TEAM")
            .trim()
            .parse()
            .unwrap(),
    );

    // build the exporter
    let endpoint = Url::parse(
        std::env::var(ENDPOINT)
            .expect("missing OTLP_TONIC_ENDPOINT")
            .as_str(),
    )
    .expect("OTLP_TONIC_ENDPOINT has to be a url");
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint.as_str())
        .with_metadata(metadata)
        .with_tls_config(ClientTlsConfig::new().domain_name(endpoint.host_str().unwrap()));

    // build the otlp pipeline
    let pipeline = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter);

    // install the tracer
    pipeline
        .with_trace_config(
            opentelemetry::sdk::trace::config().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                std::env::var(HONEYCOMB_SERVICE).expect("missing OTLP_TONIC_SERVICE_NAME"),
            )])),
        )
        .install_batch(opentelemetry::runtime::Tokio)
        .unwrap()
}
