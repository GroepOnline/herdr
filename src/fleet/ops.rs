use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Deserialize;

use crate::detect::AgentState;
use crate::plugin_paths::plugin_state_dir;
use crate::terminal::state::TerminalState;

/// Plugin ids that contribute `fleet_ops.json` fragments for the CHEF bar.
pub const FLEET_OPS_PLUGIN_IDS: &[&str] = &[
    "com.chefgroep.linear-context",
    "com.chefgroep.github-status",
    "com.chefgroep.fleet-health",
    "com.chefgroep.cloudflare-tunnel",
    "com.chefgroep.kater-bridge",
    "com.chefgroep.udo-metrics",
    "com.chefgroep.session-park",
];

#[derive(Debug, Clone, Default, Deserialize)]
#[allow(dead_code)] // reserved JSON fields for future fleet/cloudflare/parked bar segments
pub struct PluginFleetFragment {
    pub source: Option<String>,
    pub updated_at: Option<String>,
    pub ttl_seconds: Option<u64>,
    pub issue: Option<PluginIssueFragment>,
    pub pr: Option<PluginPrFragment>,
    pub fleet: Option<PluginFleetSummary>,
    pub cloudflare: Option<PluginCloudflareSummary>,
    pub parked: Option<PluginParkedSummary>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[allow(dead_code)] // title/status reserved for richer Linear bar segments
pub struct PluginIssueFragment {
    pub id: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub assignee: Option<String>,
    pub cycle: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PluginPrFragment {
    pub number: Option<u32>,
    pub checks: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[allow(dead_code)]
pub struct PluginFleetSummary {
    pub online: Option<u32>,
    pub total: Option<u32>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[allow(dead_code)]
pub struct PluginCloudflareSummary {
    pub tunnels_healthy: Option<u32>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[allow(dead_code)]
pub struct PluginParkedSummary {
    pub count: Option<u32>,
    pub oldest_hours: Option<u32>,
}

/// Fleet operations metadata for a single pane/agent.
/// Supplements (never overrides) the upstream semantic AgentState.
#[derive(Clone, Debug, Default)]
pub struct FleetOpsMetadata {
    pub repo: Option<String>,
    pub worktree: Option<String>,
    pub branch: Option<String>,
    pub linear_issue: Option<String>,
    pub linear_assignee: Option<String>,
    pub linear_cycle: Option<String>,
    pub github_pr: Option<u32>,
    pub ci_status: Option<CiStatus>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub host: String,
    pub elapsed: Option<Duration>,
    pub retry_count: u32,
    pub session_resume_available: bool,
}

/// Semantic segment kinds for fleet ops bar rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FleetOpsBarKind {
    Name,
    State,
    Git,
    Linear,
    Pr,
    Model,
    Host,
    Elapsed,
    Retry,
    Resume,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FleetOpsBarPart {
    pub kind: FleetOpsBarKind,
    pub text: String,
}

/// CI status values reserved for PR/CI wiring in the fleet ops bar.
#[derive(Clone, Debug, Default)]
#[allow(dead_code)] // Pending/Cancelled reserved for CI wiring beyond success/fail/running suffixes
pub enum CiStatus {
    #[default]
    Pending,
    Running,
    Success,
    Failed,
    Cancelled,
}

impl FleetOpsMetadata {
    /// Build metadata from terminal state + runtime context.
    pub fn from_terminal(term: &TerminalState, host: &str) -> Self {
        let (repo, worktree, branch) = derive_git_context(&term.cwd);

        let mut meta = FleetOpsMetadata {
            repo,
            worktree,
            branch,
            host: host.to_string(),
            elapsed: None,
            retry_count: 0,
            session_resume_available: term.persisted_agent_session.is_some(),
            model: term
                .agent_metadata
                .values()
                .filter_map(|m| m.display_agent.clone())
                .next(),
            provider: term
                .agent_metadata
                .values()
                .filter_map(|m| m.source.clone().into())
                .next(),
            ..Default::default()
        };
        meta.merge_plugin_fragments(&load_plugin_fleet_fragments());
        meta
    }

    /// Merge CHEF plugin `fleet_ops.json` fragments (Linear/GitHub/fleet/…).
    pub fn merge_plugin_fragments(&mut self, fragments: &[PluginFleetFragment]) {
        for fragment in fragments {
            if fragment_expired(fragment) {
                continue;
            }
            if let Some(issue) = &fragment.issue {
                if let Some(id) = issue.id.as_ref().filter(|id| !id.is_empty()) {
                    self.linear_issue = Some(id.clone());
                }
                if let Some(assignee) = issue.assignee.as_ref().filter(|v| !v.is_empty()) {
                    self.linear_assignee = Some(assignee.clone());
                }
                if let Some(cycle) = issue.cycle.as_ref().filter(|v| !v.is_empty()) {
                    self.linear_cycle = Some(cycle.clone());
                }
            }
            if let Some(pr) = &fragment.pr {
                if let Some(number) = pr.number {
                    self.github_pr = Some(number);
                }
                if let Some(checks) = pr.checks.as_deref() {
                    self.ci_status = Some(match checks.to_ascii_lowercase().as_str() {
                        "passing" | "success" | "ok" => CiStatus::Success,
                        "failing" | "failed" | "fail" => CiStatus::Failed,
                        "running" | "pending" => CiStatus::Running,
                        _ => CiStatus::Pending,
                    });
                }
            }
        }
    }

    /// Personal one-line summary used by the settings Fleet tab / toasts.
    pub fn personal_summary_line(&self) -> String {
        let mut parts = Vec::new();
        if let Some(issue) = &self.linear_issue {
            parts.push(format!("ENG-{issue}"));
        }
        if let Some(assignee) = self.linear_assignee.as_ref().filter(|v| !v.is_empty()) {
            parts.push(assignee.clone());
        }
        if let Some(cycle) = self.linear_cycle.as_ref().filter(|v| !v.is_empty()) {
            parts.push(cycle.clone());
        }
        if let Some(pr) = self.github_pr {
            let ci = match &self.ci_status {
                Some(CiStatus::Success) => " ✓",
                Some(CiStatus::Failed) => " ✗",
                Some(CiStatus::Running) => " …",
                _ => "",
            };
            parts.push(format!("PR #{pr}{ci}"));
        }
        if !self.host.is_empty() {
            parts.push(self.host.clone());
        }
        if parts.is_empty() {
            "fleet ops idle".into()
        } else {
            parts.join(" · ")
        }
    }

    /// Structured bar segments shared by plain-text and styled UI renderers.
    pub fn bar_parts(
        &self,
        agent_name: &str,
        state: AgentState,
        label: Option<&str>,
    ) -> Vec<FleetOpsBarPart> {
        let mut parts = Vec::new();

        parts.push(FleetOpsBarPart {
            kind: FleetOpsBarKind::Name,
            text: label.unwrap_or(agent_name).to_string(),
        });

        parts.push(FleetOpsBarPart {
            kind: FleetOpsBarKind::State,
            text: state_label(state).to_string(),
        });

        if let Some(repo) = &self.repo {
            let git_info = match (&self.branch, &self.worktree) {
                (Some(br), Some(wt)) => format!("{}:{} ({})", repo, br, wt),
                (Some(br), None) => format!("{}:{}", repo, br),
                (None, _) => repo.clone(),
            };
            parts.push(FleetOpsBarPart {
                kind: FleetOpsBarKind::Git,
                text: git_info,
            });
        }

        if let Some(issue) = &self.linear_issue {
            let mut text = format!("ENG-{issue}");
            if let Some(assignee) = self.linear_assignee.as_ref().filter(|v| !v.is_empty()) {
                text.push_str(" · ");
                text.push_str(assignee);
            }
            if let Some(cycle) = self.linear_cycle.as_ref().filter(|v| !v.is_empty()) {
                text.push_str(" · ");
                text.push_str(cycle);
            }
            parts.push(FleetOpsBarPart {
                kind: FleetOpsBarKind::Linear,
                text,
            });
        }

        if let Some(pr) = self.github_pr {
            let ci = match &self.ci_status {
                Some(CiStatus::Success) => " OK",
                Some(CiStatus::Failed) => " FAIL",
                Some(CiStatus::Running) => " ...",
                _ => "",
            };
            parts.push(FleetOpsBarPart {
                kind: FleetOpsBarKind::Pr,
                text: format!("#{}{}", pr, ci),
            });
        }

        if let Some(model) = &self.model {
            let provider = self.provider.as_deref().unwrap_or("");
            let text = if provider.is_empty() {
                model.clone()
            } else {
                format!("{}/{}", provider, model)
            };
            parts.push(FleetOpsBarPart {
                kind: FleetOpsBarKind::Model,
                text,
            });
        }

        parts.push(FleetOpsBarPart {
            kind: FleetOpsBarKind::Host,
            text: self.host.clone(),
        });

        if let Some(elapsed) = self.elapsed {
            parts.push(FleetOpsBarPart {
                kind: FleetOpsBarKind::Elapsed,
                text: format_duration(elapsed),
            });
        }

        if self.retry_count > 0 {
            parts.push(FleetOpsBarPart {
                kind: FleetOpsBarKind::Retry,
                text: format!("retry:{}", self.retry_count),
            });
        }

        if self.session_resume_available {
            parts.push(FleetOpsBarPart {
                kind: FleetOpsBarKind::Resume,
                text: "resume".to_string(),
            });
        }

        parts
    }
}

fn state_label(state: AgentState) -> &'static str {
    match state {
        AgentState::Idle => "idle",
        AgentState::Working => "working",
        AgentState::Blocked => "blocked",
        AgentState::Unknown => "unknown",
    }
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else {
        format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
    }
}

fn derive_git_context(cwd: &Path) -> (Option<String>, Option<String>, Option<String>) {
    let repo = find_git_repo(cwd);
    let repo_name = repo
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.trim_end_matches(".git").to_string());

    let branch = repo
        .as_ref()
        .and_then(|r| std::fs::read_to_string(r.join(".git/HEAD")).ok())
        .and_then(|head| {
            head.strip_prefix("ref: refs/heads/")
                .map(|b| b.trim().to_string())
        });

    let worktree = if repo.is_some() {
        cwd.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    } else {
        None
    };

    (repo_name, worktree, branch)
}

fn find_git_repo(path: &Path) -> Option<std::path::PathBuf> {
    let mut current = path;
    loop {
        if current.join(".git").exists() {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
}

pub fn load_plugin_fleet_fragments() -> Vec<PluginFleetFragment> {
    FLEET_OPS_PLUGIN_IDS
        .iter()
        .filter_map(|plugin_id| read_plugin_fleet_fragment(&plugin_state_dir(plugin_id)))
        .collect()
}

fn read_plugin_fleet_fragment(state_dir: &Path) -> Option<PluginFleetFragment> {
    let path = state_dir.join("fleet_ops.json");
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn fragment_expired(fragment: &PluginFleetFragment) -> bool {
    let Some(ttl) = fragment.ttl_seconds.filter(|ttl| *ttl > 0) else {
        return false;
    };
    let Some(updated_at) = fragment.updated_at.as_deref() else {
        return false;
    };
    let Ok(parsed) = chrono_lite_parse_rfc3339(updated_at) else {
        return false;
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    now.saturating_sub(parsed) > ttl
}

/// Minimal RFC3339 / ISO-8601 parser for `YYYY-MM-DDTHH:MM:SSZ` style stamps.
fn chrono_lite_parse_rfc3339(value: &str) -> Result<u64, ()> {
    // Prefer unix seconds if plugins write them.
    if let Ok(secs) = value.parse::<u64>() {
        return Ok(secs);
    }
    let trimmed = value.trim().trim_end_matches('Z');
    let (date, time) = trimmed.split_once('T').ok_or(())?;
    let mut date_parts = date.split('-');
    let year: i64 = date_parts.next().ok_or(())?.parse().map_err(|_| ())?;
    let month: u64 = date_parts.next().ok_or(())?.parse().map_err(|_| ())?;
    let day: u64 = date_parts.next().ok_or(())?.parse().map_err(|_| ())?;
    let mut time_parts = time.split(':');
    let hour: u64 = time_parts.next().ok_or(())?.parse().map_err(|_| ())?;
    let minute: u64 = time_parts.next().ok_or(())?.parse().map_err(|_| ())?;
    let second_str = time_parts.next().unwrap_or("0");
    let second: u64 = second_str
        .split(['.', '+', '-'])
        .next()
        .unwrap_or("0")
        .parse()
        .map_err(|_| ())?;
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return Err(());
    }
    // Approximate days-from-civil (Howard Hinnant) → unix seconds.
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = (y - era * 400) as u64;
    let mp = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = (era * 146097) as i64 + doe as i64 - 719468;
    Ok((days * 86400) as u64 + hour * 3600 + minute * 60 + second)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_labels() {
        assert_eq!(state_label(AgentState::Idle), "idle");
        assert_eq!(state_label(AgentState::Working), "working");
        assert_eq!(state_label(AgentState::Blocked), "blocked");
        assert_eq!(state_label(AgentState::Unknown), "unknown");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m");
        assert_eq!(format_duration(Duration::from_secs(3700)), "1h1m");
    }

    #[test]
    fn test_bar_parts_minimal() {
        let meta = FleetOpsMetadata {
            host: "sofie".to_string(),
            ..Default::default()
        };
        let texts: Vec<_> = meta
            .bar_parts("claude", AgentState::Idle, None)
            .into_iter()
            .map(|part| part.text)
            .collect();
        assert!(texts.iter().any(|t| t == "claude"));
        assert!(texts.iter().any(|t| t == "idle"));
        assert!(texts.iter().any(|t| t == "sofie"));
    }

    #[test]
    fn test_bar_parts_full() {
        let meta = FleetOpsMetadata {
            repo: Some("herdr".to_string()),
            branch: Some("main".to_string()),
            worktree: Some("feature".to_string()),
            model: Some("glm-5.2".to_string()),
            provider: Some("zai".to_string()),
            host: "sofie".to_string(),
            elapsed: Some(Duration::from_secs(300)),
            github_pr: Some(42),
            ci_status: Some(CiStatus::Success),
            session_resume_available: true,
            ..Default::default()
        };
        let texts: Vec<_> = meta
            .bar_parts("claude", AgentState::Working, Some("chef-bot"))
            .into_iter()
            .map(|part| part.text)
            .collect();
        assert!(texts.iter().any(|t| t == "chef-bot"));
        assert!(texts.iter().any(|t| t == "working"));
        assert!(texts.iter().any(|t| t == "herdr:main (feature)"));
        assert!(texts.iter().any(|t| t == "zai/glm-5.2"));
        assert!(texts.iter().any(|t| t == "#42 OK"));
        assert!(texts.iter().any(|t| t == "5m"));
        assert!(texts.iter().any(|t| t == "resume"));
    }

    #[test]
    fn test_bar_parts_include_resume() {
        let meta = FleetOpsMetadata {
            host: "local".to_string(),
            session_resume_available: true,
            ..Default::default()
        };
        let parts = meta.bar_parts("claude", AgentState::Idle, None);
        assert!(parts
            .iter()
            .any(|p| p.kind == FleetOpsBarKind::Resume && p.text == "resume"));
    }

    #[test]
    fn merge_plugin_fragments_fills_linear_and_pr() {
        let mut meta = FleetOpsMetadata {
            host: "sofie".into(),
            ..Default::default()
        };
        meta.merge_plugin_fragments(&[PluginFleetFragment {
            source: Some("linear-context".into()),
            ttl_seconds: Some(0),
            issue: Some(PluginIssueFragment {
                id: Some("432".into()),
                assignee: Some("joep".into()),
                cycle: Some("Sprint".into()),
                ..Default::default()
            }),
            pr: Some(PluginPrFragment {
                number: Some(42),
                checks: Some("passing".into()),
            }),
            ..Default::default()
        }]);
        assert_eq!(meta.linear_issue.as_deref(), Some("432"));
        assert_eq!(meta.linear_assignee.as_deref(), Some("joep"));
        assert_eq!(meta.github_pr, Some(42));
        assert!(matches!(meta.ci_status, Some(CiStatus::Success)));
        let summary = meta.personal_summary_line();
        assert!(summary.contains("ENG-432"));
        assert!(summary.contains("joep"));
        assert!(summary.contains("Sprint"));
        assert!(summary.contains("PR #42"));
    }
}
