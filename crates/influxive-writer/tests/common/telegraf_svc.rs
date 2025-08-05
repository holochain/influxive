use crate::common::telegraf_binaries::TELEGRAF_SPEC;
use std::path::Path;
use std::process::Stdio;

/// Spawns and handles a Telegraf child service
pub struct TelegrafSvc {
    process: Option<tokio::process::Child>,
    config_path: String,
    binary_dir: String,
}

impl TelegrafSvc {
    pub fn new(config_path: &str, fallback_binary_dir: &str) -> Self {
        Self {
            process: None,
            config_path: config_path.to_string(),
            binary_dir: fallback_binary_dir.to_string(),
        }
    }

    async fn download_telegraf(
        &self,
    ) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        println!("Downloading from: {}", TELEGRAF_SPEC.url);
        let filepath =
            TELEGRAF_SPEC.download(Path::new(&self.binary_dir)).await?;
        println!(
            "Telegraf binary downloaded and extracted successfully to {}",
            filepath.display()
        );
        Ok(filepath)
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure binary is available
        let filepath = self.download_telegraf().await?;

        println!(
            "Starting Telegraf with config: {} | {:?}",
            self.config_path, filepath
        );

        let child = tokio::process::Command::new(&filepath)
            .arg("--config")
            .arg(&self.config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
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
