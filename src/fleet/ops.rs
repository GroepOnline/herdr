use std::path::Path;
use std::time::Duration;

use crate::detect::AgentState;
use crate::terminal::state::TerminalState;

/// Fleet operations metadata for a single pane/agent.
/// Supplements (never overrides) the upstream semantic AgentState.
#[derive(Clone, Debug, Default)]
pub struct FleetOpsMetadata {
    pub repo: Option<String>,
    pub worktree: Option<String>,
    pub branch: Option<String>,
    pub linear_issue: Option<String>,
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

        FleetOpsMetadata {
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
            parts.push(FleetOpsBarPart {
                kind: FleetOpsBarKind::Linear,
                text: format!("LIN-{}", issue),
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
}
