use crate::orchestrator::Orchestrator;
use crate::Result;
use std::process::Command;

/// Display diff using lumen (with git diff fallback) for an agent's changes
pub async fn run(id: String, use_git: bool) -> Result<()> {
    let mut orchestrator = Orchestrator::new()?;

    // Check status (updates from status file if exists)
    orchestrator.check_status(&id)?;

    let agent = orchestrator.get_agent(&id)?;
    let base_branch = &agent.base_branch;
    let worktree_path = &agent.worktree_path;

    // Use three-dot diff to show changes since branch diverged from base
    let diff_range = format!("{base_branch}...HEAD");

    if use_git {
        Command::new("git")
            .args(["diff", &diff_range])
            .current_dir(worktree_path)
            .status()?;
        return Ok(());
    }

    // Try lumen first for interactive side-by-side diff
    let lumen_result = Command::new("lumen")
        .args(["diff", &diff_range])
        .current_dir(worktree_path)
        .status();

    match lumen_result {
        Ok(status) if status.success() => {}
        _ => {
            // Fall back to git diff if lumen is not available or failed
            eprintln!("lumen not available, falling back to git diff\n");
            Command::new("git")
                .args(["diff", &diff_range])
                .current_dir(worktree_path)
                .status()?;
        }
    }

    Ok(())
}
