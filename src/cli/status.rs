use crate::orchestrator::Orchestrator;
use crate::Result;

pub async fn run(id: String, lines: usize) -> Result<()> {
    let mut orchestrator = Orchestrator::new()?;

    // Check and update status from status file
    let status = orchestrator.check_status(&id)?;
    let agent = orchestrator.get_agent(&id)?;

    println!("Agent: {}", agent.id);
    println!("Branch: {}", agent.branch);
    println!("Status: {status}");
    println!("Task: {}", agent.task);
    println!();
    println!("--- Recent output (last {lines} lines) ---");
    println!();

    let output = orchestrator.get_output(&id, lines)?;
    print!("{output}");

    Ok(())
}
