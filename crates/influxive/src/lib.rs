#![deny(missing_docs)]
#![deny(warnings)]
#![deny(unsafe_code)]
//! High-level Rust integration of opentelemetry metrics and InfluxDB.

use std::sync::Arc;

use influxive_child_svc::InfluxiveChildSvc;
use influxive_otel::InfluxiveMeterProvider;
use influxive_writer::*;

#[doc(inline)]
pub use influxive_child_svc::InfluxiveChildSvcConfig;

#[doc(inline)]
pub use influxive_writer::InfluxiveWriterConfig;

/// Create an opentelemetry_api MeterProvider ready to provide metrics
/// to a running child process instance of InfluxDB.
pub async fn influxive_child_process_meter_provider(
    config: InfluxiveChildSvcConfig,
) -> std::io::Result<(Arc<InfluxiveChildSvc>, InfluxiveMeterProvider)> {
    let influxive = Arc::new(InfluxiveChildSvc::new(config).await?);
    let meter_provider = InfluxiveMeterProvider::new(influxive.clone());
    Ok((influxive, meter_provider))
}

/// Create an opentelemetry_api MeterProvider ready to provide metrics
/// to an InfluxDB instance that is already running as a separate process.
pub async fn influxive_external_meter_provider_token_auth<
    H: AsRef<str>,
    B: AsRef<str>,
    T: AsRef<str>,
>(
    config: InfluxiveWriterConfig,
    host: H,
    bucket: B,
    token: T,
) -> InfluxiveMeterProvider {
    let writer = InfluxiveWriter::with_token_auth(config, host, bucket, token);
    InfluxiveMeterProvider::new(Arc::new(writer))
}
