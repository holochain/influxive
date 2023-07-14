use super::*;

const LINUX_X86_64_DB_URL: &str =
    "https://dl.influxdata.com/influxdb/releases/influxdb2-2.7.1-linux-amd64.tar.gz";
const LINUX_X86_64_DB_SHA: [u8; 32] = hex_literal::hex!(
    "e5ecfc15c35af55641ffc92680ad0fb043aa51a942944252e214e2a551c60ebb"
);
const LINUX_X86_64_CLI_URL: &str =
    "https://dl.influxdata.com/influxdb/releases/influxdb2-client-2.7.3-linux-amd64.tar.gz";
const LINUX_X86_64_CLI_SHA: [u8; 32] = hex_literal::hex!(
    "a266f304547463b6bc7886bf45e37d252bcc0ceb3156ab8d78c52561558fbfe6"
);

const LINUX_AARCH64_DB_URL: &str =
    "https://dl.influxdata.com/influxdb/releases/influxdb2-2.7.1-linux-arm64.tar.gz";
const LINUX_AARCH64_DB_SHA: [u8; 32] = hex_literal::hex!(
    "b88989dae0c802fdee499fa07aae837139da3c786293c74e9d7c46b8460510d4"
);
const LINUX_AARCH64_CLI_URL: &str =
    "https://dl.influxdata.com/influxdb/releases/influxdb2-client-2.7.3-linux-arm64.tar.gz";
const LINUX_AARCH64_CLI_SHA: [u8; 32] = hex_literal::hex!(
    "d5d09f5279aa32d692362cd096d002d787b3983868487e6f27379b1e205b4ba2"
);

const DARWIN_X64_64_DB_URL: &str =
    "https://dl.influxdata.com/influxdb/releases/influxdb2-2.7.1-darwin-amd64.tar.gz";
const DARWIN_X64_64_DB_SHA: [u8; 32] = hex_literal::hex!(
    "af709215dce8767ae131802f050c139d0ae179c13f29bb68ca5baa2716aa1874"
);
const DARWIN_X64_64_CLI_URL: &str =
    "https://dl.influxdata.com/influxdb/releases/influxdb2-client-2.7.3-darwin-amd64.tar.gz";
const DARWIN_X64_64_CLI_SHA: [u8; 32] = hex_literal::hex!(
    "4d8297fc9e4ba15e432189295743c399a3e2647e9621bf36c68fbae8873f51b1"
);

const WINDOWS_X64_64_DB_URL: &str =
    "https://dl.influxdata.com/influxdb/releases/influxdb2-2.7.1-windows-amd64.zip";
const WINDOWS_X64_64_DB_SHA: [u8; 32] = hex_literal::hex!(
    "8e0acbc7dba55a794450fa53d72cd48958d11d39e619394a268e06a6c03af672"
);
const WINDOWS_X64_64_CLI_URL: &str =
    "https://dl.influxdata.com/influxdb/releases/influxdb2-client-2.7.3-windows-amd64.zip";
const WINDOWS_X64_64_CLI_SHA: [u8; 32] = hex_literal::hex!(
    "a9265771a2693269e50eeaf2ac82ac01d44305c6c6a5b425cf63e8289b6e89c4"
);

#[cfg(target_os = "linux")]
const TGT_OS: &str = "linux";
#[cfg(any(target_os = "macos", target_os = "ios", target_os = "tvos"))]
const TGT_OS: &str = "darwin";
#[cfg(target_os = "windows")]
const TGT_OS: &str = "windows";
#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "windows"
)))]
const TGT_OS: &str = "other";

#[cfg(target_arch = "x86_64")]
const TGT_ARCH: &str = "x86_64";
#[cfg(target_arch = "aarch64")]
const TGT_ARCH: &str = "aarch64";
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
const TGT_ARCH: &str = "other";

