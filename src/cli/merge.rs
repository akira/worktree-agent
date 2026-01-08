use crate::orchestrator::{MergeStrategy, Orchestrator};
use crate::Result;

pub async fn run(id: String, strategy: MergeStrategy, force: bool) -> Result<()> {
    let mut orchestrator = Orchestrator::new()?;

    // Check status (updates from status file if exists)
    orchestrator.check_status(&id)?;

    let result = orchestrator.merge(&id, strategy, force).await?;

    if result.success {
        println!("{}", result.message);
    } else {
        println!("Merge failed: {}", result.message);
        if !result.conflicts.is_empty() {
            println!("Conflicting files:");
            for file in &result.conflicts {
                println!("  - {}", file.display());
            }
        }
    }

    Ok(())
}
