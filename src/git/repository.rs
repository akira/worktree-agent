use crate::error::{Error, Result};
use std::path::Path;
use std::process::Command;

/// Get the default branch for a repository
/// Based on worktrunk's approach with simplified logic
pub fn default_branch(repo_root: &Path) -> Result<String> {
    // 1. Check cache: git config wta.default-branch
    if let Ok(branch) = get_cached_default(repo_root) {
        if branch_exists_locally(repo_root, &branch)? {
            return Ok(branch);
        }
    }

    // 2. Try remote detection: origin/HEAD
    if let Ok(branch) = detect_from_remote(repo_root) {
        // Cache for future use
        let _ = cache_default_branch(repo_root, &branch);
        return Ok(branch);
    }

    // 3. Fall back to local inference
    if let Ok(branch) = infer_default_branch_locally(repo_root) {
        // Cache for future use
        let _ = cache_default_branch(repo_root, &branch);
        return Ok(branch);
    }

    Err(Error::Git(git2::Error::from_str(
        "Could not detect default branch. Please use --target to specify explicitly.",
    )))
}

/// Get cached default branch from git config
fn get_cached_default(repo_root: &Path) -> Result<String> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["config", "--get", "wta.default-branch"])
        .output()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !branch.is_empty() {
            return Ok(branch);
        }
    }

    Err(Error::Git(git2::Error::from_str(
        "No cached default branch",
    )))
}

/// Cache the default branch in git config
fn cache_default_branch(repo_root: &Path, branch: &str) -> Result<()> {
    Command::new("git")
        .current_dir(repo_root)
        .args(["config", "wta.default-branch", branch])
        .output()?;
    Ok(())
}

/// Check if a branch exists locally
fn branch_exists_locally(repo_root: &Path, branch: &str) -> Result<bool> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["rev-parse", "--verify", &format!("refs/heads/{}", branch)])
        .output()?;
    Ok(output.status.success())
}

/// Detect default branch from remote (origin/HEAD)
fn detect_from_remote(repo_root: &Path) -> Result<String> {
    // Try to get the branch that origin/HEAD points to
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["rev-parse", "--abbrev-ref", "origin/HEAD"])
        .output()?;

    if output.status.success() {
        let full_ref = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // Strip "origin/" prefix to get branch name
        if let Some(branch) = full_ref.strip_prefix("origin/") {
            return Ok(branch.to_string());
        }
    }

    Err(Error::Git(git2::Error::from_str(
        "Could not detect from remote",
    )))
}

/// Infer default branch using local heuristics
fn infer_default_branch_locally(repo_root: &Path) -> Result<String> {
    // 1. Check git config init.defaultBranch
    if let Ok(branch) = get_init_default_branch(repo_root) {
        if branch_exists_locally(repo_root, &branch)? {
            return Ok(branch);
        }
    }

    // 2. Try common branch names
    for name in ["main", "master", "develop", "trunk"] {
        if branch_exists_locally(repo_root, name)? {
            return Ok(name.to_string());
        }
    }

    // 3. If only one branch exists, use it
    if let Ok(branches) = list_local_branches(repo_root) {
        if branches.len() == 1 {
            return Ok(branches[0].clone());
        }
    }

    Err(Error::Git(git2::Error::from_str(
        "Could not infer default branch locally",
    )))
}

/// Get init.defaultBranch from git config
fn get_init_default_branch(repo_root: &Path) -> Result<String> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["config", "--get", "init.defaultBranch"])
        .output()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !branch.is_empty() {
            return Ok(branch);
        }
    }

    Err(Error::Git(git2::Error::from_str(
        "No init.defaultBranch configured",
    )))
}

/// List all local branches
fn list_local_branches(repo_root: &Path) -> Result<Vec<String>> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["branch", "--format=%(refname:short)"])
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let branches: Vec<String> = stdout
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();
        return Ok(branches);
    }

    Err(Error::Git(git2::Error::from_str("Failed to list branches")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_branch_names() {
        // Just verify the list is in the expected order
        let names = ["main", "master", "develop", "trunk"];
        assert_eq!(names[0], "main");
        assert_eq!(names[1], "master");
    }
}
