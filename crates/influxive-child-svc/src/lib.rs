#![deny(missing_docs)]
#![deny(warnings)]
#![deny(unsafe_code)]
//! Run influxd as a child process.

use std::borrow::Cow;
use std::io::Result;
use std::sync::Arc;

#[cfg(feature = "download_binaries")]
mod download_binaries;
#[cfg(feature = "download_binaries")]
use download_binaries::download_influx;

mod types;
pub use types::*;

fn err_other<E>(error: E) -> std::io::Error
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    std::io::Error::new(std::io::ErrorKind::Other, error.into())
}

macro_rules! cmd_output {
    ($cmd:expr $(,$arg:expr)*) => {async {
        let mut proc = tokio::process::Command::new($cmd);
        proc.stdin(std::process::Stdio::null());
        proc.kill_on_drop(true);
        $(
            proc.arg($arg);
        )*
        let output = proc.output().await?;
        let err = String::from_utf8_lossy(&output.stderr);
        if !err.is_empty() {
            Err(err_other(err.to_string()))
        } else {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
    }.await}
}

/// Configure the child process.
#[derive(Debug)]
pub struct Config {
    #[cfg(feature = "download_binaries")]
    /// If true, will fall back to downloading influx release binaries.
    /// Defaults to `true`.
    pub download_binaries: bool,

    /// Path to influxd binary. If None, will try the path.
    /// Defaults to `None`.
    pub influxd_path: Option<std::path::PathBuf>,

    /// Path to influx cli binary. If None, will try the path.
    /// Defaults to `None`.
    pub influx_path: Option<std::path::PathBuf>,

    /// Path to influx database files and config directory. If None, will
    /// use `./influxive`.
    /// Defaults to `None`.
    pub database_path: Option<std::path::PathBuf>,

    /// Influx initial username.
    /// Defaults to `influxive`.
    pub user: String,

    /// Influx initial password.
    /// Defaults to `influxive`.
    pub pass: String,

    /// Influx initial organization name.
    /// Defaults to `influxive`.
    pub org: String,

    /// Influx initial bucket name.
    /// Defaults to `influxive`.
    pub bucket: String,

    /// Retention timespan.
    /// Defaults to `72h`.
    pub retention: String,

    /// Time span over which metric writes will be buffered before
    /// actually being written to InfluxDB to facilitate batching.
    /// Defaults to `100ms`.
    pub metric_write_batch_duration: std::time::Duration,

    /// The size of the metric write batch buffer. If a metric to be
    /// writen would go beyond this buffer, it will instead be ignored with
    /// a trace written at "debug" level.
    /// Defaults to `4096`.
    pub metric_write_batch_buffer_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            download_binaries: true,
            influxd_path: None,
            influx_path: None,
            database_path: None,
            user: "influxive".to_string(),
            pass: "influxive".to_string(),
            org: "influxive".to_string(),
            bucket: "influxive".to_string(),
            retention: "72h".to_string(),
            metric_write_batch_duration: std::time::Duration::from_millis(100),
            metric_write_batch_buffer_size: 4096,
        }
    }
}

/// A running child-process instance of influxd.
/// Command and control functions are provided through the influx cli tool.
/// Metric writing is provided through the http line protocol.
pub struct Influxive {
    config: Config,
    host: String,
    token: String,
    _child: tokio::process::Child,
    influx_path: std::path::PathBuf,
    write_send: tokio::sync::mpsc::Sender<Metric>,
}

impl Influxive {
    /// Spawn a new influxd child process.
    pub async fn new(config: Config) -> Result<Self> {
        let db_path = config.database_path.clone().unwrap_or_else(|| {
            let mut db_path = std::path::PathBuf::from(".");
            db_path.push("influxive");
            db_path
        });

        tokio::fs::create_dir_all(&db_path).await?;

        let influxd_path = validate_influx(&db_path, &config, false).await?;

        let influx_path = validate_influx(&db_path, &config, true).await?;

        let (_child, port) = spawn_influxd(&db_path, &influxd_path).await?;

        let host = format!("http://127.0.0.1:{port}");

        let mut configs_path = std::path::PathBuf::from(&db_path);
        configs_path.push("configs");

        if let Err(err) = cmd_output!(
            &influx_path,
            "setup",
            "--json",
            "--configs-path",
            &configs_path,
            "--host",
            &host,
            "--username",
            &config.user,
            "--password",
            &config.pass,
            "--org",
            &config.org,
            "--bucket",
            &config.bucket,
            "--retention",
            &config.retention,
            "--force"
        ) {
            let repr = format!("{err:?}");
            if !repr.contains("Error: instance has already been set up") {
                return Err(err);
            }
        }

        let token = tokio::fs::read(&configs_path).await?;
        let token = String::from_utf8_lossy(&token);
        let mut token = token.split("token = \"");
        token.next().unwrap();
        let token = token.next().unwrap();
        let mut token = token.split('\"');
        let token = token.next().unwrap().to_string();

        let client =
            influxdb::Client::new(&host, "influxive").with_token(&token);

        let (write_send, mut write_recv) =
            tokio::sync::mpsc::channel(config.metric_write_batch_buffer_size);

        let write_sleep_dur = config.metric_write_batch_duration;
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

        Ok(Self {
            config,
            host,
            token,
            _child,
            influx_path,
            write_send,
        })
    }

