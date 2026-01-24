use crate::Result;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

const SKILL_CONTENT: &str = include_str!("../../.claude/skills/wta/SKILL.md");

pub async fn run() -> Result<()> {
    let home = std::env::var("HOME").map_err(|_| {
        crate::error::Error::ExternalProcessFailed("HOME environment variable not set".to_string())
    })?;
    let skill_dir = PathBuf::from(home).join(".claude/skills/wta");
    let skill_file = skill_dir.join("SKILL.md");

    // Check if skill already exists
    if skill_file.exists() {
        println!("{}", "Claude wta skill is already installed!".green().bold());
        println!("\nLocation: {}", skill_file.display().to_string().cyan());
        println!("\nTo reinstall, first remove the existing skill:");
        println!("  {}", format!("rm -rf {}", skill_dir.display()).yellow());
        return Ok(());
    }

    // Create directory
    fs::create_dir_all(&skill_dir)?;

    // Write skill file
    fs::write(&skill_file, SKILL_CONTENT)?;

    println!("{}", "✓ Claude wta skill installed successfully!".green().bold());
    println!("\nInstalled to: {}", skill_file.display().to_string().cyan());
    println!("\n{}", "Next steps:".yellow().bold());
    println!("  1. Restart your Claude Code session");
    println!("  2. Use {} to access the skill", "/wta".cyan());
    println!("  3. Claude will automatically use wta for parallel tasks\n");

    println!("{}", "What the skill does:".yellow().bold());
    println!("  • Enables Claude to orchestrate multiple AI agents in parallel");
    println!("  • Automatically decomposes complex tasks into parallel workstreams");
    println!("  • Launches agents in isolated git worktrees");
    println!("  • Monitors progress and merges completed work\n");

    Ok(())
}
