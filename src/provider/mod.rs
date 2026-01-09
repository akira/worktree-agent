use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Available AI provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    /// Claude Code CLI (default)
    #[default]
    Claude,
    /// OpenAI Codex CLI
    Codex,
    /// Google Gemini CLI
    Gemini,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::Claude => write!(f, "claude"),
            Provider::Codex => write!(f, "codex"),
            Provider::Gemini => write!(f, "gemini"),
        }
    }
}

impl Provider {
    /// Get the binary name for this provider
    pub fn binary_name(&self) -> &'static str {
        match self {
            Provider::Claude => "claude",
            Provider::Codex => "codex",
            Provider::Gemini => "gemini",
        }
    }

    /// Build the command string to run the AI agent
    ///
    /// # Arguments
    /// * `worktree_path` - Path to the worktree directory
    /// * `prompt_file` - Path to the prompt file
    /// * `status_file` - Path to the status file
    /// * `extra_args` - Extra arguments to pass to the provider
    pub fn build_command(
        &self,
        worktree_path: &Path,
        prompt_file: &Path,
        status_file: &Path,
        extra_args: &[String],
    ) -> String {
        match self {
            Provider::Claude => {
                self.build_claude_command(worktree_path, prompt_file, status_file, extra_args)
            }
            Provider::Codex => self.build_codex_command(worktree_path, prompt_file, extra_args),
            Provider::Gemini => self.build_gemini_command(worktree_path, prompt_file, extra_args),
        }
    }

    fn build_claude_command(
        &self,
        worktree_path: &Path,
        prompt_file: &Path,
        status_file: &Path,
        extra_args: &[String],
    ) -> String {
        let extra_args_str = if extra_args.is_empty() {
            String::new()
        } else {
            format!(" {}", extra_args.join(" "))
        };

        // Default allowed tools for safe operations
        let default_allowed_tools = [
            "Bash(cargo check:*)",
            "Bash(cargo build:*)",
            "Bash(cargo test:*)",
            "Bash(cargo fmt:*)",
            "Bash(cargo clippy:*)",
            "Bash(git diff:*)",
            "Bash(git status:*)",
            "Bash(git log:*)",
            "Bash(git branch:*)",
            "Bash(git add:*)",
            "Bash(git commit:*)",
            "Bash(ls:*)",
            "Bash(pwd)",
        ];

        // Allow writing to the status directory so agent can report completion
        // Use directory wildcard pattern to ensure permission is granted
        let status_dir = status_file
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let status_file_pattern = format!("Write({status_dir}/*)");
        let allowed_tools_arg = format!(
            "--allowedTools '{},{}'",
            default_allowed_tools.join(","),
            status_file_pattern
        );

        format!(
            "cd {} && cat {} | claude --permission-mode acceptEdits {allowed_tools_arg}{extra_args_str}",
            worktree_path.display(),
            prompt_file.display()
        )
    }

    fn build_codex_command(
        &self,
        worktree_path: &Path,
        prompt_file: &Path,
        extra_args: &[String],
    ) -> String {
        let extra_args_str = if extra_args.is_empty() {
            String::new()
        } else {
            format!(" {}", extra_args.join(" "))
        };

        // Codex CLI uses --full-auto for autonomous sandboxed execution
        // and accepts prompt via stdin similar to Claude
        format!(
            "cd {} && cat {} | codex --full-auto{extra_args_str}",
            worktree_path.display(),
            prompt_file.display()
        )
    }

    fn build_gemini_command(
        &self,
        worktree_path: &Path,
        prompt_file: &Path,
        extra_args: &[String],
    ) -> String {
        let extra_args_str = if extra_args.is_empty() {
            String::new()
        } else {
            format!(" {}", extra_args.join(" "))
        };

        // Gemini CLI uses -y for auto-accept mode
        // and accepts prompt via stdin
        format!(
            "cd {} && cat {} | gemini -y{extra_args_str}",
            worktree_path.display(),
            prompt_file.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_provider_display() {
        assert_eq!(Provider::Claude.to_string(), "claude");
        assert_eq!(Provider::Codex.to_string(), "codex");
        assert_eq!(Provider::Gemini.to_string(), "gemini");
    }

    #[test]
    fn test_provider_binary_name() {
        assert_eq!(Provider::Claude.binary_name(), "claude");
        assert_eq!(Provider::Codex.binary_name(), "codex");
        assert_eq!(Provider::Gemini.binary_name(), "gemini");
    }

    #[test]
    fn test_provider_default_is_claude() {
        assert_eq!(Provider::default(), Provider::Claude);
    }

    #[test]
    fn test_provider_serialization() {
        let json = serde_json::to_string(&Provider::Claude).unwrap();
        assert_eq!(json, "\"claude\"");

        let json = serde_json::to_string(&Provider::Codex).unwrap();
        assert_eq!(json, "\"codex\"");

        let json = serde_json::to_string(&Provider::Gemini).unwrap();
        assert_eq!(json, "\"gemini\"");
    }

    #[test]
    fn test_provider_deserialization() {
        let provider: Provider = serde_json::from_str("\"claude\"").unwrap();
        assert_eq!(provider, Provider::Claude);

        let provider: Provider = serde_json::from_str("\"codex\"").unwrap();
        assert_eq!(provider, Provider::Codex);

        let provider: Provider = serde_json::from_str("\"gemini\"").unwrap();
        assert_eq!(provider, Provider::Gemini);
    }

    #[test]
    fn test_build_claude_command() {
        let worktree = PathBuf::from("/tmp/worktree");
        let prompt = PathBuf::from("/tmp/prompt.txt");
        let status = PathBuf::from("/tmp/status/agent.json");

        let cmd = Provider::Claude.build_command(&worktree, &prompt, &status, &[]);

        assert!(cmd.contains("cd /tmp/worktree"));
        assert!(cmd.contains("cat /tmp/prompt.txt"));
        assert!(cmd.contains("claude --permission-mode acceptEdits"));
        assert!(cmd.contains("--allowedTools"));
        assert!(cmd.contains("Write(/tmp/status/*)"));
    }

    #[test]
    fn test_build_claude_command_with_extra_args() {
        let worktree = PathBuf::from("/tmp/worktree");
        let prompt = PathBuf::from("/tmp/prompt.txt");
        let status = PathBuf::from("/tmp/status/agent.json");
        let extra_args = vec![
            "--verbose".to_string(),
            "--model".to_string(),
            "opus".to_string(),
        ];

        let cmd = Provider::Claude.build_command(&worktree, &prompt, &status, &extra_args);

        assert!(cmd.contains("--verbose --model opus"));
    }

    #[test]
    fn test_build_codex_command() {
        let worktree = PathBuf::from("/tmp/worktree");
        let prompt = PathBuf::from("/tmp/prompt.txt");
        let status = PathBuf::from("/tmp/status.json");

        let cmd = Provider::Codex.build_command(&worktree, &prompt, &status, &[]);

        assert!(cmd.contains("cd /tmp/worktree"));
        assert!(cmd.contains("cat /tmp/prompt.txt"));
        assert!(cmd.contains("codex --full-auto"));
    }

    #[test]
    fn test_build_codex_command_with_extra_args() {
        let worktree = PathBuf::from("/tmp/worktree");
        let prompt = PathBuf::from("/tmp/prompt.txt");
        let status = PathBuf::from("/tmp/status.json");
        let extra_args = vec!["--model".to_string(), "o3".to_string()];

        let cmd = Provider::Codex.build_command(&worktree, &prompt, &status, &extra_args);

        assert!(cmd.contains("--model o3"));
    }

    #[test]
    fn test_build_gemini_command() {
        let worktree = PathBuf::from("/tmp/worktree");
        let prompt = PathBuf::from("/tmp/prompt.txt");
        let status = PathBuf::from("/tmp/status.json");

        let cmd = Provider::Gemini.build_command(&worktree, &prompt, &status, &[]);

        assert!(cmd.contains("cd /tmp/worktree"));
        assert!(cmd.contains("cat /tmp/prompt.txt"));
        assert!(cmd.contains("gemini -y"));
    }

    #[test]
    fn test_build_gemini_command_with_extra_args() {
        let worktree = PathBuf::from("/tmp/worktree");
        let prompt = PathBuf::from("/tmp/prompt.txt");
        let status = PathBuf::from("/tmp/status.json");
        let extra_args = vec!["--sandbox".to_string()];

        let cmd = Provider::Gemini.build_command(&worktree, &prompt, &status, &extra_args);

        assert!(cmd.contains("--sandbox"));
    }

    #[test]
    fn test_provider_equality() {
        assert_eq!(Provider::Claude, Provider::Claude);
        assert_eq!(Provider::Codex, Provider::Codex);
        assert_eq!(Provider::Gemini, Provider::Gemini);
        assert_ne!(Provider::Claude, Provider::Codex);
        assert_ne!(Provider::Claude, Provider::Gemini);
        assert_ne!(Provider::Codex, Provider::Gemini);
    }
}
