use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const PENDING_RELEASE_NOTES_PATH: &str = "release-notes.json";
const BUNDLED_CHANGELOG: &str = include_str!("../CHANGELOG.md");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseNotes {
    pub version: String,
    pub body: String,
    pub preview: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredReleaseNotes {
    version: String,
    body: String,
    #[serde(default = "default_show_on_startup")]
    show_on_startup: bool,
}

fn default_show_on_startup() -> bool {
    true
}

pub fn pending_path() -> PathBuf {
    let mut path = crate::config::config_path();
    path.set_file_name(PENDING_RELEASE_NOTES_PATH);
    path
}

pub fn save_pending(version: &str, body: &str) -> std::io::Result<()> {
    save_pending_to_path(&pending_path(), version, body)
}

fn save_pending_to_path(path: &Path, version: &str, body: &str) -> std::io::Result<()> {
    let body = normalize_body(body);
    if body.is_empty() {
        return clear_pending_at(path);
    }

    write_stored_to_path(
        path,
        &StoredReleaseNotes {
            version: version.to_string(),
            body,
            show_on_startup: true,
        },
    )
}

fn write_stored_to_path(path: &Path, stored: &StoredReleaseNotes) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(stored).map_err(std::io::Error::other)?;
    let tmp_path = path.with_extension(format!("json.tmp.{}", std::process::id()));
    fs::write(&tmp_path, json)?;
    if let Err(err) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(err);
    }
    Ok(())
}

fn load_stored_from_path(path: &Path) -> Option<StoredReleaseNotes> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn load_latest() -> Option<ReleaseNotes> {
    load_latest_from_path(
        &pending_path(),
        &crate::build_info::version(),
        BUNDLED_CHANGELOG,
    )
}

fn load_latest_from_path(
    path: &Path,
    current_version: &str,
    bundled_changelog: &str,
) -> Option<ReleaseNotes> {
    latest_release_notes(
        load_stored_from_path(path),
        current_version,
        bundled_changelog,
    )
}

fn latest_release_notes(
    stored: Option<StoredReleaseNotes>,
    current_version: &str,
    bundled_changelog: &str,
) -> Option<ReleaseNotes> {
    let stored = stored.and_then(|stored| release_notes_from_stored(stored, current_version));
    let matching_prerelease = current_version.contains('-')
        && stored
            .as_ref()
            .is_some_and(|notes| notes.version == current_version);
    if stored.as_ref().is_some_and(|notes| notes.preview) || matching_prerelease {
        return stored;
    }

    bundled_release_notes(current_version, bundled_changelog).or(stored)
}

fn bundled_release_notes(current_version: &str, changelog: &str) -> Option<ReleaseNotes> {
    let base_version = current_version
        .split_once('-')
        .map_or(current_version, |(base, _)| base);
    let version = crate::update::Version::parse(base_version)?.to_string();
    let body = extract_version_section(changelog, &version)?;
    Some(ReleaseNotes {
        version,
        body: normalize_body(&body),
        preview: false,
    })
}

fn release_notes_from_stored(
    stored: StoredReleaseNotes,
    current_version: &str,
) -> Option<ReleaseNotes> {
    let body = normalize_body(&stored.body);
    if body.is_empty() {
        return None;
    }

    let preview = match (
        crate::update::Version::parse(&stored.version),
        crate::update::Version::parse(current_version),
    ) {
        (Some(stored_version), Some(current_version)) => stored_version > current_version,
        _ => false,
    };

    Some(ReleaseNotes {
        preview,
        version: stored.version,
        body,
    })
}

pub fn mark_current_version_seen() -> std::io::Result<()> {
    mark_current_version_seen_at(&pending_path(), &crate::build_info::version())
}

fn mark_current_version_seen_at(path: &Path, current_version: &str) -> std::io::Result<()> {
    let Some(mut stored) = load_stored_from_path(path) else {
        return Ok(());
    };
    if stored.version != current_version || !stored.show_on_startup {
        return Ok(());
    }
    stored.show_on_startup = false;
    write_stored_to_path(path, &stored)
}

fn clear_pending_at(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        fs::remove_file(path)
    } else {
        Ok(())
    }
}

pub fn load_preview_from_local_changelog(version: &str) -> Option<ReleaseNotes> {
    let path = Path::new("CHANGELOG.md");
    let content = fs::read_to_string(path).ok()?;
    let body = extract_version_section(&content, version)?;
    Some(ReleaseNotes {
        version: version.to_string(),
        body: normalize_body(&body),
        preview: true,
    })
}

fn extract_version_section(content: &str, version: &str) -> Option<String> {
    let header = format!("## [{version}]");
    let mut collecting = false;
    let mut lines = Vec::new();

    for line in content.lines() {
        if !collecting {
            if line.starts_with(&header) {
                collecting = true;
            }
            continue;
        }

        if line.starts_with("## [") {
            break;
        }

        lines.push(line);
    }

    let body = lines.join("\n").trim().to_string();
    (!body.is_empty()).then_some(body)
}

