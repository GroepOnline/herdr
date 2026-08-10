use crate::{api::schema::AgentStatus, detect::AgentState};

/// The single status projection shared by API helpers and TUI presentation.
///
/// `seen` is the existing shared session acknowledgement: an unseen idle
/// transition is displayed as `done`, while an acknowledged idle state is
/// displayed as `idle`. Unknown is deliberately never collapsed into idle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StatusProjection {
    pub agent_status: AgentStatus,
    pub label: &'static str,
    /// Stable ordering used when aggregating multiple panes into one status.
    pub attention_priority: u8,
}

pub(crate) fn project(state: AgentState, seen: bool) -> StatusProjection {
    match (state, seen) {
        (AgentState::Blocked, _) => StatusProjection {
            agent_status: AgentStatus::Blocked,
            label: "blocked",
            attention_priority: 4,
        },
        (AgentState::Working, _) => StatusProjection {
            agent_status: AgentStatus::Working,
            label: "working",
            attention_priority: 2,
        },
        (AgentState::Idle, false) => StatusProjection {
            agent_status: AgentStatus::Done,
            label: "done",
            attention_priority: 3,
        },
        (AgentState::Idle, true) => StatusProjection {
            agent_status: AgentStatus::Idle,
            label: "idle",
            attention_priority: 1,
        },
        (AgentState::Unknown, _) => StatusProjection {
            agent_status: AgentStatus::Unknown,
            label: "unknown",
            attention_priority: 0,
        },
    }
}

pub(crate) fn label(state: AgentState, seen: bool) -> &'static str {
    project(state, seen).label
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projects_each_runtime_state_without_collapsing_unknown_into_idle() {
        assert_eq!(
            project(AgentState::Working, true),
            StatusProjection {
                agent_status: AgentStatus::Working,
                label: "working",
                attention_priority: 2,
            }
        );
        assert_eq!(
            project(AgentState::Blocked, true),
            StatusProjection {
                agent_status: AgentStatus::Blocked,
                label: "blocked",
                attention_priority: 4,
            }
        );
        assert_eq!(
            project(AgentState::Idle, false),
            StatusProjection {
                agent_status: AgentStatus::Done,
                label: "done",
                attention_priority: 3,
            }
        );
        assert_eq!(
            project(AgentState::Idle, true),
            StatusProjection {
                agent_status: AgentStatus::Idle,
                label: "idle",
                attention_priority: 1,
            }
        );
        assert_eq!(
            project(AgentState::Unknown, true),
            StatusProjection {
                agent_status: AgentStatus::Unknown,
                label: "unknown",
                attention_priority: 0,
            }
        );
        assert_eq!(label(AgentState::Unknown, false), "unknown");
        assert_eq!(project(AgentState::Blocked, true).attention_priority, 4);
        assert_eq!(project(AgentState::Idle, false).attention_priority, 3);
        assert_eq!(project(AgentState::Working, true).attention_priority, 2);
        assert_eq!(project(AgentState::Idle, true).attention_priority, 1);
        assert_eq!(project(AgentState::Unknown, true).attention_priority, 0);
    }

    #[test]
    fn seen_only_changes_idle_presentation_and_priority_order_is_total() {
        for state in [AgentState::Blocked, AgentState::Working, AgentState::Unknown] {
            assert_eq!(project(state, false), project(state, true));
        }

        let priorities = [
            project(AgentState::Unknown, true).attention_priority,
            project(AgentState::Idle, true).attention_priority,
            project(AgentState::Working, true).attention_priority,
            project(AgentState::Idle, false).attention_priority,
            project(AgentState::Blocked, true).attention_priority,
        ];
        assert!(priorities.windows(2).all(|pair| pair[0] < pair[1]));
    }
}
