use std::path::{Path, PathBuf};
use std::process::Stdio;

static TELEGRAF_VERSION: &str = "1.28.5";

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
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        let (platform, extension) = match os {
            "linux" => ("linux", "tar.gz"),
            "macos" => ("darwin", "tar.gz"),
            "windows" => ("windows", "zip"),
            _ => return Err(format!("Unsupported OS: {}", os).into()),
        };

        let arch_name = match arch {
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            _ => {
                return Err(format!("Unsupported architecture: {}", arch).into())
            }
        };

        let filename = format!(
            "telegraf-{}_{}_{}.{}",
            TELEGRAF_VERSION, platform, arch_name, extension
        );
        let url =
            format!("https://dl.influxdata.com/telegraf/releases/{}", filename);

        println!("Downloading from: {}", url);

        // Download the archive
        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(format!(
                "Failed to download Telegraf: HTTP {}",
                response.status()
            )
            .into());
        }

        let archive_bytes = response.bytes().await?;
        let temp_file = format!("{}/{}", self.binary_dir, filename);
        std::fs::write(&temp_file, &archive_bytes)?;
        println!("Telegraf binary downloaded successfully to {}", temp_file);

        // Extract the archive
        self.extract_telegraf(&temp_file, extension).await?;

        // Clean up temp file
        std::fs::remove_file(&temp_file)?;

        println!("Telegraf binary downloaded and extracted successfully");
        Ok(())
    }

    async fn extract_telegraf(
        &self,
        archive_path: &str,
        extension: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match extension {
            "tar.gz" => {
                // Use tokio process to run tar command
                let output = tokio::process::Command::new("tar")
                    .args(&["-xzf", archive_path])
                    .args(&["-C", self.binary_dir.as_str()])
                    .output()
                    .await?;

                if !output.status.success() {
                    return Err(format!(
                        "Failed to extract {}: {} ",
                        archive_path,
                        String::from_utf8_lossy(&output.stderr)
                    )
                    .into());
                }
                println!(
                    "Telegraf binary extracted successfully: {}",
                    archive_path
                );

                // Copy to our bin directory
                let filename = format!("telegraf-{}", TELEGRAF_VERSION);
                let extracted_dir =
                    PathBuf::from(&self.binary_dir).join(filename);
                let extracted_bin_path = Path::new(&extracted_dir)
                    .join("usr")
                    .join("bin")
                    .join("telegraf");
                tokio::fs::copy(&extracted_bin_path, &self.binary_path())
                    .await?;

                // Make it executable
                tokio::process::Command::new("chmod")
                    .args(&[
                        "+x",
                        self.binary_path()
                            .as_path()
                            .as_os_str()
                            .to_str()
                            .unwrap(),
                    ])
                    .output()
                    .await?;

                println!("Telegraf binary made executable {}", self.binary_dir);
            }
            "zip" => {
                // For Windows, we'd need to handle zip extraction
                return Err("ZIP extraction not implemented for now".into());
            }
            _ => {
                return Err(format!(
                    "Unsupported archive format: {}",
                    extension
                )
                .into())
            }
        }

        Ok(())
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure binary is available
        self.ensure_binary().await?;

        println!(
            "Starting Telegraf with config: {} | {}",
            self.config_path,
            self.binary_path().as_os_str().to_str().unwrap()
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
