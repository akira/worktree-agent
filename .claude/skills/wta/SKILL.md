---
name: wta
description: Orchestrate AI agents in parallel using Worktree Agent (wta). Use when tasks can be split into independent parallel workstreams, when multiple features/fixes should be developed simultaneously, or when the user explicitly requests parallel agent execution. Claude should proactively suggest and launch wta agents for complex multi-part tasks.
allowed-tools: Bash, Read, Grep, Glob
argument-hint: [launch|list|status|diff|merge|pr|remove|prune] [args...]
disable-model-invocation: false
---

# Worktree Agent (wta) Orchestration Skill

You can use the Worktree Agent (wta) tool to launch multiple AI agents that work in parallel in isolated git worktrees. Each agent gets its own worktree, branch, and tmux session to work autonomously on a specific task.

## When to Use WTA

**PROACTIVELY use wta to launch agents when:**

1. **Multiple Independent Tasks**: The user's request involves 2+ tasks that don't depend on each other
   - Example: "Add login page, create API endpoints, and write tests" → Launch 3 agents

2. **Complex Multi-Part Features**: A large feature can be broken into parallel workstreams
   - Example: "Build authentication system" → Launch agents for: frontend forms, backend API, database schema, tests

3. **Explicit Parallel Execution**: User says "in parallel", "simultaneously", or "use wta"

4. **Time-Intensive Tasks**: Multiple slow tasks that benefit from parallelism
   - Example: "Refactor module A and module B" → Parallel refactoring

5. **Exploratory Work**: Multiple approaches or experiments to try
   - Example: "Try implementing caching with Redis and with in-memory" → Launch 2 agents

**DO NOT use wta when:**
- Single straightforward task
- Tasks have dependencies (one must finish before another starts)
- Quick changes that take less time than coordination overhead
- User wants direct interaction and iteration

## Current Repository Context

- Working directory: !`pwd`
- Current branch: !`git branch --show-current`
- Active agents: !`wta list 2>/dev/null | tail -n +2 || echo "No agents"`

## Orchestration Workflow

### Step 1: Decompose the Task

When you decide to use wta, first decompose the user's request into independent parallel tasks:

1. Identify the main components or workstreams
2. Ensure tasks are truly independent (no dependencies between them)
3. Write clear, specific task descriptions for each agent
4. Choose appropriate branch names

**Example decomposition:**
```
User: "Build a REST API for user management with authentication"

Tasks:
1. "Create user model and database schema with SQLAlchemy"
   Branch: wta/user-model

2. "Implement JWT authentication middleware and login/logout endpoints"
   Branch: wta/auth-api

3. "Build CRUD endpoints for user management (/users/*)"
   Branch: wta/user-crud

4. "Write integration tests for all API endpoints"
   Branch: wta/api-tests
```

### Step 2: Launch Agents

Launch all agents in a single batch:

```bash
# Launch multiple agents in parallel (you can run these commands sequentially)
wta launch --task "Create user model and database schema with SQLAlchemy" --branch wta/user-model
wta launch --task "Implement JWT authentication middleware and login/logout endpoints" --branch wta/auth-api
wta launch --task "Build CRUD endpoints for user management" --branch wta/user-crud
wta launch --task "Write integration tests for all API endpoints" --branch wta/api-tests
```

**Task description best practices:**
- Be specific about what needs to be done
- Include file paths or modules to work on
- Mention any constraints or requirements
- Keep descriptions concise but complete
- Don't include dependencies on other agents' work

### Step 3: Monitor Progress

After launching agents, inform the user and provide monitoring commands:

```bash
# List all agents with their status
wta list

# Get detailed status and output for a specific agent
wta status <id>
wta status <id> --lines 100    # Show more output lines
```

**Agent statuses:**
- **Running**: Agent is actively working
- **Completed**: Agent finished successfully
- **Failed**: Agent encountered an error
- **Merged**: Work merged back to base branch

**Inform the user:**
- How many agents were launched
- What each agent is working on
- How to monitor progress: `wta list`, `wta status <id>`, or `wta dashboard`
- That you'll check back on their progress

