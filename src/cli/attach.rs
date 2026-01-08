use crate::orchestrator::Orchestrator;
use crate::Result;

pub async fn run(id: String, open_code: bool) -> Result<()> {
    let orchestrator = Orchestrator::new()?;

    if open_code {
        orchestrator.open_vscode(&id)?;
    }

    orchestrator.attach(&id)?;
    Ok(())
}
