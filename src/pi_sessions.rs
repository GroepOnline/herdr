//! Pi coding-agent session watcher.
//!
//! Emits [`crate::api::schema::EventKind::PiSessionEnded`] when a Pi session
//! JSONL under `~/.pi/agent/sessions/` stops growing. This lets plugins react
//! the moment a session finishes (e.g. chef-pi-eval's `ingest`) instead of
//! waiting for a coarse hourly poll.
//!
//! Design notes:
//! - **Stateless across restarts by intent.** The emitted-set lives only in
//!   memory; on restart we re-emit recently-ended sessions once. That's fine:
//!   ingestion is idempotent (upsert / ON CONFLICT DO NOTHING).
//! - **No filesystem watcher dependency.** A cheap directory scan every poll
//!   (~60 s) keeps this dependency-free and robust on SSH mounts / fleet
//!   hosts where inotify is unreliable.
//! - **Parse only first + last line + line count.** Full-session parsing is the
//!   plugin's job; the watcher just detects "stopped growing" and carries the
//!   minimal payload plugins need to decide whether to act.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

use serde::Deserialize;

use crate::api::schema::{EventData, EventEnvelope, EventKind};

/// A session file whose mtime is older than this is considered "ended".
const STALE_AFTER_SECS: u64 = 120;
/// Minimum gap between full directory rescans.
const RESCAN_EVERY_SECS: u64 = 60;
/// Override the sessions root via env (mainly for tests).
const SESSIONS_DIR_ENV: &str = "HERDR_PI_SESSIONS_DIR";

#[derive(Debug, Clone)]
pub struct EndedSession {
    pub session_id: String,
    pub cwd: String,
    pub host: String,
    pub last_stop_reason: Option<String>,
    pub event_count: u64,
    pub session_file: PathBuf,
}

impl EndedSession {
    pub fn into_envelope(self) -> EventEnvelope {
        EventEnvelope {
            event: EventKind::PiSessionEnded,
            data: EventData::PiSessionEnded {
                session_id: self.session_id,
                cwd: self.cwd,
                host: self.host,
                last_stop_reason: self.last_stop_reason,
                event_count: self.event_count,
                session_file: self.session_file.to_string_lossy().into_owned(),
            },
        }
    }
}

#[derive(Default)]
pub struct PiSessionWatcher {
    /// Session files we have already emitted, by canonical path.
    emitted: HashSet<PathBuf>,
    /// Candidate files observed during the last scan (path → mtime).
    seen: Vec<(PathBuf, SystemTime)>,
    last_scan: Option<Instant>,
    /// Pinned sessions root. `None` (production) resolves via env/default on
    /// each rescan; `Some(_)` (tests) avoids touching process-global env so
    /// parallel tests don't clobber each other's `SESSIONS_DIR_ENV`.
    root: Option<PathBuf>,
}

impl PiSessionWatcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct with a pinned sessions root. Used by tests to stay isolated
    /// from the process-global `SESSIONS_DIR_ENV` (parallel tests would race).
    #[cfg(test)]
    pub(crate) fn for_root(root: PathBuf) -> Self {
        Self {
            root: Some(root),
            ..Self::default()
        }
    }

    /// Returns envelopes for every session that ended since the last poll.
    /// Cheap when called more often than [`RESCAN_EVERY_SECS`].
    pub fn poll(&mut self) -> Vec<EventEnvelope> {
        let now = Instant::now();
        let should_scan = self
            .last_scan
            .map(|last| now.duration_since(last).as_secs() >= RESCAN_EVERY_SECS)
            .unwrap_or(true);
        if should_scan {
            let root = self.root.clone().unwrap_or_else(sessions_dir);
            self.seen = scan_session_files(root);
            self.last_scan = Some(now);
        }

        let stale_cutoff = SystemTime::now()
            .checked_sub(std::time::Duration::from_secs(STALE_AFTER_SECS))
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let mut out = Vec::new();
        // Stable iteration order for deterministic, testable behavior.
        let mut candidates: Vec<&(PathBuf, SystemTime)> = self.seen.iter().collect();
        candidates.sort_by(|a, b| a.0.cmp(&b.0));

        for (path, mtime) in candidates {
            if *mtime > stale_cutoff {
                continue; // still growing — not ended yet
            }
            if self.emitted.contains(path) {
                continue; // already announced
            }
            // path is &PathBuf here; parse wants &Path, emitted wants PathBuf.
            if let Some(ended) = parse_ended_session(path.as_ref()) {
                out.push(ended.into_envelope());
                self.emitted.insert(path.clone());
            }
        }
        out
    }
}

