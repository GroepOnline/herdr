use serde::{Deserialize, Serialize};

pub(crate) const DEFAULT_TAB_BAR_COMMAND_INTERVAL_SECONDS: u64 = 5;
pub(crate) const DEFAULT_TAB_BAR_COMMAND_TIMEOUT_SECONDS: u64 = 2;
pub(crate) const MAX_TAB_BAR_COMMAND_INTERVAL_SECONDS: u64 = 31_536_000;
pub(crate) const MAX_TAB_BAR_COMMAND_TIMEOUT_SECONDS: u64 = 3_600;
pub(crate) const MAX_TAB_BAR_RIGHT_ENTRIES: usize = 16;

fn default_datetime_format() -> String {
    "%H:%M".to_string()
}

fn default_command_interval_seconds() -> u64 {
    DEFAULT_TAB_BAR_COMMAND_INTERVAL_SECONDS
}

fn default_command_timeout_seconds() -> u64 {
    DEFAULT_TAB_BAR_COMMAND_TIMEOUT_SECONDS
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TabBarRightEntryConfig {
    Zoom,
    Hostname,
    Datetime {
        #[serde(default = "default_datetime_format")]
        format: String,
    },
    Text {
        text: String,
    },
    Command {
        command: String,
        #[serde(default = "default_command_interval_seconds")]
        interval_seconds: u64,
        #[serde(default = "default_command_timeout_seconds")]
        timeout_seconds: u64,
    },
}

/// Validate a strftime format string for tab bar datetime entries.
///
/// We support a subset of strftime directives that map to server-local
/// wall-clock time. Directives requiring a UTC offset or Unix timestamp
/// (such as `%z` and `%s`) are rejected.
pub(crate) fn validate_tab_bar_datetime_format(value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err("datetime format is empty".into());
    }
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '%' {
            continue;
        }
        let directive = chars.next().ok_or("truncated strftime directive")?;
        // Reject directives that require a UTC offset or Unix timestamp.
        if matches!(directive, 'z' | 's' | 'Z') {
            return Err(format!(
                "unsupported datetime format: %{directive} requires a UTC offset or timestamp"
            ));
        }
    }
    Ok(())
}

pub(crate) fn tab_bar_right_diagnostics(entries: &[TabBarRightEntryConfig]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if entries.len() > MAX_TAB_BAR_RIGHT_ENTRIES {
        diagnostics.push(format!(
            "ui.tab_bar_right may contain at most {MAX_TAB_BAR_RIGHT_ENTRIES} entries; ignoring extras"
        ));
    }

    for (index, entry) in entries.iter().enumerate().take(MAX_TAB_BAR_RIGHT_ENTRIES) {
        match entry {
            TabBarRightEntryConfig::Datetime { format } => {
                if format.is_empty() {
                    diagnostics.push(format!(
                        "ui.tab_bar_right[{index}] datetime format is empty; hiding entry"
                    ));
                } else if let Err(err) = validate_tab_bar_datetime_format(format) {
                    diagnostics.push(format!("ui.tab_bar_right[{index}] has {err}; hiding entry"));
                }
            }
            TabBarRightEntryConfig::Command {
                command,
                interval_seconds,
                timeout_seconds,
            } => {
                if command.trim().is_empty() {
                    diagnostics.push(format!(
                        "ui.tab_bar_right[{index}] command is empty; hiding entry"
                    ));
                }
                if *interval_seconds == 0 {
                    diagnostics.push(format!(
                        "ui.tab_bar_right[{index}] interval_seconds must be at least 1; hiding entry"
                    ));
                }
                if *interval_seconds > MAX_TAB_BAR_COMMAND_INTERVAL_SECONDS {
                    diagnostics.push(format!(
                        "ui.tab_bar_right[{index}] interval_seconds may be at most {MAX_TAB_BAR_COMMAND_INTERVAL_SECONDS}; hiding entry"
                    ));
                }
                if *timeout_seconds == 0 {
                    diagnostics.push(format!(
                        "ui.tab_bar_right[{index}] timeout_seconds must be at least 1; hiding entry"
                    ));
                }
                if *timeout_seconds > MAX_TAB_BAR_COMMAND_TIMEOUT_SECONDS {
                    diagnostics.push(format!(
                        "ui.tab_bar_right[{index}] timeout_seconds may be at most {MAX_TAB_BAR_COMMAND_TIMEOUT_SECONDS}; hiding entry"
                    ));
                }
            }
            TabBarRightEntryConfig::Zoom
            | TabBarRightEntryConfig::Hostname
            | TabBarRightEntryConfig::Text { .. } => {}
        }
    }

    diagnostics
}