**Proactive monitoring:**
- After launching agents, wait a reasonable time (suggest user checks status)
- When user asks for updates, run `wta list` to check progress
- If agents complete, proactively offer to review and merge

### Step 4: Review and Merge Completed Work

When agents complete (or user asks to review), review their work:

```bash
# Check agent status
wta list

# View changes for completed agent
wta diff <id>
```

**Review process:**
1. Check which agents have completed
2. Review diffs for each completed agent
3. Summarize what each agent accomplished
4. Identify any issues or conflicts
5. Recommend merge strategy or fixes needed

### Step 5: Merge or Create PRs

After reviewing, merge completed work:

```bash
# Merge with default strategy
wta merge <id>

# Or create a PR instead
wta pr <id>

# Merge strategies
wta merge <id> --strategy rebase   # Rebase commits
wta merge <id> --strategy squash   # Squash into one commit
```

**Merge order:**
1. Merge agents with no conflicts first
2. If conflicts arise, resolve them or suggest the user review
3. Merge related agents in logical order
4. Consider creating PRs instead of direct merge for review

**After merging:**
- Summarize what was merged
- Run tests if appropriate
- Clean up with `wta prune`

### Create Pull Requests

```bash
# Create PR with AI-generated title and description
wta pr <id>

# With custom title (AI generates description)
wta pr <id> --title "Add user authentication"

# With custom title and body
wta pr <id> --title "Title" --body "Description"

# Force PR creation even if agent still running
wta pr <id> --force
```

**PR creation:**
- Uses `gh` CLI to create GitHub pull requests
- Automatically pushes branch to origin
- AI generates concise title and markdown description from task when not specified

### Cleanup

```bash
# Remove specific agent
wta remove <id>
wta remove <id> --force        # Force remove running agent

# Prune stale agents
wta prune                      # Remove completed/failed/merged
wta prune --all               # Remove all agents including running
wta prune --status completed  # Remove only completed agents
wta prune --status failed
wta prune --status merged
```

### Web Dashboard

```bash
# Start web dashboard
wta dashboard                  # Runs at http://localhost:3847
wta dashboard --port 8080      # Custom port
wta dashboard --open           # Auto-open browser
```

**Dashboard features:**
- Kanban board UI with Running, Completed, Failed, Merged columns
- Click tasks to view PR diff
- Merge with different strategies
- Create PRs directly from UI
- Remove agents from UI

## Example Orchestration Session

**User request:** "Build a REST API for user management with tests"

**Your response:**
```
I'll orchestrate multiple agents to work on this in parallel:

1. Database model and schema
2. Authentication endpoints
3. CRUD endpoints
4. Integration tests

Launching agents now...
```

**Launch agents:**
```bash
wta launch --task "Create SQLAlchemy user model with id, email, password_hash, created_at fields and users table migration" --branch wta/user-model
wta launch --task "Implement JWT auth with /login and /logout endpoints, authentication middleware" --branch wta/auth
wta launch --task "Create user CRUD endpoints: GET/POST /users, GET/PUT/DELETE /users/{id}" --branch wta/crud
wta launch --task "Write pytest integration tests covering all API endpoints with test database" --branch wta/tests
```

**Inform user:**
```
4 agents launched successfully:
- Agent 1: Database model (wta/user-model)
- Agent 2: Authentication (wta/auth)
- Agent 3: CRUD endpoints (wta/crud)
- Agent 4: Integration tests (wta/tests)

Monitor progress:
  wta list              # See all agent statuses
  wta status <id>       # Check specific agent
  wta dashboard --open  # Visual dashboard

I'll check back on their progress. Let me know when you'd like me to review and merge their work!
```

**Later - review and merge:**
```bash
# Check status
wta list

# Review completed agents
wta diff 1
wta diff 2
# ... review each

# Merge in logical order
wta merge 1  # Model first
wta merge 2  # Auth second
wta merge 3  # CRUD third
wta merge 4  # Tests last

# Cleanup
wta prune
```

## Agent Lifecycle

```
Launched → Running → Completed/Failed → Merged/Removed
```

