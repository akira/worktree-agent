use crate::orchestrator::{Orchestrator, LaunchRequest};
use crate::provider::Provider;
use crate::Result;

pub async fn run(
    task: String,
    branch: Option<String>,
    base: Option<String>,
    provider: Provider,
    code: bool,
    provider_args: Vec<String>,
) -> Result<()> {
    let mut orchestrator = Orchestrator::new()?;

    let request = LaunchRequest {
        task: task.clone(),
        branch,
        base,
        provider,
        provider_args,
    };

    let id = orchestrator.launch(request).await?;

    if code {
        orchestrator.open_vscode(&id.to_string())?;
    }

    println!("Launched agent {id} on branch wta/{id}");
    println!("Provider: {provider}");
    println!("Task: {task}");
    println!();
    println!("Use 'wta attach {id}' to watch the agent");
    println!("Use 'wta status {id}' to check progress");

    if code {
        orchestrator.open_vscode(&id.0)?;
    }

    Ok(())
}
