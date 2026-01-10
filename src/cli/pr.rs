use crate::orchestrator::Orchestrator;
use crate::Result;

pub async fn run(
    id: String,
    title: Option<String>,
    body: Option<String>,
    force: bool,
) -> Result<()> {
    let mut orchestrator = Orchestrator::new()?;

    // Check status (updates from status file if exists)
    orchestrator.check_status(&id)?;

    let result = orchestrator.create_pr(&id, title, body, force).await?;

    println!("{}", result.url);

    Ok(())
}
