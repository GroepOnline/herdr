//! Install identity, PATH leftover detection, and package-manager update plans.
//!
//! One [`InstallKind`] answers "what is this binary?" and "what should update do?".
//! Leftover retirement happens only after a package-manager command succeeds.

use std::env;
#[cfg(unix)]
use std::fs;
use std::path::{Path, PathBuf};

use super::{
    UpdateChannel, HERDR_UPDATE_COMMAND, HOMEBREW_UPDATE_COMMAND, HOMEBREW_UPSTREAM_UPDATE_COMMAND,
    MISE_INSTALLS_DIR_ENV, MISE_UPDATE_COMMAND, NIX_UPDATE_COMMAND, NPM_UPDATE_COMMAND,
};

const NIX_GUIDANCE: &str =
    "update Nix-managed Herdr with `nix profile upgrade` or the flake input that provides Herdr";
const NIX_GUIDANCE_PRERELEASE: &str = "preview and dev channels are only available for direct Herdr installs; update Nix-managed Herdr with `nix profile upgrade` or the flake input that provides Herdr";
const HOMEBREW_PREVIEW_NOTE: &str = "preview and dev channels are only available for direct Herdr installs; Homebrew installs stay on stable";
const NPM_PREVIEW_NOTE: &str = "preview and dev channels are only available for direct Herdr installs; npm installs stay on stable";
const MISE_PREVIEW_NOTE: &str = "preview and dev channels are only available for direct Herdr installs; mise installs stay on stable";
const NIX_PREVIEW_NOTE: &str = "preview and dev channels are only available for direct Herdr installs; Nix installs stay on stable";
const HOMEBREW_CHANNEL_GUIDANCE: &str = "Use `brew update && brew upgrade GroepOnline/tap/groeponline-herdr` (or `brew upgrade herdr` if you installed the `herdr` formula alias) to update Homebrew installs.";
const HOMEBREW_UPSTREAM_CHANNEL_GUIDANCE: &str =
    "Use `brew update && brew upgrade herdr` to update Homebrew installs.";
const NPM_CHANNEL_GUIDANCE: &str =
    "Use `npm install --global groeponline-herdr@latest` to update npm installs.";
const MISE_CHANNEL_GUIDANCE: &str = "Use `mise upgrade herdr` to update mise installs.";
const NIX_CHANNEL_GUIDANCE: &str = "Update through Nix to update Nix-managed Herdr installs.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InstallKind {
    Direct,
    Homebrew,
    Npm,
    Mise,
    Nix,
}

impl InstallKind {
    fn classify(path: &Path) -> Self {
        if matches_or_canonical(path, is_homebrew_managed_exe_path) {
            Self::Homebrew
        } else if matches_or_canonical(path, is_npm_managed_exe_path) {
            Self::Npm
        } else if matches_or_canonical(path, is_mise_managed_exe_path) {
            Self::Mise
        } else if matches_or_canonical(path, is_nix_store_exe_path) {
            Self::Nix
        } else {
            Self::Direct
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Homebrew => "homebrew",
            Self::Npm => "npm",
            Self::Mise => "mise",
            Self::Nix => "nix",
        }
    }

    fn is_package_managed(self) -> bool {
        !matches!(self, Self::Direct)
    }

