pub mod attach;
pub mod init;
pub mod list;
pub mod merge;
pub mod pr;
pub mod prune;
pub mod remove;
pub mod launch;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_task_shorter_than_max() {
        let task = "Short task";
        let result = truncate_task(task, 50);
        assert_eq!(result, "Short task");
    }

    #[test]
    fn test_truncate_task_exact_length() {
        let task = "Exact";
        let result = truncate_task(task, 5);
        assert_eq!(result, "Exact");
    }

    #[test]
    fn test_truncate_task_longer_than_max() {
        let task = "This is a very long task description that needs truncation";
        let result = truncate_task(task, 20);
        assert_eq!(result.len(), 20);
        assert!(result.ends_with("..."));
        assert_eq!(result, "This is a very lo...");
    }

    #[test]
    fn test_truncate_task_with_small_max_len() {
        let task = "Hello World";
        let result = truncate_task(task, 6);
        assert_eq!(result, "Hel...");
    }

    #[test]
    fn test_truncate_task_empty_string() {
        let task = "";
        let result = truncate_task(task, 50);
        assert_eq!(result, "");
    }

    #[test]
    fn test_truncate_task_max_len_zero() {
        let task = "Some task";
        let result = truncate_task(task, 0);
        // saturating_sub(3) returns 0, so we get "..."
        assert_eq!(result, "...");
    }

    #[test]
    fn test_truncate_task_max_len_one() {
        let task = "Some task";
        let result = truncate_task(task, 1);
        // saturating_sub(3) returns 0, so we get "..."
        assert_eq!(result, "...");
    }

    #[test]
    fn test_truncate_task_max_len_two() {
        let task = "Some task";
        let result = truncate_task(task, 2);
        // saturating_sub(3) returns 0, so we get "..."
        assert_eq!(result, "...");
    }

    #[test]
    fn test_truncate_task_max_len_three() {
        let task = "Some task";
        let result = truncate_task(task, 3);
        // saturating_sub(3) returns 0, so we get "..."
        assert_eq!(result, "...");
    }

    #[test]
    fn test_truncate_task_max_len_four() {
        let task = "Some task";
        let result = truncate_task(task, 4);
        assert_eq!(result, "S...");
    }

    #[test]
    fn test_truncate_task_preserves_original_when_not_truncated() {
        let task = "Fix the authentication bug";
        let result = truncate_task(task, 100);
        assert_eq!(result, task);
    }

    #[test]
    fn test_truncate_task_one_char_over() {
        let task = "12345678901"; // 11 chars
        let result = truncate_task(task, 10);
        assert_eq!(result, "1234567...");
    }
}
