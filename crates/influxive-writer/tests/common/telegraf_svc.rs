use crate::common::telegraf_binaries::TELEGRAF_SPEC;
use std::path::PathBuf;
use std::process::Stdio;

/// Spawns and handles a Telegraf child service
pub struct TelegrafSvc {
    process: Option<tokio::process::Child>,
    config_path: PathBuf,
    binary_dir: PathBuf,
}

impl TelegrafSvc {
    pub fn new(config_path: &str, fallback_binary_dir: &str) -> Self {
        Self {
            process: None,
            config_path: PathBuf::from(config_path),
            binary_dir: PathBuf::from(fallback_binary_dir),
        }
    }

    async fn download_telegraf(
        &self,
    ) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        println!("Downloading from: {}", TELEGRAF_SPEC.url);
        let filepath =
            TELEGRAF_SPEC.download(self.binary_dir.as_path()).await?;
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
            "Starting Telegraf with config: {} | {}",
            self.config_path.to_string_lossy(),
            filepath.to_string_lossy(),
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