pub(crate) async fn download_influx(
    db_path: &std::path::Path,
    is_cli: bool,
) -> Result<std::path::PathBuf> {
    use sha2::Digest;
    use std::io::Seek;
    use tokio::io::AsyncSeekExt;

    struct Target {
        pub url: &'static str,
        pub sha256hash: &'static [u8; 32],
        pub unpack_path: std::path::PathBuf,
        pub final_path: std::path::PathBuf,
    }

    let mut target = None;

    let mut is_tar_gz = true;

    if TGT_OS == "linux" && TGT_ARCH == "x86_64" {
        if is_cli {
            let url = LINUX_X86_64_CLI_URL;
            let sha256hash = &LINUX_X86_64_CLI_SHA;
            let mut unpack_path = std::path::PathBuf::from(db_path);
            unpack_path.push("influx_unpack");
            let mut final_path = unpack_path.clone();
            final_path.push("influx");
            target = Some(Target {
                url,
                sha256hash,
                unpack_path,
                final_path,
            });
        } else {
            let url = LINUX_X86_64_DB_URL;
            let sha256hash = &LINUX_X86_64_DB_SHA;
            let mut unpack_path = std::path::PathBuf::from(db_path);
            unpack_path.push("influxd_unpack");
            let mut final_path = unpack_path.clone();
            final_path.push("influxdb2_linux_amd64");
            final_path.push("influxd");
            target = Some(Target {
                url,
                sha256hash,
                unpack_path,
                final_path,
            });
        }
    } else if TGT_OS == "linux" && TGT_ARCH == "aarch64" {
        if is_cli {
            let url = LINUX_AARCH64_CLI_URL;
            let sha256hash = &LINUX_AARCH64_CLI_SHA;
            let mut unpack_path = std::path::PathBuf::from(db_path);
            unpack_path.push("influx_unpack");
            let mut final_path = unpack_path.clone();
            final_path.push("influx");
            target = Some(Target {
                url,
                sha256hash,
                unpack_path,
                final_path,
            });
        } else {
            let url = LINUX_AARCH64_DB_URL;
            let sha256hash = &LINUX_AARCH64_DB_SHA;
            let mut unpack_path = std::path::PathBuf::from(db_path);
            unpack_path.push("influxd_unpack");
            let mut final_path = unpack_path.clone();
            final_path.push("influxdb2_linux_arm64");
            final_path.push("influxd");
            target = Some(Target {
                url,
                sha256hash,
                unpack_path,
                final_path,
            });
        }
    } else if TGT_OS == "darwin" && TGT_ARCH == "x86_64" {
        if is_cli {
            let url = DARWIN_X64_64_CLI_URL;
            let sha256hash = &DARWIN_X64_64_CLI_SHA;
            let mut unpack_path = std::path::PathBuf::from(db_path);
            unpack_path.push("influx_unpack");
            let mut final_path = unpack_path.clone();
            final_path.push("influx");
            target = Some(Target {
                url,
                sha256hash,
                unpack_path,
                final_path,
            });
        } else {
            let url = DARWIN_X64_64_DB_URL;
            let sha256hash = &DARWIN_X64_64_DB_SHA;
            let mut unpack_path = std::path::PathBuf::from(db_path);
            unpack_path.push("influxd_unpack");
            let mut final_path = unpack_path.clone();
            final_path.push("influxdb2_darwin_amd64");
            final_path.push("influxd");
            target = Some(Target {
                url,
                sha256hash,
                unpack_path,
                final_path,
            });
        }
    } else if TGT_OS == "windows" && TGT_ARCH == "x86_64" {
        is_tar_gz = false;
        if is_cli {
            let url = WINDOWS_X64_64_CLI_URL;
            let sha256hash = &WINDOWS_X64_64_CLI_SHA;
            let mut unpack_path = std::path::PathBuf::from(db_path);
            unpack_path.push("influx_unpack");
            let mut final_path = unpack_path.clone();
            final_path.push("influx.exe");
            target = Some(Target {
                url,
                sha256hash,
                unpack_path,
                final_path,
            });
        } else {
            let url = WINDOWS_X64_64_DB_URL;
            let sha256hash = &WINDOWS_X64_64_DB_SHA;
            let mut unpack_path = std::path::PathBuf::from(db_path);
            unpack_path.push("influxd_unpack");
            let mut final_path = unpack_path.clone();
            final_path.push("influxdb2_windows_amd64");
            final_path.push("influxd.exe");
            target = Some(Target {
                url,
                sha256hash,
                unpack_path,
                final_path,
            });
        }
    }

    let Target {
        url,
        sha256hash,
        unpack_path,
        final_path,
    } = match target {
        Some(t) => t,
        None => {
            return Err(err_other(format!(
                "cannot download influxd for {TGT_OS} {TGT_ARCH}"
            )));
        }
    };

    if let Ok(true) = tokio::fs::try_exists(&final_path).await {
        return Ok(final_path);
    }

    let file = tempfile::tempfile()?;
    let mut file = tokio::fs::File::from_std(file);

    let mut data = reqwest::get(url).await.map_err(err_other)?.bytes_stream();

    let mut hasher = sha2::Sha256::new();

    use futures::stream::StreamExt;
    while let Some(bytes) = data.next().await {
        let bytes = bytes.map_err(err_other)?;

        hasher.update(&bytes);

        let mut reader: &[u8] = &bytes;
        tokio::io::copy(&mut reader, &mut file).await?;
    }

    let hash = hasher.finalize();
    if hash.as_slice() != sha256hash {
        return Err(err_other("download hash mismatch"));
    }

    file.rewind().await?;
    let mut file = file.into_std().await;

    if is_tar_gz {
        let big_file = tempfile::tempfile()?;

        let big_file = tokio::task::spawn_blocking(move || {
            let mut decoder = flate2::write::GzDecoder::new(big_file);
            std::io::copy(&mut file, &mut decoder)?;
            let mut big_file = decoder.finish()?;
            big_file.rewind()?;
            std::io::Result::Ok(big_file)
        })
        .await??;

        tokio::task::spawn_blocking(move || {
            let mut archive = tar::Archive::new(big_file);
            archive.unpack(unpack_path)
        })
        .await??;
    } else {
        tokio::task::spawn_blocking(move || {
            let mut archive = zip::ZipArchive::new(file).map_err(err_other)?;
            archive.extract(unpack_path).map_err(err_other)
        })
        .await??;
    }

    Ok(final_path)
}
