use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Agent still running: {0}")]
    AgentStillRunning(String),

    #[error("Agent already completed: {0}")]
    AgentAlreadyCompleted(String),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Tmux error: {0}")]
    Tmux(String),

    #[error("External process failed: {0}")]
    ExternalProcessFailed(String),

    #[error("Tmux session not found: {0}")]
    TmuxSessionNotFound(String),

    #[error("Tmux window not found: {0}")]
    TmuxWindowNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Merge conflict in files: {0:?}")]
    MergeConflict(Vec<PathBuf>),

    #[error("Worktree already exists: {0}")]
    WorktreeAlreadyExists(PathBuf),

    #[error("Worktree not found: {0}")]
    WorktreeNotFound(PathBuf),

    #[error("Branch already exists: {0}")]
    BranchAlreadyExists(String),

    #[error("Not a git repository")]
    NotAGitRepository,

    #[error("State file corrupted: {0}")]
    StateCorrupted(String),

    #[error("Command failed: {command}, exit code: {code:?}, stderr: {stderr}")]
    CommandFailed {
        command: String,
        code: Option<i32>,
        stderr: String,
    },

    #[error("Invalid UTF-8 path: {0}")]
    InvalidUtf8Path(PathBuf),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_agent_not_found() {
        let err = Error::AgentNotFound("42".to_string());
        assert_eq!(err.to_string(), "Agent not found: 42");
    }

    #[test]
    fn test_error_display_agent_still_running() {
        let err = Error::AgentStillRunning("1".to_string());
        assert_eq!(err.to_string(), "Agent still running: 1");
    }

    #[test]
    fn test_error_display_tmux() {
        let err = Error::Tmux("session failed".to_string());
        assert_eq!(err.to_string(), "Tmux error: session failed");
    }

    #[test]
    fn test_error_display_external_process_failed() {
        let err = Error::ExternalProcessFailed("Failed to launch VS Code: No such file".to_string());
        assert_eq!(
            err.to_string(),
            "External process failed: Failed to launch VS Code: No such file"
        );
    }

    #[test]
    fn test_error_display_merge_conflict() {
        let err = Error::MergeConflict(vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("src/lib.rs"),
        ]);
        assert!(err.to_string().contains("src/main.rs"));
        assert!(err.to_string().contains("src/lib.rs"));
    }

    #[test]
    fn test_error_display_worktree_already_exists() {
        let err = Error::WorktreeAlreadyExists(PathBuf::from(".worktrees/1"));
        assert_eq!(err.to_string(), "Worktree already exists: .worktrees/1");
    }

    #[test]
    fn test_error_display_command_failed() {
        let err = Error::CommandFailed {
            command: "git worktree add".to_string(),
            code: Some(128),
            stderr: "fatal: error".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("git worktree add"));
        assert!(msg.contains("128"));
        assert!(msg.contains("fatal: error"));
    }

    #[test]
    fn test_error_display_not_a_git_repository() {
        let err = Error::NotAGitRepository;
        assert_eq!(err.to_string(), "Not a git repository");
    }

    #[test]
    fn test_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_error_from_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err: Error = json_err.into();
        assert!(matches!(err, Error::Json(_)));
    }
}
