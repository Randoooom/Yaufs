use std::time::Duration;
use tonic::server::NamedService;
use tonic_health::proto::health_server::{Health, HealthServer};
use tonic_health::server::HealthReporter;

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