    fn preview_note(self) -> Option<&'static str> {
        match self {
            Self::Direct => None,
            Self::Homebrew => Some(HOMEBREW_PREVIEW_NOTE),
            Self::Npm => Some(NPM_PREVIEW_NOTE),
            Self::Mise => Some(MISE_PREVIEW_NOTE),
            Self::Nix => Some(NIX_PREVIEW_NOTE),
        }
    }

    fn update_command(self, path: Option<&Path>) -> Option<&'static str> {
        match self {
            Self::Direct | Self::Nix => None,
            Self::Homebrew => Some(homebrew_update_command_for_path(path)),
            Self::Npm => Some(NPM_UPDATE_COMMAND),
            Self::Mise => Some(MISE_UPDATE_COMMAND),
        }
    }

    fn channel_guidance(self, path: Option<&Path>) -> Option<&'static str> {
        match self {
            Self::Direct => None,
            Self::Homebrew => Some(match path.and_then(homebrew_formula_name) {
                Some("herdr") => HOMEBREW_UPSTREAM_CHANNEL_GUIDANCE,
                _ => HOMEBREW_CHANNEL_GUIDANCE,
            }),
            Self::Npm => Some(NPM_CHANNEL_GUIDANCE),
            Self::Mise => Some(MISE_CHANNEL_GUIDANCE),
            Self::Nix => Some(NIX_CHANNEL_GUIDANCE),
        }
    }

    fn tui_update_command(self, path: Option<&Path>) -> &'static str {
        match self {
            Self::Direct => HERDR_UPDATE_COMMAND,
            Self::Nix => NIX_UPDATE_COMMAND,
            Self::Homebrew | Self::Npm | Self::Mise => {
                self.update_command(path).unwrap_or(HERDR_UPDATE_COMMAND)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PathInstallShadow {
    pub(super) leftover: PathBuf,
    pub(super) managed: PathBuf,
    pub(super) managed_kind: InstallKind,
}

impl PathInstallShadow {
    fn is_transient_nix(&self) -> bool {
        self.managed_kind == InstallKind::Nix && self.managed.starts_with("/nix/store")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SelfUpdatePlan {
    Direct,
    ForceDirect {
        shadow: PathInstallShadow,
    },
    RunPm {
        kind: InstallKind,
        command: &'static str,
        prerelease_note: Option<&'static str>,
        leftover: Option<PathInstallShadow>,
    },
    Guide {
        message: &'static str,
        leftover: Option<PathInstallShadow>,
        exit_error: bool,
    },
}

pub(crate) fn update_install_command() -> &'static str {
    let path = env::current_exe().ok();
    current_install_kind_from(path.as_deref()).tui_update_command(path.as_deref())
}

pub(crate) fn update_install_instruction(install_command: &str) -> String {
    match install_command {
        HERDR_UPDATE_COMMAND => {
            "detach, run `herdr update`, then follow its restart guidance".to_string()
        }
        HOMEBREW_UPDATE_COMMAND => {
            "detach, run `brew update && brew upgrade GroepOnline/tap/groeponline-herdr`, then restart this Herdr session when ready".to_string()
        }
        HOMEBREW_UPSTREAM_UPDATE_COMMAND => {
            "detach, run `brew update && brew upgrade herdr`, then restart this Herdr session when ready".to_string()
        }
        NPM_UPDATE_COMMAND => {
            "detach, run `npm install --global groeponline-herdr@latest`, then restart this Herdr session when ready".to_string()
        }
        MISE_UPDATE_COMMAND => {
            "detach, run `mise upgrade herdr`, then restart this Herdr session when ready"
                .to_string()
        }
        NIX_UPDATE_COMMAND => {
            "detach, update through Nix, then restart this Herdr session when ready".to_string()
        }
        command => format!("detach, run `{command}`, then restart this Herdr session when ready"),
    }
}

pub(crate) fn preview_channel_rejection_for_current_install() -> Option<&'static str> {
    let path = env::current_exe().ok()?;
    InstallKind::classify(&path).preview_note()
}

pub(crate) fn package_manager_channel_update_guidance_for_current_install() -> Option<&'static str>
{
    let path = env::current_exe().ok();
    current_install_kind_from(path.as_deref()).channel_guidance(path.as_deref())
}

#[cfg(unix)]
pub(crate) fn is_package_manager_managed_exe_path(path: &Path) -> bool {
    InstallKind::classify(path).is_package_managed()
}

#[cfg(not(unix))]
pub(crate) fn is_package_manager_managed_exe_path(_path: &Path) -> bool {
    false
}

pub(crate) fn current_install_kind_label() -> &'static str {
    current_install_kind().as_str()
}

pub(super) fn current_install_kind() -> InstallKind {
    current_install_kind_from(env::current_exe().ok().as_deref())
}

fn current_install_kind_from(path: Option<&Path>) -> InstallKind {
    path.map(InstallKind::classify)
        .unwrap_or(InstallKind::Direct)
}

pub(crate) fn invoked_binary_label() -> String {
    match env::current_exe() {
        Ok(path) => format!(
            "{} ({})",
            path.display(),
            InstallKind::classify(&path).as_str()
        ),
        Err(err) => format!("unknown ({err})"),
    }
}

pub(crate) fn print_version_identity() {
    match env::current_exe() {
        Ok(path) => {
            eprintln!(
                "binary: {} ({})",
                path.display(),
                InstallKind::classify(&path).as_str()
            );
            if let Some(shadow) = detect_path_install_shadow_for(&path) {
                eprintln!(
                    "note: {} herdr is also on PATH at {}",
                    shadow.managed_kind.as_str(),
                    shadow.managed.display()
                );
            }
        }
        Err(err) => eprintln!("binary: unknown ({err})"),
    }
}

pub(super) fn plan_self_update(force_direct: bool, channel: UpdateChannel) -> SelfUpdatePlan {
    let path = env::current_exe().ok();
    let kind = current_install_kind_from(path.as_deref());
    let shadow = path.as_deref().and_then(detect_path_install_update_shadow_for);
    plan_from_parts(kind, path.as_deref(), channel, force_direct, shadow)
}

fn plan_from_parts(
    kind: InstallKind,
    path: Option<&Path>,
    channel: UpdateChannel,
    force_direct: bool,
    shadow: Option<PathInstallShadow>,
) -> SelfUpdatePlan {
    if force_direct {
        return match shadow {
            Some(shadow) => SelfUpdatePlan::ForceDirect { shadow },
            None => plan_for_managed(kind, path, channel, None),
        };
    }

    if let Some(shadow) = shadow.filter(|shadow| !shadow.is_transient_nix()) {
        let kind = shadow.managed_kind;
        let managed = shadow.managed.clone();
        return plan_for_managed(kind, Some(&managed), channel, Some(shadow));
    }

    plan_for_managed(kind, path, channel, None)
}

fn plan_for_managed(
    kind: InstallKind,
    path: Option<&Path>,
    channel: UpdateChannel,
    leftover: Option<PathInstallShadow>,
) -> SelfUpdatePlan {
    let prerelease_note = channel
        .is_prerelease()
        .then(|| kind.preview_note())
        .flatten();
    match kind {
        InstallKind::Direct => SelfUpdatePlan::Direct,
        InstallKind::Homebrew | InstallKind::Npm | InstallKind::Mise => {
            let Some(command) = kind.update_command(path) else {
                return SelfUpdatePlan::Direct;
            };
            SelfUpdatePlan::RunPm {
                kind,
                command,
                prerelease_note,
                leftover,
            }
        }
        InstallKind::Nix => SelfUpdatePlan::Guide {
            message: if channel.is_prerelease() {
                NIX_GUIDANCE_PRERELEASE
            } else {
                NIX_GUIDANCE
            },
            leftover,
            exit_error: leftover.is_none(),
        },
    }
}

pub(super) fn apply_managed_update(plan: SelfUpdatePlan) -> Result<(), String> {
    match plan {
        SelfUpdatePlan::RunPm {
            kind,
            command,
            prerelease_note,
            leftover,
        } => {
            if let Some(note) = prerelease_note {
                eprintln!("note: {note}");
            }
            eprintln!("this {} install updates with `{command}`", kind.as_str());
            #[cfg(unix)]
            {
                eprintln!("running `{command}`");
                crate::platform::run_shell_command(command)?;
                eprintln!("package manager update finished");
            }
            #[cfg(not(unix))]
            {
                eprintln!("run `{command}` to update");
            }
            retire_leftover_after_success(leftover.as_ref());
            eprintln!(
                "Restart any running Herdr sessions to use the {} install.",
                kind.as_str()
            );
            Ok(())
        }
        SelfUpdatePlan::Guide {
            message,
            leftover,
            exit_error,
        } => {
            eprintln!("{message}");
            retire_leftover_after_success(leftover.as_ref());
            if exit_error {
                Err(message.to_string())
            } else {
                Ok(())
            }
        }
        SelfUpdatePlan::Direct | SelfUpdatePlan::ForceDirect { .. } => Ok(()),
    }
}

fn retire_leftover_after_success(shadow: Option<&PathInstallShadow>) -> Result<(), String> {
    let Some(shadow) = shadow else {
        return Ok(());
    };
    #[cfg(not(unix))]
    {
        let _ = shadow;
    }
    #[cfg(unix)]
    match retire_leftover(shadow) {
        Ok(backup) => {
            eprintln!(
                "retired leftover direct install {} -> {}",
                shadow.leftover.display(),
                backup.display()
            );
            eprintln!(
                "herdr on PATH now uses the {} install at {}",
                shadow.managed_kind.as_str(),
                shadow.managed.display()
            );
            eprintln!(
                "use `herdr update --force-direct` only when you want to keep a leftover direct binary first on PATH"
            );
        }
        Err(err) => eprintln!("{err}"),
    }
}

#[cfg(unix)]
fn leftover_direct_backup_path(leftover: &Path) -> PathBuf {
    let file_name = leftover
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("herdr");
    let preferred = leftover.with_file_name(format!("{file_name}.direct.bak"));
    if !preferred.exists() {
        return preferred;
    }
    leftover.with_file_name(format!("{file_name}.direct.bak.{}", std::process::id()))
}

#[cfg(unix)]
fn retire_leftover(shadow: &PathInstallShadow) -> Result<PathBuf, String> {
    let backup = leftover_direct_backup_path(&shadow.leftover);
    fs::rename(&shadow.leftover, &backup).map_err(|err| {
        format!(
            "package manager finished, but leftover {} still shadows PATH. Move it aside (`mv {} {}`): {err}",
            shadow.leftover.display(),
            shadow.leftover.display(),
            backup.display()
        )
    })?;
    Ok(backup)
}

fn herdr_exe_file_name() -> &'static str {
    if cfg!(windows) {
        "herdr.exe"
    } else {
        "herdr"
    }
}