/// Format a `time::PrimitiveDateTime` using a strftime format string.
///
/// Supports the common directives: %Y, %m, %d, %H, %M, %S, %y, %p, %I.
/// Literal `%%` produces a single `%`.
pub(crate) fn format_datetime(dt: &time::PrimitiveDateTime, format: &str) -> String {
    let mut result = String::new();
    let mut chars = format.chars();
    while let Some(ch) = chars.next() {
        if ch != '%' {
            result.push(ch);
            continue;
        }
        let directive = match chars.next() {
            Some(d) => d,
            None => {
                result.push('%');
                break;
            }
        };
        match directive {
            '%' => result.push('%'),
            'Y' => result.push_str(&dt.year().to_string()),
            'y' => result.push_str(&format!("{:02}", dt.year() % 100)),
            'm' => result.push_str(&format!("{:02}", dt.month() as u8)),
            'd' => result.push_str(&format!("{:02}", dt.day())),
            'e' => result.push_str(&format!("{:2}", dt.day())),
            'H' => result.push_str(&format!("{:02}", dt.hour())),
            'M' => result.push_str(&format!("{:02}", dt.minute())),
            'S' => result.push_str(&format!("{:02}", dt.second())),
            'I' => {
                let h = dt.hour() % 12;
                let h = if h == 0 { 12 } else { h };
                result.push_str(&format!("{:02}", h));
            }
            'p' => {
                if dt.hour() < 12 {
                    result.push_str("AM");
                } else {
                    result.push_str("PM");
                }
            }
            'j' => result.push_str(&format!("{:03}", dt.day())),
            _ => {
                result.push('%');
                result.push(directive);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_bar_entries_parse_with_command_defaults() {
        #[derive(Deserialize)]
        struct Wrapper {
            entries: Vec<TabBarRightEntryConfig>,
        }

        let parsed: Wrapper = toml::from_str(
            r#"
entries = [
  { type = "zoom" },
  { type = "hostname" },
  { type = "datetime", format = "%H:%M" },
  { type = "text", text = "prod" },
  { type = "command", command = "status.sh" },
]
"#,
        )
        .expect("parse tab bar entries");

        assert_eq!(parsed.entries.len(), 5);
        assert!(matches!(
            &parsed.entries[4],
            TabBarRightEntryConfig::Command {
                interval_seconds: DEFAULT_TAB_BAR_COMMAND_INTERVAL_SECONDS,
                timeout_seconds: DEFAULT_TAB_BAR_COMMAND_TIMEOUT_SECONDS,
                ..
            }
        ));
    }

    #[test]
    fn diagnostics_reject_invalid_datetime_and_command_schedules() {
        let entries = vec![
            TabBarRightEntryConfig::Datetime {
                format: "%z".into(),
            },
            TabBarRightEntryConfig::Datetime {
                format: "%s".into(),
            },
            TabBarRightEntryConfig::Datetime {
                format: String::new(),
            },
            TabBarRightEntryConfig::Command {
                command: String::new(),
                interval_seconds: 0,
                timeout_seconds: 0,
            },
            TabBarRightEntryConfig::Command {
                command: "status.sh".into(),
                interval_seconds: MAX_TAB_BAR_COMMAND_INTERVAL_SECONDS + 1,
                timeout_seconds: MAX_TAB_BAR_COMMAND_TIMEOUT_SECONDS + 1,
            },
        ];

        let diagnostics = tab_bar_right_diagnostics(&entries).join("\n");
        assert!(diagnostics.contains("unsupported datetime format"));
        assert!(diagnostics.contains("datetime format is empty"));
        assert!(diagnostics.contains("command is empty"));
        assert!(diagnostics.contains("interval_seconds must be at least 1"));
        assert!(diagnostics.contains("interval_seconds may be at most"));
        assert!(diagnostics.contains("timeout_seconds must be at least 1"));
        assert!(diagnostics.contains("timeout_seconds may be at most"));
        assert!(validate_tab_bar_datetime_format("").is_err());
    }
}
