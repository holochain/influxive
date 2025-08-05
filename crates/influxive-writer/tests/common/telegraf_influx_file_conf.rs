use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct TelegrafLineProtocolConfig {
    pub influxdb_url: String,
    pub token: String,
    pub organization: String,
    pub bucket: String,
    pub metrics_file_path: PathBuf,
    pub config_output_path: PathBuf,
}

impl TelegrafLineProtocolConfig {
    pub fn generate_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_content = self.build_content();

        // Create parent directories if they don't exist
        if let Some(parent) = Path::new(&self.config_output_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&self.config_output_path)?;
        file.write_all(config_content.as_bytes())?;

        println!(
            "Telegraf Line Protocol configuration written to: {}",
            self.config_output_path.as_path().display()
        );
        Ok(())
    }

    /// Builds Telegraf config file content based on config
    fn build_content(&self) -> String {
        format!(
            r#"# Generated Telegraf Configuration for Line Protocol Metrics

[global_tags]
  # Global tags can be specified here in key="value" format

[agent]
  interval = "5s"
  round_interval = true
  metric_batch_size = 1000
  metric_buffer_limit = 10000
  collection_jitter = "0s"
  flush_interval = "5s"
  flush_jitter = "0s"
  precision = ""
  hostname = ""
  omit_hostname = false
  quiet = false
#  logfile = "logs_telegraf.log"

# Configuration for InfluxDB v2 output
[[outputs.influxdb_v2]]
  ## The URLs of the InfluxDB cluster nodes.
  urls = ["{}"]

  ## Token for authentication
  token = "{}"

  ## Organization is the name of the organization you wish to write to
  organization = "{}"

  ## Destination bucket to write into
  bucket = "{}"

# Input plugin for reading Line Protocol metrics from file
[[inputs.file]]
  ## Files to parse each interval. Accept standard unix glob matching rules,
  ## as well as ** to match recursive files and directories.
  files = ["{}"]

  ## Data format to consume.
  data_format = "influx"

  ## Character encoding to use when interpreting the file contents.  Invalid
  ## characters are replaced using the unicode replacement character.  When set
  ## to the empty string the encoding will be automatically determined.
  character_encoding = "utf-8"
"#,
            self.influxdb_url,
            self.token,
            self.organization,
            self.bucket,
            self.metrics_file_path.as_path().display(),
        )
    }
}

/// Builder pattern for easier configuration
pub struct TelegrafLineProtocolConfigBuilder {
    config: TelegrafLineProtocolConfig,
}

impl TelegrafLineProtocolConfigBuilder {
    /// Constructor
    pub fn new() -> Self {
        Self {
            config: TelegrafLineProtocolConfig {
                influxdb_url: String::new(),
                token: String::new(),
                organization: String::new(),
                bucket: String::new(),
                metrics_file_path: PathBuf::from("app_metrics.log"),
                config_output_path: PathBuf::from("telegraf.conf"),
            },
        }
    }

    pub fn influxdb_url(mut self, url: &str) -> Self {
        self.config.influxdb_url = url.to_string();
        self
    }

    pub fn token(mut self, token: &str) -> Self {
        self.config.token = token.to_string();
        self
    }

    pub fn organization(mut self, org: &str) -> Self {
        self.config.organization = org.to_string();
        self
    }

    pub fn bucket(mut self, bucket: &str) -> Self {
        self.config.bucket = bucket.to_string();
        self
    }

    pub fn metrics_file_path(mut self, path: &str) -> Self {
        self.config.metrics_file_path = PathBuf::from(path);
        self
    }

    pub fn config_output_path(mut self, path: &str) -> Self {
        self.config.config_output_path = PathBuf::from(path);
        self
    }

    pub fn build(self) -> TelegrafLineProtocolConfig {
        self.config
    }
}
