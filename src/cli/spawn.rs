use crate::orchestrator::{Orchestrator, SpawnRequest};
use crate::Result;
use std::process::Command;

pub async fn run(
    task: String,
    branch: Option<String>,
    base: Option<String>,
    code: bool,
    claude_args: Vec<String>,
) -> Result<()> {
    let mut orchestrator = Orchestrator::new()?;

    let request = SpawnRequest {
        task: task.clone(),
        branch,
        base,
        claude_args,
    };

    let id = orchestrator.spawn(request).await?;

    println!("Spawned agent {} on branch wta/{}", id, id);
    println!("Task: {task}");
    println!();
    println!("Use 'wta attach {}' to watch the agent", id);
    println!("Use 'wta status {}' to check progress", id);

    if code {
        let worktree_path = orchestrator.worktrees_dir().join(id.to_string());
        if let Err(e) = Command::new("code").arg(&worktree_path).spawn() {
            eprintln!("Failed to open VS Code: {e}");
        }
    }

    Ok(())
}
