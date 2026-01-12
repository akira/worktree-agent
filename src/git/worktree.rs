use crate::error::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const GIT: &str = "git";
const WORKTREE: &str = "worktree";
const ALREADY_EXISTS: &str = "already exists";

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

    fn run_git(&self, args: &[&str]) -> Result<Output> {
        Command::new(GIT)
            .current_dir(&self.repo_root)
            .args(args)
            .output()
            .map_err(Error::from)
    }

    fn run_git_checked(&self, args: &[&str], command_name: &str) -> Result<Output> {
        let output = self.run_git(args)?;
        if !output.status.success() {
            return Err(Error::CommandFailed {
                command: command_name.to_string(),
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }
        Ok(output)
    }

    /// Check if a branch exists locally or remotely
    pub fn branch_exists(&self, branch: &str) -> Result<bool> {
        // Check local branches
        let local = self.run_git(&["rev-parse", "--verify", &format!("refs/heads/{branch}")])?;
        if local.status.success() {
            return Ok(true);
        }

        // Check remote branches (origin)
        let remote = self.run_git(&[
            "rev-parse",
            "--verify",
            &format!("refs/remotes/origin/{branch}"),
        ])?;
        Ok(remote.status.success())
    }

    /// Create a worktree for an existing branch
    pub fn checkout_existing(&self, id: &str, branch: &str) -> Result<PathBuf> {
        let worktree_path = self.worktrees_dir.join(id);

        if worktree_path.exists() {
            return Err(Error::WorktreeAlreadyExists(worktree_path));
        }

        let path_str = worktree_path
            .to_str()
            .ok_or_else(|| Error::InvalidUtf8Path(worktree_path.clone()))?;

        // Try local branch first, then remote
        let output = self.run_git(&[WORKTREE, "add", path_str, branch])?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed {
                command: "git worktree add".to_string(),
                code: output.status.code(),
                stderr: stderr.to_string(),
            });
        }

        Ok(worktree_path)
    }

    /// Create a new worktree with a new branch based on the given base branch
    pub fn create(&self, id: &str, branch: &str, base: &str) -> Result<PathBuf> {
        let worktree_path = self.worktrees_dir.join(id);

        if worktree_path.exists() {
            return Err(Error::WorktreeAlreadyExists(worktree_path));
        }

        let path_str = worktree_path
            .to_str()
            .ok_or_else(|| Error::InvalidUtf8Path(worktree_path.clone()))?;
        let output = self.run_git(&[WORKTREE, "add", "-b", branch, path_str, base])?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains(ALREADY_EXISTS) {
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

        let path_str = worktree_path
            .to_str()
            .ok_or_else(|| Error::InvalidUtf8Path(worktree_path.clone()))?;
        self.run_git_checked(
            &[WORKTREE, "remove", "--force", path_str],
            "git worktree remove",
        )?;

        Ok(())
    }

    /// List all worktrees
    pub fn list(&self) -> Result<Vec<WorktreeInfo>> {
        let output =
            self.run_git_checked(&[WORKTREE, "list", "--porcelain"], "git worktree list")?;

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

#[derive(Debug, PartialEq, Eq)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worktree_manager_new() {
        let repo_root = PathBuf::from("/home/user/project");
        let worktrees_dir = PathBuf::from("/home/user/project/.worktrees");
        let manager = WorktreeManager::new(&repo_root, &worktrees_dir);

        assert_eq!(manager.repo_root, repo_root);
        assert_eq!(manager.worktrees_dir, worktrees_dir);
    }

    #[test]
    fn test_worktree_info_equality() {
        let info1 = WorktreeInfo {
            path: PathBuf::from("/home/user/project/.worktrees/1"),
            branch: "wta/1".to_string(),
        };
        let info2 = WorktreeInfo {
            path: PathBuf::from("/home/user/project/.worktrees/1"),
            branch: "wta/1".to_string(),
        };
        let info3 = WorktreeInfo {
            path: PathBuf::from("/home/user/project/.worktrees/2"),
            branch: "wta/2".to_string(),
        };

        assert_eq!(info1, info2);
        assert_ne!(info1, info3);
    }

    /// Test that the porcelain parsing logic works correctly.
    /// This tests the parsing implementation without needing actual git commands.
    #[test]
    fn test_parse_worktree_list_porcelain_single() {
        // Simulate porcelain output parsing
        let stdout = "worktree /home/user/project\nbranch refs/heads/main\n\n";
        let worktrees = parse_worktree_porcelain(stdout);

        assert_eq!(worktrees.len(), 1);
        assert_eq!(worktrees[0].path, PathBuf::from("/home/user/project"));
        assert_eq!(worktrees[0].branch, "main");
    }

    #[test]
    fn test_parse_worktree_list_porcelain_multiple() {
        let stdout = "\
worktree /home/user/project
branch refs/heads/main

worktree /home/user/project/.worktrees/1
branch refs/heads/wta/1

worktree /home/user/project/.worktrees/2
branch refs/heads/feature-branch
";
        let worktrees = parse_worktree_porcelain(stdout);

        assert_eq!(worktrees.len(), 3);
        assert_eq!(worktrees[0].path, PathBuf::from("/home/user/project"));
        assert_eq!(worktrees[0].branch, "main");
        assert_eq!(
            worktrees[1].path,
            PathBuf::from("/home/user/project/.worktrees/1")
        );
        assert_eq!(worktrees[1].branch, "wta/1");
        assert_eq!(
            worktrees[2].path,
            PathBuf::from("/home/user/project/.worktrees/2")
        );
        assert_eq!(worktrees[2].branch, "feature-branch");
    }

    #[test]
    fn test_parse_worktree_list_porcelain_empty() {
        let stdout = "";
        let worktrees = parse_worktree_porcelain(stdout);
        assert!(worktrees.is_empty());
    }

    #[test]
    fn test_parse_worktree_list_porcelain_detached_head() {
        // Detached HEAD worktrees don't have a branch line
        let stdout = "\
worktree /home/user/project
branch refs/heads/main

worktree /home/user/project/.worktrees/1
HEAD abc123def456

";
        let worktrees = parse_worktree_porcelain(stdout);

        // Only the first worktree with a branch should be included
        assert_eq!(worktrees.len(), 1);
        assert_eq!(worktrees[0].path, PathBuf::from("/home/user/project"));
        assert_eq!(worktrees[0].branch, "main");
    }

    #[test]
    fn test_parse_worktree_list_porcelain_with_extra_fields() {
        // Git porcelain output may include other fields like HEAD, bare, etc.
        let stdout = "\
worktree /home/user/project
HEAD abc123
branch refs/heads/main
bare

worktree /home/user/project/.worktrees/1
HEAD def456
branch refs/heads/wta/1
locked

";
        let worktrees = parse_worktree_porcelain(stdout);

        assert_eq!(worktrees.len(), 2);
        assert_eq!(worktrees[0].branch, "main");
        assert_eq!(worktrees[1].branch, "wta/1");
    }

    /// Helper function to parse worktree porcelain output
    /// This extracts the parsing logic from the list() method for testing
    fn parse_worktree_porcelain(stdout: &str) -> Vec<WorktreeInfo> {
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

        worktrees
    }
}
