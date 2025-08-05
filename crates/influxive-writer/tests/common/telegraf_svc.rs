use std::path::{Path, PathBuf};
use std::process::Stdio;
use influxive_downloader::{Archive, DownloadSpec, Hash};
use hex_literal::hex;


const TELEGRAF_TAR: DownloadSpec = DownloadSpec {
    url: "https://dl.influxdata.com/telegraf/releases/telegraf-1.28.5_linux_amd64.tar.gz",
    archive: Archive::TarGz {
        inner_path: "telegraf-1.28.5/usr/bin/telegraf",
    },
    archive_hash: Hash::Sha2_256(&hex!(
            "ae2f925e8e999299d4f4e6db7c20395813457edfb4128652d685cecb501ef669"
        )),
    file_hash: Hash::Sha2_256(&hex!(
            "8e9e4cf36fd7ebda5270c53453153f4d551ea291574fdaed08e376eaf6d3700b"
        )),
    file_prefix: "telegraf",
    file_extension: "",
};

const TELEGRAF_ZIP: DownloadSpec = DownloadSpec {
    url: "https://dl.influxdata.com/telegraf/releases/telegraf-1.28.5-windows-amd64.zip",
    archive: Archive::Zip {
        inner_path: "telegraf",
    },
    archive_hash: Hash::Sha2_256(&hex!(
            "a9265771a2693269e50eeaf2ac82ac01d44305c6c6a5b425cf63e8289b6e89c4"
        )),
    file_hash: Hash::Sha2_256(&hex!(
            "829bb2657149436a88a959ea223c9f85bb25431fcf2891056522d9ec061f093e"
        )),
    file_prefix: "telegraf",
    file_extension: ".exe",
};


/// Spawns and handles a Telegraf child service
pub struct TelegrafSvc {
    process: Option<tokio::process::Child>,
    config_path: String,
    binary_dir: String,
}

impl TelegrafSvc {
    fn binary_path(&self) -> PathBuf {
        PathBuf::from(&self.binary_dir).join("telegraf")
    }

    pub fn new(config_path: &str, binary_path: &str) -> Self {
        Self {
            process: None,
            config_path: config_path.to_string(),
            binary_dir: binary_path.to_string(),
        }
    }

    pub async fn ensure_binary(
        &self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.binary_path().exists() {
            println!("Telegraf binary already exists at {}", self.binary_dir);
            return Ok(());
        }

        println!("Downloading Telegraf binary...");
        self.download_telegraf().await?;
        Ok(())
    }

    async fn download_telegraf(
        &self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&self.binary_dir)?;

        // Detect OS and architecture
        let spec = match std::env::consts::OS {
            "linux" | "macos" => TELEGRAF_TAR,
            "windows" => TELEGRAF_ZIP,
            _ => return Err(format!("Unsupported OS: {}", std::env::consts::OS).into()),
        };

        println!("Downloading from: {}", spec.url);

        let filepath = spec.download(Path::new(&self.binary_dir)).await?;
        println!("Telegraf binary downloaded and extracted successfully to {}", filepath.display());

        tokio::fs::copy(&filepath, &self.binary_path()).await?;

        Ok(())
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure binary is available
        self.ensure_binary().await?;

        println!(
            "Starting Telegraf with config: {} | {:?}",
            self.config_path,
            self.binary_path()
        );

        let child = tokio::process::Command::new(&self.binary_path())
            .arg("--config")
            .arg(&self.config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.process = Some(child);

        println!("Telegraf started successfully");
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut child) = self.process.take() {
            println!("Stopping Telegraf...");
            child.kill().await?;
            child.wait().await?;
            println!("Telegraf stopped");
        }
        Ok(())
    }
}

impl Drop for TelegrafSvc {
    fn drop(&mut self) {
        if let Some(child) = self.process.take() {
            // Use blocking kill since we can't await in Drop
            let _ = std::process::Command::new("kill")
                .arg(child.id().unwrap_or(0).to_string())
                .output();
        }
    }
}
