#!/usr/bin/env python3
"""One-shot patch: make Rust update and remote assets checksum-required."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, text: str) -> None:
    (ROOT / path).write_text(text, encoding="utf-8")


def replace_once(path: str, old: str, new: str, label: str) -> None:
    text = read(path)
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one match in {path}, found {count}")
    write(path, text.replace(old, new, 1))


def patch_checksum() -> None:
    replace_once(
        "src/checksum.rs",
        '''pub(crate) fn verify_sha256(path: &Path, expected: &str) -> io::Result<()> {
    let expected = expected.trim().to_ascii_lowercase();
    if expected.len() != 64 || !expected.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected sha256 must be 64 hexadecimal characters",
        ));
    }

    let actual = file_sha256(path)?;
''',
        '''pub(crate) fn normalize_sha256(expected: &str) -> io::Result<String> {
    let expected = expected.trim().to_ascii_lowercase();
    if expected.len() != 64 || !expected.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected sha256 must be 64 hexadecimal characters",
        ));
    }
    Ok(expected)
}

pub(crate) fn verify_sha256(path: &Path, expected: &str) -> io::Result<()> {
    let expected = normalize_sha256(expected)?;
    let actual = file_sha256(path)?;
''',
        "centralize checksum syntax validation",
    )
    replace_once(
        "src/checksum.rs",
        '''    #[test]
    fn verifies_matching_sha256() {
''',
        '''    #[test]
    fn normalizes_valid_sha256_and_rejects_invalid_values() {
        assert_eq!(
            super::normalize_sha256(
                "  78193EF266C1E3C2CE4EA2A86D7FC87E8C52799653FAAAC8536533A1C9300F82  "
            )
            .unwrap(),
            "78193ef266c1e3c2ce4ea2a86d7fc87e8c52799653faaac8536533a1c9300f82"
        );
        assert!(super::normalize_sha256("").is_err());
        assert!(super::normalize_sha256("abc").is_err());
        assert!(super::normalize_sha256(&"g".repeat(64)).is_err());
    }

    #[test]
    fn verifies_matching_sha256() {
''',
        "test checksum normalization",
    )


def patch_update() -> None:
    replace_once(
        "src/update.rs",
        '''#[derive(Debug, Clone)]
struct AssetRef {
    url: String,
    sha256: Option<String>,
}

impl<'de> Deserialize<'de> for AssetRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::String(url) if !url.trim().is_empty() => Ok(Self {
                url: url.trim().to_string(),
                sha256: None,
            }),
            serde_json::Value::Object(mut object) => {
                let url = object
                    .remove("url")
                    .and_then(|value| value.as_str().map(str::to_string))
                    .ok_or_else(|| serde::de::Error::custom("asset object is missing url"))?;
                let sha256 = object
                    .remove("sha256")
                    .and_then(|value| value.as_str().map(str::to_string));
                if url.trim().is_empty() {
                    return Err(serde::de::Error::custom("asset url must not be empty"));
                }
                Ok(Self {
                    url: url.trim().to_string(),
                    sha256: sha256.filter(|value| !value.trim().is_empty()),
                })
            }
            _ => Err(serde::de::Error::custom(
                "asset must be a URL string or object with url",
            )),
        }
    }
}
''',
        '''#[derive(Debug, Clone)]
struct AssetRef {
    url: String,
    sha256: String,
}

impl<'de> Deserialize<'de> for AssetRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let serde_json::Value::Object(mut object) = value else {
            return Err(serde::de::Error::custom(
                "asset must be an object with url and sha256",
            ));
        };
        let url = object
            .remove("url")
            .and_then(|value| value.as_str().map(str::to_string))
            .ok_or_else(|| serde::de::Error::custom("asset object is missing url"))?;
        let sha256 = object
            .remove("sha256")
            .and_then(|value| value.as_str().map(str::to_string))
            .ok_or_else(|| serde::de::Error::custom("asset object is missing sha256"))?;
        if url.trim().is_empty() {
            return Err(serde::de::Error::custom("asset url must not be empty"));
        }
        let sha256 = crate::checksum::normalize_sha256(&sha256)
            .map_err(|error| serde::de::Error::custom(error.to_string()))?;
        Ok(Self {
            url: url.trim().to_string(),
            sha256,
        })
    }
}

#[cfg(test)]
mod asset_checksum_tests {
    use super::AssetRef;

    #[test]
    fn update_asset_requires_valid_sha256() {
        assert!(serde_json::from_str::<AssetRef>(r#"{"url":"https://example.test/herdr"}"#)
            .is_err());
        assert!(serde_json::from_str::<AssetRef>(
            r#"{"url":"https://example.test/herdr","sha256":"abc"}"#
        )
        .is_err());
        let parsed = serde_json::from_str::<AssetRef>(
            r#"{"url":"https://example.test/herdr","sha256":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"}"#,
        )
        .unwrap();
        assert_eq!(parsed.sha256, "a".repeat(64));
    }

    #[test]
    fn update_asset_rejects_legacy_url_string() {
        assert!(serde_json::from_str::<AssetRef>(r#""https://example.test/herdr""#).is_err());
    }
}
''',
        "make update manifest checksum required",
    )
    replace_once(
        "src/update.rs",
        '''    sha256: Option<String>,
    notes_body: String,
''',
        '''    sha256: String,
    notes_body: String,
''',
        "make release checksum non-optional",
    )
    replace_once(
        "src/update.rs",
        '''    if let Some(expected) = &release.sha256 {
        if let Err(e) = crate::checksum::verify_sha256(&tmp_path, expected) {
            let _ = fs::remove_file(&tmp_path);
            return Err(format!(
                "downloaded update checksum verification failed: {e}"
            ));
        }
        tracing::info!(sha256 = %expected, "downloaded update checksum verified");
    }
''',
        '''    if let Err(e) = crate::checksum::verify_sha256(&tmp_path, &release.sha256) {
        let _ = fs::remove_file(&tmp_path);
        return Err(format!(
            "downloaded update checksum verification failed: {e}"
        ));
    }
    tracing::info!(sha256 = %release.sha256, "downloaded update checksum verified");
''',
        "verify self-update checksum unconditionally",
    )


def patch_remote() -> None:
    replace_once(
        "src/remote/unix.rs",
        "use serde::Deserialize;\n",
        "use serde::{Deserialize, Deserializer};\n",
        "import custom deserializer",
    )
    replace_once(
        "src/remote/unix.rs",
        '''#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RemoteAssetRef {
    Url(String),
    Object { url: String, sha256: Option<String> },
}

impl RemoteAssetRef {
    fn url(&self) -> &str {
        match self {
            Self::Url(url) => url,
            Self::Object { url, .. } => url,
        }
    }

    fn sha256(&self) -> Option<&str> {
        match self {
            Self::Url(_) => None,
            Self::Object { sha256, .. } => {
                sha256.as_deref().filter(|value| !value.trim().is_empty())
            }
        }
    }
}
''',
        '''#[derive(Debug, Clone, PartialEq, Eq)]
struct RemoteAssetRef {
    url: String,
    sha256: String,
}

impl<'de> Deserialize<'de> for RemoteAssetRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let serde_json::Value::Object(mut object) = value else {
            return Err(serde::de::Error::custom(
                "remote asset must be an object with url and sha256",
            ));
        };
        let url = object
            .remove("url")
            .and_then(|value| value.as_str().map(str::to_string))
            .ok_or_else(|| serde::de::Error::custom("remote asset object is missing url"))?;
        let sha256 = object
            .remove("sha256")
            .and_then(|value| value.as_str().map(str::to_string))
            .ok_or_else(|| serde::de::Error::custom("remote asset object is missing sha256"))?;
        if url.trim().is_empty() {
            return Err(serde::de::Error::custom("remote asset url must not be empty"));
        }
        let sha256 = crate::checksum::normalize_sha256(&sha256)
            .map_err(|error| serde::de::Error::custom(error.to_string()))?;
        Ok(Self {
            url: url.trim().to_string(),
            sha256,
        })
    }
}

impl RemoteAssetRef {
    fn url(&self) -> &str {
        &self.url
    }

    fn sha256(&self) -> &str {
        &self.sha256
    }
}

#[cfg(test)]
mod remote_asset_checksum_tests {
    use super::RemoteAssetRef;

    #[test]
    fn remote_asset_requires_valid_sha256() {
        assert!(serde_json::from_str::<RemoteAssetRef>(
            r#"{"url":"https://example.test/herdr"}"#
        )
        .is_err());
        assert!(serde_json::from_str::<RemoteAssetRef>(
            r#"{"url":"https://example.test/herdr","sha256":"abc"}"#
        )
        .is_err());
        let parsed = serde_json::from_str::<RemoteAssetRef>(
            r#"{"url":"https://example.test/herdr","sha256":"BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB"}"#,
        )
        .unwrap();
        assert_eq!(parsed.sha256(), "b".repeat(64));
    }

    #[test]
    fn remote_asset_rejects_legacy_url_string() {
        assert!(serde_json::from_str::<RemoteAssetRef>(r#""https://example.test/herdr""#)
            .is_err());
    }
}
''',
        "make remote manifest checksum required",
    )
    replace_once(
        "src/remote/unix.rs",
        '''struct RemoteReleaseAsset {
    url: String,
    sha256: Option<String>,
}
''',
        '''struct RemoteReleaseAsset {
    url: String,
    sha256: String,
}
''',
        "make remote release checksum non-optional",
    )
    replace_once(
        "src/remote/unix.rs",
        '''    if let Some(expected) = &asset.sha256 {
        if let Err(err) = crate::checksum::verify_sha256(&path, expected) {
            let _ = fs::remove_dir_all(&dir);
            return Err(io::Error::new(
                err.kind(),
                format!("downloaded remote asset checksum verification failed: {err}"),
            ));
        }
    }
''',
        '''    if let Err(err) = crate::checksum::verify_sha256(&path, &asset.sha256) {
        let _ = fs::remove_dir_all(&dir);
        return Err(io::Error::new(
            err.kind(),
            format!("downloaded remote asset checksum verification failed: {err}"),
        ));
    }
''',
        "verify remote checksum unconditionally",
    )
    replace_once(
        "src/remote/unix.rs",
        '''fn remote_asset_info(asset: &RemoteAssetRef) -> RemoteReleaseAsset {
    RemoteReleaseAsset {
        url: asset.url().to_string(),
        sha256: asset.sha256().map(str::to_string),
    }
}
''',
        '''fn remote_asset_info(asset: &RemoteAssetRef) -> RemoteReleaseAsset {
    RemoteReleaseAsset {
        url: asset.url().to_string(),
        sha256: asset.sha256().to_string(),
    }
}
''',
        "carry required remote checksum",
    )


def main() -> None:
    patch_checksum()
    patch_update()
    patch_remote()
    print("required Rust checksum patch applied")


if __name__ == "__main__":
    main()
