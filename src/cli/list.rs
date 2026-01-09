use crate::cli::truncate_task;
use crate::orchestrator::{AgentStatus, Orchestrator};
use crate::Result;
use tabled::{Table, Tabled};

const TASK_MAX_LEN: usize = 50;

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
    let mut orchestrator = Orchestrator::new()?;

    // Collect agent IDs that need status refresh (those currently showing as Running)
    let running_ids: Vec<String> = orchestrator
        .list()
        .iter()
        .filter(|a| a.status == AgentStatus::Running)
        .map(|a| a.id.0.clone())
        .collect();

    // Refresh status for running agents (checks status file and tmux window existence)
    for id in &running_ids {
        let _ = orchestrator.check_status(id);
    }

    // Now get the updated list
    let agents = orchestrator.list();

    if agents.is_empty() {
        println!("No agents running.");
        return Ok(());
    }

    let mut rows = Vec::with_capacity(agents.len());
    for a in agents {
        let task = truncate_task(&a.task, TASK_MAX_LEN);

        rows.push(AgentRow {
            id: a.id.0.clone(),
            branch: a.branch.clone(),
            status: a.status.to_string(),
            task,
        });
    }

    let table = Table::new(rows).to_string();
    println!("{table}");

    Ok(())
}
