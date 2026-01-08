use crate::error::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct WorktreeManager {
    repo_root: PathBuf,
    worktrees_dir: PathBuf,
}

impl WorktreeManager {
    pub fn new(repo_root: &Path, worktrees_dir: &Path) -> Self {
        Self {
            repo_root: repo_root.to_path_buf(),
            worktrees_dir: worktrees_dir.to_path_buf(),
        }
    }

    /// Create a new worktree with a new branch based on the given base branch
    pub fn create(&self, id: &str, branch: &str, base: &str) -> Result<PathBuf> {
        let worktree_path = self.worktrees_dir.join(id);

        if worktree_path.exists() {
            return Err(Error::WorktreeAlreadyExists(worktree_path));
        }

        // Create worktree with new branch
        let output = Command::new("git")
            .current_dir(&self.repo_root)
            .args([
                "worktree",
                "add",
                "-b",
                branch,
                worktree_path.to_str().unwrap(),
                base,
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("already exists") {
                return Err(Error::BranchAlreadyExists(branch.to_string()));
            }
            return Err(Error::CommandFailed {
                command: "git worktree add".to_string(),
                code: output.status.code(),
                stderr: stderr.to_string(),
            });
        }

        Ok(worktree_path)
    }

    /// Remove a worktree
    pub fn remove(&self, id: &str) -> Result<()> {
        let worktree_path = self.worktrees_dir.join(id);

        if !worktree_path.exists() {
            return Err(Error::WorktreeNotFound(worktree_path));
        }

        // Remove worktree (force to handle uncommitted changes)
        let output = Command::new("git")
            .current_dir(&self.repo_root)
            .args([
                "worktree",
                "remove",
                "--force",
                worktree_path.to_str().unwrap(),
            ])
            .output()?;

        if !output.status.success() {
            return Err(Error::CommandFailed {
                command: "git worktree remove".to_string(),
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        Ok(())
    }

    /// List all worktrees
    pub fn list(&self) -> Result<Vec<WorktreeInfo>> {
        let output = Command::new("git")
            .current_dir(&self.repo_root)
            .args(["worktree", "list", "--porcelain"])
            .output()?;

        if !output.status.success() {
            return Err(Error::CommandFailed {
                command: "git worktree list".to_string(),
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut worktrees = Vec::new();
        let mut current_path = None;
        let mut current_branch = None;

        for line in stdout.lines() {
            if let Some(path) = line.strip_prefix("worktree ") {
                if let (Some(p), Some(b)) = (current_path.take(), current_branch.take()) {
                    worktrees.push(WorktreeInfo {
                        path: PathBuf::from(p),
                        branch: b,
                    });
                }
                current_path = Some(path.to_string());
            } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
                current_branch = Some(branch.to_string());
            }
        }

        // Don't forget the last one
        if let (Some(p), Some(b)) = (current_path, current_branch) {
            worktrees.push(WorktreeInfo {
                path: PathBuf::from(p),
                branch: b,
            });
        }

        Ok(worktrees)
    }
}

#[derive(Debug)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
}
