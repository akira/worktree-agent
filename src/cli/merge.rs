use crate::error::Error;
use crate::orchestrator::{MergeStrategy, Orchestrator};
use crate::Result;
use colored::Colorize;

pub async fn run(id: String, strategy: MergeStrategy, force: bool) -> Result<()> {
    let mut orchestrator = Orchestrator::new()?;

    // Check status (updates from status file if exists)
    orchestrator.check_status(&id)?;

    // Get agent info before merge for error messages
    let agent = orchestrator.get_agent(&id)?;
    let branch = agent.branch.clone();

    let result = match orchestrator.merge(&id, strategy, force).await {
        Ok(result) => result,
        Err(Error::MergeConflict(conflicts)) => {
            println!("{}", "Merge conflict detected!".red().bold());
            println!();
            println!("Conflicting files:");
            for file in &conflicts {
                println!("  {} {}", "-".red(), file.display());
            }
            println!();
            println!(
                "{} Resolve conflicts in {}, then run: {}",
                "Fix:".yellow().bold(),
                branch.cyan(),
                format!("wta merge {id}").green()
            );
            return Ok(());
        }
        Err(e) => return Err(e),
    };

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
