use crate::common::telegraf_binaries::TELEGRAF_SPEC;
use std::path::PathBuf;
use std::process::Stdio;

/// Spawns and handles a Telegraf child service
pub struct TelegrafSvc {
    _process: tokio::process::Child,
}

impl TelegrafSvc {
    pub async fn spawn(
        config_path: &str,
        fallback_binary_dir: &str,
    ) -> std::io::Result<Self> {
        // Ensure binary is available
        let filepath =
            TelegrafSvc::download_telegraf(fallback_binary_dir).await?;

        println!(
            "Starting Telegraf with config: {} | {}",
            config_path,
            filepath.to_string_lossy(),
        );

        let child = tokio::process::Command::new(&filepath)
            .arg("--config")
            .arg(&config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        println!("Telegraf started successfully");

        Ok(Self { _process: child })
    }

    async fn download_telegraf(binary_dir: &str) -> std::io::Result<PathBuf> {
        println!("Downloading from: {}", TELEGRAF_SPEC.url);
        let filepath = TELEGRAF_SPEC
            .download(PathBuf::from(binary_dir).as_path())
            .await?;
        println!(
            "Telegraf binary downloaded and extracted successfully to {}",
            filepath.display()
        );
        Ok(filepath)
    }
}

impl Drop for TelegrafSvc {
    fn drop(&mut self) {
        println!("Telegraf stopped");
    }
}
