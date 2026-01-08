pub mod attach;
pub mod list;
pub mod merge;
pub mod prune;
pub mod remove;
pub mod spawn;
pub mod status;
pub mod worktree;

/// Truncates a task string to `max_len` characters, adding "..." suffix when truncated.
pub fn truncate_task(task: &str, max_len: usize) -> String {
    if task.len() > max_len {
        format!("{}...", &task[..max_len.saturating_sub(3)])
    } else {
        task.to_string()
    }
}
