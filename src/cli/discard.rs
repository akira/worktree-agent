use crate::orchestrator::Orchestrator;
use crate::Result;

pub async fn run(id: String, force: bool) -> Result<()> {
    let mut orchestrator = Orchestrator::new()?;

    orchestrator.discard(&id, force).await?;

    println!("Discarded agent {id}");

    Ok(())
}