fn herdr_binaries_on_path_var(path_var: impl AsRef<std::ffi::OsStr>) -> Vec<PathBuf> {
    env::split_paths(path_var.as_ref())
        .filter_map(|dir| {
            let candidate = dir.join(herdr_exe_file_name());
            candidate
                .is_file()
                .then(|| {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        (candidate.metadata().ok()?.permissions().mode() & 0o111 != 0)
                            .then_some(candidate)
                    }
                    #[cfg(not(unix))]
                    {
                        Some(candidate)
                    }
                })
                .flatten()
        })
        .collect()
}

fn path_install_shadow_impl(
    current_exe: &Path,
    path_binaries: &[PathBuf],
    skip_transient_nix: bool,
) -> Option<PathInstallShadow> {
    if InstallKind::classify(current_exe).is_package_managed() {
        return None;
    }

    let mut leftover = None;
    for candidate in path_binaries {
        if leftover.is_none() {
            if paths_match(candidate, current_exe) {
                leftover = Some(candidate.clone());
            }
            continue;
        }

        let kind = InstallKind::classify(candidate);
        if !kind.is_package_managed() {
            continue;
        }
        // A raw /nix/store path is usually a transient `nix shell` rather
        // than a durable install. When planning an update, keep scanning
        // past it instead of treating it as the managed install to retire
        // the leftover for.
        if skip_transient_nix && kind == InstallKind::Nix && candidate.starts_with("/nix/store") {
            continue;
        }
        return Some(PathInstallShadow {
            leftover: leftover?,
            managed: candidate.clone(),
            managed_kind: kind,
        });
    }

    None
}

