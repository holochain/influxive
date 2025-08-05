use crate::common::telegraf_influx_file_conf::TelegrafLineProtocolConfigBuilder;
use crate::common::telegraf_svc::TelegrafSvc;
use influxive_child_svc::{InfluxiveChildSvc, InfluxiveChildSvcConfig};
use influxive_core::*;
use influxive_writer::*;
use std::path::{Path, PathBuf};
use std::time::Duration;

mod common;

/// Setup InfluxiveWriter to use LineProtocolFileBackendFactory
pub fn create_influx_file_writer(test_path: &PathBuf) -> InfluxiveWriter {
    let _ = std::fs::remove_file(&test_path);
    let mut config =
        InfluxiveWriterConfig::create_with_influx_file(test_path.clone());
    config.batch_duration = std::time::Duration::from_millis(30);
    let writer = InfluxiveWriter::with_token_auth(config.clone(), "", "", "");
    writer
}

/// Spawn influxDB with the default config
async fn spawn_influx(path: &Path) -> InfluxiveChildSvc {
    let child = InfluxiveChildSvc::new(
        InfluxiveChildSvcConfig::default()
            .with_database_path(Some(path.into()))
            .with_metric_write(
                InfluxiveWriterConfig::default()
                    .with_batch_duration(std::time::Duration::from_millis(5)),
            ),
    )
    .await
    .unwrap();

    child.ping().await.unwrap();

    child
}

async fn write_metrics_to_file(test_path: &PathBuf) {
    use std::io::BufRead;

    let writer = create_influx_file_writer(test_path);

    // Write one metric
    writer.write_metric(
        Metric::new(std::time::SystemTime::now(), "my-metric")
            .with_field("f1", 1.77)
            .with_field("f2", 2.77)
            .with_field("f3", 3.77)
            .with_tag("tag", "test-tag")
            .with_tag("tag2", "test-tag2"),
    );

    // Write many metrics with different timestamps
    let now = std::time::SystemTime::now()
        .checked_sub(Duration::from_secs(10))
        .unwrap();
    for n in 0..10 {
        writer.write_metric(
            Metric::new(
                now.checked_add(Duration::from_secs(n)).unwrap(),
                "my-second-metric",
            )
            .with_field("val", n),
        );
    }

    // Wait for batch processing to trigger
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Make sure metrics have been written to disk
    let file = std::fs::File::open(&test_path).unwrap();
    let reader = std::io::BufReader::new(file);
    let count = reader.lines().count();
    assert_eq!(count, 11);
}

#[tokio::test(flavor = "multi_thread")]
async fn write_to_file_then_read() {
    let test_dir = tempfile::tempdir().unwrap().path().to_owned();
    std::fs::create_dir_all(&test_dir).unwrap();
    let metrics_path = test_dir.join("test_metrics.influx");
    let telegraf_config_path = test_dir.join("test_telegraf.conf");

    // Write metrics to disk
    write_metrics_to_file(&metrics_path).await;

    // Launch influxDB
    let influx_process = spawn_influx(&test_dir).await;

    // Generate Telegraf config
    let config = TelegrafLineProtocolConfigBuilder::new()
        .influxdb_url(influx_process.get_host())
        .token(influx_process.get_token())
        .organization("influxive")
        .bucket("influxive")
        .metrics_file_path(metrics_path.to_str().unwrap())
        .config_output_path(telegraf_config_path.to_str().unwrap())
        .build();
    assert!(config.generate_file().is_ok());

    // Launch Telegraf
    let mut telegraf_process = TelegrafSvc::new(
        telegraf_config_path.to_str().unwrap(),
        test_dir.to_str().unwrap(),
    );
    telegraf_process.start().await.unwrap();

    // Wait for telegraf to process by querying influxDB every second until we get the expected
    // result or a timeout
    let start = std::time::Instant::now();
    let mut line_count = 0;
    while start.elapsed() < std::time::Duration::from_secs(20) {
        let result = influx_process
            .query(
                r#"from(bucket: "influxive")
|> range(start: -15m, stop: now())
|> filter(fn: (r) => r["_measurement"] == "my-second-metric")
|> filter(fn: (r) => r["_field"] == "val")"#,
            )
            .await
            .unwrap();

        line_count = result
            .split('\n')
            .filter(|l| l.contains("my-second-metric"))
            .count();
        if line_count == 10 {
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    assert_eq!(line_count, 10);

    telegraf_process.stop().await.unwrap();
}