pub fn normalize_body(body: &str) -> String {
    body.lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_version_section() {
        let changelog = "# Changelog\n\n## [0.2.3] - 2026-03-31\n\n### Changed\n- One\n\n## [0.2.2] - 2026-03-30\n\n### Fixed\n- Two\n";
        assert_eq!(
            extract_version_section(changelog, "0.2.3").as_deref(),
            Some("### Changed\n- One")
        );
    }

    #[test]
    fn preserves_headings() {
        assert_eq!(
            normalize_body("### Changed\n- One\n\n### Fixed\n- Two"),
            "### Changed\n- One\n\n### Fixed\n- Two"
        );
    }

    #[test]
    fn load_latest_keeps_future_version_previewable_before_restart() {
        let path = std::env::temp_dir().join(format!(
            "herdr-release-notes-{}-{}.json",
            std::process::id(),
            "preview"
        ));
        let _ = clear_pending_at(&path);
        save_pending_to_path(&path, "0.3.2", "### Changed\n- One").unwrap();

        let notes = load_latest_from_path(&path, "0.3.1", "").expect("latest notes");
        assert_eq!(notes.version, "0.3.2");
        assert_eq!(notes.body, "### Changed\n- One");
        assert!(notes.preview);

        clear_pending_at(&path).unwrap();
    }

    #[test]
    fn upgrading_from_0_8_1_to_0_8_6_replaces_stale_notes_with_bundled_release() {
        let path = std::env::temp_dir().join(format!(
            "herdr-release-notes-{}-{}.json",
            std::process::id(),
            "stale"
        ));
        let _ = clear_pending_at(&path);
        save_pending_to_path(&path, "0.8.1", "### Changed\n- Old notes").unwrap();

        let notes = load_latest_from_path(
            &path,
            "0.8.6",
            "# Changelog\n\n## [0.8.6] - 2026-08-19\n\n### Fixed\n- Current notes\n\n## [0.8.1] - 2026-08-15\n\n### Changed\n- Old notes\n",
        )
        .expect("latest notes");
        assert_eq!(notes.version, "0.8.6");
        assert_eq!(notes.body, "### Fixed\n- Current notes");
        assert!(!notes.preview);

        clear_pending_at(&path).unwrap();
    }

    #[test]
    fn marking_current_version_seen_preserves_latest_notes() {
        let path = std::env::temp_dir().join(format!(
            "herdr-release-notes-{}-{}.json",
            std::process::id(),
            "seen"
        ));
        let _ = clear_pending_at(&path);
        save_pending_to_path(&path, "0.3.1", "### Changed\n- One").unwrap();

        mark_current_version_seen_at(&path, "0.3.1").unwrap();

        let stored = load_stored_from_path(&path).expect("stored notes");
        assert!(!stored.show_on_startup);
        let latest = load_latest_from_path(&path, "0.3.1", "").expect("latest notes");
        assert_eq!(latest.version, "0.3.1");
        assert!(!latest.preview);

        clear_pending_at(&path).unwrap();
    }

    #[test]
    fn legacy_notes_without_show_on_startup_remain_available_as_latest() {
        let path = std::env::temp_dir().join(format!(
            "herdr-release-notes-{}-{}.json",
            std::process::id(),
            "legacy"
        ));
        let _ = clear_pending_at(&path);
        fs::write(
            &path,
            "{\n  \"version\": \"0.3.1\",\n  \"body\": \"### Changed\\n- One\"\n}",
        )
        .unwrap();

        assert!(load_latest_from_path(&path, "0.3.1", "").is_some());

        clear_pending_at(&path).unwrap();
    }

    #[test]
    fn fresh_install_uses_bundled_notes_for_the_package_version() {
        let notes = latest_release_notes(
            None,
            "0.8.6",
            "# Changelog\n\n## [0.8.6]\n\n### Fixed\n- Current notes\n",
        )
        .expect("bundled notes");

        assert_eq!(notes.version, "0.8.6");
        assert_eq!(notes.body, "### Fixed\n- Current notes");
        assert!(!notes.preview);
    }

    #[test]
    fn bundled_changelog_contains_notes_for_cargo_package_version() {
        let notes = bundled_release_notes(crate::build_info::BASE_VERSION, BUNDLED_CHANGELOG)
            .expect("Cargo package version must have bundled release notes");

        assert_eq!(notes.version, crate::build_info::BASE_VERSION);
        assert!(!notes.body.is_empty());
        assert!(!notes.preview);
    }

    #[test]
    fn current_prerelease_uses_matching_persisted_channel_notes() {
        let current_version = "0.8.6-dev.2026-08-22-ed0e658b";
        let notes = latest_release_notes(
            Some(StoredReleaseNotes {
                version: current_version.to_string(),
                body: "### Changed\n- Current dev build".to_string(),
                show_on_startup: false,
            }),
            current_version,
            "# Changelog\n\n## [0.8.6]\n\n### Fixed\n- Stable notes\n",
        )
        .expect("current dev notes");

        assert_eq!(notes.version, current_version);
        assert_eq!(notes.body, "### Changed\n- Current dev build");
        assert!(!notes.preview);
    }

    #[test]
    fn already_seen_current_release_uses_canonical_bundled_notes() {
        let notes = latest_release_notes(
            Some(StoredReleaseNotes {
                version: "0.8.6".to_string(),
                body: "### Fixed\n- Non-canonical cached notes".to_string(),
                show_on_startup: false,
            }),
            "0.8.6",
            "# Changelog\n\n## [0.8.6]\n\n### Fixed\n- Canonical notes\n",
        )
        .expect("bundled notes");

        assert_eq!(notes.version, "0.8.6");
        assert_eq!(notes.body, "### Fixed\n- Canonical notes");
        assert!(!notes.preview);
    }
}
