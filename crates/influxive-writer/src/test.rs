use crate::types::*;
use crate::*;

struct TestBackend(usize, Arc<std::sync::atomic::AtomicUsize>);

impl Backend for TestBackend {
    fn buffer_metric(&mut self, _metric: Metric) {
        self.0 += 1;
    }

    fn buffer_count(&self) -> usize {
        self.0
    }

    fn send(
        &mut self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = ()> + '_ + Send + Sync>,
    > {
        Box::pin(async move {
            // simulate it taking a while to do things
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            self.1
                .fetch_add(self.0, std::sync::atomic::Ordering::SeqCst);
            self.0 = 0;
        })
    }
}

#[derive(Debug)]
struct TestFactory {
    write_count: Arc<std::sync::atomic::AtomicUsize>,
}

impl TestFactory {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
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
        let out: Box<dyn Backend + 'static + Send + Sync> =
            Box::new(TestBackend(0, self.write_count.clone()));
        out
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn writer_stress() {
    let factory = TestFactory::new();

    let config = InfluxiveWriterConfig {
        batch_duration: std::time::Duration::from_millis(3),
        batch_buffer_size: 10,
        backend: factory.clone(),
    };

    let writer = InfluxiveWriter::with_token_auth(config, "", "", "");

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // this should be well within our cadence
    for _ in 0..5 {
        for _ in 0..5 {
            writer.write_metric(
                Metric::new(std::time::SystemTime::now(), "my.metric")
                    .with_field("val", 3.14)
                    .with_tag("tag", "test-tag"),
            );
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    assert_eq!(25, factory.get_write_count());

    // this should be well outside our cadence
    for _ in 0..5 {
        for _ in 0..20 {
            writer.write_metric(
                Metric::new(std::time::SystemTime::now(), "my.metric")
                    .with_field("val", 3.14)
                    .with_tag("tag", "test-tag"),
            );
        }
        tokio::time::sleep(std::time::Duration::from_millis(3)).await;
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    assert!(factory.get_write_count() < 100);
}
