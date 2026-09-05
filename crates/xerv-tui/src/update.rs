//! Update System — niezależny od TUI.
//!
//! Odpowiada za:
//! - wykrywanie najnowszego GitHub Release (`check_for_update`),
//! - pobieranie i bezpieczną instalację (`download_and_install`),
//! - porównywanie wersji SemVer (`cmp_versions`),
//! - wybór artifactu (`select_asset`),
//! - weryfikację SHA256 (`verify_sha256`).
//!
//! Logika porównań i wyboru używa surowych danych (czyste funkcje) —
//! testowalna bez HTTP. `check_for_update` i `download_and_install`
//! używają `reqwest` (blocking) — izolowane właśnie jako jedyne miejsca sieci.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;
use xerv_core::api::{ApiError, ApiResult, Version};

/// Repozytorium GitHub (owner/repo).
const GITHUB_API: &str = "https://api.github.com/repos/";

/// Reprezentuje jednego assetu w GitHub Release.
#[derive(Debug, Deserialize, Clone)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
    #[serde(default)]
    pub size: u64,
}

/// GitHub Release (minimalny payload).
#[derive(Debug, Deserialize, Clone)]
pub struct GitHubRelease {
    pub tag_name: String,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub assets: Vec<Asset>,
}

/// Wyciągnięta informacja o dostępności updat'u.
#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub version: Version,
    pub is_prerelease: bool,
    pub asset_url: String,
    pub sha256_url: String,
}

/// Wynik porównania wersji.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpResult {
    Newer,
    Same,
    Older,
}

/// Porównuje bieżącą wersję z najnowszą.
pub fn cmp_versions(current: &Version, latest: &Version) -> CmpResult {
    if latest > current {
        CmpResult::Newer
    } else if latest == current {
        CmpResult::Same
    } else {
        CmpResult::Older
    }
}

/// Parsuje tag GitHub release na SemVer. Obsługuje prefiks `v`.
pub fn parse_tag(tag: &str) -> ApiResult<Version> {
    let t = tag.strip_prefix('v').unwrap_or(tag);
    Version::parse(t).map_err(|e| ApiError::Other(format!("invalid semver tag '{tag}': {e}")))
}

/// Wybiera asset dla targetu (np. `x86_64-unknown-linux-gnu`).
pub fn select_asset<'a>(release: &'a GitHubRelease, target: &str) -> ApiResult<Option<&'a Asset>> {
    for a in &release.assets {
        let lname = a.name.to_lowercase();
        if lname.contains(target) && (lname.ends_with(".tar.gz") || lname.ends_with(".tar.zst")) {
            return Ok(Some(a));
        }
    }
    for a in &release.assets {
        let lname = a.name.to_lowercase();
        if lname.contains(target) {
            return Ok(Some(a));
        }
    }
    Ok(None)
}

/// Generuje URL SHA256 — konwencja: `<asset>.sha256`.
pub fn sha_url_for(asset_url: &str) -> String {
    format!("{asset_url}.sha256")
}

/// Map target triple → GitHub release asset target.
pub fn detect_target() -> String {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    match (arch, os) {
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        ("aarch64", "macos") => "aarch64-apple-darwin",
        ("x86_64", "macos") => "x86_64-apple-darwin",
        _ => "unknown",
    }
    .to_string()
}

/// Fetch release info z GitHub API (latest).
/// `Ok(None)` = brak nowszej wersji; `Ok(Some)` = dostępny update.
pub fn check_for_update(current: &Version) -> ApiResult<Option<ReleaseInfo>> {
    let target = detect_target();
    let url = format!("{GITHUB_API}XonofiliusPL/Xerv-Core/releases/latest");
    let resp =
        reqwest::blocking::get(&url).map_err(|e| ApiError::Other(format!("network: {e}")))?;
    if !resp.status().is_success() {
        return Ok(None);
    }
    let gh: GitHubRelease = resp
        .json()
        .map_err(|e| ApiError::Other(format!("parse: {e}")))?;
    if gh.draft {
        return Ok(None);
    }
    let latest = parse_tag(&gh.tag_name)?;
    match cmp_versions(current, &latest) {
        CmpResult::Older | CmpResult::Same => Ok(None),
        CmpResult::Newer => {
            let asset = select_asset(&gh, &target)?
                .ok_or_else(|| ApiError::Other(format!("no asset for target '{target}'")))?;
            Ok(Some(ReleaseInfo {
                version: latest,
                is_prerelease: gh.prerelease,
                asset_url: asset.browser_download_url.clone(),
                sha256_url: sha_url_for(&asset.browser_download_url),
            }))
        }
    }
}

