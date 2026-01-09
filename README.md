# Worktree Agent (WTA)

A Rust CLI tool for launching Claude Code AI agents in isolated Git worktrees, enabling parallel autonomous task execution with clean branch management.

## Overview

WTA orchestrates Claude Code agents by:
- Creating isolated Git worktrees for each agent
- Managing agents within tmux sessions
- Tracking agent status and task completion
- Providing flexible merge strategies for completed work

## Installation

```bash
cargo install --path .
```

## Usage

### Launch an Agent

```bash
# Basic launch with auto-generated branch name
wta launch --task "Implement user authentication"

# With custom branch name
wta launch --task "Add dark mode" --branch feature/dark-mode

# From a specific base branch
wta launch --task "Fix login bug" --base develop
```

### Monitor Agents

```bash
# List all agents
wta list

# Get status and output for a specific agent
wta status <id>

# Get more lines of output
wta status <id> --lines 100

# Attach to an agent's tmux window
wta attach <id>
```

### Merge Completed Work

```bash
# Merge with default strategy
wta merge <id>

# Rebase strategy
wta merge <id> --strategy rebase

# Squash merge
wta merge <id> --strategy squash

# Force merge even if agent is still running
wta merge <id> --force
```

### Remove Agents

```bash
# Remove an agent, kill window, and cleanup worktree
wta remove <id>

# Force remove even if agent is still running
wta remove <id> --force
```

### Prune Stale Agents

```bash
# Prune stale agents (completed, failed, or merged)
wta prune

# Prune all agents including running ones
wta prune --all

# Prune only agents with a specific status
wta prune --status completed
wta prune --status failed
wta prune --status merged
```

## Agent Lifecycle

```
Launched → Running → Completed/Failed → Merged/Removed
```

1. **Running**: Agent is actively working in its tmux window
2. **Completed**: Agent finished successfully (wrote status file)
3. **Failed**: Agent encountered an error
4. **Merged**: Work merged back to base branch
5. **Removed**: Agent's worktree and branch removed

## Directory Structure

WTA creates the following directories in your repository:

```
.worktrees/           # Git worktrees for each agent
.worktree-agents/
├── state.json        # Agent registry
├── status/           # Agent completion status files
└── prompts/          # Task instructions for agents
```

## Requirements

- Git
- tmux
- Claude Code CLI (`claude`)

## License

MIT
