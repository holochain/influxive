#![deny(missing_docs)]
#![deny(warnings)]
#![deny(unsafe_code)]
//! High-level Rust integration of opentelemetry metrics and InfluxDB.
//!
//! ## Examples
//!
//! ### Easy, zero-configuration InfluxDB as a child process
//!
//! ```
//! # #[tokio::main(flavor = "multi_thread")]
//! # async fn main() {
//! let tmp = tempfile::tempdir().unwrap();
//!
//! // create our meter provider
//! let (_influxive, meter_provider) = influxive::influxive_child_process_meter_provider(
//!     influxive::InfluxiveChildSvcConfig::default()
//!         .with_database_path(Some(tmp.path().to_owned())),
//!     influxive::InfluxiveMeterProviderConfig::default(),
//! ).await.unwrap();
//!
//! // register our meter provider
//! opentelemetry_api::global::set_meter_provider(meter_provider);
//!
//! // create a metric
//! let m = opentelemetry_api::global::meter("my.meter")
//!     .f64_histogram("my.metric")
//!     .init();
//!
//! // make a recording
//! m.record(&opentelemetry_api::Context::new(), 3.14, &[]);
//! # _influxive.shutdown();
//! # }
//! ```
//!
//! ### Connecting to an already running InfluxDB system process
//!
//! ```
//! # #[tokio::main(flavor = "multi_thread")]
//! # async fn main() {
//! // create our meter provider
//! let meter_provider = influxive::influxive_external_meter_provider_token_auth(
//!     influxive::InfluxiveWriterConfig::default(),
//!     influxive::InfluxiveMeterProviderConfig::default(),
//!     "http://127.0.0.1:8086",
//!     "my.bucket",
//!     "my.token",
//! );
//!
//! // register our meter provider
//! opentelemetry_api::global::set_meter_provider(meter_provider);
//!
//! // create a metric
//! let m = opentelemetry_api::global::meter("my.meter")
//!     .f64_histogram("my.metric")
//!     .init();
//!
//! // make a recording
//! m.record(&opentelemetry_api::Context::new(), 3.14, &[]);
//! # }
//! ```

use std::sync::Arc;

use influxive_child_svc::InfluxiveChildSvc;
use influxive_otel::InfluxiveMeterProvider;
use influxive_writer::*;

#[doc(inline)]
pub use influxive_child_svc::InfluxiveChildSvcConfig;

#[doc(inline)]
pub use influxive_writer::InfluxiveWriterConfig;

#[doc(inline)]
pub use influxive_otel::InfluxiveMeterProviderConfig;

/// Create an opentelemetry_api MeterProvider ready to provide metrics
/// to a running child process instance of InfluxDB.
pub async fn influxive_child_process_meter_provider(
    svc_config: InfluxiveChildSvcConfig,
    otel_config: InfluxiveMeterProviderConfig,
) -> std::io::Result<(Arc<InfluxiveChildSvc>, InfluxiveMeterProvider)> {
    let influxive = Arc::new(InfluxiveChildSvc::new(svc_config).await?);
    let meter_provider =
        InfluxiveMeterProvider::new(otel_config, influxive.clone());
    Ok((influxive, meter_provider))
}

/// Create an opentelemetry_api MeterProvider ready to provide metrics
/// to an InfluxDB instance that is already running as a separate process.
pub fn influxive_external_meter_provider_token_auth<
    H: AsRef<str>,
    B: AsRef<str>,
    T: AsRef<str>,
>(
    writer_config: InfluxiveWriterConfig,
    otel_config: InfluxiveMeterProviderConfig,
    host: H,
    bucket: B,
    token: T,
) -> InfluxiveMeterProvider {
    let writer =
        InfluxiveWriter::with_token_auth(writer_config, host, bucket, token);
    InfluxiveMeterProvider::new(otel_config, Arc::new(writer))
}
