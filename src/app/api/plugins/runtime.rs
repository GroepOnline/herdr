use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;

use super::manifest::{effective_platforms, ensure_platform_supported};
use super::plugin_manifest_available;
use crate::api::schema::{
    InstalledPluginInfo, PluginCommandLogInfo, PluginCommandStatus, PluginInvocationContext,
};
use crate::app::App;

const PLUGIN_COMMAND_OUTPUT_MAX_BYTES: usize = 64 * 1024;
pub(super) const MAX_PLUGIN_COMMANDS_IN_FLIGHT: usize = 32;
const PLUGIN_COMMAND_LOG_LIMIT: usize = 200;
const PLUGIN_COMMAND_HISTORY_MAX_BYTES: u64 = 8 * 1024 * 1024;
const PLUGIN_COMMAND_HISTORY_ROTATIONS: usize = 7;

impl App {
    pub(super) fn start_plugin_command(
        &mut self,
        plugin: &InstalledPluginInfo,
        action_id: Option<String>,
        event: Option<String>,
        command: Vec<String>,
        context: &PluginInvocationContext,
        event_json: Option<String>,
    ) -> Result<PluginCommandLogInfo, (&'static str, String)> {
        let Some(program) = command.first().cloned() else {
            return Err((
                "invalid_plugin_command",
                "command must not be empty".to_string(),
            ));
        };
        let args = command.iter().skip(1).cloned().collect::<Vec<_>>();
        let context_json = serde_json::to_string(context)
            .map_err(|err| ("invalid_plugin_context", err.to_string()))?;
        super::env::ensure_plugin_user_dirs(plugin)
            .map_err(|err| ("plugin_user_dir_create_failed", err.to_string()))?;
        let log_id = format!("plugin-log-{}", self.state.next_plugin_command_log_id);
        self.state.next_plugin_command_log_id += 1;
        let started_unix_ms = current_unix_ms();
        let mut env = super::env::plugin_path_env(plugin);
        env.extend([
            (
                crate::api::SOCKET_PATH_ENV_VAR.to_string(),
                crate::api::socket_path().display().to_string(),
            ),
            ("HERDR_ENV".to_string(), "1".to_string()),
            ("HERDR_PLUGIN_ID".to_string(), plugin.plugin_id.clone()),
            ("HERDR_PLUGIN_CONTEXT_JSON".to_string(), context_json),
        ]);
        if let Ok(current_exe) = std::env::current_exe() {
            env.push((
                "HERDR_BIN_PATH".to_string(),
                current_exe.display().to_string(),
            ));
        }
        if let Some(action_id) = action_id.as_ref() {
            env.push(("HERDR_PLUGIN_ACTION_ID".to_string(), action_id.clone()));
        }
        if let Some(event) = event.as_ref() {
            env.push(("HERDR_PLUGIN_EVENT".to_string(), event.clone()));
        }
        if let Some(event_json) = event_json {
            env.push(("HERDR_PLUGIN_EVENT_JSON".to_string(), event_json));
        }
        if let Some(workspace_id) = context.workspace_id.as_ref() {
            env.push(("HERDR_WORKSPACE_ID".to_string(), workspace_id.clone()));
        }
        if let Some(tab_id) = context.tab_id.as_ref() {
            env.push(("HERDR_TAB_ID".to_string(), tab_id.clone()));
        }
        if let Some(pane_id) = context.focused_pane_id.as_ref() {
            env.push(("HERDR_PANE_ID".to_string(), pane_id.clone()));
        }
        if let Some(clicked_url) = context.clicked_url.as_ref() {
            env.push(("HERDR_PLUGIN_CLICKED_URL".to_string(), clicked_url.clone()));
        }
        if let Some(link_handler_id) = context.link_handler_id.as_ref() {
            env.push((
                "HERDR_PLUGIN_LINK_HANDLER_ID".to_string(),
                link_handler_id.clone(),
            ));
        }
        if self.state.plugin_commands_in_flight >= MAX_PLUGIN_COMMANDS_IN_FLIGHT {
            let message = format!(
                "maximum concurrent plugin commands reached ({MAX_PLUGIN_COMMANDS_IN_FLIGHT})"
            );
            let log = PluginCommandLogInfo {
                log_id,
                plugin_id: plugin.plugin_id.clone(),
                action_id,
                event,
                command,
                status: PluginCommandStatus::Failed,
                started_unix_ms,
                finished_unix_ms: Some(started_unix_ms),
                exit_code: None,
                stdout: Some(String::new()),
                stderr: Some(String::new()),
                error: Some(message.clone()),
            };
            self.push_plugin_command_log(log);
            return Err(("plugin_command_limit_reached", message));
        }
        let plugin_root = std::path::PathBuf::from(&plugin.plugin_root);
        let log = PluginCommandLogInfo {
            log_id: log_id.clone(),
            plugin_id: plugin.plugin_id.clone(),
            action_id,
            event,
            command: command.clone(),
            status: PluginCommandStatus::Running,
            started_unix_ms,
            finished_unix_ms: None,
            exit_code: None,
            stdout: None,
            stderr: None,
            error: None,
        };
        self.push_plugin_command_log(log.clone());
        self.state.plugin_commands_in_flight += 1;
        let event_tx = self.event_tx.clone();
        std::thread::spawn(move || {
            let child =
                crate::plugin_command::command_for_argv_in_dir(&program, &args, &plugin_root)
                    .envs(env)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn();
            let finished = match child {
                Ok(mut child) => {
                    let stdout = child.stdout.take();
                    let stderr = child.stderr.take();
                    let stdout_reader = stdout.map(|stdout| {
                        std::thread::spawn(move || {
                            read_capped_plugin_output(stdout, PLUGIN_COMMAND_OUTPUT_MAX_BYTES)
                        })
                    });
                    let stderr_reader = stderr.map(|stderr| {
                        std::thread::spawn(move || {
                            read_capped_plugin_output(stderr, PLUGIN_COMMAND_OUTPUT_MAX_BYTES)
                        })
                    });
                    match child.wait() {
                        Ok(status) => crate::events::AppEvent::PluginCommandFinished {
                            log_id,
                            finished_unix_ms: current_unix_ms(),
                            exit_code: status.code(),
                            stdout: stdout_reader
                                .and_then(|reader| reader.join().ok())
                                .unwrap_or_default(),
                            stderr: stderr_reader
                                .and_then(|reader| reader.join().ok())
                                .unwrap_or_default(),
                            error: None,
                        },
                        Err(err) => crate::events::AppEvent::PluginCommandFinished {
                            log_id,
                            finished_unix_ms: current_unix_ms(),
                            exit_code: None,
                            stdout: stdout_reader
                                .and_then(|reader| reader.join().ok())
                                .unwrap_or_default(),
                            stderr: stderr_reader
                                .and_then(|reader| reader.join().ok())
                                .unwrap_or_default(),
                            error: Some(err.to_string()),
                        },
                    }
                }
                Err(err) => crate::events::AppEvent::PluginCommandFinished {
                    log_id,
                    finished_unix_ms: current_unix_ms(),
                    exit_code: None,
                    stdout: String::new(),
                    stderr: String::new(),
                    error: Some(err.to_string()),
                },
            };
            let _ = event_tx.blocking_send(finished);
        });
        Ok(log)
    }

