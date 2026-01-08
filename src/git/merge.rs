use crate::error::{Error, Result};
use crate::orchestrator::{MergeResult, MergeStrategy};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const GIT: &str = "git";
const CONFLICT_UPPER: &str = "CONFLICT";
const CONFLICT_LOWER: &str = "conflict";

fn run_git(repo_root: &Path, args: &[&str]) -> Result<Output> {
    Command::new(GIT)
        .current_dir(repo_root)
        .args(args)
        .output()
        .map_err(Error::from)
}

fn run_git_checked(repo_root: &Path, args: &[&str], command_name: &str) -> Result<Output> {
    let output = run_git(repo_root, args)?;
    if !output.status.success() {
        return Err(Error::CommandFailed {
            command: command_name.to_string(),
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    Ok(output)
}

fn has_conflict(stderr: &str) -> bool {
    stderr.contains(CONFLICT_UPPER) || stderr.contains(CONFLICT_LOWER)
}

fn checkout(repo_root: &Path, branch: &str) -> Result<()> {
    run_git_checked(repo_root, &["checkout", branch], "git checkout")?;
    Ok(())
}

/// Merge a branch back into the base branch
pub fn merge_branch(
    repo_root: &Path,
    branch: &str,
    base_branch: &str,
    strategy: MergeStrategy,
) -> Result<MergeResult> {
    checkout(repo_root, base_branch)?;

    match strategy {
        MergeStrategy::Merge => do_merge(repo_root, branch),
        MergeStrategy::Rebase => do_rebase(repo_root, branch, base_branch),
        MergeStrategy::Squash => do_squash_merge(repo_root, branch),
    }
}

fn do_merge(repo_root: &Path, branch: &str) -> Result<MergeResult> {
    let output = run_git(repo_root, &["merge", branch, "--no-edit"])?;

    if output.status.success() {
        return Ok(MergeResult {
            success: true,
            message: format!("Successfully merged {branch}"),
            conflicts: Vec::new(),
        });
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if has_conflict(&stderr) {
        let conflicts = get_conflict_files(repo_root)?;
        let _ = run_git(repo_root, &["merge", "--abort"]);
        return Err(Error::MergeConflict(conflicts));
    }

    Err(Error::CommandFailed {
        command: "git merge".to_string(),
        code: output.status.code(),
        stderr: stderr.to_string(),
    })
}

fn do_rebase(repo_root: &Path, branch: &str, base_branch: &str) -> Result<MergeResult> {
    checkout(repo_root, branch)?;

    let output = run_git(repo_root, &["rebase", base_branch])?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if has_conflict(&stderr) {
            let conflicts = get_conflict_files(repo_root)?;
            let _ = run_git(repo_root, &["rebase", "--abort"]);
            return Err(Error::MergeConflict(conflicts));
        }
        return Err(Error::CommandFailed {
            command: "git rebase".to_string(),
            code: output.status.code(),
            stderr: stderr.to_string(),
        });
    }

    checkout(repo_root, base_branch)?;
    run_git_checked(
        repo_root,
        &["merge", "--ff-only", branch],
        "git merge --ff-only",
    )?;

    Ok(MergeResult {
        success: true,
        message: format!("Successfully rebased and merged {branch}"),
        conflicts: Vec::new(),
    })
}

fn do_squash_merge(repo_root: &Path, branch: &str) -> Result<MergeResult> {
    let output = run_git(repo_root, &["merge", "--squash", branch])?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if has_conflict(&stderr) {
            let conflicts = get_conflict_files(repo_root)?;
            let _ = run_git(repo_root, &["reset", "--hard", "HEAD"]);
            return Err(Error::MergeConflict(conflicts));
        }
        return Err(Error::CommandFailed {
            command: "git merge --squash".to_string(),
            code: output.status.code(),
            stderr: stderr.to_string(),
        });
    }

    run_git_checked(repo_root, &["commit", "--no-edit"], "git commit")?;

    Ok(MergeResult {
        success: true,
        message: format!("Successfully squash-merged {branch}"),
        conflicts: Vec::new(),
    })
}

fn get_conflict_files(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let output = run_git(repo_root, &["diff", "--name-only", "--diff-filter=U"])?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().map(|l| PathBuf::from(l.trim())).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_conflict_detects_uppercase() {
        assert!(has_conflict("CONFLICT (content): Merge conflict in file.rs"));
    }

    #[test]
    fn test_has_conflict_detects_lowercase() {
        assert!(has_conflict("Auto-merging file.rs\nconflict: could not merge"));
    }

    #[test]
    fn test_has_conflict_returns_false_for_clean_merge() {
        assert!(!has_conflict("Auto-merging file.rs\nMerge made by recursive strategy."));
    }

    #[test]
    fn test_has_conflict_returns_false_for_empty() {
        assert!(!has_conflict(""));
    }

    #[test]
    fn test_has_conflict_with_mixed_case() {
        // The function checks for both CONFLICT and conflict
        assert!(has_conflict("some text with CONFLICT in it"));
        assert!(has_conflict("some text with conflict in it"));
        assert!(!has_conflict("some text with Conflict in it")); // Mixed case not detected
    }

    #[test]
    fn test_merge_strategy_enum_values() {
        // Ensure all merge strategies are distinct
        assert_ne!(
            std::mem::discriminant(&MergeStrategy::Merge),
            std::mem::discriminant(&MergeStrategy::Rebase)
        );
        assert_ne!(
            std::mem::discriminant(&MergeStrategy::Merge),
            std::mem::discriminant(&MergeStrategy::Squash)
        );
        assert_ne!(
            std::mem::discriminant(&MergeStrategy::Rebase),
            std::mem::discriminant(&MergeStrategy::Squash)
        );
    }

    #[test]
    fn test_merge_result_success_fields() {
        let result = MergeResult {
            success: true,
            message: "Successfully merged branch".to_string(),
            conflicts: Vec::new(),
        };

        assert!(result.success);
        assert!(result.conflicts.is_empty());
        assert!(result.message.contains("Successfully"));
    }

    #[test]
    fn test_merge_result_with_conflicts() {
        let conflicts = vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("src/lib.rs"),
        ];
        let result = MergeResult {
            success: false,
            message: "Merge failed due to conflicts".to_string(),
            conflicts: conflicts.clone(),
        };

        assert!(!result.success);
        assert_eq!(result.conflicts.len(), 2);
        assert_eq!(result.conflicts[0], PathBuf::from("src/main.rs"));
        assert_eq!(result.conflicts[1], PathBuf::from("src/lib.rs"));
    }

    /// Test helper for parsing conflict file output
    #[test]
    fn test_parse_conflict_files_output() {
        let output = "src/main.rs\nsrc/lib.rs\nCargo.toml\n";
        let files: Vec<PathBuf> = output.lines().map(|l| PathBuf::from(l.trim())).collect();

        assert_eq!(files.len(), 3);
        assert_eq!(files[0], PathBuf::from("src/main.rs"));
        assert_eq!(files[1], PathBuf::from("src/lib.rs"));
        assert_eq!(files[2], PathBuf::from("Cargo.toml"));
    }

    #[test]
    fn test_parse_conflict_files_empty() {
        let output = "";
        let files: Vec<PathBuf> = output
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| PathBuf::from(l.trim()))
            .collect();

        assert!(files.is_empty());
    }

    #[test]
    fn test_parse_conflict_files_with_whitespace() {
        let output = "  src/main.rs  \n  src/lib.rs  \n";
        let files: Vec<PathBuf> = output.lines().map(|l| PathBuf::from(l.trim())).collect();

        assert_eq!(files.len(), 2);
        assert_eq!(files[0], PathBuf::from("src/main.rs"));
        assert_eq!(files[1], PathBuf::from("src/lib.rs"));
    }
}
