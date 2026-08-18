//! Read-only access to OpenCode's SQLite session store.
//!
//! OpenCode's plugin/socket integration remains the live authority for pane
//! state and session identity. This module is only used during offline restore
//! to avoid resuming an OpenCode session that a present, readable database can
//! prove no longer exists. It never reads or writes JSONL, messages, prompts,
//! or other session payloads.

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OpenFlags};
use tracing::{debug, warn};

/// Explicit override for installations whose OpenCode data directory is not at
/// the platform default. The value is a directory containing the database files.
const OPENCODE_DATA_DIR_ENV_VAR: &str = "OPENCODE_DATA_DIR";
const OPENCODE_DB_NAMES: [&str; 2] = ["opencode-next.db", "opencode.db"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DatabaseLookup {
    Found,
    NotFound,
    Unavailable,
}

/// Returns whether an OpenCode session should be resumed.
///
/// A readable database containing the `session` table is authoritative for a
/// negative lookup. The preferred database is `opencode-next.db`; the legacy
/// database is consulted only when the preferred database is unavailable or
/// has no compatible schema. If no usable database is available, preserve the
/// existing snapshot-only behavior so restores still work for remote, moved,
/// or older OpenCode installations whose data directory is not visible locally.
pub(crate) fn should_resume_session(session_id: &str) -> bool {
    let paths = database_paths();

    for path in paths {
        match lookup_session(&path, session_id) {
            DatabaseLookup::Found => return true,
            DatabaseLookup::NotFound => {
                debug!(path = %path.display(), "OpenCode session is absent from the active SQLite database");
                return false;
            }
            DatabaseLookup::Unavailable => {}
        }
    }

    debug!(
        session_id,
        "OpenCode SQLite session store unavailable; preserving snapshot restore"
    );
    true
}

fn database_paths() -> Vec<PathBuf> {
    let Some(data_dir) = data_directory() else {
        return Vec::new();
    };

    OPENCODE_DB_NAMES
        .iter()
        .map(|name| data_dir.join(name))
        .collect()
}

fn data_directory() -> Option<PathBuf> {
    if let Some(value) =
        std::env::var_os(OPENCODE_DATA_DIR_ENV_VAR).filter(|value| !value.is_empty())
    {
        return Some(PathBuf::from(value));
    }

    if let Some(value) = std::env::var_os("XDG_DATA_HOME").filter(|value| !value.is_empty()) {
        return Some(PathBuf::from(value).join("opencode"));
    }

    #[cfg(target_os = "macos")]
    if let Some(home) = std::env::var_os("HOME").filter(|value| !value.is_empty()) {
        return Some(PathBuf::from(home).join("Library/Application Support/opencode"));
    }

    #[cfg(windows)]
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA").filter(|value| !value.is_empty())
    {
        return Some(PathBuf::from(local_app_data).join("opencode"));
    }

    std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(|value| PathBuf::from(value).join(".local/share/opencode"))
}

fn lookup_session(path: &Path, session_id: &str) -> DatabaseLookup {
    if !path.is_file() {
        return DatabaseLookup::Unavailable;
    }

    let connection = match Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(connection) => connection,
        Err(error) => {
            warn!(path = %path.display(), %error, "unable to open OpenCode SQLite database read-only");
            return DatabaseLookup::Unavailable;
        }
    };

    if !has_session_schema(&connection) {
        debug!(path = %path.display(), "OpenCode SQLite database has no compatible session schema");
        return DatabaseLookup::Unavailable;
    }

    match connection.query_row(
        "SELECT 1 FROM session WHERE id = ?1 LIMIT 1",
        params![session_id],
        |_| Ok(()),
    ) {
        Ok(()) => DatabaseLookup::Found,
        Err(rusqlite::Error::QueryReturnedNoRows) => DatabaseLookup::NotFound,
        Err(error) => {
            warn!(path = %path.display(), %error, "unable to query OpenCode SQLite sessions");
            DatabaseLookup::Unavailable
        }
    }
}

