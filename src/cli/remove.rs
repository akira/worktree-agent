use crate::orchestrator::Orchestrator;
use crate::Result;

pub async fn run(id: String, force: bool) -> Result<()> {
    let mut orchestrator = Orchestrator::new()?;

    orchestrator.remove(&id, force).await?;

    println!("Removed agent {id}");

    Ok(())
}
