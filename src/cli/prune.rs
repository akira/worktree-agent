use crate::orchestrator::{AgentStatus, Orchestrator, PruneFilter};
use crate::Result;
use tabled::{Table, Tabled};

const TASK_MAX_LEN: usize = 50;
const TASK_TRUNCATE_LEN: usize = 47;

#[derive(Tabled)]
struct PrunedAgentRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "BRANCH")]
    branch: String,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "TASK")]
    task: String,
}

pub async fn run(all: bool, status: Option<AgentStatus>) -> Result<()> {
    let mut orchestrator = Orchestrator::new()?;

    let filter = if all {
        PruneFilter::All
    } else if let Some(s) = status {
        PruneFilter::Status(s)
    } else {
        // Default: prune Merged, Completed, and Failed
        PruneFilter::Inactive
    };

    let pruned = orchestrator.prune(filter).await?;

    if pruned.is_empty() {
        println!("No agents to prune.");
        return Ok(());
    }

    let mut rows = Vec::with_capacity(pruned.len());
    for agent in &pruned {
        let task = if agent.task.len() > TASK_MAX_LEN {
            format!("{}...", &agent.task[..TASK_TRUNCATE_LEN])
        } else {
            agent.task.clone()
        };

        rows.push(PrunedAgentRow {
            id: agent.id.0.clone(),
            branch: agent.branch.clone(),
            status: agent.status.to_string(),
            task,
        });
    }

    println!("Pruned {} agent(s):", pruned.len());
    let table = Table::new(rows).to_string();
    println!("{table}");

    Ok(())
}