    pub(crate) fn run_plugin_startup_hooks(&mut self) {
        let mut context = self.current_plugin_context("plugin.startup");
        context.invocation_source = Some("startup".to_string());
        let mut plugins = self
            .state
            .installed_plugins
            .values()
            .filter(|plugin| {
                plugin.enabled && plugin_manifest_available(plugin) && !plugin.startup.is_empty()
            })
            .cloned()
            .collect::<Vec<_>>();
        plugins.sort_by(|left, right| left.plugin_id.cmp(&right.plugin_id));
        for plugin in plugins {
            for startup in &plugin.startup {
                if ensure_platform_supported(
                    effective_platforms(&startup.platforms, &plugin.platforms),
                    "startup",
                )
                .is_err()
                {
                    continue;
                }
                let _ = self.start_plugin_command(
                    &plugin,
                    None,
                    Some("startup".to_string()),
                    startup.command.clone(),
                    &context,
                    None,
                );
            }
        }
    }

    pub(crate) fn run_plugin_event_hooks(&mut self, event: &crate::api::schema::EventEnvelope) {
        let event_name = event.event.dot_name();
        if !crate::api::schema::PLUGIN_HOOK_EVENT_KINDS.contains(&event.event) {
            return;
        }
        if let Err(err) = self.refresh_installed_plugins() {
            tracing::warn!(err = %err, "failed to refresh plugin registry before event hooks");
            return;
        }
        let plugins = self
            .state
            .installed_plugins
            .values()
            .filter(|plugin| {
                plugin.enabled
                    && plugin_manifest_available(plugin)
                    && plugin.events.iter().any(|hook| hook.on == event_name)
            })
            .cloned()
            .collect::<Vec<_>>();
        if plugins.is_empty() {
            return;
        }
        let event_json = serde_json::to_string(event).ok();
        let context = self.plugin_context_for_event(event, event_name);
        for plugin in plugins {
            for hook in &plugin.events {
                if hook.on != event_name {
                    continue;
                }
                if ensure_platform_supported(
                    effective_platforms(&hook.platforms, &plugin.platforms),
                    event_name,
                )
                .is_err()
                {
                    continue;
                }
                let _ = self.start_plugin_command(
                    &plugin,
                    None,
                    Some(event_name.to_string()),
                    hook.command.clone(),
                    &context,
                    event_json.clone(),
                );
            }
        }
    }

