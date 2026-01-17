use crate::orchestrator::{Agent, MergeStrategy, Orchestrator};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type AppState = Arc<Mutex<Orchestrator>>;

#[derive(Serialize)]
pub struct ApiError {
    pub error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}

fn map_err(e: impl std::fmt::Display) -> (StatusCode, Json<ApiError>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            error: e.to_string(),
        }),
    )
}

#[derive(Serialize)]
pub struct AgentResponse {
    pub id: String,
    pub task: String,
    pub branch: String,
    pub base_branch: String,
    pub status: String,
    pub provider: String,
    pub launched_at: String,
    pub completed_at: Option<String>,
}

impl From<&Agent> for AgentResponse {
    fn from(agent: &Agent) -> Self {
        Self {
            id: agent.id.0.clone(),
            task: agent.task.clone(),
            branch: agent.branch.clone(),
            base_branch: agent.base_branch.clone(),
            status: agent.status.to_string(),
            provider: agent.provider.to_string(),
            launched_at: agent.launched_at.to_rfc3339(),
            completed_at: agent.completed_at.map(|t| t.to_rfc3339()),
        }
    }
}

pub async fn list_agents(
    State(state): State<AppState>,
) -> std::result::Result<Json<Vec<AgentResponse>>, (StatusCode, Json<ApiError>)> {
    let mut orchestrator = state.lock().await;

    // Update status for all agents
    let ids: Vec<String> = orchestrator.list().iter().map(|a| a.id.0.clone()).collect();
    for id in ids {
        let _ = orchestrator.check_status(&id);
    }

    let agents: Vec<AgentResponse> = orchestrator
        .list()
        .iter()
        .map(|a| AgentResponse::from(*a))
        .collect();
    Ok(Json(agents))
}

pub async fn get_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> std::result::Result<Json<AgentResponse>, (StatusCode, Json<ApiError>)> {
    let mut orchestrator = state.lock().await;

    // Update status
    let _ = orchestrator.check_status(&id);

    let agent = orchestrator.get_agent(&id).map_err(map_err)?;
    Ok(Json(AgentResponse::from(agent)))
}

#[derive(Serialize)]
pub struct DiffResponse {
    pub diff: String,
    pub files_changed: Vec<String>,
    pub stats: DiffStats,
}

#[derive(Serialize)]
pub struct DiffStats {
    pub additions: usize,
    pub deletions: usize,
    pub files_changed: usize,
}

pub async fn get_diff(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> std::result::Result<Json<DiffResponse>, (StatusCode, Json<ApiError>)> {
    let orchestrator = state.lock().await;
    let agent = orchestrator.get_agent(&id).map_err(map_err)?;

    // Check if worktree still exists (it's removed after merge)
    if !agent.worktree_path.exists() {
        return Ok(Json(DiffResponse {
            diff: String::new(),
            files_changed: Vec::new(),
            stats: DiffStats {
                additions: 0,
                deletions: 0,
                files_changed: 0,
            },
        }));
    }

    let diff_range = format!("{}...HEAD", agent.base_branch);

    // Get the diff content
    let diff_output = Command::new("git")
        .args(["diff", &diff_range])
        .current_dir(&agent.worktree_path)
        .output()
        .map_err(map_err)?;

    let diff = String::from_utf8_lossy(&diff_output.stdout).to_string();

    // Get list of changed files
    let files_output = Command::new("git")
        .args(["diff", "--name-only", &diff_range])
        .current_dir(&agent.worktree_path)
        .output()
        .map_err(map_err)?;

    let files_changed: Vec<String> = String::from_utf8_lossy(&files_output.stdout)
        .lines()
        .map(String::from)
        .collect();

    // Get diff stats
    let stat_output = Command::new("git")
        .args(["diff", "--shortstat", &diff_range])
        .current_dir(&agent.worktree_path)
        .output()
        .map_err(map_err)?;

    let stat_str = String::from_utf8_lossy(&stat_output.stdout);
    let stats = parse_diff_stats(&stat_str);

    Ok(Json(DiffResponse {
        diff,
        files_changed,
        stats,
    }))
}

fn parse_diff_stats(stat_str: &str) -> DiffStats {
    let mut stats = DiffStats {
        additions: 0,
        deletions: 0,
        files_changed: 0,
    };

    // Parse "3 files changed, 10 insertions(+), 5 deletions(-)"
    for part in stat_str.split(',') {
        let part = part.trim();
        if part.contains("file") {
            if let Some(num) = part.split_whitespace().next() {
                stats.files_changed = num.parse().unwrap_or(0);
            }
        } else if part.contains("insertion") {
            if let Some(num) = part.split_whitespace().next() {
                stats.additions = num.parse().unwrap_or(0);
            }
        } else if part.contains("deletion") {
            if let Some(num) = part.split_whitespace().next() {
                stats.deletions = num.parse().unwrap_or(0);
            }
        }
    }

    stats
}

#[derive(Deserialize)]
pub struct MergeRequest {
    pub strategy: Option<String>,
    pub force: Option<bool>,
}

#[derive(Serialize)]
pub struct MergeResponse {
    pub success: bool,
    pub message: String,
    pub conflicts: Vec<String>,
}

pub async fn merge_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<MergeRequest>,
) -> std::result::Result<Json<MergeResponse>, (StatusCode, Json<ApiError>)> {
    let mut orchestrator = state.lock().await;

    let strategy = match req.strategy.as_deref() {
        Some("rebase") => MergeStrategy::Rebase,
        Some("squash") => MergeStrategy::Squash,
        _ => MergeStrategy::Merge,
    };

    let force = req.force.unwrap_or(false);

    let result = orchestrator
        .merge(&id, strategy, force)
        .await
        .map_err(map_err)?;

    Ok(Json(MergeResponse {
        success: result.success,
        message: result.message,
        conflicts: result
            .conflicts
            .iter()
            .map(|p| p.display().to_string())
            .collect(),
    }))
}

