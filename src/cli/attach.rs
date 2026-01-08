use crate::orchestrator::Orchestrator;
use crate::Result;

pub async fn run(id: String) -> Result<()> {
    let orchestrator = Orchestrator::new()?;
    orchestrator.attach(&id)?;
    Ok(())
}
