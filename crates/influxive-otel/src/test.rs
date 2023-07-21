use super::*;
use influxive_child_svc::*;

#[tokio::test(flavor = "multi_thread")]
async fn sanity() {
    use opentelemetry_api::metrics::MeterProvider;

    let tmp = tempfile::tempdir().unwrap();

    let i = Arc::new(
        InfluxiveChildSvc::new(InfluxiveChildSvcConfig {
            influxd_path: Some("bad".into()),
            influx_path: Some("bad".into()),
            database_path: Some(tmp.path().into()),
            metric_write: InfluxiveWriterConfig {
                batch_duration: std::time::Duration::from_millis(5),
                ..Default::default()
            },
            ..Default::default()
        })
        .await
        .unwrap(),
    );

    println!("{}", i.get_host());

    i.ping().await.unwrap();

    let meter_provider = InfluxiveMeterProvider::new(i.clone());
    opentelemetry_api::global::set_meter_provider(meter_provider);

    let meter = opentelemetry_api::global::meter_provider().versioned_meter(
        "my_metrics",
        None::<&'static str>,
        None::<&'static str>,
        Some(vec![opentelemetry_api::KeyValue::new(
            "test-metric-attribute-key",
            "test-metric-attribute-value",
        )]),
    );

    let m_cnt_f64 = meter.f64_counter("m_cnt_f64").init();
    let m_hist_f64 = meter.f64_histogram("m_hist_f64").init();
    let m_obs_cnt_f64 = meter.f64_observable_counter("m_obs_cnt_f64").init();
    let m_obs_g_f64 = meter.f64_observable_gauge("m_obs_g_f64").init();
    let m_obs_ud_f64 =
        meter.f64_observable_up_down_counter("m_obs_ud_f64").init();
    let m_ud_f64 = meter.f64_up_down_counter("m_ud_f64").init();

    let m_hist_i64 = meter.i64_histogram("m_hist_i64").init();
    let m_obs_g_i64 = meter.i64_observable_gauge("m_obs_g_i64").init();
    let m_obs_ud_i64 =
        meter.i64_observable_up_down_counter("m_obs_ud_i64").init();
    let m_ud_i64 = meter.i64_up_down_counter("m_ud_i64").init();

    let m_cnt_u64 = meter.u64_counter("m_cnt_u64").init();
    let m_hist_u64 = meter.u64_histogram("m_hist_u64").init();
    let m_obs_cnt_u64 = meter.u64_observable_counter("m_obs_cnt_u64").init();
    let m_obs_g_u64 = meter.u64_observable_gauge("m_obs_g_u64").init();

    for _ in 0..12 {
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;

        let cx = opentelemetry_api::Context::new();

        macro_rules! obs {
            ($n:ident, $f:ident, $v:literal) => {{
                let $n = $n.clone();
                meter
                    .register_callback(&[$n.as_any()], move |obs| {
                        obs.$f(&$n, $v, &[])
                    })
                    .unwrap()
                    .unregister()
                    .unwrap();
            }};
        }

        m_cnt_f64.add(&cx, 1.1, &[]);
        m_hist_f64.record(&cx, 1.1, &[]);
        obs!(m_obs_cnt_f64, observe_f64, 1.1);
        obs!(m_obs_g_f64, observe_f64, -1.1);
        obs!(m_obs_ud_f64, observe_f64, -1.1);
        m_ud_f64.add(&cx, -1.1, &[]);

        m_hist_i64.record(&cx, -1, &[]);
        obs!(m_obs_g_i64, observe_i64, -1);
        obs!(m_obs_ud_i64, observe_i64, -1);
        m_ud_i64.add(&cx, -1, &[]);

        m_cnt_u64.add(&cx, 1, &[]);
        m_hist_u64.record(&cx, 1, &[]);
        obs!(m_obs_cnt_u64, observe_u64, 1);
        obs!(m_obs_g_u64, observe_u64, 1);
    }

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let result = i
        .query(
            r#"from(bucket: "influxive")
|> range(start: -15m, stop: now())
"#,
        )
        .await
        .unwrap();

    println!("{result}");

    assert_eq!(12, result.matches("m_cnt_f64").count());
    assert_eq!(12, result.matches("m_hist_f64").count());
    assert_eq!(12, result.matches("m_obs_cnt_f64").count());
    assert_eq!(12, result.matches("m_obs_g_f64").count());
    assert_eq!(12, result.matches("m_obs_ud_f64").count());
    assert_eq!(12, result.matches("m_ud_f64").count());

    assert_eq!(12, result.matches("m_hist_i64").count());
    assert_eq!(12, result.matches("m_obs_g_i64").count());
    assert_eq!(12, result.matches("m_obs_ud_i64").count());
    assert_eq!(12, result.matches("m_ud_i64").count());

    assert_eq!(12, result.matches("m_cnt_u64").count());
    assert_eq!(12, result.matches("m_hist_u64").count());
    assert_eq!(12, result.matches("m_obs_cnt_u64").count());
    assert_eq!(12, result.matches("m_obs_g_u64").count());

    println!("about to shutdown influxive-child-svc");
    i.shutdown();

    println!("about to drop influxive-child-svc");
    drop(i);

    println!("about to close tempfile::tempdir");
    // okay if this fails on windows...
    let _ = tmp.close();

    println!("test complete");
}
