use crate::orchestrator::{MergeStrategy, Orchestrator};
use crate::Result;

pub async fn run(id: String, strategy: MergeStrategy) -> Result<()> {
    let mut orchestrator = Orchestrator::new()?;

    // First check status
    orchestrator.check_status(&id)?;

    let result = orchestrator.merge(&id, strategy).await?;

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
