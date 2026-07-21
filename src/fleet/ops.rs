use std::path::Path;
use std::time::Duration;

use crate::detect::AgentState;
use crate::terminal::state::TerminalState;

/// Fleet operations metadata for a single pane/agent.
/// Supplements (never overrides) the upstream semantic AgentState.
#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
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

#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
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

    /// Render a compact single-line status string.
    /// Format: `agent | state | repo:branch | model | host | elapsed`
    #[allow(dead_code)]
    pub fn render_bar(&self, agent_name: &str, state: AgentState, label: Option<&str>) -> String {
        let mut parts: Vec<String> = Vec::new();

        let display_name = label.unwrap_or(agent_name);
        parts.push(display_name.to_string());

        parts.push(state_label(state).to_string());

        if let Some(repo) = &self.repo {
            let git_info = match (&self.branch, &self.worktree) {
                (Some(br), Some(wt)) => format!("{}:{} ({})", repo, br, wt),
                (Some(br), None) => format!("{}:{}", repo, br),
                (None, _) => repo.clone(),
            };
            parts.push(git_info);
        }

        if let Some(issue) = &self.linear_issue {
            parts.push(format!("LIN-{}", issue));
        }

        if let Some(pr) = self.github_pr {
            let ci = match &self.ci_status {
                Some(CiStatus::Success) => " OK",
                Some(CiStatus::Failed) => " FAIL",
                Some(CiStatus::Running) => " ...",
                _ => "",
            };
            parts.push(format!("#{}{}", pr, ci));
        }

        if let Some(model) = &self.model {
            let provider = self.provider.as_deref().unwrap_or("");
            if provider.is_empty() {
                parts.push(model.clone());
            } else {
                parts.push(format!("{}/{}", provider, model));
            }
        }

        parts.push(self.host.clone());

        if let Some(elapsed) = self.elapsed {
            parts.push(format_duration(elapsed));
        }

        if self.retry_count > 0 {
            parts.push(format!("retry:{}", self.retry_count));
        }

        if self.session_resume_available {
            parts.push("resume".to_string());
        }

        parts.join(" | ")
    }
}

#[allow(dead_code)]
fn state_label(state: AgentState) -> &'static str {
    match state {
        AgentState::Idle => "idle",
        AgentState::Working => "working",
        AgentState::Blocked => "blocked",
        AgentState::Unknown => "unknown",
    }
}

#[allow(dead_code)]
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
    fn test_render_bar_minimal() {
        let meta = FleetOpsMetadata {
            host: "sofie".to_string(),
            ..Default::default()
        };
        let bar = meta.render_bar("claude", AgentState::Idle, None);
        assert!(bar.contains("claude"));
        assert!(bar.contains("idle"));
        assert!(bar.contains("sofie"));
    }

    #[test]
    fn test_render_bar_full() {
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
        let bar = meta.render_bar("claude", AgentState::Working, Some("chef-bot"));
        assert!(bar.contains("chef-bot"));
        assert!(bar.contains("working"));
        assert!(bar.contains("herdr:main (feature)"));
        assert!(bar.contains("zai/glm-5.2"));
        assert!(bar.contains("#42 OK"));
        assert!(bar.contains("5m"));
        assert!(bar.contains("resume"));
    }
}
