use crate::cli::truncate_task;
use crate::orchestrator::{AgentStatus, Orchestrator};
use crate::Result;
use colored::Colorize;
use tabled::settings::style::Style;
use tabled::settings::Padding;
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

fn colorize_status(status: &AgentStatus) -> String {
    match status {
        AgentStatus::Running => status.to_string().bright_blue().bold().to_string(),
        AgentStatus::Completed => status.to_string().magenta().to_string(),
        AgentStatus::Failed => status.to_string().red().bold().to_string(),
        AgentStatus::Merged => status.to_string().green().to_string(),
        AgentStatus::Conflict => status.to_string().yellow().bold().to_string(),
    }
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
            id: a.id.0.bright_white().to_string(),
            branch: a.branch.cyan().to_string(),
            status: colorize_status(&a.status),
            task: task.white().to_string(),
        });
    }

    let table = Table::new(rows)
        .with(Style::rounded())
        .with(Padding::new(1, 1, 0, 0))
        .to_string();
    println!("{table}");

    Ok(())
}
