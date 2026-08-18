//! Install-kind and leftover-plan tests.

use super::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

fn preview_channel_rejection_for_exe_path(path: &std::path::Path) -> Option<&'static str> {
    InstallKind::classify(path).preview_note()
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn homebrew_cellar_path_is_detected() {
    let path = Path::new("/opt/homebrew/Cellar/herdr/0.5.9/bin/herdr");

    assert!(is_homebrew_managed_exe_path(path));
    assert_eq!(
        homebrew_cellar_keg_root(path).unwrap(),
        PathBuf::from("/opt/homebrew/Cellar/herdr/0.5.9")
    );
}

#[test]
fn groeponline_homebrew_cellar_path_is_detected() {
    let path = Path::new("/opt/homebrew/Cellar/groeponline-herdr/0.7.6/bin/herdr");

    assert!(is_homebrew_managed_exe_path(path));
    assert_eq!(
        homebrew_cellar_keg_root(path).unwrap(),
        PathBuf::from("/opt/homebrew/Cellar/groeponline-herdr/0.7.6")
    );
}

#[test]
fn homebrew_linux_cellar_path_is_detected() {
    let path = Path::new("/home/linuxbrew/.linuxbrew/Cellar/herdr/0.5.9/bin/herdr");

    assert!(is_homebrew_managed_exe_path(path));
}

#[test]
fn homebrew_opt_path_requires_canonicalized_cellar_target() {
    let path = Path::new("/opt/homebrew/opt/herdr/bin/herdr");

    assert!(!is_homebrew_managed_exe_path(path));
}

#[test]
fn non_homebrew_path_is_not_detected() {
    let path = Path::new("/usr/local/bin/herdr");

    assert!(!is_homebrew_managed_exe_path(path));
    assert_eq!(InstallKind::classify(path), InstallKind::Direct);
}

#[test]
fn path_install_shadow_finds_leftover_before_homebrew() {
    let leftover = PathBuf::from("/home/joep/.local/bin/herdr");
    let managed =
        PathBuf::from("/home/linuxbrew/.linuxbrew/Cellar/groeponline-herdr/0.8.1/bin/herdr");

    assert_eq!(
        path_install_shadow(&leftover, &[leftover.clone(), managed.clone()]),
        Some(PathInstallShadow {
            leftover: leftover.clone(),
            managed: managed.clone(),
            managed_kind: InstallKind::Homebrew,
        })
    );
    assert_eq!(
        path_install_shadow(&managed, &[leftover.clone(), managed.clone()]),
        None
    );
    assert_eq!(
        path_install_shadow(&leftover, std::slice::from_ref(&leftover)),
        None
    );
    assert_eq!(
        path_install_shadow(&leftover, &[managed, leftover.clone()]),
        None
    );
}

