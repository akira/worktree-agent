use clap::{Parser, Subcommand, ValueEnum};
use tracing_subscriber::{fmt, EnvFilter};
use worktree_agent::cli;

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
    },

    /// Open dashboard with all agents in split panes
    Dashboard,

    /// Merge agent's work back to base branch
    Merge {
        /// Agent ID
        id: String,

        /// Merge strategy
        #[arg(long, default_value = "merge")]
        strategy: MergeStrategy,
    },

    /// Discard agent's work and cleanup
    Discard {
        /// Agent ID
        id: String,

        /// Force discard even if agent is still running
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum MergeStrategy {
    Merge,
    Rebase,
    Squash,
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

        Commands::Attach { id } => cli::attach::run(id).await?,

        Commands::Dashboard => cli::dashboard::run().await?,

        Commands::Merge { id, strategy } => {
            let strategy = match strategy {
                MergeStrategy::Merge => worktree_agent::orchestrator::MergeStrategy::Merge,
                MergeStrategy::Rebase => worktree_agent::orchestrator::MergeStrategy::Rebase,
                MergeStrategy::Squash => worktree_agent::orchestrator::MergeStrategy::Squash,
            };
            cli::merge::run(id, strategy).await?
        }

        Commands::Discard { id, force } => cli::discard::run(id, force).await?,
    }

    Ok(())
}