## Directory Structure

wta creates these directories in the repository:

```
.worktrees/              # Git worktrees for each agent
.worktree-agents/
├── state.json          # Agent registry
├── status/             # Agent completion status files
└── prompts/            # Task instructions for agents
```

## Requirements

- **Git**: Worktree and branch management
- **tmux**: Session management
- **claude** (or other AI provider CLI): For running agents
- **gh**: GitHub CLI (for `wta pr` command)
- **lumen** (optional): For interactive diffs

## Tips and Best Practices

1. **Task Descriptions**: Write clear, specific task descriptions. Include:
   - What needs to be done
   - Any specific requirements or constraints
   - Files or areas of the codebase to focus on

2. **Branch Naming**: Use descriptive branch names that indicate the feature or fix

3. **Monitoring**: Use `wta attach <id>` to watch agents work in real-time, or `wta dashboard` for a visual overview

4. **Review Before Merge**: Always use `wta diff <id>` to review changes before merging

5. **Parallel Work**: Launch multiple agents for independent tasks to maximize parallelism

6. **Cleanup**: Regularly run `wta prune` to clean up completed agents and keep the workspace tidy

## Troubleshooting

If an agent fails:
1. Check status: `wta status <id> --lines 100`
2. Attach to see what happened: `wta attach <id>`
3. Review the prompt file: Check `.worktree-agents/prompts/<id>.md`
4. Try relaunching with a clearer task description

## Advanced Techniques

### Handling Dependencies

If tasks have dependencies, launch them in stages:

```bash
# Stage 1: Foundation
wta launch --task "Set up database schema" --branch wta/schema

# After stage 1 completes, launch stage 2
wta launch --task "Build API endpoints using the schema" --branch wta/api --base wta/schema
```

### Speculative Exploration

Launch agents to explore multiple approaches:

```bash
wta launch --task "Implement caching with Redis" --branch wta/redis-cache
wta launch --task "Implement caching with in-memory LRU" --branch wta/lru-cache

# Review both, pick the best approach
wta diff 1
wta diff 2
# Merge winner, remove loser
wta merge 1
wta remove 2
```

### Large Features

For very large features, use wta to parallelize the initial implementation:

```bash
# Frontend and backend in parallel
wta launch --task "Build React components for user dashboard" --branch wta/dashboard-ui
wta launch --task "Create GraphQL API for dashboard data" --branch wta/dashboard-api

# Review, test integration, then merge
```

## Orchestration Best Practices

1. **Clear Task Boundaries**: Ensure each agent's task is self-contained
2. **Descriptive Branch Names**: Use `wta/<feature>` or descriptive names
3. **Inform the User**: Always explain what you're doing and why
4. **Monitor Proactively**: Check agent progress and report updates
5. **Review Before Merge**: Always review diffs before merging
6. **Handle Conflicts**: Help resolve merge conflicts when they arise
7. **Clean Up**: Run `wta prune` after merging to clean up worktrees

## Command Reference

| Command | Purpose |
|---------|---------|
| `wta launch --task "..." --branch <name>` | Launch new agent |
| `wta list` | List all agents and their status |
| `wta status <id>` | Get detailed agent status and output |
| `wta diff <id>` | View agent's changes |
| `wta merge <id>` | Merge agent's work |
| `wta pr <id>` | Create PR for agent's work |
| `wta remove <id>` | Remove agent and worktree |
| `wta prune` | Clean up completed/failed/merged agents |
| `wta dashboard` | Start web dashboard |

## Troubleshooting

If an agent fails:
1. Check status: `wta status <id> --lines 100`
2. Review the task: Was it clear and self-contained?
3. Check for dependency issues: Did it need another agent's work first?
4. Suggest relaunching with clearer instructions or different approach

## Remember

- **Be proactive**: Suggest wta when you see opportunities for parallelism
- **Decompose thoughtfully**: Break tasks into truly independent units
- **Communicate clearly**: Tell the user what you're doing and why
- **Monitor and follow up**: Check on agents and help merge their work
- **Handle failures gracefully**: Help troubleshoot and relaunch if needed
