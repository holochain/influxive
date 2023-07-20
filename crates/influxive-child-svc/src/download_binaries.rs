use influxive_downloader::*;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod tgt {
    use super::*;
    pub const DL_DB: Option<DownloadSpec> = Some(DownloadSpec {
        url: "https://dl.influxdata.com/influxdb/releases/influxdb2-2.7.1-linux-amd64.tar.gz",
        archive: Archive::TarGz {
            inner_path: "influxdb2_linux_amd64/influxd",
        },
        archive_hash: Hash::Sha2_256(&hex_literal::hex!(
            "e5ecfc15c35af55641ffc92680ad0fb043aa51a942944252e214e2a551c60ebb"
        )),
        file_hash: Hash::Sha2_256(&hex_literal::hex!(
            "68547e6e8b05088f1d824c9923412d22045003026f4f6e844630a126c10a97e1"
        )),
        file_prefix: "influxd",
        file_extension: "",
    });
    pub const DL_CLI: Option<DownloadSpec> = Some(DownloadSpec {
        url: "https://dl.influxdata.com/influxdb/releases/influxdb2-client-2.7.3-linux-amd64.tar.gz",
        archive: Archive::TarGz {
            inner_path: "influx",
        },
        archive_hash: Hash::Sha2_256(&hex_literal::hex!(
            "a266f304547463b6bc7886bf45e37d252bcc0ceb3156ab8d78c52561558fbfe6"
        )),
        file_hash: Hash::Sha2_256(&hex_literal::hex!(
            "63a2aa0112bba8cd357656b5393c5e6655da6c85590374342b5f0ef14c60fa75"
        )),
        file_prefix: "influx",
        file_extension: "",
    });
}

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
mod tgt {
    use super::*;
    pub const DL_DB: Option<DownloadSpec> = Some(DownloadSpec {
        url: "https://dl.influxdata.com/influxdb/releases/influxdb2-2.7.1-linux-arm64.tar.gz",
        archive: Archive::TarGz {
            inner_path: "influxdb2_linux_arm64/influxd",
        },
        archive_hash: Hash::Sha2_256(&hex_literal::hex!(
            "b88989dae0c802fdee499fa07aae837139da3c786293c74e9d7c46b8460510d4"
        )),
        file_hash: Hash::Sha2_256(&hex_literal::hex!(
            "0000000000000000000000000000000000000000000000000000000000000000"
        )),
        file_prefix: "influxd",
        file_extension: "",
    });
    pub const DL_CLI: Option<DownloadSpec> = Some(DownloadSpec {
        url: "https://dl.influxdata.com/influxdb/releases/influxdb2-client-2.7.3-linux-arm64.tar.gz",
        archive: Archive::TarGz {
            inner_path: "influx",
        },
        archive_hash: Hash::Sha2_256(&hex_literal::hex!(
            "d5d09f5279aa32d692362cd096d002d787b3983868487e6f27379b1e205b4ba2"
        )),
        file_hash: Hash::Sha2_256(&hex_literal::hex!(
            "0000000000000000000000000000000000000000000000000000000000000000"
        )),
        file_prefix: "influx",
        file_extension: "",
    });
}

#[cfg(all(
    any(target_os = "macos", target_os = "ios", target_os = "tvos"),
    target_arch = "x86_64"
))]
mod tgt {
    use super::*;
    pub const DL_DB: Option<DownloadSpec> = Some(DownloadSpec {
        url: "https://dl.influxdata.com/influxdb/releases/influxdb2-2.7.1-darwin-amd64.tar.gz",
        archive: Archive::TarGz {
            inner_path: "influxdb2_darwin_amd64/influxd",
        },
        archive_hash: Hash::Sha2_256(&hex_literal::hex!(
            "af709215dce8767ae131802f050c139d0ae179c13f29bb68ca5baa2716aa1874"
        )),
        file_hash: Hash::Sha2_256(&hex_literal::hex!(
            "0000000000000000000000000000000000000000000000000000000000000000"
        )),
        file_prefix: "influxd",
        file_extension: "",
    });
    pub const DL_CLI: Option<DownloadSpec> = Some(DownloadSpec {
        url: "https://dl.influxdata.com/influxdb/releases/influxdb2-client-2.7.3-darwin-amd64.tar.gz",
        archive: Archive::TarGz {
            inner_path: "influx",
        },
        archive_hash: Hash::Sha2_256(&hex_literal::hex!(
            "4d8297fc9e4ba15e432189295743c399a3e2647e9621bf36c68fbae8873f51b1"
        )),
        file_hash: Hash::Sha2_256(&hex_literal::hex!(
            "0000000000000000000000000000000000000000000000000000000000000000"
        )),
        file_prefix: "influx",
        file_extension: "",
    });
}

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
mod tgt {
    use super::*;
    pub const DL_DB: Option<DownloadSpec> = Some(DownloadSpec {
        url: "https://dl.influxdata.com/influxdb/releases/influxdb2-2.7.1-windows-amd64.zip",
        archive: Archive::Zip {
            inner_path: "influxdb2_windows_amd64\\influxd.exe",
        },
        archive_hash: Hash::Sha2_256(&hex_literal::hex!(
            "8e0acbc7dba55a794450fa53d72cd48958d11d39e619394a268e06a6c03af672"
        )),
        file_hash: Hash::Sha2_256(&hex_literal::hex!(
            "0000000000000000000000000000000000000000000000000000000000000000"
        )),
        file_prefix: "influxd",
        file_extension: ".exe",
    });
    pub const DL_CLI: Option<DownloadSpec> = Some(DownloadSpec {
        url: "https://dl.influxdata.com/influxdb/releases/influxdb2-client-2.7.3-windows-amd64.zip",
        archive: Archive::Zip {
            inner_path: "influx.exe",
        },
        archive_hash: Hash::Sha2_256(&hex_literal::hex!(
            "a9265771a2693269e50eeaf2ac82ac01d44305c6c6a5b425cf63e8289b6e89c4"
        )),
        file_hash: Hash::Sha2_256(&hex_literal::hex!(
            "829bb2657149436a88a959ea223c9f85bb25431fcf2891056522d9ec061f093e"
        )),
        file_prefix: "influx",
        file_extension: ".exe",
    });
}

#[cfg(not(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(
        any(target_os = "macos", target_os = "ios", target_os = "tvos"),
        target_arch = "x86_64"
    ),
    all(target_os = "windows", target_arch = "x86_64")
)))]
mod tgt {
    use super::*;
    pub const DL_DB: Option<DownloadSpec> = None;
    pub const DL_CLI: Option<DownloadSpec> = None;
}

pub use tgt::*;
