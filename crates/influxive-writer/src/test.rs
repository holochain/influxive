use crate::types::*;
use crate::*;

struct TestBackend {
    test_start: std::time::Instant,
    buffer_count: usize,
    write_count: Arc<std::sync::atomic::AtomicUsize>,
}

impl TestBackend {
    pub fn new(
        test_start: std::time::Instant,
        write_count: Arc<std::sync::atomic::AtomicUsize>,
    ) -> Self {
        Self {
            test_start,
            buffer_count: 0,
            write_count,
        }
    }
}

impl Backend for TestBackend {
    fn buffer_metric(&mut self, _metric: Metric) {
        self.buffer_count += 1;
        println!(
            "@@@ {:0.2} - buffer {}",
            self.test_start.elapsed().as_secs_f64(),
            self.buffer_count
        );
    }

    fn buffer_count(&self) -> usize {
        self.buffer_count
    }

    fn send(
        &mut self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = ()> + '_ + Send + Sync>,
    > {
        Box::pin(async move {
            // simulate it taking a while to do things
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            self.write_count.fetch_add(
                self.buffer_count,
                std::sync::atomic::Ordering::SeqCst,
            );
            self.buffer_count = 0;

            println!(
                "@@@ {:0.2} - write",
                self.test_start.elapsed().as_secs_f64()
            );
        })
    }
}

#[derive(Debug)]
struct TestFactory {
    test_start: std::time::Instant,
    write_count: Arc<std::sync::atomic::AtomicUsize>,
}

impl TestFactory {
    pub fn new(test_start: std::time::Instant) -> Arc<Self> {
        Arc::new(Self {
            test_start,
            write_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        })
    }

    pub fn get_write_count(&self) -> usize {
        self.write_count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl BackendFactory for TestFactory {
    fn with_token_auth(
        &self,
        _host: String,
        _bucket: String,
        _token: String,
    ) -> Box<dyn Backend + 'static + Send + Sync> {
        let out: Box<dyn Backend + 'static + Send + Sync> = Box::new(
            TestBackend::new(self.test_start, self.write_count.clone()),
        );
        out
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn writer_stress() {
    let test_start = std::time::Instant::now();

    let factory = TestFactory::new(test_start);

    let config = InfluxiveWriterConfig {
        batch_duration: std::time::Duration::from_millis(30),
        batch_buffer_size: 10,
        backend: factory.clone(),
    };

    let writer = InfluxiveWriter::with_token_auth(config, "", "", "");

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    println!(
        "@@@ {:0.2} - start easy",
        test_start.elapsed().as_secs_f64()
    );

    let mut cnt = 0;

    // this should be well within our cadence
    for _ in 0..5 {
        for _ in 0..5 {
            cnt += 1;
            println!(
                "@@@ {:0.2} - submit {}",
                test_start.elapsed().as_secs_f64(),
                cnt
            );
            writer.write_metric(
                Metric::new(std::time::SystemTime::now(), "my.metric")
                    .with_field("val", 3.77)
                    .with_tag("tag", "test-tag"),
            );
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    assert_eq!(25, factory.get_write_count());

    println!(
        "@@@ {:0.2} - start stress",
        test_start.elapsed().as_secs_f64()
    );

    // this should be well outside our cadence
    for _ in 0..5 {
        for _ in 0..50 {
            cnt += 1;
            println!(
                "@@@ {:0.2} - submit {}",
                test_start.elapsed().as_secs_f64(),
                cnt
            );
            writer.write_metric(
                Metric::new(std::time::SystemTime::now(), "my.metric")
                    .with_field("val", 3.77)
                    .with_tag("tag", "test-tag"),
            );
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    assert!(factory.get_write_count() < 250);
}
