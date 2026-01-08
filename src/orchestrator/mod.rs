mod agent;
mod state;

pub use agent::{Agent, AgentId, AgentStatus};
pub use state::State;

use crate::error::{Error, Result};
use crate::git::WorktreeManager;
use crate::tmux::TmuxManager;
use clap::ValueEnum;
use std::path::PathBuf;

const TMUX_SESSION_NAME: &str = "wta";
const WORKTREES_DIR: &str = ".worktrees";
const STATE_DIR: &str = ".worktree-agents";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum MergeStrategy {
    Merge,
    Rebase,
    Squash,
}

pub struct SpawnRequest {
    pub task: String,
    pub branch: Option<String>,
    pub base: Option<String>,
    pub claude_args: Vec<String>,
}

pub struct MergeResult {
    pub success: bool,
    pub message: String,
    pub conflicts: Vec<PathBuf>,
}

pub struct Orchestrator {
    state: State,
    repo_root: PathBuf,
    worktree_manager: WorktreeManager,
    tmux: TmuxManager,
}

impl Orchestrator {
    /// Create a new orchestrator for the current directory
    pub fn new() -> Result<Self> {
        let repo_root = Self::find_repo_root()?;
        let worktrees_dir = repo_root.join(WORKTREES_DIR);
        let state_dir = repo_root.join(STATE_DIR);

        // Ensure directories exist
        std::fs::create_dir_all(&worktrees_dir)?;
        std::fs::create_dir_all(&state_dir)?;
        std::fs::create_dir_all(state_dir.join("status"))?;

        let state = State::load_or_create(&state_dir)?;
        let worktree_manager = WorktreeManager::new(&repo_root, &worktrees_dir);
        let tmux = TmuxManager::new(TMUX_SESSION_NAME);

        Ok(Self {
            state,
            repo_root,
            worktree_manager,
            tmux,
        })
    }

    fn find_repo_root() -> Result<PathBuf> {
        let current_dir = std::env::current_dir()?;
        let repo = git2::Repository::discover(&current_dir)?;
        repo.workdir()
            .map(|p| p.to_path_buf())
            .ok_or(Error::NotAGitRepository)
    }

    fn get_current_branch(&self) -> Result<String> {
        let repo = git2::Repository::open(&self.repo_root)?;
        let head = repo.head()?;
        head.shorthand()
            .map(|s| s.to_string())
            .ok_or_else(|| Error::Git(git2::Error::from_str("HEAD is not a branch")))
    }

    pub async fn spawn(&mut self, request: SpawnRequest) -> Result<AgentId> {
        // 1. Generate ID
        let id = AgentId(self.state.next_id().to_string());

        // 2. Determine branch name
        let branch = request.branch.unwrap_or_else(|| format!("wta/{}", id.0));

        // 3. Get base branch
        let base_branch = match request.base {
            Some(b) => b,
            None => self.get_current_branch()?,
        };

        // 4. Create worktree
        let worktree_path = self.worktree_manager.create(&id.0, &branch, &base_branch)?;

        // 5. Ensure tmux session exists
        self.tmux.ensure_session()?;

        // 6. Create tmux window
        self.tmux.create_window(&id.0, &worktree_path)?;

        // 7. Build status file path for the agent to write
        let status_file = self
            .repo_root
            .join(STATE_DIR)
            .join("status")
            .join(format!("{}.json", id.0));

        // 8. Build claude command with task and status file instructions
        let task_with_instructions = format!(
            "{}\n\n---\nIMPORTANT: When you complete this task:\n1. Commit your changes (do NOT include Co-Authored-By in commits)\n2. Write a JSON status file to: {}\n   Format: {{\"status\": \"completed\"|\"failed\", \"summary\": \"brief description\", \"files_changed\": [\"file1\", \"file2\"], \"error\": null}}",
            request.task,
            status_file.display()
        );

        // 9. Write prompt to a file (avoids shell quoting issues with newlines)
        let prompts_dir = self.repo_root.join(STATE_DIR).join("prompts");
        std::fs::create_dir_all(&prompts_dir)?;
        let prompt_file = prompts_dir.join(format!("{}.txt", id.0));
        std::fs::write(&prompt_file, &task_with_instructions)?;

        let claude_args_str = if request.claude_args.is_empty() {
            String::new()
        } else {
            format!(" {}", request.claude_args.join(" "))
        };

        // Default allowed tools for safe operations
        let default_allowed_tools = [
            "Bash(cargo check:*)",
            "Bash(cargo build:*)",
            "Bash(cargo test:*)",
            "Bash(cargo fmt:*)",
            "Bash(cargo clippy:*)",
            "Bash(git diff:*)",
            "Bash(git status:*)",
            "Bash(git log:*)",
            "Bash(git branch:*)",
            "Bash(git add:*)",
            "Bash(git commit:*)",
            "Bash(ls:*)",
            "Bash(pwd)",
        ];
        let allowed_tools_arg = format!("--allowedTools '{}'", default_allowed_tools.join(","));

        // Use cat to pipe the prompt to claude (explicit cd to ensure we're in worktree)
        let claude_cmd = format!(
            "cd {} && cat {} | claude {allowed_tools_arg}{claude_args_str}",
            worktree_path.display(),
            prompt_file.display()
        );

        // 10. Send command to tmux
        self.tmux.send_keys(&id.0, &claude_cmd)?;

        // 11. Register agent in state
        let agent = Agent::new(
            id.clone(),
            request.task,
            branch,
            base_branch,
            worktree_path,
            TMUX_SESSION_NAME.to_string(),
            id.0.clone(),
        );

        self.state.add_agent(agent)?;

        Ok(id)
    }

