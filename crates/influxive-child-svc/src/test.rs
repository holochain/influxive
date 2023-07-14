use super::*;

#[tokio::test(flavor = "multi_thread")]
async fn sanity() {
    let tmp = tempfile::tempdir().unwrap();

    const METRIC: &'static str = "my.metric";
    const VALUE: &'static str = "value";

    let i = Influxive::new(Config {
        influxd_path: Some("bad".into()),
        influx_path: Some("bad".into()),
        database_path: Some(tmp.path().into()),
        metric_write_batch_duration: std::time::Duration::from_millis(5),
        ..Default::default()
    })
    .await
    .unwrap();

    println!("{}", i.get_host());

    i.ping().await.unwrap();

    let mut last_time = std::time::Instant::now();

    for _ in 0..12 {
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;

        i.write_metric(
            Metric::new(std::time::SystemTime::now(), METRIC)
                .with_field(VALUE, last_time.elapsed().as_secs_f64())
                .with_tag("tag-name", "tag-value"),
        );

        last_time = std::time::Instant::now();
    }

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let result = i
        .query(
            r#"from(bucket: "influxive")
|> range(start: -15m, stop: now())
|> filter(fn: (r) => r["_measurement"] == "my.metric")
|> filter(fn: (r) => r["_field"] == "value")"#,
        )
        .await
        .unwrap();

    // make sure the result contains at least 10 of the entries
    // we just added plus the header lines
    let line_count = result.split("\n").count();
    assert_eq!(18, line_count);

    drop(i);
    tmp.close().unwrap();
}