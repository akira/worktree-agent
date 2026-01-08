use crate::orchestrator::Orchestrator;
use crate::Result;

pub async fn run() -> Result<()> {
    let orchestrator = Orchestrator::new()?;
    orchestrator.dashboard()?;
    Ok(())
}
