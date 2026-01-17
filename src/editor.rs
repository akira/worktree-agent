use std::env;
use std::fs;
use std::io::Write;
use std::process::Command;

use crate::error::{Error, Result};

const DEFAULT_EDITOR: &str = "vi";
const TASK_TEMPLATE: &str = "\
# Enter your task description below.
# Lines starting with '#' will be ignored.
# An empty message aborts the launch.
";

/// Opens the user's preferred editor to compose a task description.
/// If `custom_editor` is provided, it overrides all other editor settings.
/// Returns the task string or an error if aborted.
pub fn open_editor_for_task(custom_editor: Option<String>) -> Result<String> {
    let editor = normalize_editor(custom_editor.unwrap_or_else(get_editor));
    let temp_path = create_temp_file()?;

    // Open editor and wait for it to close
    // Use shell to handle editors with arguments (e.g., "code --wait")
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &format!("{editor} \"{temp_path}\"")])
            .status()
    } else {
        Command::new("sh")
            .args(["-c", &format!("{editor} \"{temp_path}\"")])
            .status()
    }
    .map_err(|e| Error::EditorAborted(format!("Failed to launch editor '{editor}': {e}")))?;

    if !status.success() {
        // Clean up temp file on error
        let _ = fs::remove_file(&temp_path);
        return Err(Error::EditorAborted(format!(
            "Editor exited with status: {status}"
        )));
    }

    // Read the contents
    let contents = fs::read_to_string(&temp_path)?;

    // Clean up temp file
    let _ = fs::remove_file(&temp_path);

    // Process the contents: remove comment lines and trim
    let task = process_editor_content(&contents);

    if task.is_empty() {
        return Err(Error::EditorAborted("Empty task description".to_string()));
    }

    Ok(task)
}

/// Get the editor following git's precedence:
/// 1. VISUAL environment variable
/// 2. EDITOR environment variable
/// 3. git config core.editor
/// 4. Default to vi
fn get_editor() -> String {
    // Check VISUAL first (typically for graphical editors)
    if let Ok(editor) = env::var("VISUAL") {
        if !editor.is_empty() {
            return editor;
        }
    }

    // Check EDITOR
    if let Ok(editor) = env::var("EDITOR") {
        if !editor.is_empty() {
            return editor;
        }
    }

    // Try git config core.editor
    if let Some(editor) = get_git_editor() {
        return editor;
    }

    DEFAULT_EDITOR.to_string()
}

fn get_git_editor() -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--get", "core.editor"])
        .output()
        .ok()?;

    if output.status.success() {
        let editor = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !editor.is_empty() {
            return Some(editor);
        }
    }

    None
}

/// Normalizes editor commands, adding required flags for known editors.
/// For example, "code" becomes "code --wait" since VS Code needs --wait to block.
fn normalize_editor(editor: String) -> String {
    match editor.as_str() {
        "code" => "code --wait".to_string(),
        "code-insiders" => "code-insiders --wait".to_string(),
        "subl" | "sublime" => "subl --wait".to_string(),
        "atom" => "atom --wait".to_string(),
        _ => editor,
    }
}

fn create_temp_file() -> Result<String> {
    let temp_dir = env::temp_dir();
    let temp_path = temp_dir.join(format!("wta_task_{}.txt", std::process::id()));
    let temp_path_str = temp_path
        .to_str()
        .ok_or_else(|| Error::InvalidUtf8Path(temp_path.clone()))?
        .to_string();

    let mut file = fs::File::create(&temp_path)?;
    file.write_all(TASK_TEMPLATE.as_bytes())?;

    Ok(temp_path_str)
}

fn process_editor_content(contents: &str) -> String {
    contents
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_editor_content_removes_comments() {
        let content = "# This is a comment\nActual task\n# Another comment\nMore task";
        let result = process_editor_content(content);
        assert_eq!(result, "Actual task\n\nMore task");
    }

    #[test]
    fn test_process_editor_content_trims_whitespace() {
        let content = "\n\n  Task description  \n\n";
        let result = process_editor_content(content);
        assert_eq!(result, "Task description");
    }

    #[test]
    fn test_process_editor_content_empty_after_comments() {
        let content = "# Only comments\n# Nothing else";
        let result = process_editor_content(content);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_editor_default() {
        // Remove both env vars to test default
        env::remove_var("VISUAL");
        env::remove_var("EDITOR");
        let editor = get_editor();
        assert_eq!(editor, DEFAULT_EDITOR);
    }
}
