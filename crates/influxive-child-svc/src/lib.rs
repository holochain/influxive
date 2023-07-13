#![deny(missing_docs)]
#![deny(warnings)]
#![deny(unsafe_code)]
//! Run influxd as a child process.

const LINUX_X86_64_CLI_URL: &str = "https://dl.influxdata.com/influxdb/releases/influxdb2-client-2.7.3-linux-amd64.tar.gz";
const LINUX_X86_64_CLI_SHA: &str = "a266f304547463b6bc7886bf45e37d252bcc0ceb3156ab8d78c52561558fbfe6";
const LINUX_X86_64_DB_URL: &str = "https://dl.influxdata.com/influxdb/releases/influxdb2-2.7.1-linux-amd64.tar.gz";
const LINUX_X86_64_DB_SHA: &str = "e5ecfc15c35af55641ffc92680ad0fb043aa51a942944252e214e2a551c60ebb";

use std::io::Result;

macro_rules! cmd_output {
    ($cmd:expr $(,$arg:expr)*) => {async {
        let mut proc = tokio::process::Command::new($cmd);
        proc.kill_on_drop(true);
        $(
            proc.arg($arg);
        )*
        let output = proc.output().await?;
        ::std::io::Result::Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }.await}
}


fn err_other<E>(error: E) -> std::io::Error
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    std::io::Error::new(std::io::ErrorKind::Other, error.into())
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

        cmd_output!(
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
        )?;

        let token = tokio::fs::read(&configs_path).await?;
        let token = String::from_utf8_lossy(&token);
        let mut token = token.split("token = \"");
        token.next().unwrap();
        let token = token.next().unwrap();
        let mut token = token.split("\"");
        let token = token.next().unwrap().to_string();

        Ok(Self {
            config,
            host,
            token,
            _child,
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
    } else {
        if let Some(path) = &config.influxd_path {
            bin_path = path.clone();
        }
    };

    let ver = match cmd_output!(&bin_path, "version") {
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
                                err_list.push(err_other(format!("failed to run {bin_path:?}")));
                                err_list.push(err);
                                return Err(err_other(format!("{:?}", err_list)));
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

    println!("{ver}");

    Ok(bin_path)
}

#[cfg(target_os = "linux")]
const TGT_OS: &str = "linux";
#[cfg(not(any(target_os = "linux")))]
const TGT_OS: &str = "other";

#[cfg(target_arch = "x86_64")]
const TGT_ARCH: &str = "x86_64";
#[cfg(not(any(target_arch = "x86_64")))]
const TGT_ARCH: &str = "other";

#[cfg(feature = "download_binaries")]
async fn download_influx(
    db_path: &std::path::Path,
    is_cli: bool,
) -> Result<std::path::PathBuf> {
    use tokio::io::AsyncSeekExt;
    use std::io::Seek;

    struct Target {
        pub url: &'static str,
        pub _sha: &'static str,
        pub unpack_path: std::path::PathBuf,
        pub final_path: std::path::PathBuf,
    }

    let mut target = None;

    if TGT_OS == "linux" && TGT_ARCH == "x86_64" {
        if is_cli {
            let url = LINUX_X86_64_CLI_URL;
            let _sha = LINUX_X86_64_CLI_SHA;
            let mut unpack_path = std::path::PathBuf::from(db_path);
            unpack_path.push("influx_unpack");
            let mut final_path = unpack_path.clone();
            final_path.push("influx");
            target = Some(Target {
                url,
                _sha,
                unpack_path,
                final_path,
            });
        } else {
            let url = LINUX_X86_64_DB_URL;
            let _sha = LINUX_X86_64_DB_SHA;
            let mut unpack_path = std::path::PathBuf::from(db_path);
            unpack_path.push("influxd_unpack");
            let mut final_path = unpack_path.clone();
            final_path.push("influxdb2_linux_amd64");
            final_path.push("influxd");
            target = Some(Target {
                url,
                _sha,
                unpack_path,
                final_path,
            });
        }
    }

    let Target { url, _sha, unpack_path, final_path } = match target {
        Some(t) => t,
        None => {
            return Err(err_other(format!("cannot download influxd for {TGT_OS} {TGT_ARCH}")));
        }
    };

    if let Ok(true) = tokio::fs::try_exists(&final_path).await {
        return Ok(final_path);
    }

    let file = tempfile::tempfile()?;
    let mut file = tokio::fs::File::from_std(file);

    let mut data = reqwest::get(url).await.map_err(err_other)?.bytes_stream();

    use futures::stream::StreamExt;
    while let Some(bytes) = data.next().await {
        let bytes = bytes.map_err(err_other)?;
        let mut reader: &[u8] = &bytes;
        tokio::io::copy(&mut reader, &mut file).await?;
    }

    file.rewind().await?;
    let mut file = file.into_std().await;

    let big_file = tempfile::tempfile()?;

    let big_file = tokio::task::spawn_blocking(move || {
        let mut decoder = flate2::write::GzDecoder::new(big_file);
        std::io::copy(&mut file, &mut decoder)?;
        let mut big_file = decoder.finish()?;
        big_file.rewind()?;
        std::io::Result::Ok(big_file)
    }).await??;

    let _ = tokio::fs::remove_dir_all(&unpack_path).await;

    tokio::task::spawn_blocking(move || {
        let mut archive = tar::Archive::new(big_file);
        archive.unpack(unpack_path)
    }).await??;

    Ok(final_path)
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

    let mut child = proc.spawn().map_err(|err| {
        err_other(format!("{proc_err}: {err:?}"))
    })?;

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
mod test {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn sanity() {
        let _i = Influxive::new(Config::default()).await.unwrap();
    }
}