/// Resolve the sessions root: env override, else `~/.pi/agent/sessions`.
fn sessions_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os(SESSIONS_DIR_ENV) {
        return PathBuf::from(dir);
    }
    let home = std::env::var_os("HOME").unwrap_or_default();
    PathBuf::from(home)
        .join(".pi")
        .join("agent")
        .join("sessions")
}

/// One-shot directory scan → (file, mtime) for every `*.jsonl` under root.
fn scan_session_files(root: PathBuf) -> Vec<(PathBuf, SystemTime)> {
    let subdirs = match fs::read_dir(&root) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect::<Vec<_>>(),
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for entry in subdirs {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let dir = match fs::read_dir(entry.path()) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for file in dir.flatten() {
            let path = file.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }
            let mtime = file
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            out.push((path, mtime));
        }
    }
    out
}

#[derive(Deserialize)]
struct SessionHeader {
    #[serde(rename = "type")]
    kind: Option<String>,
    id: Option<String>,
    cwd: Option<String>,
}

#[derive(Deserialize)]
struct LastLineMessage {
    message: Option<LastMessageInner>,
}

#[derive(Deserialize)]
struct LastMessageInner {
    #[serde(rename = "stopReason")]
    stop_reason: Option<String>,
}

/// Read the first line (session header) + last line (stopReason) + line count.
fn parse_ended_session(path: &Path) -> Option<EndedSession> {
    let bytes = fs::read_to_string(path).ok()?;
    let mut lines = bytes.lines().filter(|l| !l.is_empty());

    let first = lines.next()?;
    let header: SessionHeader = serde_json::from_str(first).ok()?;
    if header.kind.as_deref() != Some("session") {
        return None;
    }

    // Walk to the last non-empty line and count as we go (first already counted).
    let mut last_line = first;
    let mut event_count: u64 = 1;
    for line in lines {
        event_count += 1;
        last_line = line;
    }

    let last_stop_reason = serde_json::from_str::<LastLineMessage>(last_line)
        .ok()
        .and_then(|wrap| wrap.message)
        .and_then(|m| m.stop_reason);

    Some(EndedSession {
        session_id: header.id.unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string()
        }),
        cwd: header.cwd.unwrap_or_default(),
        host: hostname(),
        last_stop_reason,
        event_count,
        session_file: path.to_path_buf(),
    })
}

fn hostname() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::time::Duration;

    fn tmp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("herdr-pi-watch-{}-{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_session(root: &Path, cwd_dir: &str, id: &str, stop: Option<&str>) -> PathBuf {
        let session_dir = root.join(cwd_dir);
        fs::create_dir_all(&session_dir).unwrap();
        let path = session_dir.join(format!("{id}.jsonl"));
        let mut f = fs::File::create(&path).unwrap();
        writeln!(
            f,
            "{{\"type\":\"session\",\"id\":\"{id}\",\"cwd\":\"/home/joep\"}}"
        )
        .unwrap();
        if let Some(s) = stop {
            writeln!(
                f,
                "{{\"type\":\"message\",\"message\":{{\"stopReason\":\"{s}\"}}}}"
            )
            .unwrap();
        }
        // backdate mtime so it reads as stale
        let old = SystemTime::now() - Duration::from_secs(STALE_AFTER_SECS + 60);
        let _ = filetime::set_file_mtime(&path, filetime::FileTime::from_system_time(old));
        path
    }

    #[test]
    fn emits_for_stale_session_and_not_again() {
        let root = tmp("emits");
        write_session(&root, "--home-joep--", "abc123", Some("completed"));

        let mut w = PiSessionWatcher::for_root(root);

        let first = w.poll();
        assert_eq!(first.len(), 1, "first poll emits the ended session");
        let second = w.poll();
        assert!(second.is_empty(), "second poll does not re-emit");
    }

    #[test]
    fn ignores_still_growing_session() {
        let root = tmp("growing");
        let dir = root.join("--home-joep--");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("fresh.jsonl");
        fs::write(
            &path,
            "{\"type\":\"session\",\"id\":\"x\",\"cwd\":\"/h\"}\n",
        )
        .unwrap();
        // fresh mtime → not stale

        let mut w = PiSessionWatcher::for_root(root);
        assert!(w.poll().is_empty(), "freshly-written session is not ended");
    }

    #[test]
    fn parse_extracts_header_and_stop_reason() {
        let root = tmp("parse");
        let path = write_session(&root, "--home-joep--", "id-parse", Some("aborted"));
        let ended = parse_ended_session(&path).unwrap();
        assert_eq!(ended.session_id, "id-parse");
        assert_eq!(ended.cwd, "/home/joep");
        assert_eq!(ended.last_stop_reason.as_deref(), Some("aborted"));
        assert_eq!(ended.event_count, 2);
    }
}
