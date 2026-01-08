use crate::error::{Error, Result};
use crate::orchestrator::Agent;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const STATE_FILE: &str = "state.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    #[serde(skip)]
    state_dir: PathBuf,
    next_id: u64,
    agents: Vec<Agent>,
}

impl State {
    pub fn load_or_create(state_dir: &Path) -> Result<Self> {
        let state_file = state_dir.join(STATE_FILE);

        if state_file.exists() {
            let content = std::fs::read_to_string(&state_file)?;
            let mut state: State =
                serde_json::from_str(&content).map_err(|e| Error::StateCorrupted(e.to_string()))?;
            state.state_dir = state_dir.to_path_buf();
            Ok(state)
        } else {
            Ok(Self {
                state_dir: state_dir.to_path_buf(),
                next_id: 1,
                agents: Vec::new(),
            })
        }
    }

    pub fn save(&self) -> Result<()> {
        let state_file = self.state_dir.join(STATE_FILE);
        let content = serde_json::to_string_pretty(&self)?;
        std::fs::write(state_file, content)?;
        Ok(())
    }

    pub fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add_agent(&mut self, agent: Agent) -> Result<()> {
        self.agents.push(agent);
        self.save()
    }

    pub fn agents(&self) -> Vec<&Agent> {
        self.agents.iter().collect()
    }

    pub fn get_agent(&self, id: &str) -> Option<&Agent> {
        self.agents.iter().find(|a| a.id.0 == id)
    }

    pub fn get_agent_mut(&mut self, id: &str) -> Option<&mut Agent> {
        self.agents.iter_mut().find(|a| a.id.0 == id)
    }

    pub fn remove_agent(&mut self, id: &str) -> Result<()> {
        self.agents.retain(|a| a.id.0 != id);
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::AgentStatus;
    use tempfile::TempDir;

    #[test]
    fn test_state_load_or_create_new() {
        let temp_dir = TempDir::new().unwrap();
        let state = State::load_or_create(temp_dir.path()).unwrap();

        assert_eq!(state.next_id, 1);
        assert!(state.agents.is_empty());
    }

    #[test]
    fn test_state_next_id_increments() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = State::load_or_create(temp_dir.path()).unwrap();

        assert_eq!(state.next_id(), 1);
        assert_eq!(state.next_id(), 2);
        assert_eq!(state.next_id(), 3);
    }

    #[test]
    fn test_state_add_agent() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = State::load_or_create(temp_dir.path()).unwrap();

        let agent = Agent::create_test_agent(1);
        state.add_agent(agent).unwrap();

        assert_eq!(state.agents().len(), 1);
        assert_eq!(state.agents()[0].id.0, "1");
    }

    #[test]
    fn test_state_get_agent() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = State::load_or_create(temp_dir.path()).unwrap();

        state.add_agent(Agent::create_test_agent(1)).unwrap();
        state.add_agent(Agent::create_test_agent(2)).unwrap();

        let agent = state.get_agent("1").unwrap();
        assert_eq!(agent.task, "Task 1");

        let agent = state.get_agent("2").unwrap();
        assert_eq!(agent.task, "Task 2");

        assert!(state.get_agent("99").is_none());
    }

    #[test]
    fn test_state_get_agent_mut() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = State::load_or_create(temp_dir.path()).unwrap();

        state.add_agent(Agent::create_test_agent(1)).unwrap();

        let agent = state.get_agent_mut("1").unwrap();
        agent.status = AgentStatus::Completed;
        agent.completed_at = Some(chrono::Utc::now());

        let agent = state.get_agent("1").unwrap();
        assert_eq!(agent.status, AgentStatus::Completed);
        assert!(agent.completed_at.is_some());
    }

    #[test]
    fn test_state_save_and_load() {
        let temp_dir = TempDir::new().unwrap();

        // Create state and add agents
        {
            let mut state = State::load_or_create(temp_dir.path()).unwrap();
            let _ = state.next_id(); // Consume ID 1
            let _ = state.next_id(); // Consume ID 2
            state.add_agent(Agent::create_test_agent(1)).unwrap();
            state.add_agent(Agent::create_test_agent(2)).unwrap();
            state.save().unwrap();
        }

        // Load state and verify
        {
            let state = State::load_or_create(temp_dir.path()).unwrap();
            assert_eq!(state.next_id, 3); // Next ID should be 3
            assert_eq!(state.agents().len(), 2);
            assert_eq!(state.get_agent("1").unwrap().task, "Task 1");
            assert_eq!(state.get_agent("2").unwrap().task, "Task 2");
        }
    }

    #[test]
    fn test_state_persistence_across_sessions() {
        let temp_dir = TempDir::new().unwrap();

        // Session 1: Create and add agent
        {
            let mut state = State::load_or_create(temp_dir.path()).unwrap();
            state.add_agent(Agent::create_test_agent(1)).unwrap();
        }

        // Session 2: Modify agent status
        {
            let mut state = State::load_or_create(temp_dir.path()).unwrap();
            let agent = state.get_agent_mut("1").unwrap();
            agent.status = AgentStatus::Completed;
            state.save().unwrap();
        }

        // Session 3: Verify status persisted
        {
            let state = State::load_or_create(temp_dir.path()).unwrap();
            let agent = state.get_agent("1").unwrap();
            assert_eq!(agent.status, AgentStatus::Completed);
        }
    }

    #[test]
    fn test_state_corrupted_file() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("state.json");

        // Write invalid JSON
        std::fs::write(&state_file, "not valid json").unwrap();

        let result = State::load_or_create(temp_dir.path());
        assert!(result.is_err());

        match result.unwrap_err() {
            Error::StateCorrupted(_) => {}
            e => panic!("Expected StateCorrupted error, got: {e:?}"),
        }
    }
}
