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
    /// Deep Agents CLI
    Deepagents,
    /// Amp Code CLI
    Amp,
    /// Opencode CLI
    Opencode,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::Claude => write!(f, "claude"),
            Provider::Codex => write!(f, "codex"),
            Provider::Gemini => write!(f, "gemini"),
            Provider::Deepagents => write!(f, "deepagents"),
            Provider::Amp => write!(f, "amp"),
            Provider::Opencode => write!(f, "opencode"),
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
            Provider::Deepagents => "deepagents",
            Provider::Amp => "amp",
            Provider::Opencode => "opencode",
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
            Provider::Deepagents => {
                self.build_deepagents_command(worktree_path, prompt_file, extra_args)
            }
            Provider::Amp => self.build_amp_command(worktree_path, prompt_file, extra_args),
            Provider::Opencode => {
                self.build_opencode_command(worktree_path, prompt_file, extra_args)
            }
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

        // Check if dangerously-allow-all is in extra_args
        let dangerously_allow_all = extra_args.contains(&"--dangerously-allow-all".to_string());

        if dangerously_allow_all {
            // Skip permission restrictions entirely
            return format!(
                "cd {} && cat {} | claude --dangerously-allow-all{extra_args_str}",
                worktree_path.display(),
                prompt_file.display()
            );
        }

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

        // Codex CLI uses `codex exec -` to read prompt from stdin
        // --full-auto enables autonomous sandboxed execution
        format!(
            "cd {} && cat {} | codex exec --full-auto{extra_args_str} -",
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

    fn build_deepagents_command(
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

        // Deep Agents CLI uses --auto-approve to skip confirmation prompts
        // and accepts prompt via stdin
        format!(
            "cd {} && cat {} | deepagents --auto-approve{extra_args_str}",
            worktree_path.display(),
            prompt_file.display()
        )
    }

    fn build_amp_command(
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

        // Check if dangerously-allow-all is explicitly requested
        // Amp supports this flag for explicit permission skipping
        let has_dangerously_allow_all = extra_args.contains(&"--dangerously-allow-all".to_string());

        if has_dangerously_allow_all {
            // Amp CLI uses --dangerously-allow-all to skip all permission prompts
            format!(
                "cd {} && cat {} | amp --dangerously-allow-all{extra_args_str}",
                worktree_path.display(),
                prompt_file.display()
            )
        } else {
            // By default, Amp also runs with --dangerously-allow-all for consistency
            format!(
                "cd {} && cat {} | amp --dangerously-allow-all{extra_args_str}",
                worktree_path.display(),
                prompt_file.display()
            )
        }
    }

    fn build_opencode_command(
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

        // Opencode CLI accepts prompt via stdin
        format!(
            "cd {} && cat {} | opencode{extra_args_str}",
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
        assert_eq!(Provider::Deepagents.to_string(), "deepagents");
        assert_eq!(Provider::Amp.to_string(), "amp");
        assert_eq!(Provider::Opencode.to_string(), "opencode");
    }

    #[test]
    fn test_provider_binary_name() {
        assert_eq!(Provider::Claude.binary_name(), "claude");
        assert_eq!(Provider::Codex.binary_name(), "codex");
        assert_eq!(Provider::Gemini.binary_name(), "gemini");
        assert_eq!(Provider::Deepagents.binary_name(), "deepagents");
        assert_eq!(Provider::Amp.binary_name(), "amp");
        assert_eq!(Provider::Opencode.binary_name(), "opencode");
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

        let json = serde_json::to_string(&Provider::Deepagents).unwrap();
        assert_eq!(json, "\"deepagents\"");

        let json = serde_json::to_string(&Provider::Amp).unwrap();
        assert_eq!(json, "\"amp\"");

        let json = serde_json::to_string(&Provider::Opencode).unwrap();
        assert_eq!(json, "\"opencode\"");
    }

    #[test]
    fn test_provider_deserialization() {
        let provider: Provider = serde_json::from_str("\"claude\"").unwrap();
        assert_eq!(provider, Provider::Claude);

        let provider: Provider = serde_json::from_str("\"codex\"").unwrap();
        assert_eq!(provider, Provider::Codex);

        let provider: Provider = serde_json::from_str("\"gemini\"").unwrap();
        assert_eq!(provider, Provider::Gemini);

        let provider: Provider = serde_json::from_str("\"deepagents\"").unwrap();
        assert_eq!(provider, Provider::Deepagents);

        let provider: Provider = serde_json::from_str("\"amp\"").unwrap();
        assert_eq!(provider, Provider::Amp);

        let provider: Provider = serde_json::from_str("\"opencode\"").unwrap();
        assert_eq!(provider, Provider::Opencode);
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
    fn test_build_claude_command_dangerously_allow_all() {
        let worktree = PathBuf::from("/tmp/worktree");
        let prompt = PathBuf::from("/tmp/prompt.txt");
        let status = PathBuf::from("/tmp/status/agent.json");
        let extra_args = vec!["--dangerously-allow-all".to_string()];

        let cmd = Provider::Claude.build_command(&worktree, &prompt, &status, &extra_args);

        assert!(cmd.contains("claude --dangerously-allow-all"));
        // Should not contain allowedTools when dangerously-allow-all is used
        assert!(!cmd.contains("--allowedTools"));
        assert!(!cmd.contains("--permission-mode"));
    }

    #[test]
    fn test_build_codex_command() {
        let worktree = PathBuf::from("/tmp/worktree");
        let prompt = PathBuf::from("/tmp/prompt.txt");
        let status = PathBuf::from("/tmp/status.json");

        let cmd = Provider::Codex.build_command(&worktree, &prompt, &status, &[]);

        assert!(cmd.contains("cd /tmp/worktree"));
        assert!(cmd.contains("cat /tmp/prompt.txt"));
        assert!(cmd.contains("codex exec --full-auto"));
        assert!(cmd.ends_with(" -"));
    }

    #[test]
    fn test_build_codex_command_with_extra_args() {
        let worktree = PathBuf::from("/tmp/worktree");
        let prompt = PathBuf::from("/tmp/prompt.txt");
        let status = PathBuf::from("/tmp/status.json");
        let extra_args = vec!["--model".to_string(), "o3".to_string()];

        let cmd = Provider::Codex.build_command(&worktree, &prompt, &status, &extra_args);

        assert!(cmd.contains("--model o3"));
        assert!(cmd.ends_with(" -"));
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
    fn test_build_deepagents_command() {
        let worktree = PathBuf::from("/tmp/worktree");
        let prompt = PathBuf::from("/tmp/prompt.txt");
        let status = PathBuf::from("/tmp/status.json");

        let cmd = Provider::Deepagents.build_command(&worktree, &prompt, &status, &[]);

        assert!(cmd.contains("cd /tmp/worktree"));
        assert!(cmd.contains("cat /tmp/prompt.txt"));
        assert!(cmd.contains("deepagents --auto-approve"));
    }

    #[test]
    fn test_build_deepagents_command_with_extra_args() {
        let worktree = PathBuf::from("/tmp/worktree");
        let prompt = PathBuf::from("/tmp/prompt.txt");
        let status = PathBuf::from("/tmp/status.json");
        let extra_args = vec!["--agent".to_string(), "backend-dev".to_string()];

        let cmd = Provider::Deepagents.build_command(&worktree, &prompt, &status, &extra_args);

        assert!(cmd.contains("--agent backend-dev"));
    }

    #[test]
    fn test_build_amp_command() {
        let worktree = PathBuf::from("/tmp/worktree");
        let prompt = PathBuf::from("/tmp/prompt.txt");
        let status = PathBuf::from("/tmp/status.json");

        let cmd = Provider::Amp.build_command(&worktree, &prompt, &status, &[]);

        assert!(cmd.contains("cd /tmp/worktree"));
        assert!(cmd.contains("cat /tmp/prompt.txt"));
        assert!(cmd.contains("amp --dangerously-allow-all"));
    }

    #[test]
    fn test_build_amp_command_with_extra_args() {
        let worktree = PathBuf::from("/tmp/worktree");
        let prompt = PathBuf::from("/tmp/prompt.txt");
        let status = PathBuf::from("/tmp/status.json");
        let extra_args = vec!["--mode".to_string(), "rush".to_string()];

        let cmd = Provider::Amp.build_command(&worktree, &prompt, &status, &extra_args);

        assert!(cmd.contains("--mode rush"));
    }

    #[test]
    fn test_build_amp_command_dangerously_allow_all() {
        let worktree = PathBuf::from("/tmp/worktree");
        let prompt = PathBuf::from("/tmp/prompt.txt");
        let status = PathBuf::from("/tmp/status.json");
        let extra_args = vec!["--dangerously-allow-all".to_string()];

        let cmd = Provider::Amp.build_command(&worktree, &prompt, &status, &extra_args);

        assert!(cmd.contains("amp --dangerously-allow-all"));
    }

    #[test]
    fn test_build_opencode_command() {
        let worktree = PathBuf::from("/tmp/worktree");
        let prompt = PathBuf::from("/tmp/prompt.txt");
        let status = PathBuf::from("/tmp/status.json");

        let cmd = Provider::Opencode.build_command(&worktree, &prompt, &status, &[]);

        assert!(cmd.contains("cd /tmp/worktree"));
        assert!(cmd.contains("cat /tmp/prompt.txt"));
        assert!(cmd.contains("opencode"));
    }

    #[test]
    fn test_build_opencode_command_with_extra_args() {
        let worktree = PathBuf::from("/tmp/worktree");
        let prompt = PathBuf::from("/tmp/prompt.txt");
        let status = PathBuf::from("/tmp/status.json");
        let extra_args = vec!["--verbose".to_string()];

        let cmd = Provider::Opencode.build_command(&worktree, &prompt, &status, &extra_args);

        assert!(cmd.contains("--verbose"));
    }

    #[test]
    fn test_provider_equality() {
        assert_eq!(Provider::Claude, Provider::Claude);
        assert_eq!(Provider::Codex, Provider::Codex);
        assert_eq!(Provider::Gemini, Provider::Gemini);
        assert_eq!(Provider::Deepagents, Provider::Deepagents);
        assert_eq!(Provider::Amp, Provider::Amp);
        assert_eq!(Provider::Opencode, Provider::Opencode);
        assert_ne!(Provider::Claude, Provider::Codex);
        assert_ne!(Provider::Claude, Provider::Gemini);
        assert_ne!(Provider::Claude, Provider::Deepagents);
        assert_ne!(Provider::Claude, Provider::Amp);
        assert_ne!(Provider::Claude, Provider::Opencode);
        assert_ne!(Provider::Codex, Provider::Gemini);
        assert_ne!(Provider::Codex, Provider::Deepagents);
        assert_ne!(Provider::Codex, Provider::Amp);
        assert_ne!(Provider::Codex, Provider::Opencode);
        assert_ne!(Provider::Gemini, Provider::Deepagents);
        assert_ne!(Provider::Gemini, Provider::Amp);
        assert_ne!(Provider::Gemini, Provider::Opencode);
        assert_ne!(Provider::Deepagents, Provider::Amp);
        assert_ne!(Provider::Deepagents, Provider::Opencode);
        assert_ne!(Provider::Amp, Provider::Opencode);
    }
}