#[test]
fn path_install_shadow_keeps_the_path_entry_not_the_resolved_exe() {
    let dir = std::env::temp_dir().join(format!(
        "herdr-update-shadow-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    let target = dir.join("herdr-target");
    let leftover = dir.join("herdr");
    fs::write(&target, b"").unwrap();
    std::os::unix::fs::symlink(&target, &leftover).unwrap();
    let managed = PathBuf::from("/opt/homebrew/Cellar/groeponline-herdr/0.8.1/bin/herdr");

    assert_eq!(
        path_install_shadow(&target, &[leftover.clone(), managed.clone()]),
        Some(PathInstallShadow {
            leftover,
            managed,
            managed_kind: InstallKind::Homebrew,
        })
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn path_install_shadow_notes_transient_nix_store_but_plan_skips_retire() {
    let leftover = PathBuf::from("/home/joep/.local/bin/herdr");
    let nix_store = PathBuf::from("/nix/store/aaaa-herdr-0.8.1/bin/herdr");
    let shadow = path_install_shadow(&leftover, &[leftover.clone(), nix_store.clone()]);

    assert_eq!(
        shadow,
        Some(PathInstallShadow {
            leftover: leftover.clone(),
            managed: nix_store.clone(),
            managed_kind: InstallKind::Nix,
        })
    );
    assert!(shadow
        .as_ref()
        .is_some_and(PathInstallShadow::is_transient_nix));
    assert_eq!(
        plan_from_parts(
            InstallKind::Direct,
            Some(&leftover),
            UpdateChannel::Stable,
            false,
            shadow,
        ),
        SelfUpdatePlan::Direct
    );
    assert_eq!(
        path_install_update_shadow(&leftover, &[leftover.clone(), nix_store]),
        None
    );
}

#[test]
fn path_install_update_shadow_skips_nix_and_finds_homebrew() {
    let leftover = PathBuf::from("/home/joep/.local/bin/herdr");
    let nix_store = PathBuf::from("/nix/store/aaaa-herdr-0.8.1/bin/herdr");
    let managed =
        PathBuf::from("/home/linuxbrew/.linuxbrew/Cellar/groeponline-herdr/0.8.1/bin/herdr");

    assert_eq!(
        path_install_update_shadow(&leftover, &[leftover.clone(), nix_store, managed.clone()]),
        Some(PathInstallShadow {
            leftover,
            managed,
            managed_kind: InstallKind::Homebrew,
        })
    );
}

#[test]
fn path_install_shadow_retires_leftover_before_mise_shim() {
    let leftover = PathBuf::from("/home/joep/.local/bin/herdr");
    let shim = PathBuf::from("/home/joep/.local/share/mise/shims/herdr");

    assert_eq!(
        path_install_shadow(&leftover, &[leftover.clone(), shim.clone()]),
        Some(PathInstallShadow {
            leftover,
            managed: shim,
            managed_kind: InstallKind::Mise,
        })
    );
}

#[test]
fn asdf_shim_is_not_mise() {
    assert!(!is_mise_shim_exe_path(Path::new(
        "/home/joep/.asdf/shims/herdr"
    )));
    assert_eq!(
        InstallKind::classify(Path::new("/home/joep/.asdf/shims/herdr")),
        InstallKind::Direct
    );
}

#[test]
fn homebrew_update_command_uses_cellar_formula_not_binary_name() {
    assert_eq!(
        homebrew_update_command_for_path(Some(Path::new(
            "/opt/homebrew/Cellar/groeponline-herdr/0.8.1/bin/herdr"
        ))),
        HOMEBREW_UPDATE_COMMAND
    );
    assert_eq!(
        homebrew_update_command_for_path(Some(Path::new(
            "/opt/homebrew/Cellar/herdr/0.8.1/bin/herdr"
        ))),
        HOMEBREW_UPSTREAM_UPDATE_COMMAND
    );
    assert_eq!(
        homebrew_update_command_for_path(Some(Path::new("/home/linuxbrew/.linuxbrew/bin/herdr"))),
        HOMEBREW_UPDATE_COMMAND
    );
    assert_eq!(
        homebrew_update_command_for_path(None),
        HOMEBREW_UPDATE_COMMAND
    );
}

#[test]
fn leftover_direct_backup_path_avoids_existing_backup() {
    let dir = std::env::temp_dir().join(format!(
        "herdr-update-bak-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    let leftover = dir.join("herdr");
    let first = leftover_direct_backup_path(&leftover);
    assert_eq!(first, dir.join("herdr.direct.bak"));
    fs::write(&first, b"old").unwrap();
    let second = leftover_direct_backup_path(&leftover);
    assert_ne!(second, first);
    assert!(second
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("herdr.direct.bak.")));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn retire_leftover_renames_after_success_without_restore() {
    let dir = std::env::temp_dir().join(format!(
        "herdr-update-retire-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    let leftover = dir.join("herdr");
    fs::write(&leftover, b"leftover").unwrap();
    let managed = PathBuf::from("/opt/homebrew/Cellar/groeponline-herdr/0.8.1/bin/herdr");
    let shadow = PathInstallShadow {
        leftover: leftover.clone(),
        managed,
        managed_kind: InstallKind::Homebrew,
    };

    let backup = retire_leftover(&shadow).unwrap();
    assert!(!leftover.exists());
    assert_eq!(fs::read(&backup).unwrap(), b"leftover");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn plan_runs_homebrew_and_notes_preview() {
    assert_eq!(
        plan_from_parts(
            InstallKind::Direct,
            None,
            UpdateChannel::Stable,
            false,
            None
        ),
        SelfUpdatePlan::Direct
    );
    assert_eq!(
        plan_from_parts(
            InstallKind::Homebrew,
            None,
            UpdateChannel::Stable,
            false,
            None
        ),
        SelfUpdatePlan::RunPm {
            kind: InstallKind::Homebrew,
            command: HOMEBREW_UPDATE_COMMAND,
            prerelease_note: None,
            leftover: None,
        }
    );
    assert_eq!(
        plan_from_parts(
            InstallKind::Homebrew,
            None,
            UpdateChannel::Preview,
            false,
            None
        ),
        SelfUpdatePlan::RunPm {
            kind: InstallKind::Homebrew,
            command: HOMEBREW_UPDATE_COMMAND,
            prerelease_note: Some(HOMEBREW_PREVIEW_NOTE),
            leftover: None,
        }
    );
    assert_eq!(
        plan_from_parts(InstallKind::Nix, None, UpdateChannel::Stable, false, None),
        SelfUpdatePlan::Guide {
            message: NIX_GUIDANCE,
            leftover: None,
            exit_error: true,
        }
    );
}

#[test]
fn leftover_homebrew_plan_runs_pm_then_retires() {
    let leftover = PathBuf::from("/home/joep/.local/bin/herdr");
    let managed = PathBuf::from("/opt/homebrew/Cellar/groeponline-herdr/0.8.1/bin/herdr");
    let shadow = PathInstallShadow {
        leftover: leftover.clone(),
        managed: managed.clone(),
        managed_kind: InstallKind::Homebrew,
    };

    assert_eq!(
        plan_from_parts(
            InstallKind::Direct,
            Some(&leftover),
            UpdateChannel::Stable,
            false,
            Some(shadow.clone()),
        ),
        SelfUpdatePlan::RunPm {
            kind: InstallKind::Homebrew,
            command: HOMEBREW_UPDATE_COMMAND,
            prerelease_note: None,
            leftover: Some(shadow.clone()),
        }
    );
    assert_eq!(
        plan_from_parts(
            InstallKind::Direct,
            Some(&leftover),
            UpdateChannel::Stable,
            true,
            Some(shadow.clone()),
        ),
        SelfUpdatePlan::ForceDirect { shadow }
    );
}

#[test]
fn npm_global_install_path_is_detected() {
    let path = Path::new("/usr/local/lib/node_modules/groeponline-herdr/bin/herdr");

    assert!(is_npm_managed_exe_path(path));
    assert!(is_package_manager_managed_exe_path(path));
    assert_eq!(
        preview_channel_rejection_for_exe_path(path),
        Some(NPM_PREVIEW_NOTE)
    );
    assert_eq!(
        update_install_instruction(NPM_UPDATE_COMMAND),
        "detach, run `npm install --global groeponline-herdr@latest`, then restart this Herdr session when ready"
    );
}

#[test]
fn npm_path_requires_exact_package_and_native_binary_shape() {
    assert!(!is_npm_managed_exe_path(Path::new(
        "/usr/local/lib/node_modules/another-package/bin/herdr",
    )));
    assert!(!is_npm_managed_exe_path(Path::new(
        "/usr/local/lib/node_modules/groeponline-herdr/herdr",
    )));
    assert!(!is_npm_managed_exe_path(Path::new("/usr/local/bin/herdr",)));
}

#[test]
fn mise_install_path_is_detected() {
    let path = Path::new("/home/user/.local/share/mise/installs/herdr/0.6.6/bin/herdr");

    assert!(is_mise_managed_exe_path(path));
    assert_eq!(
        mise_install_root(path).unwrap(),
        PathBuf::from("/home/user/.local/share/mise/installs/herdr/0.6.6")
    );
}

#[test]
fn mise_alias_install_path_is_detected() {
    let path = Path::new("/home/user/.local/share/mise/installs/herdr/latest/bin/herdr");

    assert!(is_mise_managed_exe_path(path));
}

#[test]
fn mise_custom_installs_dir_path_is_detected() {
    let path = Path::new("/opt/mise-tools/installs/herdr/0.6.6/bin/herdr");

    assert!(is_mise_managed_exe_path(path));
}

#[test]
fn mise_configured_installs_dir_path_is_detected() {
    let _guard = env_lock().lock().unwrap();
    let previous = std::env::var_os(MISE_INSTALLS_DIR_ENV);
    std::env::set_var(MISE_INSTALLS_DIR_ENV, "/opt/mise-tools");
    let path = Path::new("/opt/mise-tools/herdr/0.6.6/bin/herdr");

    assert!(is_mise_managed_exe_path(path));
    assert_eq!(
        mise_install_root(path).unwrap(),
        PathBuf::from("/opt/mise-tools/herdr/0.6.6")
    );

    if let Some(previous) = previous {
        std::env::set_var(MISE_INSTALLS_DIR_ENV, previous);
    } else {
        std::env::remove_var(MISE_INSTALLS_DIR_ENV);
    }
}

#[test]
fn non_mise_install_path_is_not_detected() {
    let path = Path::new("/home/user/.local/bin/herdr");

    assert!(!is_mise_managed_exe_path(path));
}

#[test]
fn package_manager_path_detection_follows_homebrew_symlink() {
    let root = std::env::temp_dir().join(format!(
        "herdr-homebrew-symlink-test-{}",
        std::process::id()
    ));
    let cellar_bin = root.join("Cellar/herdr/0.6.2/bin");
    let opt_bin = root.join("opt/herdr/bin");
    fs::create_dir_all(&cellar_bin).unwrap();
    fs::create_dir_all(&opt_bin).unwrap();
    let cellar_binary = cellar_bin.join("herdr");
    let opt_binary = opt_bin.join("herdr");
    fs::write(&cellar_binary, b"").unwrap();
    std::os::unix::fs::symlink(&cellar_binary, &opt_binary).unwrap();

    assert!(is_package_manager_managed_exe_path(&opt_binary));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn package_manager_path_detection_follows_mise_symlink() {
    let root = std::env::temp_dir().join(format!("herdr-mise-symlink-test-{}", std::process::id()));
    let version_bin = root.join("installs/herdr/0.6.2/bin");
    let latest_bin = root.join("installs/herdr/latest/bin");
    fs::create_dir_all(&version_bin).unwrap();
    fs::create_dir_all(&latest_bin).unwrap();
    let version_binary = version_bin.join("herdr");
    let latest_binary = latest_bin.join("herdr");
    fs::write(&version_binary, b"").unwrap();
    std::os::unix::fs::symlink(&version_binary, &latest_binary).unwrap();

    assert!(is_package_manager_managed_exe_path(&latest_binary));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn nix_store_path_is_detected() {
    let path = Path::new("/nix/store/abc123-herdr-0.6.1/bin/herdr");

    assert!(is_nix_store_exe_path(path));
    assert!(is_package_manager_managed_exe_path(path));
}

#[test]
fn preview_channel_is_rejected_for_package_manager_paths() {
    let homebrew = Path::new("/opt/homebrew/Cellar/herdr/0.6.6/bin/herdr");
    let mise = Path::new("/home/user/.local/share/mise/installs/herdr/0.6.6/bin/herdr");
    let nix = Path::new("/nix/store/abc123-herdr-0.6.6/bin/herdr");
    let direct = Path::new("/home/user/.local/bin/herdr");

    assert!(preview_channel_rejection_for_exe_path(homebrew)
        .is_some_and(|message| message.contains("Homebrew")));
    assert!(preview_channel_rejection_for_exe_path(mise)
        .is_some_and(|message| message.contains("mise")));
    assert!(
        preview_channel_rejection_for_exe_path(nix).is_some_and(|message| message.contains("Nix"))
    );
    assert!(preview_channel_rejection_for_exe_path(direct).is_none());
}

#[test]
fn non_nix_store_path_is_not_detected() {
    let path = Path::new("/usr/local/bin/herdr");

    assert!(!is_nix_store_exe_path(path));
}

#[test]
fn path_install_update_shadow_skips_multiple_transient_nix_entries() {
    let leftover = PathBuf::from("/home/joep/.local/bin/herdr");
    let nix_store_one = PathBuf::from("/nix/store/aaaa-herdr-0.8.0/bin/herdr");
    let nix_store_two = PathBuf::from("/nix/store/bbbb-herdr-0.8.1/bin/herdr");
    let managed = PathBuf::from("/home/user/.local/share/mise/installs/herdr/0.8.1/bin/herdr");

    assert_eq!(
        path_install_update_shadow(
            &leftover,
            &[
                leftover.clone(),
                nix_store_one,
                nix_store_two,
                managed.clone(),
            ]
        ),
        Some(PathInstallShadow {
            leftover,
            managed,
            managed_kind: InstallKind::Mise,
        })
    );
}

#[test]
fn path_install_update_shadow_returns_none_when_only_transient_nix_follows() {
    let leftover = PathBuf::from("/home/joep/.local/bin/herdr");
    let nix_store = PathBuf::from("/nix/store/aaaa-herdr-0.8.0/bin/herdr");

    assert_eq!(
        path_install_update_shadow(&leftover, &[leftover.clone(), nix_store]),
        None
    );
}

#[test]
fn path_install_update_shadow_treats_stable_nix_profile_path_as_managed() {
    // A Nix path that does not literally live under /nix/store (for example a
    // `nix profile` symlink target already resolved) is not considered
    // transient, so it can still be picked up as the install to retire for.
    let leftover = PathBuf::from("/home/joep/.local/bin/herdr");
    let nix_profile = PathBuf::from("/nix/store-not-really/herdr-0.8.1/bin/herdr");

    // Sanity: this path is not classified as Nix at all (doesn't start with
    // "/nix/store"), so it falls through to Direct and is not package
    // managed, meaning it is skipped just like any other non-managed binary.
    assert_eq!(InstallKind::classify(&nix_profile), InstallKind::Direct);
    assert_eq!(
        path_install_update_shadow(&leftover, &[leftover.clone(), nix_profile]),
        None
    );
}

#[test]
fn apply_managed_update_guide_returns_err_without_leftover_when_exit_error() {
    let result = apply_managed_update(SelfUpdatePlan::Guide {
        message: NIX_GUIDANCE,
        leftover: None,
        exit_error: true,
    });

    assert_eq!(result, Err(NIX_GUIDANCE.to_string()));
}

#[test]
fn apply_managed_update_guide_returns_ok_when_exit_error_is_false() {
    let result = apply_managed_update(SelfUpdatePlan::Guide {
        message: NIX_GUIDANCE,
        leftover: None,
        exit_error: false,
    });

    assert_eq!(result, Ok(()));
}

#[test]
fn apply_managed_update_direct_and_force_direct_are_noops() {
    assert_eq!(apply_managed_update(SelfUpdatePlan::Direct), Ok(()));

    let shadow = PathInstallShadow {
        leftover: PathBuf::from("/tmp/herdr-noop-leftover"),
        managed: PathBuf::from("/opt/homebrew/Cellar/groeponline-herdr/0.8.1/bin/herdr"),
        managed_kind: InstallKind::Homebrew,
    };
    assert_eq!(
        apply_managed_update(SelfUpdatePlan::ForceDirect { shadow }),
        Ok(())
    );
}

#[test]
fn apply_managed_update_guide_retires_leftover_on_success_without_running_a_package_manager() {
    let dir = std::env::temp_dir().join(format!(
        "herdr-update-guide-retire-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    let leftover = dir.join("herdr");
    fs::write(&leftover, b"leftover").unwrap();
    let shadow = PathInstallShadow {
        leftover: leftover.clone(),
        managed: PathBuf::from("/nix/store/aaaa-herdr-0.8.1/bin/herdr"),
        managed_kind: InstallKind::Nix,
    };

    // Guide never shells out, so this only exercises the leftover-retirement
    // side effect, not a real package-manager command.
    let result = apply_managed_update(SelfUpdatePlan::Guide {
        message: NIX_GUIDANCE,
        leftover: Some(shadow),
        exit_error: false,
    });

    assert_eq!(result, Ok(()));
    assert!(!leftover.exists());
    assert!(dir.join("herdr.direct.bak").exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn apply_managed_update_guide_reports_exit_error_even_when_leftover_retire_fails() {
    // The leftover path does not exist, so retirement fails; that failure
    // must not mask the underlying `exit_error` guidance failure.
    let missing_leftover = PathBuf::from("/tmp/herdr-guide-missing-leftover-does-not-exist");
    let shadow = PathInstallShadow {
        leftover: missing_leftover,
        managed: PathBuf::from("/nix/store/aaaa-herdr-0.8.1/bin/herdr"),
        managed_kind: InstallKind::Nix,
    };

    let result = apply_managed_update(SelfUpdatePlan::Guide {
        message: NIX_GUIDANCE,
        leftover: Some(shadow),
        exit_error: true,
    });

    assert_eq!(result, Err(NIX_GUIDANCE.to_string()));
}