    /// Get the config this instance was created with.
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    /// Get the host url of this running influxd instance.
    pub fn get_host(&self) -> &str {
        &self.host
    }

    /// Get the operator token of this running influxd instance.
    pub fn get_token(&self) -> &str {
        &self.token
    }

    /// "Ping" the running InfluxDB instance, returning the result.
    pub async fn ping(&self) -> Result<()> {
        cmd_output!(&self.influx_path, "ping", "--host", &self.host)?;
        Ok(())
    }

    /// Run a query against the running InfluxDB instance.
    /// Note, if you are writing metrics, prefer the 'write_metric' api
    /// as that will be more efficient.
    pub async fn query<Q: Into<StringType>>(
        &self,
        flux_query: Q,
    ) -> Result<String> {
        cmd_output!(
            &self.influx_path,
            "query",
            "--raw",
            "--org",
            &self.config.org,
            "--host",
            &self.host,
            "--token",
            &self.token,
            flux_query.into().into_string()
        )
    }

    /// Log a metric to the running InfluxDB instance.
    /// Note, this function itself is an efficiency abstraction,
    /// which will return quickly if there is space in the buffer.
    /// The actual call to log the metrics will be made a configurable
    /// timespan later to facilitate batching of metric writes.
    pub fn write_metric(&self, metric: Metric) {
        match self.write_send.try_send(metric) {
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

async fn validate_influx(
    _db_path: &std::path::Path,
    config: &Config,
    is_cli: bool,
) -> Result<std::path::PathBuf> {
    let mut bin_path = if is_cli {
        "influx".into()
    } else {
        "influxd".into()
    };

    if is_cli {
        if let Some(path) = &config.influx_path {
            bin_path = path.clone();
        }
    } else if let Some(path) = &config.influxd_path {
        bin_path = path.clone();
    };

    let _ver = match cmd_output!(&bin_path, "version") {
        Ok(ver) => ver,
        Err(err) => {
            let mut err_list = Vec::new();
            err_list.push(err_other(format!("failed to run {bin_path:?}")));
            err_list.push(err);

            #[cfg(feature = "download_binaries")]
            {
                match download_influx(_db_path, is_cli).await {
                    Ok(path) => {
                        bin_path = path;
                        match cmd_output!(&bin_path, "version") {
                            Ok(ver) => ver,
                            Err(err) => {
                                err_list.push(err_other(format!(
                                    "failed to run {bin_path:?}"
                                )));
                                err_list.push(err);
                                return Err(err_other(format!(
                                    "{:?}",
                                    err_list
                                )));
                            }
                        }
                    }
                    Err(err) => {
                        err_list.push(err_other("failed to download"));
                        err_list.push(err);
                        return Err(err_other(format!("{:?}", err_list)));
                    }
                }
            }

            #[cfg(not(feature = "download_binaries"))]
            {
                return Err(err_other(format!("{:?}", err_list)));
            }
        }
    };

    Ok(bin_path)
}

async fn spawn_influxd(
    db_path: &std::path::Path,
    influxd_path: &std::path::Path,
) -> Result<(tokio::process::Child, u16)> {
    use tokio::io::AsyncBufReadExt;

    let (s, r) = tokio::sync::oneshot::channel();

    let mut s = Some(s);

    let mut engine_path = std::path::PathBuf::from(db_path);
    engine_path.push("engine");
    let mut bolt_path = std::path::PathBuf::from(db_path);
    bolt_path.push("influxd.bolt");

    let mut proc = tokio::process::Command::new(influxd_path);
    proc.kill_on_drop(true);
    proc.arg("--engine-path").arg(engine_path);
    proc.arg("--bolt-path").arg(bolt_path);
    proc.arg("--http-bind-address").arg("127.0.0.1:0");
    proc.arg("--metrics-disabled");
    proc.arg("--reporting-disabled");
    proc.stdout(std::process::Stdio::piped());

    let proc_err = format!("{proc:?}");

    let mut child = proc
        .spawn()
        .map_err(|err| err_other(format!("{proc_err}: {err:?}")))?;

    let stdout = child.stdout.take().unwrap();
    let mut reader = tokio::io::BufReader::new(stdout).lines();

    tokio::task::spawn(async move {
        while let Some(line) = reader.next_line().await.expect("got line") {
            if line.contains("msg=Listening")
                && line.contains("service=tcp-listener")
                && line.contains("transport=http")
            {
                let mut iter = line.split(" port=");
                iter.next().unwrap();
                let item = iter.next().unwrap();
                let port: u16 = item.parse().unwrap();
                if let Some(s) = s.take() {
                    let _ = s.send(port);
                }
            }
        }
    });

    let port = r.await.map_err(|_| err_other("Failed to scrape port"))?;

    Ok((child, port))
}

#[cfg(test)]
mod test;
