pub extern crate axum_tracing_opentelemetry;
extern crate simple_logger;

mod telemetry;

use log::LevelFilter;
#[cfg(not(debug_assertions))]
use tracing_subscriber::layer::SubscriberExt;

/// This function has to be called at the top of every main function belonging
/// to the yaufs-stack. Here we setup the most important functionalities for the monitoring
/// (observability and metrics).
pub fn init() {
    // load dotenv
    dotenv::dotenv().ok();

    // set the logging format
    simple_logger::SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .ok();

    // if we do not run in debug mode install the telemetry module
    #[cfg(not(debug_assertions))]
    {
        let tracer = telemetry::install_tracer();
        let subscriber = tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(tracing_opentelemetry::layer().with_tracer(tracer));

        // activate global
        tracing::subscriber::set_global_default(subscriber)
            .expect("Error while setting global subscriber");
    }
}
