use crate::orchestrator::Orchestrator;
use crate::Result;
use tabled::{Table, Tabled};

const TASK_MAX_LEN: usize = 50;
const TASK_TRUNCATE_LEN: usize = 47;

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

    let mut rows = Vec::with_capacity(agents.len());
    for a in agents {
        let task = if a.task.len() > TASK_MAX_LEN {
            format!("{}...", &a.task[..TASK_TRUNCATE_LEN])
        } else {
            a.task.clone()
        };

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
