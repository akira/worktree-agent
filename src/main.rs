use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, EnvFilter};
use worktree_agent::cli;
use worktree_agent::orchestrator::{AgentStatus, MergeStrategy};

#[derive(Parser)]
#[command(
    name = "wta",
    about = "Worktree Agent - spawn Claude Code agents in isolated git worktrees"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Spawn a new agent with a task
    Spawn {
        /// The task for the agent to perform
        #[arg(short, long)]
        task: String,

        /// Branch name (auto-generated if not provided)
        #[arg(short, long)]
        branch: Option<String>,

        /// Base branch to fork from (default: current branch)
        #[arg(long)]
        base: Option<String>,

        /// Extra arguments to pass to claude
        #[arg(last = true)]
        claude_args: Vec<String>,
    },

    /// List all agents
    List,

    /// Get status and output of an agent
    Status {
        /// Agent ID
        id: String,

        /// Number of lines to capture from tmux
        #[arg(short, long, default_value = "50")]
        lines: usize,
    },

    /// Attach to an agent's tmux window
    Attach {
        /// Agent ID
        id: String,

        /// Open VS Code in the worktree directory
        #[arg(long)]
        code: bool,
    },

    /// Merge agent's work back to base branch
    Merge {
        /// Agent ID
        id: String,

        /// Merge strategy
        #[arg(long, value_enum, default_value = "merge")]
        strategy: MergeStrategy,

        /// Force merge even if agent status is unknown
        #[arg(short, long)]
        force: bool,
    },

    /// Remove agent, kill window, and cleanup worktree
    Remove {
        /// Agent ID
        id: String,

        /// Force remove even if agent is still running
        #[arg(short, long)]
        force: bool,
    },

    /// Prune stale agents and clean up their resources
    Prune {
        /// Prune all agents including running ones
        #[arg(short, long)]
        all: bool,

        /// Only prune agents with this status (merged, completed, failed, running)
        #[arg(short, long, value_enum)]
        status: Option<AgentStatus>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Spawn {
            task,
            branch,
            base,
            claude_args,
        } => cli::spawn::run(task, branch, base, claude_args).await?,

        Commands::List => cli::list::run().await?,

        Commands::Status { id, lines } => cli::status::run(id, lines).await?,

        Commands::Attach { id, code } => cli::attach::run(id, code).await?,

        Commands::Merge { id, strategy, force } => cli::merge::run(id, strategy, force).await?,

        Commands::Remove { id, force } => cli::remove::run(id, force).await?,

        Commands::Prune { all, status } => cli::prune::run(all, status).await?,
    }

    Ok(())
}