fn has_session_schema(connection: &Connection) -> bool {
    let Ok(mut statement) = connection.prepare("PRAGMA table_info(session)") else {
        return false;
    };

    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .collect::<std::collections::HashSet<_>>();

    columns.contains("id") && columns.contains("directory")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_database_path(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("herdr-opencode-{label}-{nonce}.db"))
    }

    fn create_database(path: &Path, schema: &str, session_id: Option<&str>) {
        let connection = Connection::open(path).expect("test database should open");
        connection
            .execute_batch(schema)
            .expect("test schema should be valid");
        if let Some(session_id) = session_id {
            connection
                .execute("INSERT INTO session (id) VALUES (?1)", params![session_id])
                .expect("test session should insert");
        }
        drop(connection);
    }

    fn remove_database(path: &Path) {
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(path.with_extension("db-wal"));
        let _ = fs::remove_file(path.with_extension("db-shm"));
    }

    fn with_data_dir<T>(data_dir: &Path, test: impl FnOnce() -> T) -> T {
        let _lock = crate::integration::integration_env_lock();
        let previous = std::env::var_os(OPENCODE_DATA_DIR_ENV_VAR);
        std::env::set_var(OPENCODE_DATA_DIR_ENV_VAR, data_dir);
        let result = test();
        match previous {
            Some(value) => std::env::set_var(OPENCODE_DATA_DIR_ENV_VAR, value),
            None => std::env::remove_var(OPENCODE_DATA_DIR_ENV_VAR),
        }
        result
    }

    #[test]
    fn readable_session_schema_finds_exact_id_without_reading_payloads() {
        let path = temp_database_path("exact-id");
        create_database(
            &path,
            "CREATE TABLE session (id TEXT PRIMARY KEY, directory TEXT, data TEXT);",
            Some("session-a"),
        );

        assert_eq!(lookup_session(&path, "session-a"), DatabaseLookup::Found);
        assert_eq!(lookup_session(&path, "session-b"), DatabaseLookup::NotFound);

        remove_database(&path);
    }

    #[test]
    fn incompatible_schema_is_not_authoritative() {
        let path = temp_database_path("schema");
        create_database(&path, "CREATE TABLE other (id TEXT PRIMARY KEY);", None);

        assert_eq!(
            lookup_session(&path, "session-a"),
            DatabaseLookup::Unavailable
        );

        remove_database(&path);
    }

    #[test]
    fn lookup_is_read_only() {
        let path = temp_database_path("readonly");
        create_database(
            &path,
            "CREATE TABLE session (id TEXT PRIMARY KEY, directory TEXT);",
            Some("session-a"),
        );

        let connection = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("read-only connection should open");
        assert!(connection
            .execute("INSERT INTO session (id) VALUES ('should-fail')", [])
            .is_err());
        drop(connection);

        remove_database(&path);
    }

    #[test]
    fn compatible_preferred_database_is_authoritative_over_legacy_database() {
        let data_dir =
            std::env::temp_dir().join(format!("herdr-opencode-preferred-{}", std::process::id()));
        let _ = fs::create_dir_all(&data_dir);
        let preferred = data_dir.join("opencode-next.db");
        let legacy = data_dir.join("opencode.db");
        create_database(
            &preferred,
            "CREATE TABLE session (id TEXT PRIMARY KEY, directory TEXT);",
            None,
        );
        create_database(
            &legacy,
            "CREATE TABLE session (id TEXT PRIMARY KEY, directory TEXT);",
            Some("legacy-only-session"),
        );

        with_data_dir(&data_dir, || {
            assert!(!should_resume_session("legacy-only-session"));
        });

        remove_database(&preferred);
        remove_database(&legacy);
        let _ = fs::remove_dir(&data_dir);
    }

    #[test]
    fn incompatible_preferred_database_falls_back_to_legacy_database() {
        let data_dir =
            std::env::temp_dir().join(format!("herdr-opencode-fallback-{}", std::process::id()));
        let _ = fs::create_dir_all(&data_dir);
        let preferred = data_dir.join("opencode-next.db");
        let legacy = data_dir.join("opencode.db");
        create_database(
            &preferred,
            "CREATE TABLE other (id TEXT PRIMARY KEY);",
            None,
        );
        create_database(
            &legacy,
            "CREATE TABLE session (id TEXT PRIMARY KEY, directory TEXT);",
            Some("legacy-session"),
        );

        with_data_dir(&data_dir, || {
            assert!(should_resume_session("legacy-session"));
            assert!(!should_resume_session("missing-session"));
        });

        remove_database(&preferred);
        remove_database(&legacy);
        let _ = fs::remove_dir(&data_dir);
    }

    #[test]
    fn unavailable_databases_preserve_snapshot_restore() {
        let data_dir =
            std::env::temp_dir().join(format!("herdr-opencode-missing-{}", std::process::id()));
        let _ = fs::create_dir_all(&data_dir);

        with_data_dir(&data_dir, || {
            assert!(should_resume_session("session-not-locally-visible"));
        });

        let _ = fs::remove_dir(&data_dir);
    }
}
