use hex_literal::hex;
use influxive_downloader::{Archive, DownloadSpec, Hash};

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub const TELEGRAF_SPEC: DownloadSpec = DownloadSpec {
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

#[cfg(all(
    any(target_os = "macos", target_os = "ios", target_os = "tvos"),
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
pub const TELEGRAF_SPEC: DownloadSpec = DownloadSpec {
    url: "https://dl.influxdata.com/telegraf/releases/telegraf-1.28.5_darwin_amd64.zip",
    archive: Archive::Zip {
        inner_path: "telegraf-1.28.5/telegraf",
    },
    archive_hash: Hash::Sha2_256(&hex!(
            "0848074b210d4a40e4b22f6a8b3c48450428ad02f9f796c1e2d55dee8d441c5b"
        )),
    file_hash: Hash::Sha2_256(&hex!(
            "e6e2c820431aa9a89ee1a8ada2408c0a058e138bb5126ae27bcadb9624e5f2dc"
        )),
    file_prefix: "telegraf",
    file_extension: ".exe",
};

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub const TELEGRAF_SPEC: DownloadSpec = DownloadSpec {
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