fn path_install_shadow(current_exe: &Path, path_binaries: &[PathBuf]) -> Option<PathInstallShadow> {
    path_install_shadow_impl(current_exe, path_binaries, false)
}

/// Like [`path_install_shadow`], but skips transient `/nix/store` entries so
/// a package-manager install later on `PATH` can still be found and used to
/// plan an update / leftover retirement.
fn path_install_update_shadow(
    current_exe: &Path,
    path_binaries: &[PathBuf],
) -> Option<PathInstallShadow> {
    path_install_shadow_impl(current_exe, path_binaries, true)
}

fn detect_path_install_shadow_for(current_exe: &Path) -> Option<PathInstallShadow> {
    let path_var = env::var_os("PATH")?;
    path_install_shadow(current_exe, &herdr_binaries_on_path_var(&path_var))
}

fn detect_path_install_update_shadow_for(current_exe: &Path) -> Option<PathInstallShadow> {
    let path_var = env::var_os("PATH")?;
    path_install_update_shadow(current_exe, &herdr_binaries_on_path_var(&path_var))
}

fn homebrew_update_command_for_path(path: Option<&Path>) -> &'static str {
    match path.and_then(homebrew_formula_name) {
        Some("herdr") => HOMEBREW_UPSTREAM_UPDATE_COMMAND,
        _ => HOMEBREW_UPDATE_COMMAND,
    }
}

