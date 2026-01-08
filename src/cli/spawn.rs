use crate::orchestrator::{Orchestrator, SpawnRequest};
use crate::Result;

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

    if code {
        orchestrator.open_vscode(&id.0)?;
    }

    println!("Spawned agent {} on branch wta/{}", id, id);
    println!("Task: {task}");
    println!();
    println!("Use 'wta attach {}' to watch the agent", id);
    println!("Use 'wta status {}' to check progress", id);

    Ok(())
}
