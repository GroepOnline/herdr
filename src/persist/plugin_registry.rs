use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

use tracing::warn;

use crate::api::schema::InstalledPluginInfo;

pub const MANIFEST_UNAVAILABLE_WARNING_PREFIX: &str = "manifest unavailable: ";
const REGISTRY_LOCK_FILE: &str = ".plugins.lock";

fn registry_path() -> PathBuf {
    crate::config::config_dir().join("plugins.json")
}

fn registry_lock_path() -> PathBuf {
    crate::config::config_dir().join(REGISTRY_LOCK_FILE)
}

fn with_registry_lock<T>(operation: impl FnOnce() -> std::io::Result<T>) -> std::io::Result<T> {
    let lock_path = registry_lock_path();
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(lock_path)?;
    lock.lock()?;
    operation()
}

fn save_json_to_path<T: serde::Serialize + ?Sized>(path: &Path, value: &T) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(value)?;
    let tmp_path = path.with_extension("json.tmp");
    std::fs::write(&tmp_path, json)?;
    #[cfg(windows)]
    if path.exists() {
        if let Err(err) = std::fs::remove_file(path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(err);
        }
    }
    if let Err(err) = std::fs::rename(&tmp_path, path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(err);
    }
    Ok(())
}

pub fn save_to_path(path: &Path, plugins: &[InstalledPluginInfo]) -> std::io::Result<()> {
    save_json_to_path(path, plugins)
}

pub fn update<T>(
    mutation: impl FnOnce(&mut Vec<InstalledPluginInfo>) -> T,
) -> std::io::Result<(T, Vec<InstalledPluginInfo>)> {
    with_registry_lock(|| {
        let mut plugins = load_from_path_strict(&registry_path())?;
        let result = mutation(&mut plugins);
        plugins.sort_by(|left, right| left.plugin_id.cmp(&right.plugin_id));
        save_to_path(&registry_path(), &plugins)?;
        Ok((result, plugins))
    })
}

pub fn try_load() -> std::io::Result<Vec<InstalledPluginInfo>> {
    with_registry_lock(|| load_from_path_strict(&registry_path()))
}

/// Load the global registry. Returns an empty vec on failure so a corrupt or
/// missing file never blocks server startup; mutations still use strict reads.
pub fn load() -> Vec<InstalledPluginInfo> {
    match try_load() {
        Ok(plugins) => plugins,
        Err(err) => {
            warn!(path = %registry_path().display(), err = %err, "failed to load plugin registry");
            Vec::new()
        }
    }
}

#[cfg(test)]
pub fn load_from_path(path: &Path) -> Vec<InstalledPluginInfo> {
    match load_from_path_strict(path) {
        Ok(entries) => entries,
        Err(err) => {
            warn!(path = %path.display(), err = %err, "failed to read plugin registry");
            Vec::new()
        }
    }
}

fn load_from_path_strict(path: &Path) -> std::io::Result<Vec<InstalledPluginInfo>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str::<Vec<InstalledPluginInfo>>(&content)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
}

