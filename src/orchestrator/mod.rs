mod agent;
mod state;

pub use agent::{Agent, AgentId, AgentStatus};
pub use state::State;

use crate::error::{Error, Result};
use crate::git::WorktreeManager;
use crate::provider::Provider;
use crate::tmux::TmuxManager;
use clap::ValueEnum;
use std::path::{Path, PathBuf};

const TMUX_SESSION_PREFIX: &str = "wta";
const WORKTREES_DIR: &str = ".worktrees";
const STATE_DIR: &str = ".worktree-agents";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum MergeStrategy {
    Merge,
    Rebase,
    Squash,
}

/// Filter for selecting which agents to prune
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PruneFilter {
    /// Prune all agents regardless of status
    All,
    /// Prune only agents with a specific status
    Status(AgentStatus),
    /// Prune inactive agents (Merged, Completed, Failed)
    Inactive,
}

pub struct LaunchRequest {
    pub task: String,
    pub branch: Option<String>,
    pub base: Option<String>,
    pub provider: Provider,
    pub provider_args: Vec<String>,
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
    tmux_session_name: String,
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
        let tmux_session_name = Self::generate_session_name(&repo_root);
        let tmux = TmuxManager::new(&tmux_session_name);

        Ok(Self {
            state,
            repo_root,
            worktree_manager,
            tmux,
            tmux_session_name,
        })
    }

    /// Generate a unique tmux session name based on the repository path.
    /// Uses the last component of the path (project name) plus a short hash
    /// to ensure uniqueness when multiple projects have the same name.
    fn generate_session_name(repo_root: &Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Get the project directory name
        let project_name = repo_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Create a short hash of the full path for uniqueness
        let mut hasher = DefaultHasher::new();
        repo_root.hash(&mut hasher);
        let hash = hasher.finish();
        let short_hash = format!("{:x}", hash).chars().take(6).collect::<String>();

        format!("{TMUX_SESSION_PREFIX}-{project_name}-{short_hash}")
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

    pub async fn launch(&mut self, request: LaunchRequest) -> Result<AgentId> {
        // 1. Generate ID
        let id = AgentId(self.state.next_id().to_string());

        // 2. Determine branch name and whether it's an existing branch
        let (branch, base_branch, worktree_path) = match &request.branch {
            Some(specified_branch) if self.worktree_manager.branch_exists(specified_branch)? => {
                // Existing branch - check it out directly
                let worktree_path = self
                    .worktree_manager
                    .checkout_existing(&id.0, specified_branch)?;
                // For existing branches, base_branch is the branch itself (no fork point)
                (
                    specified_branch.clone(),
                    specified_branch.clone(),
                    worktree_path,
                )
            }
            _ => {
                // New branch - create it from base
                let branch = request
                    .branch
                    .clone()
                    .unwrap_or_else(|| format!("wta/{}", id.0));
                let base_branch = match &request.base {
                    Some(b) => b.clone(),
                    None => self.get_current_branch()?,
                };
                let worktree_path =
                    self.worktree_manager
                        .create(&id.0, &branch, &base_branch)?;
                (branch, base_branch, worktree_path)
            }
        };

        // 5. Copy .claude settings from main repo to worktree for permission inheritance
        let main_claude_dir = self.repo_root.join(".claude");
        if main_claude_dir.exists() {
            let worktree_claude_dir = worktree_path.join(".claude");
            if let Err(e) = Self::copy_dir_recursive(&main_claude_dir, &worktree_claude_dir) {
                eprintln!("Warning: could not copy .claude settings to worktree: {e}");
            }
        }

        // 6. Ensure tmux session exists
        self.tmux.ensure_session()?;

        // 7. Create tmux window
        self.tmux.create_window(&id.0, &worktree_path)?;

        // 8. Build status file path for the agent to write
        let status_file = self
            .repo_root
            .join(STATE_DIR)
            .join("status")
            .join(format!("{}.json", id.0));

        // 9. Build command with task and status file instructions
        let task_with_instructions = format!(
            "{}\n\n---\nIMPORTANT: When you complete this task:\n1. Commit your changes (do NOT include Co-Authored-By in commits)\n2. Write a JSON status file to: {}\n   Format: {{\"status\": \"completed\"|\"failed\", \"summary\": \"brief description\", \"files_changed\": [\"file1\", \"file2\"], \"error\": null}}",
            request.task,
            status_file.display()
        );

        // 10. Write prompt to a file (avoids shell quoting issues with newlines)
        let prompts_dir = self.repo_root.join(STATE_DIR).join("prompts");
        std::fs::create_dir_all(&prompts_dir)?;
        let prompt_file = prompts_dir.join(format!("{}.txt", id.0));
        std::fs::write(&prompt_file, &task_with_instructions)?;

        // 11. Build provider-specific command
        let provider_cmd = request.provider.build_command(
            &worktree_path,
            &prompt_file,
            &status_file,
            &request.provider_args,
        );

        // 12. Send command to tmux
        self.tmux.send_keys(&id.0, &provider_cmd)?;

        // 13. Register agent in state
        let agent = Agent::new(
            id.clone(),
            request.task,
            branch,
            base_branch,
            worktree_path,
            self.tmux_session_name.clone(),
            id.0.clone(),
            request.provider,
        );

        self.state.add_agent(agent)?;

        Ok(id)
    }

    pub fn list(&self) -> Vec<&Agent> {
        self.state.agents()
    }

    /// Get the path to the worktrees directory
    pub fn worktrees_dir(&self) -> PathBuf {
        self.repo_root.join(WORKTREES_DIR)
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

    /// Open VS Code in the agent's worktree directory
    pub fn open_vscode(&self, id: &str) -> Result<()> {
        let agent = self.get_agent(id)?;
        std::process::Command::new("code")
            .arg(&agent.worktree_path)
            .spawn()
            .map_err(|e| Error::ExternalProcessFailed(format!("Failed to launch VS Code: {e}")))?;
        Ok(())
    }

    pub fn check_status(&mut self, id: &str) -> Result<AgentStatus> {
        let agent = self.get_agent(id)?;

        // If agent is already in a terminal state, no need to check further
        if agent.status != AgentStatus::Running {
            return Ok(agent.status);
        }

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

        // No status file exists - check if tmux window is still running
        // If the window is gone, the agent has exited (possibly crashed or user exited)
        let window_exists = self.tmux.window_exists(&agent.tmux_window);
        if !window_exists {
            let agent = self.get_agent_mut(id)?;
            agent.status = AgentStatus::Failed;
            agent.completed_at = Some(chrono::Utc::now());
            self.state.save()?;

            return Ok(AgentStatus::Failed);
        }

        Ok(agent.status)
    }

    pub async fn merge(
        &mut self,
        id: &str,
        strategy: MergeStrategy,
        force: bool,
    ) -> Result<MergeResult> {
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
            let prompt_file = self
                .repo_root
                .join(STATE_DIR)
                .join("prompts")
                .join(format!("{id}.txt"));
            let status_file = self
                .repo_root
                .join(STATE_DIR)
                .join("status")
                .join(format!("{id}.json"));
            let _ = std::fs::remove_file(prompt_file);
            let _ = std::fs::remove_file(status_file);

            let agent = self.get_agent_mut(id)?;
            agent.status = AgentStatus::Merged;
            self.state.save()?;
        }

        Ok(result)
    }

    pub async fn remove(&mut self, id: &str, force: bool) -> Result<()> {
        // First check the status file to get latest status
        self.check_status(id)?;

        let agent = self.get_agent(id)?;

        // Check both the status AND if tmux window actually exists
        // Agent is only truly running if status says Running AND window exists
        let window_exists = self.tmux.window_exists(&agent.tmux_window);
        let is_running = agent.status == AgentStatus::Running && window_exists;

        if is_running && !force {
            return Err(Error::AgentStillRunning(id.to_string()));
        }

        // Kill tmux window if it exists
        let _ = self.tmux.kill_window(&agent.tmux_window);

        // Remove worktree (warn if already gone)
        if let Err(e) = self.worktree_manager.remove(id) {
            eprintln!("Warning: could not remove worktree: {e}");
        }

        // Delete branch
        let repo = git2::Repository::open(&self.repo_root)?;
        if let Ok(mut branch) = repo.find_branch(&agent.branch, git2::BranchType::Local) {
            let _ = branch.delete();
        }

        // Remove prompt and status files if they exist
        let prompt_file = self
            .repo_root
            .join(STATE_DIR)
            .join("prompts")
            .join(format!("{id}.txt"));
        let status_file = self
            .repo_root
            .join(STATE_DIR)
            .join("status")
            .join(format!("{id}.json"));
        let _ = std::fs::remove_file(prompt_file);
        let _ = std::fs::remove_file(status_file);

        // Remove agent from state entirely
        self.state.remove_agent(id)?;

        Ok(())
    }

    /// Prune agents matching the filter, cleaning up all associated resources
    /// Returns the list of pruned agents
    pub async fn prune(&mut self, filter: PruneFilter) -> Result<Vec<Agent>> {
        // Collect agents to prune based on filter
        let agents_to_prune: Vec<Agent> = self
            .state
            .agents()
            .iter()
            .filter(|agent| match filter {
                PruneFilter::All => true,
                PruneFilter::Status(status) => agent.status == status,
                PruneFilter::Inactive => {
                    matches!(
                        agent.status,
                        AgentStatus::Merged | AgentStatus::Completed | AgentStatus::Failed
                    )
                }
            })
            .map(|a| (*a).clone())
            .collect();

        let mut pruned = Vec::with_capacity(agents_to_prune.len());

        for agent in agents_to_prune {
            // Perform cleanup, ignoring errors for resources that may already be gone
            self.cleanup_agent_resources(&agent);

            // Remove agent from state
            self.state.remove_agent(&agent.id.0)?;

            pruned.push(agent);
        }

        Ok(pruned)
    }

    /// Clean up all resources associated with an agent
    /// Ignores errors for resources that may already be cleaned up
    fn cleanup_agent_resources(&self, agent: &Agent) {
        // Kill tmux window if it exists
        let _ = self.tmux.kill_window(&agent.tmux_window);

        // Remove worktree if it exists
        let _ = self.worktree_manager.remove(&agent.id.0);

        // Delete branch if it exists
        if let Ok(repo) = git2::Repository::open(&self.repo_root) {
            if let Ok(mut branch) = repo.find_branch(&agent.branch, git2::BranchType::Local) {
                let _ = branch.delete();
            }
        }

        // Remove prompt and status files if they exist
        let prompt_file = self
            .repo_root
            .join(STATE_DIR)
            .join("prompts")
            .join(format!("{}.txt", agent.id.0));
        let status_file = self
            .repo_root
            .join(STATE_DIR)
            .join("status")
            .join(format!("{}.json", agent.id.0));
        let _ = std::fs::remove_file(prompt_file);
        let _ = std::fs::remove_file(status_file);
    }

    /// Recursively copy a directory and its contents
    fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dst)?;

        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                Self::copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_agent_with_status(id: u128, status: AgentStatus) -> Agent {
        let mut agent = Agent::create_test_agent(id);
        agent.status = status;
        agent
    }

    #[test]
    fn test_prune_filter_all_matches_all_statuses() {
        let agents = vec![
            create_test_agent_with_status(1, AgentStatus::Running),
            create_test_agent_with_status(2, AgentStatus::Completed),
            create_test_agent_with_status(3, AgentStatus::Failed),
            create_test_agent_with_status(4, AgentStatus::Merged),
        ];

        let filter = PruneFilter::All;
        let matched: Vec<_> = agents
            .iter()
            .filter(|agent| match filter {
                PruneFilter::All => true,
                PruneFilter::Status(s) => agent.status == s,
                PruneFilter::Inactive => {
                    matches!(
                        agent.status,
                        AgentStatus::Merged | AgentStatus::Completed | AgentStatus::Failed
                    )
                }
            })
            .collect();

        assert_eq!(matched.len(), 4);
    }

    #[test]
    fn test_prune_filter_inactive_excludes_running() {
        let agents = vec![
            create_test_agent_with_status(1, AgentStatus::Running),
            create_test_agent_with_status(2, AgentStatus::Completed),
            create_test_agent_with_status(3, AgentStatus::Failed),
            create_test_agent_with_status(4, AgentStatus::Merged),
        ];

        let filter = PruneFilter::Inactive;
        let matched: Vec<_> = agents
            .iter()
            .filter(|agent| match filter {
                PruneFilter::All => true,
                PruneFilter::Status(s) => agent.status == s,
                PruneFilter::Inactive => {
                    matches!(
                        agent.status,
                        AgentStatus::Merged | AgentStatus::Completed | AgentStatus::Failed
                    )
                }
            })
            .collect();

        assert_eq!(matched.len(), 3);
        assert!(matched.iter().all(|a| a.status != AgentStatus::Running));
    }

    #[test]
    fn test_prune_filter_status_matches_specific_status() {
        let agents = vec![
            create_test_agent_with_status(1, AgentStatus::Running),
            create_test_agent_with_status(2, AgentStatus::Completed),
            create_test_agent_with_status(3, AgentStatus::Failed),
            create_test_agent_with_status(4, AgentStatus::Merged),
        ];

        let filter = PruneFilter::Status(AgentStatus::Completed);
        let matched: Vec<_> = agents
            .iter()
            .filter(|agent| match filter {
                PruneFilter::All => true,
                PruneFilter::Status(s) => agent.status == s,
                PruneFilter::Inactive => {
                    matches!(
                        agent.status,
                        AgentStatus::Merged | AgentStatus::Completed | AgentStatus::Failed
                    )
                }
            })
            .collect();

        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].status, AgentStatus::Completed);
    }

    #[test]
    fn test_prune_filter_status_merged_only() {
        let agents = vec![
            create_test_agent_with_status(1, AgentStatus::Merged),
            create_test_agent_with_status(2, AgentStatus::Merged),
            create_test_agent_with_status(3, AgentStatus::Failed),
        ];

        let filter = PruneFilter::Status(AgentStatus::Merged);
        let matched: Vec<_> = agents
            .iter()
            .filter(|agent| match filter {
                PruneFilter::All => true,
                PruneFilter::Status(s) => agent.status == s,
                PruneFilter::Inactive => {
                    matches!(
                        agent.status,
                        AgentStatus::Merged | AgentStatus::Completed | AgentStatus::Failed
                    )
                }
            })
            .collect();

        assert_eq!(matched.len(), 2);
        assert!(matched.iter().all(|a| a.status == AgentStatus::Merged));
    }

    #[test]
    fn test_prune_filter_inactive_with_no_inactive_agents() {
        let agents = vec![
            create_test_agent_with_status(1, AgentStatus::Running),
            create_test_agent_with_status(2, AgentStatus::Running),
        ];

        let filter = PruneFilter::Inactive;
        let matched: Vec<_> = agents
            .iter()
            .filter(|agent| match filter {
                PruneFilter::All => true,
                PruneFilter::Status(s) => agent.status == s,
                PruneFilter::Inactive => {
                    matches!(
                        agent.status,
                        AgentStatus::Merged | AgentStatus::Completed | AgentStatus::Failed
                    )
                }
            })
            .collect();

        assert!(matched.is_empty());
    }

    #[test]
    fn test_generate_session_name_includes_project_name() {
        let path = PathBuf::from("/home/user/projects/my-project");
        let session_name = Orchestrator::generate_session_name(&path);

        assert!(session_name.starts_with("wta-my-project-"));
    }

    #[test]
    fn test_generate_session_name_is_deterministic() {
        let path = PathBuf::from("/home/user/projects/my-project");
        let session_name1 = Orchestrator::generate_session_name(&path);
        let session_name2 = Orchestrator::generate_session_name(&path);

        assert_eq!(session_name1, session_name2);
    }

    #[test]
    fn test_generate_session_name_different_paths_different_names() {
        let path1 = PathBuf::from("/home/user/projects/project-a");
        let path2 = PathBuf::from("/home/user/projects/project-b");
        let session_name1 = Orchestrator::generate_session_name(&path1);
        let session_name2 = Orchestrator::generate_session_name(&path2);

        assert_ne!(session_name1, session_name2);
    }

    #[test]
    fn test_generate_session_name_same_name_different_location_different_hash() {
        // Two projects with same name but different paths should have different session names
        let path1 = PathBuf::from("/home/user/work/my-project");
        let path2 = PathBuf::from("/home/user/personal/my-project");
        let session_name1 = Orchestrator::generate_session_name(&path1);
        let session_name2 = Orchestrator::generate_session_name(&path2);

        // Both start with same prefix but have different hashes
        assert!(session_name1.starts_with("wta-my-project-"));
        assert!(session_name2.starts_with("wta-my-project-"));
        assert_ne!(session_name1, session_name2);
    }

    #[test]
    fn test_copy_dir_recursive_copies_files() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        let dst = temp_dir.path().join("dst");

        // Create source directory with files
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("file1.txt"), "content1").unwrap();
        std::fs::write(src.join("file2.txt"), "content2").unwrap();

        // Copy directory
        Orchestrator::copy_dir_recursive(&src, &dst).unwrap();

        // Verify files were copied
        assert!(dst.exists());
        assert_eq!(
            std::fs::read_to_string(dst.join("file1.txt")).unwrap(),
            "content1"
        );
        assert_eq!(
            std::fs::read_to_string(dst.join("file2.txt")).unwrap(),
            "content2"
        );
    }

    #[test]
    fn test_copy_dir_recursive_copies_nested_directories() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        let dst = temp_dir.path().join("dst");

        // Create source directory with nested structure
        std::fs::create_dir_all(src.join("subdir1/subdir2")).unwrap();
        std::fs::write(src.join("root.txt"), "root").unwrap();
        std::fs::write(src.join("subdir1/level1.txt"), "level1").unwrap();
        std::fs::write(src.join("subdir1/subdir2/level2.txt"), "level2").unwrap();

        // Copy directory
        Orchestrator::copy_dir_recursive(&src, &dst).unwrap();

        // Verify structure was preserved
        assert!(dst.join("subdir1").is_dir());
        assert!(dst.join("subdir1/subdir2").is_dir());
        assert_eq!(
            std::fs::read_to_string(dst.join("root.txt")).unwrap(),
            "root"
        );
        assert_eq!(
            std::fs::read_to_string(dst.join("subdir1/level1.txt")).unwrap(),
            "level1"
        );
        assert_eq!(
            std::fs::read_to_string(dst.join("subdir1/subdir2/level2.txt")).unwrap(),
            "level2"
        );
    }

    #[test]
    fn test_copy_dir_recursive_empty_directory() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        let dst = temp_dir.path().join("dst");

        // Create empty source directory
        std::fs::create_dir_all(&src).unwrap();

        // Copy directory
        Orchestrator::copy_dir_recursive(&src, &dst).unwrap();

        // Verify destination was created
        assert!(dst.exists());
        assert!(dst.is_dir());
    }

    #[test]
    fn test_copy_dir_recursive_creates_destination() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        let dst = temp_dir.path().join("dst/nested/path");

        // Create source directory with a file
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("test.txt"), "test").unwrap();

        // Destination doesn't exist
        assert!(!dst.exists());

        // Copy directory - should create destination
        Orchestrator::copy_dir_recursive(&src, &dst).unwrap();

        // Verify destination was created with contents
        assert!(dst.exists());
        assert_eq!(
            std::fs::read_to_string(dst.join("test.txt")).unwrap(),
            "test"
        );
    }

    #[test]
    fn test_merge_strategy_equality() {
        assert_eq!(MergeStrategy::Merge, MergeStrategy::Merge);
        assert_eq!(MergeStrategy::Rebase, MergeStrategy::Rebase);
        assert_eq!(MergeStrategy::Squash, MergeStrategy::Squash);
        assert_ne!(MergeStrategy::Merge, MergeStrategy::Rebase);
        assert_ne!(MergeStrategy::Merge, MergeStrategy::Squash);
        assert_ne!(MergeStrategy::Rebase, MergeStrategy::Squash);
    }

    #[test]
    fn test_prune_filter_equality() {
        assert_eq!(PruneFilter::All, PruneFilter::All);
        assert_eq!(PruneFilter::Inactive, PruneFilter::Inactive);
        assert_eq!(
            PruneFilter::Status(AgentStatus::Completed),
            PruneFilter::Status(AgentStatus::Completed)
        );
        assert_ne!(PruneFilter::All, PruneFilter::Inactive);
        assert_ne!(
            PruneFilter::Status(AgentStatus::Completed),
            PruneFilter::Status(AgentStatus::Failed)
        );
    }

    #[test]
    fn test_launch_request_fields() {
        use crate::provider::Provider;

        let request = LaunchRequest {
            task: "Fix the bug".to_string(),
            branch: Some("fix/bug".to_string()),
            base: Some("main".to_string()),
            provider: Provider::Claude,
            provider_args: vec!["--verbose".to_string()],
        };

        assert_eq!(request.task, "Fix the bug");
        assert_eq!(request.branch, Some("fix/bug".to_string()));
        assert_eq!(request.base, Some("main".to_string()));
        assert_eq!(request.provider, Provider::Claude);
        assert_eq!(request.provider_args.len(), 1);
        assert_eq!(request.provider_args[0], "--verbose");
    }

    #[test]
    fn test_launch_request_optional_fields() {
        use crate::provider::Provider;

        let request = LaunchRequest {
            task: "Simple task".to_string(),
            branch: None,
            base: None,
            provider: Provider::default(),
            provider_args: Vec::new(),
        };

        assert!(request.branch.is_none());
        assert!(request.base.is_none());
        assert!(request.provider_args.is_empty());
    }

    #[test]
    fn test_launch_request_with_different_providers() {
        use crate::provider::Provider;

        let claude_request = LaunchRequest {
            task: "Task".to_string(),
            branch: None,
            base: None,
            provider: Provider::Claude,
            provider_args: Vec::new(),
        };
        assert_eq!(claude_request.provider, Provider::Claude);

        let codex_request = LaunchRequest {
            task: "Task".to_string(),
            branch: None,
            base: None,
            provider: Provider::Codex,
            provider_args: Vec::new(),
        };
        assert_eq!(codex_request.provider, Provider::Codex);

        let gemini_request = LaunchRequest {
            task: "Task".to_string(),
            branch: None,
            base: None,
            provider: Provider::Gemini,
            provider_args: Vec::new(),
        };
        assert_eq!(gemini_request.provider, Provider::Gemini);
    }

    #[test]
    fn test_merge_result_fields() {
        let result = MergeResult {
            success: true,
            message: "Merged successfully".to_string(),
            conflicts: Vec::new(),
        };

        assert!(result.success);
        assert_eq!(result.message, "Merged successfully");
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn test_merge_result_with_conflicts() {
        let result = MergeResult {
            success: false,
            message: "Merge failed".to_string(),
            conflicts: vec![PathBuf::from("src/main.rs")],
        };

        assert!(!result.success);
        assert_eq!(result.conflicts.len(), 1);
        assert_eq!(result.conflicts[0], PathBuf::from("src/main.rs"));
    }

    #[test]
    fn test_generate_session_name_handles_unknown_project() {
        // When the path has no file name (root), it should use "unknown"
        let path = PathBuf::from("/");
        let session_name = Orchestrator::generate_session_name(&path);

        // Should still generate a valid session name
        assert!(session_name.starts_with("wta-"));
    }

    #[test]
    fn test_constants() {
        assert_eq!(TMUX_SESSION_PREFIX, "wta");
        assert_eq!(WORKTREES_DIR, ".worktrees");
        assert_eq!(STATE_DIR, ".worktree-agents");
    }
}
