use crate::Result;
use colored::Colorize;

pub async fn run() -> Result<()> {
    let title = "wta - Worktree Agent".bright_white().bold();
    let subtitle = "Launch AI agents in isolated git worktrees for parallel task execution.";

    println!("{title}\n");
    println!("{subtitle}\n");

    // CLAUDE CODE INTEGRATION
    println!("{}", "CLAUDE CODE INTEGRATION".yellow().bold());
    print_command("wta claude-skill");
    println!("            Install Claude Code skill for automatic agent orchestration");
    println!("            After install: Claude will use wta for parallel tasks automatically");
    println!(
        "            Enables {} slash command in Claude\n",
        "/wta".cyan()
    );

    // GETTING STARTED
    println!("{}", "GETTING STARTED".yellow().bold());
    print_command("wta launch --task \"Fix login bug\"");
    println!("            Creates a new worktree, launches Claude Code agent with your task");
    println!(
        "            Auto-generates branch name as {} (e.g., {})\n",
        "wta/<id>".green(),
        "wta/42".green()
    );

    print_command("wta launch --task \"Add auth\" --branch feature/auth");
    println!("            Launch with custom branch name\n");

    print_command("wta launch --editor");
    println!("            Opens your editor to compose multi-line task descriptions");
    println!(
        "            Supports: {}, {}, or defaults to {}\n",
        "--editor vim".cyan(),
        "--editor code".cyan(),
        "$EDITOR".green()
    );

    // MONITORING AGENTS
    println!("{}", "MONITORING AGENTS".yellow().bold());
    print_command_desc("wta list", "List all agents with status and task");
    print_command_desc("wta status <id>", "Show detailed status and recent output");
    print_command_desc(
        "wta status <id> -l 100",
        "Show more output lines (default: 50)",
    );
    print_command_desc(
        "wta attach <id>",
        "Attach to agent's tmux window (watch live)",
    );
    print_command_desc(
        "wta attach <id> --code",
        "Attach and open VS Code in worktree",
    );
    println!();

    // AI PROVIDERS
    println!("{}", "AI PROVIDERS".yellow().bold());
    print_command_desc(
        "wta launch --task \"...\" --provider claude",
        "Claude Code (default)",
    );
    print_command_desc("wta launch --task \"...\" --provider codex", "OpenAI Codex");
    print_command_desc(
        "wta launch --task \"...\" --provider gemini",
        "Google Gemini",
    );
    print_command_desc(
        "wta launch --task \"...\" --provider deepagents",
        "DeepAgents",
    );
    println!();
    println!("  Pass extra args to provider:");
    print_command("wta launch --task \"...\" -- --model opus");
    print_command("wta launch --task \"...\" -- --agent backend-dev");
    println!();

    // REVIEWING WORK
    println!("{}", "REVIEWING WORK".yellow().bold());
    print_command_desc(
        "wta diff <id>",
        "View changes between agent's branch and base",
    );
    println!(
        "                        Uses {} if available, else {}",
        "lumen".green(),
        "git diff".cyan()
    );
    print_command_desc("wta diff <id> --viewer git", "Force git diff");
    println!();

    // MERGING WORK
    println!("{}", "MERGING WORK".yellow().bold());
    print_command_desc("wta merge <id>", "Merge agent's work back to base branch");
    print_command_desc(
        "wta merge <id> --strategy rebase",
        "Rebase instead of merge",
    );
    print_command_desc(
        "wta merge <id> --strategy squash",
        "Squash commits into one",
    );
    print_command_desc(
        "wta merge <id> --force",
        "Force merge even if agent still running",
    );
    println!();

    // CREATING PULL REQUESTS
    println!("{}", "CREATING PULL REQUESTS".yellow().bold());
    print_command_desc("wta pr <id>", "Create GitHub PR (AI generates title/body)");
    print_command_desc(
        "wta pr <id> --title \"Add OAuth2 support\"",
        "Create PR with custom title",
    );
    print_command_desc(
        "wta pr <id> --title \"Title\" --body \"Description\"",
        "Create PR with custom title and body",
    );
    println!();

    // CLEANUP
    println!("{}", "CLEANUP".yellow().bold());
    print_command_desc(
        "wta remove <id>",
        "Remove agent, kill window, cleanup worktree",
    );
    print_command_desc("wta remove <id> --force", "Force remove running agent");
    print_command_desc("wta prune", "Remove all completed/failed/merged agents");
    print_command_desc("wta prune --all", "Remove all agents including running");
    print_command_desc(
        "wta prune --status completed",
        "Remove only completed agents",
    );
    println!();

    // WEB DASHBOARD
    println!("{}", "WEB DASHBOARD".yellow().bold());
    print_command_desc(
        "wta dashboard",
        "Start web dashboard at http://localhost:3847",
    );
    print_command_desc("wta dashboard --port 8080", "Use custom port");
    print_command_desc("wta dashboard --open", "Auto-open browser");
    println!();
    println!("  Dashboard features:");
    println!(
        "    {} Kanban board: {}, {}, {}, {} columns",
        "•".green(),
        "Running".bright_blue(),
        "Completed".magenta(),
        "Failed".red(),
        "Merged".green()
    );
    println!("    {} Click tasks to view PR diff", "•".green());
    println!("    {} Merge with different strategies", "•".green());
    println!("    {} Create PRs directly from UI", "•".green());
    println!("    {} Remove agents from UI", "•".green());
    println!();

    // WORKTREE MANAGEMENT
    println!("{}", "WORKTREE MANAGEMENT (LOW-LEVEL)".yellow().bold());
    print_command_desc("wta worktree list", "List all git worktrees");
    print_command_desc("wta worktree add <branch>", "Create worktree without agent");
    print_command_desc("wta worktree remove <name>", "Remove a worktree");
    print_command_desc("wta switch <name>", "Switch to a worktree directory");
    println!();

    // SHELL INTEGRATION
    println!("{}", "SHELL INTEGRATION".yellow().bold());
    print_command_desc("wta init", "Print shell function for wta integration");
    println!(
        "                        Add output to {} or {} to enable",
        ".bashrc".green(),
        ".zshrc".green()
    );
    println!(
        "                        directory switching with '{}'",
        "wta switch".cyan()
    );
    println!();

    // AGENT LIFECYCLE
    println!("{}", "AGENT LIFECYCLE".yellow().bold());
    println!(
        "  {} → {} → {} → {}",
        "Launched".white(),
        "Running".bright_blue(),
        "Completed/Failed".magenta(),
        "Merged/Removed".green()
    );
    println!();
    println!(
        "  {}    Agent actively working in tmux window",
        "Running".bright_blue()
    );
    println!("  {}  Agent finished successfully", "Completed".magenta());
    println!("  {}     Agent encountered an error", "Failed".red());
    println!("  {}     Work merged back to base branch", "Merged".green());
    println!();

    // DIRECTORY STRUCTURE
    println!("{}", "DIRECTORY STRUCTURE".yellow().bold());
    println!(
        "  {}                  Git worktrees for each agent",
        ".worktrees/".green()
    );
    println!("  {}", ".worktree-agents/".green());
    println!(
        "  ├── {}               Agent registry",
        "state.json".white()
    );
    println!(
        "  ├── {}                  Completion status files",
        "status/".white()
    );
    println!(
        "  └── {}                 Task instructions",
        "prompts/".white()
    );
    println!();

    // REQUIREMENTS
    println!("{}", "REQUIREMENTS".yellow().bold());
    println!(
        "  {} {}           Worktree and branch management",
        "•".green(),
        "Git".bright_white()
    );
    println!(
        "  {} {}          Session management",
        "•".green(),
        "tmux".bright_white()
    );
    println!(
        "  {} {}        Claude Code CLI (default provider)",
        "•".green(),
        "claude".bright_white()
    );
    println!(
        "  {} {}            GitHub CLI (for '{}' command)",
        "•".green(),
        "gh".bright_white(),
        "wta pr".cyan()
    );
    println!(
        "  {} {}         Optional, for interactive diffs",
        "•".green(),
        "lumen".bright_white()
    );
    println!();

    // PARALLEL WORKFLOW EXAMPLE
    println!("{}", "PARALLEL WORKFLOW EXAMPLE".yellow().bold());
    println!("  # Launch multiple agents working in parallel");
    print_command("wta launch --task \"Build authentication module\" --branch auth");
    print_command("wta launch --task \"Create user dashboard\" --branch dashboard");
    print_command("wta launch --task \"Add dark mode toggle\" --branch dark-mode");
    println!();
    println!("  # Monitor progress");
    print_command_desc("wta list", "See all agents");
    print_command_desc("wta dashboard --open", "Or use web UI");
    println!();
    println!("  # Review and merge when done");
    print_command_desc("wta diff 1", "Review first agent's changes");
    print_command_desc("wta merge 1", "Merge to base branch");
    print_command_desc("wta pr 2", "Create PR for second agent");
    println!();
    println!("  # Cleanup");
    print_command_desc("wta prune", "Remove completed agents");

    Ok(())
}

fn print_command(cmd: &str) {
    println!("  {}", cmd.cyan());
}

fn print_command_desc(cmd: &str, desc: &str) {
    println!("  {}  {}", cmd.cyan(), desc.white());
}
