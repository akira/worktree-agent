use crate::error::{Error, Result};
use crate::orchestrator::MergeStrategy;
use crate::orchestrator::MergeResult;
use std::path::Path;
use std::process::Command;

/// Merge a branch back into the base branch
pub fn merge_branch(
    repo_root: &Path,
    branch: &str,
    base_branch: &str,
    strategy: MergeStrategy,
) -> Result<MergeResult> {
    // First, checkout the base branch
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["checkout", base_branch])
        .output()?;

    if !output.status.success() {
        return Err(Error::CommandFailed {
            command: "git checkout".to_string(),
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    match strategy {
        MergeStrategy::Merge => do_merge(repo_root, branch),
        MergeStrategy::Rebase => do_rebase(repo_root, branch, base_branch),
        MergeStrategy::Squash => do_squash_merge(repo_root, branch),
    }
}

fn do_merge(repo_root: &Path, branch: &str) -> Result<MergeResult> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["merge", branch, "--no-edit"])
        .output()?;

    if output.status.success() {
        Ok(MergeResult {
            success: true,
            message: format!("Successfully merged {branch}"),
            conflicts: Vec::new(),
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("CONFLICT") || stderr.contains("conflict") {
            // Get list of conflicting files
            let conflicts = get_conflict_files(repo_root)?;

            // Abort the merge
            let _ = Command::new("git")
                .current_dir(repo_root)
                .args(["merge", "--abort"])
                .output();

            Err(Error::MergeConflict(conflicts))
        } else {
            Err(Error::CommandFailed {
                command: "git merge".to_string(),
                code: output.status.code(),
                stderr: stderr.to_string(),
            })
        }
    }
}

fn do_rebase(repo_root: &Path, branch: &str, base_branch: &str) -> Result<MergeResult> {
    // Checkout the feature branch
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["checkout", branch])
        .output()?;

    if !output.status.success() {
        return Err(Error::CommandFailed {
            command: "git checkout".to_string(),
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    // Rebase onto base branch
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["rebase", base_branch])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("CONFLICT") || stderr.contains("conflict") {
            let conflicts = get_conflict_files(repo_root)?;

            // Abort the rebase
            let _ = Command::new("git")
                .current_dir(repo_root)
                .args(["rebase", "--abort"])
                .output();

            return Err(Error::MergeConflict(conflicts));
        }
        return Err(Error::CommandFailed {
            command: "git rebase".to_string(),
            code: output.status.code(),
            stderr: stderr.to_string(),
        });
    }

    // Fast-forward merge back to base
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["checkout", base_branch])
        .output()?;

    if !output.status.success() {
        return Err(Error::CommandFailed {
            command: "git checkout".to_string(),
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["merge", "--ff-only", branch])
        .output()?;

    if output.status.success() {
        Ok(MergeResult {
            success: true,
            message: format!("Successfully rebased and merged {branch}"),
            conflicts: Vec::new(),
        })
    } else {
        Err(Error::CommandFailed {
            command: "git merge --ff-only".to_string(),
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

fn do_squash_merge(repo_root: &Path, branch: &str) -> Result<MergeResult> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["merge", "--squash", branch])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("CONFLICT") || stderr.contains("conflict") {
            let conflicts = get_conflict_files(repo_root)?;

            // Reset to abort the squash merge
            let _ = Command::new("git")
                .current_dir(repo_root)
                .args(["reset", "--hard", "HEAD"])
                .output();

            return Err(Error::MergeConflict(conflicts));
        }
        return Err(Error::CommandFailed {
            command: "git merge --squash".to_string(),
            code: output.status.code(),
            stderr: stderr.to_string(),
        });
    }

    // Commit the squashed changes
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["commit", "--no-edit"])
        .output()?;

    if output.status.success() {
        Ok(MergeResult {
            success: true,
            message: format!("Successfully squash-merged {branch}"),
            conflicts: Vec::new(),
        })
    } else {
        Err(Error::CommandFailed {
            command: "git commit".to_string(),
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

fn get_conflict_files(repo_root: &Path) -> Result<Vec<std::path::PathBuf>> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["diff", "--name-only", "--diff-filter=U"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .map(|l| std::path::PathBuf::from(l.trim()))
        .collect())
}