    fn push_plugin_command_log(&mut self, log: PluginCommandLogInfo) {
        persist_plugin_command_log(&log);
        self.state.plugin_command_logs.push(log);
        if self.state.plugin_command_logs.len() > PLUGIN_COMMAND_LOG_LIMIT {
            let extra = self.state.plugin_command_logs.len() - PLUGIN_COMMAND_LOG_LIMIT;
            self.state.plugin_command_logs.drain(0..extra);
        }
    }

    pub(crate) fn persist_finished_plugin_command_log(&self, log: &PluginCommandLogInfo) {
        persist_plugin_command_log(log);
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct PersistentPluginSummary {
    schema_version: u8,
    session: String,
    updated_unix_ms: u64,
    plugins: BTreeMap<String, PersistentPluginStats>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct PersistentPluginStats {
    total: u64,
    succeeded: u64,
    failed: u64,
    last_status: String,
    last_action_id: Option<String>,
    last_event: Option<String>,
    last_finished_unix_ms: Option<u64>,
    last_duration_ms: Option<u64>,
}

fn plugin_history_dir() -> PathBuf {
    let session = crate::session::active_name()
        .unwrap_or_else(|| crate::session::DEFAULT_SESSION_NAME.to_string());
    crate::config::state_dir()
        .join("plugin-history")
        .join(session)
}

fn persistent_status(status: PluginCommandStatus) -> &'static str {
    match status {
        PluginCommandStatus::Running => "running",
        PluginCommandStatus::Succeeded => "succeeded",
        PluginCommandStatus::Failed => "failed",
    }
}

fn persist_plugin_command_log(log: &PluginCommandLogInfo) {
    if let Err(err) = persist_plugin_command_log_inner(log) {
        tracing::warn!(err = %err, plugin_id = %log.plugin_id, log_id = %log.log_id, "failed to persist plugin command metadata");
    }
}

fn persist_plugin_command_log_inner(log: &PluginCommandLogInfo) -> std::io::Result<()> {
    let dir = plugin_history_dir();
    fs::create_dir_all(&dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
    }

    let history = dir.join("commands.jsonl");
    rotate_plugin_history(&history)?;
    let status = persistent_status(log.status);
    let duration_ms = log
        .finished_unix_ms
        .map(|finished| finished.saturating_sub(log.started_unix_ms));
    let program = log
        .command
        .first()
        .and_then(|value| Path::new(value).file_name())
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let session = crate::session::active_name()
        .unwrap_or_else(|| crate::session::DEFAULT_SESSION_NAME.to_string());
    let record = persistent_plugin_record(log, &session, status, duration_ms, program);
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
    let mut file = options.open(&history)?;
    serde_json::to_writer(&mut file, &record)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
    file.write_all(b"\n")?;

    if !matches!(log.status, PluginCommandStatus::Running) {
        update_plugin_summary(&dir, log, status, duration_ms)?;
    }
    Ok(())
}

fn persistent_plugin_record(
    log: &PluginCommandLogInfo,
    session: &str,
    status: &str,
    duration_ms: Option<u64>,
    program: &str,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "session": session,
        "log_id": log.log_id,
        "plugin_id": log.plugin_id,
        "action_id": log.action_id,
        "event": log.event,
        "status": status,
        "started_unix_ms": log.started_unix_ms,
        "finished_unix_ms": log.finished_unix_ms,
        "duration_ms": duration_ms,
        "exit_code": log.exit_code,
        "program": program,
        "stdout_bytes": log.stdout.as_ref().map(|value| value.len()).unwrap_or(0),
        "stderr_bytes": log.stderr.as_ref().map(|value| value.len()).unwrap_or(0),
        "error_present": log.error.is_some(),
        "privacy": "metadata-only",
    })
}

