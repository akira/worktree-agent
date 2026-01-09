use crate::git::WorktreeManager;
use crate::Result;
use clap::Subcommand;
use std::io::IsTerminal;
use std::path::PathBuf;
use tabled::{Table, Tabled};

const WTA_DIRECTIVE_FILE_ENV: &str = "WTA_DIRECTIVE_FILE";

const WORKTREES_DIR: &str = ".worktrees";

#[derive(Subcommand)]
pub enum WorktreeCommands {
    /// List all git worktrees in this repository
    List,

    /// Add a new worktree with a branch
    Add {
        /// Branch name for the new worktree
        branch: String,

        /// Base branch to create from (default: current branch)
        #[arg(long)]
        base: Option<String>,

        /// Custom path for the worktree (default: .worktrees/<branch>)
        #[arg(long)]
        path: Option<String>,
    },

    /// Remove a worktree
    Remove {
        /// Branch name or worktree path to remove
        name: String,
    },

    /// Prune stale worktree references
    Prune,

    /// Switch to a worktree directory (prints cd command to eval)
    Switch {
        /// Worktree ID, branch name, or agent ID
        name: String,
    },
}

#[derive(Tabled)]
struct WorktreeRow {
    #[tabled(rename = "Path")]
    path: String,
    #[tabled(rename = "Branch")]
    branch: String,
}

fn get_repo_root() -> Result<PathBuf> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;

    if !output.status.success() {
        return Err(crate::Error::NotAGitRepository);
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(path))
}

fn get_current_branch() -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;

    if !output.status.success() {
        return Err(crate::Error::NotAGitRepository);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub async fn run(command: WorktreeCommands) -> Result<()> {
    match command {
        WorktreeCommands::List => run_list().await,
        WorktreeCommands::Add { branch, base, path } => run_add(branch, base, path).await,
        WorktreeCommands::Remove { name } => run_remove(name).await,
        WorktreeCommands::Prune => run_prune().await,
        WorktreeCommands::Switch { name } => run_switch(name).await,
    }
}

async fn run_list() -> Result<()> {
    let repo_root = get_repo_root()?;
    let worktrees_dir = repo_root.join(WORKTREES_DIR);
    let manager = WorktreeManager::new(&repo_root, &worktrees_dir);

    let worktrees = manager.list()?;

    if worktrees.is_empty() {
        println!("No worktrees found.");
        return Ok(());
    }

    let rows: Vec<WorktreeRow> = worktrees
        .into_iter()
        .map(|wt| WorktreeRow {
            path: wt.path.display().to_string(),
            branch: wt.branch,
        })
        .collect();

    let table = Table::new(rows).to_string();
    println!("{table}");

    Ok(())
}

async fn run_add(branch: String, base: Option<String>, path: Option<String>) -> Result<()> {
    let repo_root = get_repo_root()?;
    let worktrees_dir = repo_root.join(WORKTREES_DIR);

    // Create worktrees directory if it doesn't exist
    if !worktrees_dir.exists() {
        std::fs::create_dir_all(&worktrees_dir)?;
    }

    let manager = WorktreeManager::new(&repo_root, &worktrees_dir);

    // Determine base branch
    let base_branch = match base {
        Some(b) => b,
        None => get_current_branch()?,
    };

    // Determine worktree id/path
    let worktree_id = path.unwrap_or_else(|| branch.replace('/', "-"));

    let worktree_path = manager.create(&worktree_id, &branch, &base_branch)?;

    println!(
        "Created worktree at {} on branch {}",
        worktree_path.display(),
        branch
    );

    Ok(())
}

async fn run_remove(name: String) -> Result<()> {
    let repo_root = get_repo_root()?;
    let worktrees_dir = repo_root.join(WORKTREES_DIR);
    let manager = WorktreeManager::new(&repo_root, &worktrees_dir);

    // Check if name is a path or a worktree id
    let worktree_path = if name.contains('/') || name.contains('\\') {
        PathBuf::from(&name)
    } else {
        worktrees_dir.join(&name)
    };

    // Get the worktree id from the path
    let worktree_id = worktree_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&name);

    manager.remove(worktree_id)?;

    println!("Removed worktree: {}", worktree_path.display());

    Ok(())
}

async fn run_prune() -> Result<()> {
    let repo_root = get_repo_root()?;

    let output = std::process::Command::new("git")
        .current_dir(&repo_root)
        .args(["worktree", "prune"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crate::Error::CommandFailed {
            command: "git worktree prune".to_string(),
            code: output.status.code(),
            stderr: stderr.to_string(),
        });
    }

    println!("Pruned stale worktree references.");

    Ok(())
}

async fn run_switch(name: String) -> Result<()> {
    let repo_root = get_repo_root()?;
    let worktrees_dir = repo_root.join(WORKTREES_DIR);
    let manager = WorktreeManager::new(&repo_root, &worktrees_dir);

    let worktrees = manager.list()?;

    // Helper to output the cd command via directive file or instructions
    let output_switch = |path: &std::path::Path| -> Result<()> {
        let cd_cmd = format!("cd '{}'", path.display());

        // Check if we have a directive file to write to (shell function integration)
        if let Ok(directive_file) = std::env::var(WTA_DIRECTIVE_FILE_ENV) {
            std::fs::write(&directive_file, format!("{cd_cmd}\n"))?;
            println!("Switched to worktree: {}", path.display());
        } else if std::io::stdout().is_terminal() {
            // Running interactively without shell function - show instructions
            eprintln!("To switch to this worktree, run:");
            eprintln!();
            eprintln!("  {cd_cmd}");
            eprintln!();
            eprintln!("Tip: Add the wta shell function to enable direct switching.");
            eprintln!("Run: wta init --help");
        } else {
            // Output is being captured (e.g., via eval) - just print cd command
            println!("{cd_cmd}");
        }
        Ok(())
    };

    // Try to find worktree by exact ID match first
    let direct_path = worktrees_dir.join(&name);
    if direct_path.exists() {
        return output_switch(&direct_path);
    }

    // Try to find by branch name
    for wt in &worktrees {
        if wt.branch == name || wt.branch.ends_with(&format!("/{name}")) {
            return output_switch(&wt.path);
        }
    }

    // Try partial match on path
    for wt in &worktrees {
        if let Some(file_name) = wt.path.file_name() {
            if file_name.to_string_lossy() == name {
                return output_switch(&wt.path);
            }
        }
    }

    Err(crate::Error::WorktreeNotFound(PathBuf::from(name)))
}
