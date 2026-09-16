//! Go installer - fetch the latest stable Go release tarball into `{data}/tools/go/`
//! and expose `go`/`gofmt`.
//!
//! Go distributes official archives (.tar.gz) from `https://go.dev/dl/` whose release
//! metadata is published at `https://go.dev/dl/?mode=json`.
//! The tarball unpacks to a `go/` directory containing `bin/go` and `bin/gofmt`.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use yerd_php::{current_os_arch, is_safe_member, Arch, Downloader, Os};
use yerd_platform::PlatformDirs;

use super::{
    extract_root_dir, stage_and_swap, tool_dir, verify_sha256, Tool, ToolError,
};

const GO_DL_JSON: &str = "https://go.dev/dl/?mode=json";
const GO_DL_BASE: &str = "https://go.dev/dl";

#[derive(Debug, Deserialize)]
struct GoRelease {
    #[allow(dead_code)]
    version: String,
    stable: bool,
    files: Vec<GoFile>,
}

#[derive(Debug, Deserialize)]
struct GoFile {
    filename: String,
    os: String,
    arch: String,
    version: String,
    sha256: String,
    kind: String,
}

/// The platform tokens Go uses in download filenames: (os, arch).
fn host_platform() -> Option<(&'static str, &'static str)> {
    let (os, arch) = current_os_arch().ok()?;
    let os_str = match os {
        Os::Linux => "linux",
        Os::Macos => "darwin",
    };
    let arch_str = match arch {
        Arch::X86_64 => "amd64",
        Arch::Aarch64 => "arm64",
    };
    Some((os_str, arch_str))
}

/// Find the latest stable Go release file matching the host os and arch.
fn find_release_file(json_bytes: &[u8], os: &str, arch: &str) -> Option<(String, String, String)> {
    let releases: Vec<GoRelease> = serde_json::from_slice(json_bytes).ok()?;
    for rel in releases {
        if !rel.stable {
            continue;
        }
        for f in rel.files {
            if f.os == os && f.arch == arch && f.kind == "archive" {
                return Some((f.version, f.filename, f.sha256));
            }
        }
    }
    None
}

/// Install the latest stable Go release for the host into `{data}/tools/go/`.
pub async fn install(dirs: &PlatformDirs, dl: &dyn Downloader) -> Result<(), ToolError> {
    let (os_str, arch_str) = host_platform().ok_or(ToolError::UnsupportedHost("Go"))?;
    let index = dl
        .download(GO_DL_JSON)
        .await
        .map_err(|e| ToolError::Download(format!("go release index: {e}")))?;
    let (version, filename, want_sha) = find_release_file(&index, os_str, arch_str)
        .ok_or_else(|| ToolError::Download(format!("go: no stable archive found for {os_str}/{arch_str}")))?;

    let tarball_url = format!("{GO_DL_BASE}/{filename}");
    let bytes = dl
        .download(&tarball_url)
        .await
        .map_err(|e| ToolError::Download(format!("{filename}: {e}")))?;
    verify_sha256(&bytes, &want_sha, &filename)?;

    stage_and_swap(dirs, Tool::Go, &version, |staging| {
        unpack_tar_gz(&bytes, staging, &filename)
    })?;
    tracing::info!(version = %version, "installed Go");
    Ok(())
}

/// `(name_in_bin, target)` links for an installed Go: `go`/`gofmt` → the dist `bin/`.
#[cfg(unix)]
pub(crate) fn shim_links(dirs: &PlatformDirs) -> Vec<(String, PathBuf)> {
    let Ok(root) = extract_root_dir(&tool_dir(dirs, Tool::Go)) else {
        return Vec::new();
    };
    let bin = root.join("bin");
    ["go", "gofmt"]
        .into_iter()
        .map(|n| (n.to_owned(), bin.join(n)))
        .collect()
}

/// Safely unpack a Go `.tar.gz` full tree into `dest`, preserving permissions.
fn unpack_tar_gz(gz_bytes: &[u8], dest: &Path, label: &str) -> Result<(), ToolError> {
    let decoder = flate2::read::GzDecoder::new(gz_bytes);
    let mut archive = tar::Archive::new(decoder);
    archive.set_preserve_permissions(true);
    let entries = archive
        .entries()
        .map_err(|e| ToolError::Unpack(format!("{label}: {e}")))?;
    for entry in entries {
        let mut entry = entry.map_err(|e| ToolError::Unpack(format!("{label}: {e}")))?;
        let path = entry
            .path()
            .map_err(|e| ToolError::Unpack(format!("{label}: {e}")))?
            .into_owned();
        let name = path.to_string_lossy().into_owned();
        if !is_safe_member(&name) {
            return Err(ToolError::Unpack(format!("unsafe archive member {name:?}")));
        }
        let out = dest.join(&path);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ToolError::Unpack(format!("{}: {e}", parent.display())))?;
        }
        entry
            .unpack(&out)
            .map_err(|e| ToolError::Unpack(format!("{name}: {e}")))?;
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn find_release_file_picks_stable_archive() {
        let json = br#"[
            {
                "version": "go1.24.1",
                "stable": true,
                "files": [
                    {
                        "filename": "go1.24.1.src.tar.gz",
                        "os": "",
                        "arch": "",
                        "version": "go1.24.1",
                        "sha256": "abcdef",
                        "kind": "source"
                    },
                    {
                        "filename": "go1.24.1.linux-amd64.tar.gz",
                        "os": "linux",
                        "arch": "amd64",
                        "version": "go1.24.1",
                        "sha256": "123456",
                        "kind": "archive"
                    }
                ]
            }
        ]"#;
        let res = find_release_file(json, "linux", "amd64");
        assert_eq!(
            res,
            Some((
                "go1.24.1".into(),
                "go1.24.1.linux-amd64.tar.gz".into(),
                "123456".into()
            ))
        );
        assert_eq!(find_release_file(json, "darwin", "arm64"), None);
    }
}
