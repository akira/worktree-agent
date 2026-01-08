use crate::orchestrator::{Orchestrator, SpawnRequest};
use crate::Result;

pub async fn run(
    task: String,
    branch: Option<String>,
    base: Option<String>,
    open_code: bool,
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

    if open_code {
        orchestrator.open_vscode(&id.to_string())?;
    }

    println!("Spawned agent {id} on branch wta/{id}");
    println!("Task: {task}");
    println!();
    println!("Use 'wta attach {id}' to watch the agent");
    println!("Use 'wta status {id}' to check progress");

    Ok(())
}
