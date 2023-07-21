#![deny(missing_docs)]
#![deny(warnings)]
#![deny(unsafe_code)]
//! Rust utility for efficiently writing metrics to a running InfluxDB instance.

use influxive_core::*;

trait DataTypeExt {
    fn into_type(self) -> influxdb::Type;
}

impl DataTypeExt for DataType {
    fn into_type(self) -> influxdb::Type {
        match self {
            DataType::Bool(b) => influxdb::Type::Boolean(b),
            DataType::F64(f) => influxdb::Type::Float(f),
            DataType::I64(i) => influxdb::Type::SignedInteger(i),
            DataType::U64(u) => influxdb::Type::UnsignedInteger(u),
            DataType::String(s) => influxdb::Type::Text(s.into_string()),
        }
    }
}

/// InfluxDB metric writer configuration.
#[derive(Debug, Clone)]
pub struct InfluxiveWriterConfig {
    /// Time span over which metric writes will be buffered before
    /// actually being written to InfluxDB to facilitate batching.
    /// Defaults to `100ms`.
    pub batch_duration: std::time::Duration,

    /// The size of the metric write batch buffer. If a metric to be
    /// writen would go beyond this buffer, it will instead be ignored with
    /// a trace written at "debug" level.
    /// Defaults to `4096`.
    pub batch_buffer_size: usize,
}

impl Default for InfluxiveWriterConfig {
    fn default() -> Self {
        Self {
            batch_duration: std::time::Duration::from_millis(100),
            batch_buffer_size: 4096,
        }
    }
}

/// InfluxDB metric writer instance.
pub struct InfluxiveWriter(tokio::sync::mpsc::Sender<Metric>);

impl InfluxiveWriter {
    /// Construct a new writer authenticated by a token.
    pub fn with_token_auth<H: AsRef<str>, B: AsRef<str>, T: AsRef<str>>(
        config: InfluxiveWriterConfig,
        host: H,
        bucket: B,
        token: T,
    ) -> Self {
        let client = influxdb::Client::new(host.as_ref(), bucket.as_ref())
            .with_token(token.as_ref());

        let (write_send, mut write_recv) =
            tokio::sync::mpsc::channel(config.batch_buffer_size);

        let write_sleep_dur = config.batch_duration;
        tokio::task::spawn(async move {
            let mut q = Vec::new();

            loop {
                tokio::time::sleep(write_sleep_dur).await;

                loop {
                    match write_recv.try_recv() {
                        Ok(Metric {
                            timestamp,
                            name,
                            fields,
                            tags,
                        }) => {
                            let mut query = influxdb::WriteQuery::new(
                                influxdb::Timestamp::Nanoseconds(
                                    timestamp
                                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_nanos()
                                ),
                                name.into_string(),
                            );
                            for (k, v) in fields {
                                query = query.add_field(k.into_string(), v.into_type());
                            }
                            for (k, v) in tags {
                                query = query.add_tag(k.into_string(), v.into_type());
                            }
                            q.push(query)
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                        Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => return,
                    }
                }

                if let Err(err) = client.query(std::mem::take(&mut q)).await {
                    tracing::warn!(?err, "write metrics error");
                }
            }
        });

        Self(write_send)
    }

    /// Log a metric to the running InfluxDB instance.
    /// Note, this function itself is an efficiency abstraction,
    /// which will return quickly if there is space in the buffer.
    /// The actual call to log the metrics will be made a configurable
    /// timespan later to facilitate batching of metric writes.
    pub fn write_metric(&self, metric: Metric) {
        match self.0.try_send(metric) {
            Ok(()) => (),
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                tracing::warn!("metrics overloaded, dropping metric");
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                unreachable!("should be impossible, sender task panic?");
            }
        }
    }
}

impl influxive_core::MetricWriter for InfluxiveWriter {
    fn write_metric(&self, metric: Metric) {
        InfluxiveWriter::write_metric(self, metric);
    }
}
