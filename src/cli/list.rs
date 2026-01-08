use crate::orchestrator::Orchestrator;
use crate::Result;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct AgentRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "BRANCH")]
    branch: String,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "TASK")]
    task: String,
}

pub async fn run() -> Result<()> {
    let orchestrator = Orchestrator::new()?;
    let agents = orchestrator.list();

    if agents.is_empty() {
        println!("No agents running.");
        return Ok(());
    }

    let rows: Vec<AgentRow> = agents
        .iter()
        .map(|a| {
            let task = if a.task.len() > 50 {
                format!("{}...", &a.task[..47])
            } else {
                a.task.clone()
            };

            AgentRow {
                id: a.id.0.clone(),
                branch: a.branch.clone(),
                status: a.status.to_string(),
                task,
            }
        })
        .collect();

    let table = Table::new(rows).to_string();
    println!("{table}");

    Ok(())
}
