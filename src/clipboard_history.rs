//! Persistent clipboard history ("clipboard manager") for the fork.
//!
//! Every clipboard write that flows through the server
//! (`AppEvent::ClipboardWrite`) is recorded here, so users get a searchable
//! history instead of only the last item. The store is a small embedded
//! `redb` database; values are `bincode`-encoded entries.
//!
//! This module is intentionally non-fatal: any storage error is logged and
//! swallowed so clipboard behaviour is never broken by the history feature.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use redb::{
    Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition,
    WriteTransaction,
};

use crate::config::Config;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Entry {
    ts: u64,
    source: String,
    text: String,
}

/// One row returned to callers.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct ClipboardEntry {
    pub seq: u64,
    pub timestamp: u64,
    pub source: String,
    pub text: String,
}

const TABLE: TableDefinition<u64, &[u8]> = TableDefinition::new("clipboard_history");

fn default_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let base = match std::env::var("XDG_CONFIG_HOME") {
        Ok(x) if !x.is_empty() => PathBuf::from(x),
        _ => PathBuf::from(home).join(".config"),
    };
    base.join("herdr").join("clipboard.redb")
}

pub struct ClipboardHistory {
    db: Database,
    seq: AtomicU64,
    max_entries: u64,
    max_bytes: usize,
}

impl ClipboardHistory {
    fn open(cfg: &crate::config::ClipboardConfig) -> std::io::Result<Self> {
        let path = default_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::create(&path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let count = db
            .begin_read()
            .ok()
            .and_then(|tx| tx.open_table(TABLE).ok())
            .map(|t| t.len().unwrap_or(0))
            .unwrap_or(0);
        Ok(Self {
            db,
            seq: AtomicU64::new(count),
            max_entries: cfg.max_entries.max(1) as u64,
            max_bytes: cfg.max_bytes,
        })
    }

    /// Record a clipboard write. Non-UTF-8 and whitespace-only content is
    /// skipped. Oversized text is truncated to `max_bytes`.
    pub fn record(&self, content: &[u8], source: &str) {
        let text = match std::str::from_utf8(content) {
            Ok(t) => t,
            Err(_) => return,
        };
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        let text = if text.len() > self.max_bytes {
            &text[..self.max_bytes]
        } else {
            text
        };
        let entry = Entry {
            ts: now_ms(),
            source: source.to_string(),
            text: text.to_string(),
        };
        let encoded = match bincode::serde::encode_to_vec(&entry, bincode::config::standard()) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("clipboard history encode failed: {e}");
                return;
            }
        };
        let seq = self.seq.fetch_add(1, Ordering::SeqCst);
        let write_tx = match self.db.begin_write() {
            Ok(tx) => tx,
            Err(e) => {
                tracing::warn!("clipboard history begin_write failed: {e}");
                return;
            }
        };
        if let Err(e) = self.persist(write_tx, seq, &encoded) {
            tracing::warn!("clipboard history persist failed: {e}");
        }
    }

    fn persist(
        &self,
        write_tx: WriteTransaction,
        seq: u64,
        encoded: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut table = write_tx.open_table(TABLE)?;
            table.insert(&seq, encoded)?;
            let len = table.len()?;
            if len > self.max_entries {
                let to_delete = len - self.max_entries;
                let keys: Vec<u64> = table
                    .iter()?
                    .flatten()
                    .map(|(k, _)| k.value())
                    .take(to_delete as usize)
                    .collect();
                for key in keys {
                    table.remove(&key)?;
                }
            }
        }
        write_tx.commit()?;
        Ok(())
    }

    /// Return up to `limit` most-recent entries, newest first.
    pub fn recent(&self, limit: usize) -> Vec<ClipboardEntry> {
        let read_tx = match self.db.begin_read() {
            Ok(tx) => tx,
            Err(_) => return Vec::new(),
        };
        let table = match read_tx.open_table(TABLE) {
            Ok(t) => t,
            Err(_) => return Vec::new(),
        };
        let mut rows: Vec<ClipboardEntry> = Vec::new();
        let iter = match table.iter() {
            Ok(i) => i,
            Err(_) => return Vec::new(),
        };
        for item in iter.flatten() {
            let (k, v) = item;
            match bincode::serde::decode_from_slice::<Entry, _>(
                v.value(),
                bincode::config::standard(),
            ) {
                Ok((entry, _)) => rows.push(ClipboardEntry {
                    seq: k.value(),
                    timestamp: entry.ts,
                    source: entry.source,
                    text: entry.text,
                }),
                Err(_) => continue,
            }
        }
        rows.sort_by(|a, b| b.seq.cmp(&a.seq));
        rows.truncate(limit);
        rows
    }

    pub fn clear(&self) {
        if let Ok(write_tx) = self.db.begin_write() {
            if let Ok(mut table) = write_tx.open_table(TABLE) {
                let _ = table.retain(|_, _| false);
            }
            let _ = write_tx.commit();
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

static HISTORY: OnceLock<Option<ClipboardHistory>> = OnceLock::new();

/// Initialise the global history store from config. Safe to call multiple
/// times; only the first call has an effect.
pub fn init(config: &Config) {
    HISTORY.get_or_init(|| {
        if !config.clipboard.history_enabled {
            return None;
        }
        match ClipboardHistory::open(&config.clipboard) {
            Ok(h) => Some(h),
            Err(e) => {
                tracing::warn!("clipboard history disabled (open failed): {e}");
                None
            }
        }
    });
}

/// Record a clipboard write if the history store is enabled.
pub fn record_clipboard(content: &[u8]) {
    if let Some(Some(history)) = HISTORY.get() {
        history.record(content, "pane");
    }
}

/// Return up to `limit` most-recent clipboard entries (newest first), if the
/// history store is enabled. Empty when disabled or not yet initialised.
pub fn recent_global(limit: usize) -> Vec<ClipboardEntry> {
    HISTORY
        .get()
        .and_then(|opt| opt.as_ref())
        .map(|history| history.recent(limit))
        .unwrap_or_default()
}

/// Clear the clipboard history if the store is enabled.
pub fn clear_global() {
    if let Some(Some(history)) = HISTORY.get() {
        history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db() -> ClipboardHistory {
        static TMP: AtomicU64 = AtomicU64::new(0);
        let n = TMP.fetch_add(1, Ordering::SeqCst);
        let dir =
            std::env::temp_dir().join(format!("herdr-clip-test-{}-{}", std::process::id(), n));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let db = Database::create(dir.join("clipboard.redb")).unwrap();
        ClipboardHistory {
            db,
            seq: AtomicU64::new(0),
            max_entries: 3,
            max_bytes: 1_000_000,
        }
    }

    #[test]
    fn records_and_returns_newest_first() {
        let h = temp_db();
        h.record(b"first", "pane");
        h.record(b"second", "pane");
        h.record(b"third", "pane");
        let recent = h.recent(10);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].text, "third");
        assert_eq!(recent[2].text, "first");
    }

    #[test]
    fn prunes_to_max_entries() {
        let h = temp_db();
        for i in 0..5 {
            h.record(format!("item-{i}").as_bytes(), "pane");
        }
        let recent = h.recent(100);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].text, "item-4");
    }

    #[test]
    fn skips_non_utf8_and_blank() {
        let h = temp_db();
        h.record(&[0xff, 0xfe], "pane");
        h.record(b"   \n\t ", "pane");
        assert!(h.recent(10).is_empty());
    }
}
