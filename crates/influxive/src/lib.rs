#![deny(missing_docs)]
#![deny(warnings)]
#![deny(unsafe_code)]
//! High-level Rust integration of opentelemetry metrics and InfluxDB.

use std::sync::Arc;

use influxive_child_svc::Influxive;
use influxive_otel::InfluxiveMeterProvider;

#[doc(inline)]
pub use influxive_child_svc::Config;

/// Create an opentelemetry_api MeterProvider ready to provide metrics
/// to a running child process instance of InfluxDB.
pub async fn influxive_meter_provider(
    config: Config,
) -> std::io::Result<(Arc<Influxive>, InfluxiveMeterProvider)> {
    let influxive = Arc::new(Influxive::new(config).await?);
    let meter_provider = InfluxiveMeterProvider::new(influxive.clone());
    Ok((influxive, meter_provider))
}