    pub fn list(&self) -> Vec<&Agent> {
        self.state.agents()
    }

    pub fn get_agent(&self, id: &str) -> Result<&Agent> {
        self.state
            .get_agent(id)
            .ok_or_else(|| Error::AgentNotFound(id.to_string()))
    }

    pub fn get_agent_mut(&mut self, id: &str) -> Result<&mut Agent> {
        self.state
            .get_agent_mut(id)
            .ok_or_else(|| Error::AgentNotFound(id.to_string()))
    }

    pub fn get_output(&self, id: &str, lines: usize) -> Result<String> {
        let agent = self.get_agent(id)?;
        self.tmux.capture_pane(&agent.tmux_window, lines)
    }

    pub fn attach(&self, id: &str) -> Result<()> {
        let agent = self.get_agent(id)?;
        self.tmux.attach(Some(&agent.tmux_window))
    }

    pub fn check_status(&mut self, id: &str) -> Result<AgentStatus> {
        let agent = self.get_agent(id)?;

        // Check if status file exists
        let status_file = self
            .repo_root
            .join(STATE_DIR)
            .join("status")
            .join(format!("{}.json", id));

        if status_file.exists() {
            let content = std::fs::read_to_string(&status_file)?;
            let status_data: serde_json::Value = serde_json::from_str(&content)?;

            let new_status = match status_data.get("status").and_then(|s| s.as_str()) {
                Some("completed") => AgentStatus::Completed,
                Some("failed") => AgentStatus::Failed,
                _ => return Ok(agent.status),
            };

            // Kill tmux window since agent is done
            let _ = self.tmux.kill_window(&agent.tmux_window);

            // Update agent status
            let agent = self.get_agent_mut(id)?;
            agent.status = new_status;
            agent.completed_at = Some(chrono::Utc::now());
            self.state.save()?;

            return Ok(new_status);
        }

        Ok(agent.status)
    }

    pub async fn merge(&mut self, id: &str, strategy: MergeStrategy, force: bool) -> Result<MergeResult> {
        let agent = self.get_agent(id)?;

        if agent.status == AgentStatus::Running && !force {
            return Err(Error::AgentStillRunning(id.to_string()));
        }

        let result = crate::git::merge::merge_branch(
            &self.repo_root,
            &agent.branch,
            &agent.base_branch,
            strategy,
        )?;

        if result.success {
            // Clean up: remove worktree and delete branch
            let _ = self.worktree_manager.remove(id);

            let repo = git2::Repository::open(&self.repo_root)?;
            if let Ok(mut branch) = repo.find_branch(&agent.branch, git2::BranchType::Local) {
                let _ = branch.delete();
            }

            // Remove prompt and status files
            let prompt_file = self.repo_root.join(STATE_DIR).join("prompts").join(format!("{id}.txt"));
            let status_file = self.repo_root.join(STATE_DIR).join("status").join(format!("{id}.json"));
            let _ = std::fs::remove_file(prompt_file);
            let _ = std::fs::remove_file(status_file);

            let agent = self.get_agent_mut(id)?;
            agent.status = AgentStatus::Merged;
            self.state.save()?;
        }

        Ok(result)
    }

    pub async fn discard(&mut self, id: &str, force: bool) -> Result<()> {
        let agent = self.get_agent(id)?;

        if agent.status == AgentStatus::Running && !force {
            return Err(Error::AgentStillRunning(id.to_string()));
        }

        // Kill tmux window if it exists
        let _ = self.tmux.kill_window(&agent.tmux_window);

        // Remove worktree
        self.worktree_manager.remove(id)?;

        // Delete branch
        let repo = git2::Repository::open(&self.repo_root)?;
        if let Ok(mut branch) = repo.find_branch(&agent.branch, git2::BranchType::Local) {
            let _ = branch.delete();
        }

        // Remove prompt and status files if they exist
        let prompt_file = self.repo_root.join(STATE_DIR).join("prompts").join(format!("{id}.txt"));
        let status_file = self.repo_root.join(STATE_DIR).join("status").join(format!("{id}.json"));
        let _ = std::fs::remove_file(prompt_file);
        let _ = std::fs::remove_file(status_file);

        // Remove agent from state entirely
        self.state.remove_agent(id)?;

        Ok(())
    }
}