/// Re-read each entry's manifest from disk using the provided reload function.
///
/// If the manifest parses successfully, replace cached fields but keep the
/// stored `enabled` flag.  If the file is gone or unparseable, keep the stored
/// entry and append a warning so `plugin.list` surfaces it.
pub fn reload_manifests(
    mut entries: Vec<InstalledPluginInfo>,
    reload_fn: impl Fn(&str, bool) -> Result<InstalledPluginInfo, String>,
) -> Vec<InstalledPluginInfo> {
    for entry in &mut entries {
        entry.warnings.clear();
        match reload_fn(&entry.manifest_path, entry.enabled) {
            Ok(mut fresh) => {
                fresh.enabled = entry.enabled;
                fresh.source = entry.source.clone();
                *entry = fresh;
            }
            Err(warn_msg) => {
                entry
                    .warnings
                    .push(format!("{MANIFEST_UNAVAILABLE_WARNING_PREFIX}{warn_msg}"));
            }
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_registry_path(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir()
            .join(format!(
                "herdr-registry-{name}-{}-{nanos}",
                std::process::id()
            ))
            .join("plugins.json")
    }

    fn sample_plugin(id: &str) -> InstalledPluginInfo {
        InstalledPluginInfo {
            plugin_id: id.to_string(),
            name: "Test Plugin".to_string(),
            version: "0.1.0".to_string(),
            min_herdr_version: crate::build_info::BASE_VERSION.to_string(),
            description: None,
            manifest_path: format!("/tmp/{id}/herdr-plugin.toml"),
            plugin_root: format!("/tmp/{id}"),
            enabled: true,
            platforms: None,
            build: vec![],
            startup: vec![],
            actions: vec![],
            events: vec![],
            panes: vec![],
            link_handlers: vec![],
            source: Default::default(),
            warnings: vec![],
        }
    }

    #[test]
    fn save_and_load_roundtrip() {
        let path = temp_registry_path("roundtrip");
        let plugins = vec![sample_plugin("example.a"), sample_plugin("example.b")];

        save_to_path(&path, &plugins).unwrap();

        let loaded = load_from_path(&path);
        assert_eq!(loaded.len(), 2);
        let ids: Vec<_> = loaded.iter().map(|p| p.plugin_id.as_str()).collect();
        assert!(ids.contains(&"example.a"));
        assert!(ids.contains(&"example.b"));
    }

    #[test]
    fn missing_file_returns_empty() {
        let path = temp_registry_path("missing");
        let loaded = load_from_path(&path);
        assert!(loaded.is_empty());
    }

    #[test]
    fn corrupt_file_returns_empty_without_panic() {
        let path = temp_registry_path("corrupt");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let corrupt = b"this is not valid json {{{{";
        std::fs::write(&path, corrupt).unwrap();

        assert!(load_from_path_strict(&path).is_err());
        assert!(load_from_path(&path).is_empty());
        assert_eq!(std::fs::read(path).unwrap(), corrupt);
    }

    #[test]
    fn reload_manifests_keeps_entry_with_warning_on_missing_manifest() {
        let entry = sample_plugin("example.missing");
        let entries = vec![entry];

        let result = reload_manifests(entries, |path, _enabled| {
            Err(format!("manifest not found at {path}"))
        });

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].plugin_id, "example.missing");
        assert!(!result[0].warnings.is_empty());
        assert!(result[0].warnings[0].contains("manifest not found"));
    }

    #[test]
    fn reload_manifests_uses_fresh_parse_and_keeps_enabled_flag() {
        let mut entry = sample_plugin("example.reload");
        entry.enabled = false;
        entry.source = crate::api::schema::PluginSourceInfo {
            kind: crate::api::schema::PluginSourceKind::Github,
            owner: Some("ogulcancelik".into()),
            repo: Some("herdr-plugin-examples".into()),
            subdir: Some("worktree-bootstrap".into()),
            requested_ref: Some("main".into()),
            resolved_commit: Some("abc123".into()),
            managed_path: Some("/tmp/herdr/plugins/github/example.reload".into()),
            installed_unix_ms: Some(42),
        };

        let result = reload_manifests(vec![entry], |_path, _enabled| {
            Ok(InstalledPluginInfo {
                plugin_id: "example.reload".to_string(),
                name: "Fresh Name".to_string(),
                version: "0.2.0".to_string(),
                min_herdr_version: crate::build_info::BASE_VERSION.to_string(),
                description: Some("refreshed".to_string()),
                manifest_path: "/tmp/example.reload/herdr-plugin.toml".to_string(),
                plugin_root: "/tmp/example.reload".to_string(),
                enabled: true, // caller would pass stored enabled; fresh parse returns true
                platforms: None,
                build: vec![],
                startup: vec![],
                actions: vec![],
                events: vec![],
                panes: vec![],
                link_handlers: vec![],
                source: Default::default(),
                warnings: vec![],
            })
        });

        assert_eq!(result[0].name, "Fresh Name");
        assert_eq!(result[0].version, "0.2.0");
        // enabled preserved from stored entry
        assert!(!result[0].enabled);
        assert_eq!(
            result[0].source.kind,
            crate::api::schema::PluginSourceKind::Github
        );
        assert_eq!(result[0].source.owner.as_deref(), Some("ogulcancelik"));
        assert!(result[0].warnings.is_empty());
    }

    #[test]
    fn save_replaces_existing_registry_file() {
        let path = temp_registry_path("replace-existing");
        save_to_path(&path, &[sample_plugin("example.first")]).unwrap();
        save_to_path(&path, &[sample_plugin("example.second")]).unwrap();

        let loaded = load_from_path(&path);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].plugin_id, "example.second");
    }

    fn with_temp_config_dir(name: &str, test: impl FnOnce(PathBuf)) {
        let _guard = crate::config::test_config_env_lock().lock().unwrap();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let config_dir = std::env::temp_dir().join(format!(
            "herdr-registry-config-{name}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&config_dir).unwrap();

        let old_xdg = std::env::var_os("XDG_CONFIG_HOME");
        std::env::set_var("XDG_CONFIG_HOME", &config_dir);

        let expected_registry_path = config_dir
            .join(crate::config::app_dir_name())
            .join("plugins.json");

        // Ensure its parent exists
        std::fs::create_dir_all(expected_registry_path.parent().unwrap()).unwrap();

        struct EnvGuard {
            old_xdg: Option<std::ffi::OsString>,
            temp_dir: PathBuf,
        }

        impl Drop for EnvGuard {
            fn drop(&mut self) {
                if let Some(ref old) = self.old_xdg {
                    std::env::set_var("XDG_CONFIG_HOME", old);
                } else {
                    std::env::remove_var("XDG_CONFIG_HOME");
                }
                let _ = std::fs::remove_dir_all(&self.temp_dir);
            }
        }

        let _guard_env = EnvGuard {
            old_xdg,
            temp_dir: config_dir,
        };

        test(expected_registry_path);
    }

    #[test]
    fn try_load_returns_ok_on_missing_file() {
        with_temp_config_dir("try_load_missing", |_path| {
            let loaded = try_load().unwrap();
            assert!(loaded.is_empty());
        });
    }

    #[test]
    fn try_load_returns_error_on_corrupt_file() {
        with_temp_config_dir("try_load_corrupt", |path| {
            std::fs::write(&path, b"not valid json").unwrap();
            assert!(try_load().is_err());
        });
    }

    #[test]
    fn try_load_returns_plugins_on_success() {
        with_temp_config_dir("try_load_success", |path| {
            let plugins = vec![sample_plugin("example.tryload")];
            save_to_path(&path, &plugins).unwrap();

            let loaded = try_load().unwrap();
            assert_eq!(loaded.len(), 1);
            assert_eq!(loaded[0].plugin_id, "example.tryload");
        });
    }

    #[test]
    fn load_returns_empty_on_missing_file() {
        with_temp_config_dir("load_missing", |_path| {
            let loaded = load();
            assert!(loaded.is_empty());
        });
    }

    #[test]
    fn load_returns_empty_on_corrupt_file() {
        with_temp_config_dir("load_corrupt", |path| {
            std::fs::write(&path, b"not valid json").unwrap();
            let loaded = load();
            assert!(loaded.is_empty());
        });
    }

    #[test]
    fn load_returns_plugins_on_success() {
        with_temp_config_dir("load_success", |path| {
            let plugins = vec![sample_plugin("example.load")];
            save_to_path(&path, &plugins).unwrap();

            let loaded = load();
            assert_eq!(loaded.len(), 1);
            assert_eq!(loaded[0].plugin_id, "example.load");
        });
    }
}