fn homebrew_formula_name(path: &Path) -> Option<&'static str> {
    let version_dir = homebrew_cellar_keg_root(path).or_else(|| {
        path.canonicalize()
            .ok()
            .and_then(|canonical| homebrew_cellar_keg_root(&canonical))
    })?;
    match version_dir.parent()?.file_name()?.to_str()? {
        "herdr" => Some("herdr"),
        "groeponline-herdr" => Some("groeponline-herdr"),
        _ => None,
    }
}

fn matches_or_canonical(path: &Path, pred: fn(&Path) -> bool) -> bool {
    pred(path) || path.canonicalize().is_ok_and(|canonical| pred(&canonical))
}

fn is_npm_managed_exe_path(path: &Path) -> bool {
    if path.file_name() != Some(std::ffi::OsStr::new("herdr")) {
        return false;
    }
    let Some(bin_dir) = path.parent() else {
        return false;
    };
    if bin_dir.file_name() != Some(std::ffi::OsStr::new("bin")) {
        return false;
    }
    let Some(package_dir) = bin_dir.parent() else {
        return false;
    };
    if package_dir.file_name() != Some(std::ffi::OsStr::new("groeponline-herdr")) {
        return false;
    }
    package_dir
        .parent()
        .and_then(Path::file_name)
        .is_some_and(|name| name == "node_modules")
}

fn is_nix_store_exe_path(path: &Path) -> bool {
    path.starts_with("/nix/store")
}

fn is_mise_managed_exe_path(path: &Path) -> bool {
    mise_install_root(path).is_some() || is_mise_shim_exe_path(path)
}

pub(crate) fn is_mise_shim_exe_path(path: &Path) -> bool {
    if path.file_name() != Some(std::ffi::OsStr::new("herdr")) {
        return false;
    }
    let Some(parent) = path.parent() else {
        return false;
    };
    if parent.file_name() != Some(std::ffi::OsStr::new("shims")) {
        return false;
    }
    path.components()
        .any(|component| component.as_os_str() == "mise")
}

fn mise_install_root(path: &Path) -> Option<PathBuf> {
    if let Some(root) = mise_install_root_under_configured_installs_dir(path) {
        return Some(root);
    }

    mise_install_root_under_named_installs_dir(path)
}

fn mise_install_root_under_configured_installs_dir(path: &Path) -> Option<PathBuf> {
    let installs_dir = env::var_os(MISE_INSTALLS_DIR_ENV)
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())?;
    let version_dir = mise_tool_version_dir(path)?;
    let tool_dir = version_dir.parent()?;
    paths_match(tool_dir.parent()?, &installs_dir).then_some(version_dir.to_path_buf())
}

fn mise_install_root_under_named_installs_dir(path: &Path) -> Option<PathBuf> {
    let version_dir = mise_tool_version_dir(path)?;
    let tool_dir = version_dir.parent()?;
    let installs_dir = tool_dir.parent()?;
    if installs_dir.file_name()? != "installs" {
        return None;
    }
    Some(version_dir.to_path_buf())
}

fn mise_tool_version_dir(path: &Path) -> Option<&Path> {
    if path.file_name()? != "herdr" {
        return None;
    }
    let bin_dir = path.parent()?;
    if bin_dir.file_name()? != "bin" {
        return None;
    }
    let version_dir = bin_dir.parent()?;
    let tool_dir = version_dir.parent()?;
    if tool_dir.file_name()? != "herdr" {
        return None;
    }
    Some(version_dir)
}

fn paths_match(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }

    let Ok(left) = left.canonicalize() else {
        return false;
    };
    let Ok(right) = right.canonicalize() else {
        return false;
    };
    left == right
}

fn is_homebrew_managed_exe_path(path: &Path) -> bool {
    homebrew_cellar_keg_root(path).is_some()
}

fn homebrew_cellar_keg_root(path: &Path) -> Option<PathBuf> {
    if path.file_name()? != "herdr" {
        return None;
    }
    let bin_dir = path.parent()?;
    if bin_dir.file_name()? != "bin" {
        return None;
    }
    let version_dir = bin_dir.parent()?;
    let formula_dir = version_dir.parent()?;
    let formula_name = formula_dir.file_name()?.to_str()?;
    if formula_name != "herdr" && formula_name != "groeponline-herdr" {
        return None;
    }
    let cellar_dir = formula_dir.parent()?;
    if cellar_dir.file_name()? != "Cellar" {
        return None;
    }
    Some(version_dir.to_path_buf())
}

#[cfg(all(test, unix))]
#[path = "install_tests.rs"]
mod tests;