fn rotate_plugin_history(path: &Path) -> std::io::Result<()> {
    let Ok(meta) = fs::metadata(path) else {
        return Ok(());
    };
    if meta.len() < PLUGIN_COMMAND_HISTORY_MAX_BYTES {
        return Ok(());
    }
    for index in (1..=PLUGIN_COMMAND_HISTORY_ROTATIONS).rev() {
        let source = if index == 1 {
            path.to_path_buf()
        } else {
            path.with_file_name(format!("commands.jsonl.{}", index - 1))
        };
        if !source.exists() {
            continue;
        }
        let destination = path.with_file_name(format!("commands.jsonl.{index}"));
        if destination.exists() {
            let _ = fs::remove_file(&destination);
        }
        fs::rename(source, destination)?;
    }
    Ok(())
}

fn update_plugin_summary(
    dir: &Path,
    log: &PluginCommandLogInfo,
    status: &str,
    duration_ms: Option<u64>,
) -> std::io::Result<()> {
    let path = dir.join("summary.json");
    let session = crate::session::active_name()
        .unwrap_or_else(|| crate::session::DEFAULT_SESSION_NAME.to_string());
    let mut summary = fs::read(&path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<PersistentPluginSummary>(&bytes).ok())
        .unwrap_or_else(|| PersistentPluginSummary {
            schema_version: 1,
            session: session.clone(),
            updated_unix_ms: 0,
            plugins: BTreeMap::new(),
        });
    summary.schema_version = 1;
    summary.session = session;
    summary.updated_unix_ms = log.finished_unix_ms.unwrap_or_else(current_unix_ms);
    let stats = summary.plugins.entry(log.plugin_id.clone()).or_default();
    stats.total = stats.total.saturating_add(1);
    match log.status {
        PluginCommandStatus::Succeeded => stats.succeeded = stats.succeeded.saturating_add(1),
        PluginCommandStatus::Failed => stats.failed = stats.failed.saturating_add(1),
        PluginCommandStatus::Running => {}
    }
    stats.last_status = status.to_string();
    stats.last_action_id = log.action_id.clone();
    stats.last_event = log.event.clone();
    stats.last_finished_unix_ms = log.finished_unix_ms;
    stats.last_duration_ms = duration_ms;

    let temp = dir.join(format!("summary.json.{}.tmp", std::process::id()));
    let bytes = serde_json::to_vec_pretty(&summary)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
    fs::write(&temp, bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let _ = fs::set_permissions(&temp, fs::Permissions::from_mode(0o600));
    }
    fs::rename(temp, path)
}

fn current_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

pub(super) fn read_capped_plugin_output(mut reader: impl Read, cap: usize) -> String {
    let mut kept = Vec::with_capacity(cap.min(8192));
    let mut buf = [0u8; 8192];
    let mut truncated = false;
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let remaining = cap.saturating_sub(kept.len());
                if remaining > 0 {
                    kept.extend_from_slice(&buf[..n.min(remaining)]);
                }
                if n > remaining {
                    truncated = true;
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
    let mut output = String::from_utf8_lossy(&kept).into_owned();
    if truncated {
        output.push_str(&format!(
            "\n[herdr truncated plugin output after {cap} bytes]"
        ));
    }
    output
}

#[cfg(test)]
mod persistent_log_tests {
    use super::*;

    #[test]
    fn persistent_plugin_record_is_metadata_only() {
        let log = PluginCommandLogInfo {
            log_id: "plugin-log-7".to_string(),
            plugin_id: "com.chefgroep.example".to_string(),
            action_id: Some("status".to_string()),
            event: None,
            command: vec!["node".to_string(), "secret-argument".to_string()],
            status: PluginCommandStatus::Failed,
            started_unix_ms: 1_000,
            finished_unix_ms: Some(1_250),
            exit_code: Some(1),
            stdout: Some("sensitive stdout body".to_string()),
            stderr: Some("sensitive stderr body".to_string()),
            error: Some("sensitive error text".to_string()),
        };
        let record = persistent_plugin_record(&log, "default", "failed", Some(250), "node");
        let encoded = serde_json::to_string(&record).unwrap();
        assert!(encoded.contains("com.chefgroep.example"));
        assert!(encoded.contains("\"stdout_bytes\":21"));
        assert!(!encoded.contains("secret-argument"));
        assert!(!encoded.contains("sensitive stdout body"));
        assert!(!encoded.contains("sensitive stderr body"));
        assert!(!encoded.contains("sensitive error text"));
    }

    #[test]
    fn persistent_status_uses_stable_lowercase_values() {
        assert_eq!(persistent_status(PluginCommandStatus::Running), "running");
        assert_eq!(
            persistent_status(PluginCommandStatus::Succeeded),
            "succeeded"
        );
        assert_eq!(persistent_status(PluginCommandStatus::Failed), "failed");
    }
}
