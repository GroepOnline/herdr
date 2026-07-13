use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::api::schema::{LayoutApplyParams, LayoutNode, LayoutPane, SplitDirection};

#[derive(Debug, Deserialize)]
pub struct WorkspaceTemplate {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<u32>,
    pub workspace: TemplateWorkspace,
    #[serde(default, rename = "tabs")]
    pub tabs: Vec<TemplateTab>,
}

#[derive(Debug, Deserialize)]
pub struct TemplateWorkspace {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub focus: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TemplateTab {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub focus: Option<bool>,
    #[serde(default)]
    pub panes: Vec<TemplatePane>,
}

#[derive(Debug, Deserialize)]
pub struct TemplatePane {
    #[serde(default)]
    pub split: Option<String>,
    #[serde(default)]
    pub ratio: Option<f32>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub command: Option<Vec<String>>,
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
}

pub fn resolve_template_path(name_or_path: &str) -> Option<PathBuf> {
    let p = PathBuf::from(name_or_path);
    if p.is_file() {
        return Some(p);
    }
    let dir = crate::config::config_dir().join("templates");
    let f = dir.join(format!("{}.toml", name_or_path));
    f.is_file().then_some(f)
}

pub fn load_template(path: &Path) -> Result<WorkspaceTemplate, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read template {}: {}", path.display(), e))?;
    let tmpl: WorkspaceTemplate = toml::from_str(&content)
        .map_err(|e| format!("failed to parse template {}: {}", path.display(), e))?;
    validate_template(&tmpl)?;
    Ok(tmpl)
}

fn validate_template(tmpl: &WorkspaceTemplate) -> Result<(), String> {
    if tmpl.tabs.is_empty() {
        return Err("template must have at least one tab".to_string());
    }
    for (i, tab) in tmpl.tabs.iter().enumerate() {
        if tab.panes.is_empty() {
            return Err(format!("tab {} must have at least one pane", i));
        }
        if tab.panes.len() > 24 {
            return Err(format!("tab {} has too many panes (max 24)", i));
        }
        if tab.panes[0].split.is_some() {
            return Err(format!("tab {} first pane must not have a split", i));
        }
    }
    Ok(())
}

pub fn tab_to_layout_apply(tab: &TemplateTab, workspace_id: &str) -> LayoutApplyParams {
    LayoutApplyParams {
        workspace_id: Some(workspace_id.to_string()),
        tab_id: None,
        tab_label: tab.label.clone(),
        focus: tab.focus.unwrap_or(false),
        root: panes_to_layout_node(&tab.panes),
    }
}

fn panes_to_layout_node(panes: &[TemplatePane]) -> LayoutNode {
    if panes.is_empty() {
        return LayoutNode::Pane {
            pane: LayoutPane::default(),
        };
    }
    let mut root = pane_to_layout_node(&panes[0]);
    for p in &panes[1..] {
        let node = pane_to_layout_node(p);
        let direction = match p.split.as_deref().unwrap_or("right") {
            "down" | "below" | "vertical" => SplitDirection::Down,
            _ => SplitDirection::Right,
        };
        let ratio = p.ratio.unwrap_or(0.5).clamp(0.1, 0.9);
        root = LayoutNode::Split {
            direction,
            ratio,
            first: Box::new(root),
            second: Box::new(node),
        };
    }
    root
}

fn pane_to_layout_node(p: &TemplatePane) -> LayoutNode {
    LayoutNode::Pane {
        pane: LayoutPane {
            pane_id: None,
            label: p.label.clone(),
            cwd: p.cwd.clone(),
            command: p.command.clone(),
            env: p.env.clone().unwrap_or_default(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_tabs() {
        let tmpl = WorkspaceTemplate {
            name: "test".to_string(),
            description: None,
            version: None,
            workspace: TemplateWorkspace {
                label: None,
                cwd: None,
                focus: None,
            },
            tabs: vec![],
        };
        assert!(validate_template(&tmpl).is_err());
    }

    #[test]
    fn test_panes_to_layout_single() {
        let panes = vec![TemplatePane {
            split: None,
            ratio: None,
            label: Some("editor".to_string()),
            cwd: None,
            command: Some(vec!["nvim".to_string()]),
            env: None,
        }];
        let node = panes_to_layout_node(&panes);
        assert!(matches!(node, LayoutNode::Pane { .. }));
    }

    #[test]
    fn test_panes_to_layout_split() {
        let panes = vec![
            TemplatePane {
                split: None,
                ratio: None,
                label: Some("a".to_string()),
                cwd: None,
                command: None,
                env: None,
            },
            TemplatePane {
                split: Some("right".to_string()),
                ratio: Some(0.4),
                label: Some("b".to_string()),
                cwd: None,
                command: None,
                env: None,
            },
        ];
        let node = panes_to_layout_node(&panes);
        match node {
            LayoutNode::Split {
                direction, ratio, ..
            } => {
                assert_eq!(direction, SplitDirection::Right);
                assert!((ratio - 0.4).abs() < 0.01);
            }
            _ => panic!("expected Split"),
        }
    }
}