#[derive(Deserialize)]
pub struct PrRequest {
    pub title: Option<String>,
    pub body: Option<String>,
    pub force: Option<bool>,
}

#[derive(Serialize)]
pub struct PrResponse {
    pub url: String,
}

pub async fn create_pr(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<PrRequest>,
) -> std::result::Result<Json<PrResponse>, (StatusCode, Json<ApiError>)> {
    let mut orchestrator = state.lock().await;

    let force = req.force.unwrap_or(false);

    let result = orchestrator
        .create_pr(&id, req.title, req.body, force)
        .await
        .map_err(map_err)?;

    Ok(Json(PrResponse { url: result.url }))
}

#[derive(Deserialize)]
pub struct RemoveRequest {
    pub force: Option<bool>,
}

pub async fn remove_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(req): Query<RemoveRequest>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let mut orchestrator = state.lock().await;
    let force = req.force.unwrap_or(false);

    orchestrator.remove(&id, force).await.map_err(map_err)?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct OutputResponse {
    pub output: String,
}

#[derive(Deserialize)]
pub struct OutputQuery {
    pub lines: Option<usize>,
}

pub async fn get_output(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<OutputQuery>,
) -> std::result::Result<Json<OutputResponse>, (StatusCode, Json<ApiError>)> {
    let orchestrator = state.lock().await;
    let lines = query.lines.unwrap_or(100);

    // Try to get output, but return empty string if tmux window is gone
    let output = orchestrator
        .get_output(&id, lines)
        .unwrap_or_else(|_| "Output not available (tmux window closed)".to_string());

    Ok(Json(OutputResponse { output }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_diff_stats_full() {
        let stat_str = " 3 files changed, 10 insertions(+), 5 deletions(-)";
        let stats = parse_diff_stats(stat_str);
        assert_eq!(stats.files_changed, 3);
        assert_eq!(stats.additions, 10);
        assert_eq!(stats.deletions, 5);
    }

    #[test]
    fn test_parse_diff_stats_no_deletions() {
        let stat_str = " 1 file changed, 5 insertions(+)";
        let stats = parse_diff_stats(stat_str);
        assert_eq!(stats.files_changed, 1);
        assert_eq!(stats.additions, 5);
        assert_eq!(stats.deletions, 0);
    }

    #[test]
    fn test_parse_diff_stats_no_insertions() {
        let stat_str = " 2 files changed, 3 deletions(-)";
        let stats = parse_diff_stats(stat_str);
        assert_eq!(stats.files_changed, 2);
        assert_eq!(stats.additions, 0);
        assert_eq!(stats.deletions, 3);
    }

    #[test]
    fn test_parse_diff_stats_empty() {
        let stat_str = "";
        let stats = parse_diff_stats(stat_str);
        assert_eq!(stats.files_changed, 0);
        assert_eq!(stats.additions, 0);
        assert_eq!(stats.deletions, 0);
    }
}
