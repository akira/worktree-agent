use crate::orchestrator::Orchestrator;
use crate::Result;

pub async fn run(id: String, force: bool, delete_branch: bool) -> Result<()> {
    let mut orchestrator = Orchestrator::new()?;

    orchestrator.remove(&id, force, delete_branch).await?;

    println!("Removed agent {id}");

    Ok(())
}
