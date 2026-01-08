use crate::provider::Provider;
use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Running,
    Completed,
    Failed,
    Merged,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentStatus::Running => write!(f, "running"),
            AgentStatus::Completed => write!(f, "completed"),
            AgentStatus::Failed => write!(f, "failed"),
            AgentStatus::Merged => write!(f, "merged"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AgentId(pub String);

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub task: String,
    pub branch: String,
    pub base_branch: String,
    pub worktree_path: PathBuf,
    pub tmux_session: String,
    pub tmux_window: String,
    pub status: AgentStatus,
    #[serde(default)]
    pub provider: Provider,
    pub spawned_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Agent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: AgentId,
        task: String,
        branch: String,
        base_branch: String,
        worktree_path: PathBuf,
        tmux_session: String,
        tmux_window: String,
        provider: Provider,
    ) -> Self {
        Self {
            id,
            task,
            branch,
            base_branch,
            worktree_path,
            tmux_session,
            tmux_window,
            status: AgentStatus::Running,
            provider,
            spawned_at: Utc::now(),
            completed_at: None,
        }
    }

    #[cfg(test)]
    pub fn create_test_agent(id: u128) -> Self {
        Self::new(
            AgentId(id.to_string()),
            format!("Task {id}"),
            format!("wta/{id}"),
            "main".to_string(),
            PathBuf::from(format!(".worktrees/{id}")),
            "wta".to_string(),
            id.to_string(),
            Provider::default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_status_display() {
        assert_eq!(AgentStatus::Running.to_string(), "running");
        assert_eq!(AgentStatus::Completed.to_string(), "completed");
        assert_eq!(AgentStatus::Failed.to_string(), "failed");
        assert_eq!(AgentStatus::Merged.to_string(), "merged");
    }

    #[test]
    fn test_agent_id_display() {
        let id = AgentId("42".to_string());
        assert_eq!(id.to_string(), "42");
    }

    #[test]
    fn test_agent_new_sets_running_status() {
        let agent = Agent::create_test_agent(1);
        assert_eq!(agent.status, AgentStatus::Running);
        assert!(agent.completed_at.is_none());
    }

    #[test]
    fn test_agent_new_sets_fields_correctly() {
        let agent = Agent::create_test_agent(1);
        assert_eq!(agent.id.0, "1");
        assert_eq!(agent.task, "Task 1");
        assert_eq!(agent.branch, "wta/1");
        assert_eq!(agent.base_branch, "main");
        assert_eq!(agent.worktree_path, PathBuf::from(".worktrees/1"));
        assert_eq!(agent.tmux_session, "wta");
        assert_eq!(agent.tmux_window, "1");
    }

    #[test]
    fn test_agent_status_serialization() {
        let json = serde_json::to_string(&AgentStatus::Running).unwrap();
        assert_eq!(json, "\"running\"");

        let status: AgentStatus = serde_json::from_str("\"completed\"").unwrap();
        assert_eq!(status, AgentStatus::Completed);
    }

    #[test]
    fn test_agent_serialization_roundtrip() {
        let agent = Agent::create_test_agent(1);
        let json = serde_json::to_string(&agent).unwrap();
        let deserialized: Agent = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id.0, agent.id.0);
        assert_eq!(deserialized.task, agent.task);
        assert_eq!(deserialized.branch, agent.branch);
        assert_eq!(deserialized.status, agent.status);
    }
}
