use std::path::Path;
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
        inner_path: "telegraf-1.28.5/telegraf",
    },
    archive_hash: Hash::Sha2_256(&hex!(
            "924e103b33016a44247f97c7ee87bb807882d73d83181e31e251f6903b74fa1e"
        )),
    file_hash: Hash::Sha2_256(&hex!(
            "d64176c8a102043e578dbba69181f75cfd975b5d5118d41ddfc621523ab8f7c9"
        )),
    file_prefix: "telegraf",
    file_extension: ".exe",
};


/// Spawns and handles a Telegraf child service
pub struct TelegrafSvc {
    process: Option<tokio::process::Child>,
    config_path: String,
    binary_dir: String,
    spec: DownloadSpec,
}

impl TelegrafSvc {
    pub fn new(config_path: &str, fallback_binary_dir: &str) -> Self {
        // Detect OS
        let spec = match std::env::consts::OS {
            "linux" | "macos" => TELEGRAF_TAR,
            "windows" => TELEGRAF_ZIP,
            _ => panic!("Unsupported OS: {}", std::env::consts::OS),
        };

        Self {
            process: None,
            config_path: config_path.to_string(),
            binary_dir: fallback_binary_dir.to_string(),
            spec,
        }
    }

    async fn download_telegraf(
        &self,
    ) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        println!("Downloading from: {}", self.spec.url);
        let filepath = self.spec.download(Path::new(&self.binary_dir)).await?;
        println!("Telegraf binary downloaded and extracted successfully to {}", filepath.display());
        Ok(filepath)
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure binary is available
        let filepath = self.download_telegraf().await?;

        println!(
            "Starting Telegraf with config: {} | {:?}",
            self.config_path,
            filepath
        );

        let child = tokio::process::Command::new(&filepath)
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
