//! GitHub credential and repository helpers for status refresh.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Open PR/issue counts for one workspace after a successful refresh.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceGithubStatus {
    pub workspace_id: String,
    pub status: crate::workspace::GithubStatus,
}

/// Load a GitHub.com token from env, `gh auth token`, or `~/.config/gh/hosts.yml`.
///
/// Preference order:
/// 1. `GH_TOKEN` / `GITHUB_TOKEN`
/// 2. `gh auth token` subprocess
/// 3. `oauth_token` under the `github.com:` section of hosts.yml (enterprise hosts skipped)
pub fn load_github_token() -> Option<String> {
    for key in ["GH_TOKEN", "GITHUB_TOKEN"] {
        if let Ok(token) = std::env::var(key) {
            let token = token.trim().to_string();
            if !token.is_empty() {
                return Some(token);
            }
        }
    }

    if let Some(token) = load_token_from_gh_cli() {
        return Some(token);
    }

    let hosts_path = gh_hosts_path()?;
    let content = std::fs::read_to_string(hosts_path).ok()?;
    parse_github_com_oauth_token(&content)
}

fn load_token_from_gh_cli() -> Option<String> {
    let output = Command::new("gh").args(["auth", "token"]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let token = String::from_utf8(output.stdout).ok()?;
    let token = token.trim().to_string();
    (!token.is_empty()).then_some(token)
}

fn gh_hosts_path() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        let path = Path::new(&home).join(".config/gh/hosts.yml");
        if path.is_file() {
            return Some(path);
        }
    }
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            let path = Path::new(&appdata).join("GitHub CLI").join("hosts.yml");
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

/// Parse `oauth_token` from the `github.com:` section of a gh hosts.yml document.
pub fn parse_github_com_oauth_token(content: &str) -> Option<String> {
    let mut in_github_com = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let indent = line.len() - line.trim_start().len();
        if indent == 0 {
            in_github_com = trimmed == "github.com:" || trimmed == "github.com";
            continue;
        }

        if !in_github_com {
            continue;
        }

        let Some(value) = trimmed.strip_prefix("oauth_token:") else {
            continue;
        };
        let token = strip_yaml_quotes(value.trim());
        if !token.is_empty() {
            return Some(token.to_string());
        }
    }
    None
}

fn strip_yaml_quotes(value: &str) -> &str {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
        {
            return &value[1..value.len() - 1];
        }
    }
    value
}

/// True when `segment` is a safe GitHub path segment for URL interpolation.
pub fn is_valid_repo_path_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

/// Parse `owner/repo` from a github.com remote URL (https or ssh).
pub fn parse_github_owner_repo(url: &str) -> Option<(String, String)> {
    let url = url.trim().trim_end_matches('/').trim_end_matches(".git");
    let path = if let Some(rest) = url.split_once("github.com:") {
        rest.1
    } else if let Some(rest) = url.split_once("github.com/") {
        rest.1
    } else {
        return None;
    };

    let mut parts = path.split('/');
    let owner = parts.next()?.trim();
    let repo = parts.next()?.trim();
    if parts.next().is_some() {
        // Extra path segments are unexpected for a repo remote.
        return None;
    }
    if !is_valid_repo_path_segment(owner) || !is_valid_repo_path_segment(repo) {
        return None;
    }
    Some((owner.to_string(), repo.to_string()))
}

/// Discover origin github.com owner/repo by walking parents for `.git/config`.
pub fn discover_github_owner_repo(cwd: &Path) -> Option<(String, String)> {
    let config_path = find_git_config(cwd)?;
    let content = std::fs::read_to_string(config_path).ok()?;
    let mut in_origin = false;
    for line in content.lines() {
        let line = line.trim();
        if line == "[remote \"origin\"]" {
            in_origin = true;
        } else if line.starts_with('[') {
            in_origin = false;
        } else if in_origin {
            if let Some(url) = line
                .strip_prefix("url = ")
                .or_else(|| line.strip_prefix("url="))
            {
                return parse_github_owner_repo(url.trim());
            }
        }
    }
    None
}

fn find_git_config(start: &Path) -> Option<PathBuf> {
    let mut root = start.to_path_buf();
    loop {
        let config_path = root.join(".git/config");
        if config_path.is_file() {
            return Some(config_path);
        }
        // Linked worktree: `.git` is a file with `gitdir: ...`.
        let git_file = root.join(".git");
        if git_file.is_file() {
            if let Ok(content) = std::fs::read_to_string(&git_file) {
                for line in content.lines() {
                    if let Some(gitdir) = line.trim().strip_prefix("gitdir:") {
                        let gitdir = gitdir.trim();
                        let gitdir_path = if Path::new(gitdir).is_absolute() {
                            PathBuf::from(gitdir)
                        } else {
                            root.join(gitdir)
                        };
                        let config_path = gitdir_path.join("config");
                        if config_path.is_file() {
                            return Some(config_path);
                        }
                        // Common linked-worktree layout: gitdir points at
                        // `<repo>/.git/worktrees/<name>`; config lives at `<repo>/.git/config`.
                        if let Some(parent) = gitdir_path.parent().and_then(|p| p.parent()) {
                            let config_path = parent.join("config");
                            if config_path.is_file() {
                                return Some(config_path);
                            }
                        }
                    }
                }
            }
        }

        let parent = root.parent()?;
        if parent == root {
            return None;
        }
        root = parent.to_path_buf();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_github_com_token_and_skips_enterprise() {
        let content = r#"
github.example.com:
    oauth_token: enterprise-secret
github.com:
    user: alice
    oauth_token: "gho_github_token"
other.com:
    oauth_token: other-secret
"#;
        assert_eq!(
            parse_github_com_oauth_token(content).as_deref(),
            Some("gho_github_token")
        );
    }

    #[test]
    fn ignores_enterprise_only_hosts_yml() {
        let content = r#"
github.example.com:
    oauth_token: enterprise-secret
"#;
        assert_eq!(parse_github_com_oauth_token(content), None);
    }

    #[test]
    fn validates_repo_path_segments() {
        assert!(is_valid_repo_path_segment("OnlineChefGroep"));
        assert!(is_valid_repo_path_segment("herdr"));
        assert!(is_valid_repo_path_segment("foo.bar_baz-1"));
        assert!(!is_valid_repo_path_segment(""));
        assert!(!is_valid_repo_path_segment("foo/bar"));
        assert!(!is_valid_repo_path_segment("foo bar"));
        assert!(!is_valid_repo_path_segment("../etc"));
    }

    #[test]
    fn parses_https_and_ssh_github_urls() {
        assert_eq!(
            parse_github_owner_repo("https://github.com/acme/widgets.git"),
            Some(("acme".into(), "widgets".into()))
        );
        assert_eq!(
            parse_github_owner_repo("git@github.com:acme/widgets.git"),
            Some(("acme".into(), "widgets".into()))
        );
        assert_eq!(
            parse_github_owner_repo("https://github.com/acme/widgets/extra"),
            None
        );
        assert_eq!(
            parse_github_owner_repo("https://gitlab.com/acme/widgets.git"),
            None
        );
        assert_eq!(
            parse_github_owner_repo("https://github.com/acme/wid gets.git"),
            None
        );
    }
}