/// Bezpieczna instalacja: pobierz → weryfikuj SHA → zastąp binarkę.
/// Stara binarka → backup, rollback na błąd.
pub fn download_and_install(rel: &ReleaseInfo, bin_path: &Path) -> ApiResult<()> {
    let parent = bin_path.parent().unwrap_or(Path::new("."));
    let backup = parent.join(format!(
        "{}.bak",
        bin_path.file_name().unwrap().to_string_lossy()
    ));
    let tmp = std::env::temp_dir().join(format!("xerv-update-{}", rel.version));
    fs::create_dir_all(&tmp).map_err(|e| ApiError::Other(format!("mkdir: {e}")))?;
    let archive = tmp.join("xerv.tar.gz");
    let sha_file = tmp.join("xerv.tar.gz.sha256");

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| ApiError::Other(format!("client: {e}")))?;
    let download = || -> ApiResult<()> {
        let resp = client
            .get(&rel.asset_url)
            .send()
            .map_err(|e| ApiError::Other(format!("http: {e}")))?;
        if !resp.status().is_success() {
            return Err(ApiError::Other(format!(
                "download failed: HTTP {}",
                resp.status()
            )));
        }
        let mut file =
            fs::File::create(&archive).map_err(|e| ApiError::Other(format!("create: {e}")))?;
        let bytes = resp
            .bytes()
            .map_err(|e| ApiError::Other(format!("bytes: {e}")))?;
        file.write_all(&bytes)
            .map_err(|e| ApiError::Other(format!("write: {e}")))?;
        let sha_resp = client
            .get(&rel.sha256_url)
            .send()
            .map_err(|e| ApiError::Other(format!("sha http: {e}")))?;
        let sha_text = sha_resp
            .text()
            .map_err(|e| ApiError::Other(format!("sha text: {e}")))?;
        let mut sha_f =
            fs::File::create(&sha_file).map_err(|e| ApiError::Other(format!("sha create: {e}")))?;
        sha_f
            .write_all(sha_text.as_bytes())
            .map_err(|e| ApiError::Other(format!("sha write: {e}")))?;
        Ok(())
    };
    if let Err(e) = download() {
        let _ = fs::remove_dir_all(&tmp);
        return Err(ApiError::Other(format!("fetch: {e}")));
    }

    let archive_name = archive.file_name().unwrap().to_string_lossy().to_string();
    let expected = fs::read_to_string(&sha_file)
        .map_err(|e| ApiError::Other(format!("read sha: {e}")))?
        .lines()
        .find_map(|l| {
            let parts: Vec<&str> = l.split_whitespace().collect();
            if (parts.len() >= 2 && parts[1] == archive_name)
                || parts.first().is_some_and(|p| p.len() == 64)
            {
                Some(parts[0].to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| ApiError::Other("checksum not found in sha file".into()))?;
    let ok = verify_sha256(&archive, &expected)?;
    if !ok {
        let _ = fs::remove_dir_all(&tmp);
        return Err(ApiError::Other("checksum verification failed".into()));
    }

    let bin_name = rel.asset_url.split('/').next_back().unwrap_or("xerv");
    let extracted = tmp.join(bin_name);
    let status = Command::new("tar")
        .arg("xzf")
        .arg(&archive)
        .arg("-C")
        .arg(&tmp)
        .status()
        .map_err(|e| ApiError::Other(format!("tar spawn: {e}")))?;
    if !status.success() {
        let _ = fs::remove_dir_all(&tmp);
        return Err(ApiError::Other("tar extraction failed".into()));
    }
    if !extracted.exists() {
        let _ = fs::remove_dir_all(&tmp);
        return Err(ApiError::Other("binary not found in archive".into()));
    }

    if bin_path.exists() {
        fs::copy(bin_path, &backup).map_err(|e| ApiError::Other(format!("backup: {e}")))?;
    }
    let result = fs::rename(&extracted, bin_path);
    if let Err(e) = result {
        let _ = fs::remove_dir_all(&tmp);
        if backup.exists() {
            let _ = fs::rename(&backup, bin_path);
        }
        return Err(ApiError::Other(format!("install failed: {e}, rolled back")));
    }
    let _ = fs::remove_file(&backup);
    let _ = fs::remove_dir_all(&tmp);

    Ok(())
}

/// Weryfikuje SHA256. `ok` = match.
fn verify_sha256(path: &Path, expected: &str) -> ApiResult<bool> {
    let content = fs::read(path).map_err(|e| ApiError::Other(format!("read: {e}")))?;
    let actual = sha256_bytes(&content);
    Ok(actual.eq_ignore_ascii_case(expected))
}

/// SHA256 — używa `sha2` crate (dependency).
fn sha256_bytes(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(data);
    let result = h.finalize();
    result.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    #[test]
    fn cmp_versions_newer() {
        assert_eq!(cmp_versions(&v("0.1.0"), &v("0.2.0")), CmpResult::Newer);
    }

    #[test]
    fn cmp_versions_same() {
        assert_eq!(cmp_versions(&v("0.1.0"), &v("0.1.0")), CmpResult::Same);
    }

    #[test]
    fn cmp_versions_older() {
        assert_eq!(cmp_versions(&v("0.2.0"), &v("0.1.0")), CmpResult::Older);
    }

    #[test]
    fn cmp_versions_prerelease_is_older_than_release() {
        // 0.2.0-alpha < 0.2.0 (SemVer: pre-release < release)
        assert_eq!(
            cmp_versions(&v("0.2.0"), &v("0.2.0-alpha")),
            CmpResult::Older
        );
    }

    #[test]
    fn cmp_versions_current_release_beats_prerelease() {
        // 0.2.0 > 0.2.0-alpha → brak update
        assert!(!matches!(
            cmp_versions(&v("0.2.0"), &v("0.2.0-alpha")),
            CmpResult::Newer
        ));
    }

    #[test]
    fn parse_tag_strips_v_prefix() {
        assert_eq!(parse_tag("v0.2.0").unwrap(), v("0.2.0"));
        assert_eq!(parse_tag("0.2.0").unwrap(), v("0.2.0"));
    }

    #[test]
    fn parse_tag_invalid_returns_error() {
        assert!(parse_tag("not-a-version").is_err());
    }

    #[test]
    fn parse_tag_prerelease_ok() {
        let parsed = parse_tag("v0.2.0-rc.1").unwrap();
        assert!(
            !parsed.pre.is_empty(),
            "pre-release tag should have pre segment"
        );
    }

    #[test]
    fn select_asset_prefers_targz_over_generic() {
        let rel = GitHubRelease {
            tag_name: "v0.2.0".into(),
            prerelease: false,
            draft: false,
            assets: vec![
                Asset {
                    name: "xerv-linux-x86_64".into(),
                    browser_download_url: "http://x".into(),
                    size: 0,
                },
                Asset {
                    name: "xerv-x86_64-unknown-linux-gnu.tar.gz".into(),
                    browser_download_url: "http://tar".into(),
                    size: 0,
                },
            ],
        };
        let picked = select_asset(&rel, "x86_64-unknown-linux-gnu")
            .unwrap()
            .unwrap();
        assert_eq!(picked.name, "xerv-x86_64-unknown-linux-gnu.tar.gz");
    }

    #[test]
    fn select_asset_no_match_returns_none() {
        let rel = GitHubRelease {
            tag_name: "v0.2.0".into(),
            prerelease: false,
            draft: false,
            assets: vec![Asset {
                name: "other".into(),
                browser_download_url: "http://x".into(),
                size: 0,
            }],
        };
        assert!(select_asset(&rel, "x86_64-unknown-linux-gnu")
            .unwrap()
            .is_none());
    }

    #[test]
    fn sha_url_for_appends_sha256() {
        assert_eq!(
            sha_url_for("https://x.com/a.tar.gz"),
            "https://x.com/a.tar.gz.sha256"
        );
    }

    #[test]
    fn sha256_bytes_known_value() {
        // SHA256 of "abc" de facto: ba7816bf8f01cfea...
        let h = sha256_bytes(b"abc");
        assert_eq!(
            h,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn verify_sha256_accepts_correct() {
        use std::io::Write;
        let dir = std::env::temp_dir().join("xerv_update_test_ok");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("xerv");
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(b"abc").unwrap();
        let expected = sha256_bytes(b"abc");
        assert!(verify_sha256(&p, &expected).unwrap());
        // Case-insensitive
        assert!(verify_sha256(&p, &expected.to_uppercase()).unwrap());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_sha256_rejects_mismatch() {
        use std::io::Write;
        let dir = std::env::temp_dir().join("xerv_update_test_bad");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("xerv");
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(b"abc").unwrap();
        assert!(!verify_sha256(
            &p,
            "0000000000000000000000000000000000000000000000000000000000000000"
        )
        .unwrap());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_sha256_missing_file_errors() {
        assert!(verify_sha256(std::path::Path::new("/nonexistent/xerv"), "abc").is_err());
    }

    #[test]
    fn detect_target_not_empty() {
        let t = detect_target();
        assert!(!t.is_empty());
    }
}
