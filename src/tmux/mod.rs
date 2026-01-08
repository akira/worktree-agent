use crate::error::{Error, Result};
use std::path::Path;
use std::process::Command;

pub struct TmuxManager {
    session_name: String,
}

impl TmuxManager {
    pub fn new(session_name: &str) -> Self {
        Self {
            session_name: session_name.to_string(),
        }
    }

    /// Check if tmux session exists
    fn session_exists(&self) -> bool {
        Command::new("tmux")
            .args(["has-session", "-t", &self.session_name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Ensure the tmux session exists, creating it if necessary
    pub fn ensure_session(&self) -> Result<()> {
        if !self.session_exists() {
            let output = Command::new("tmux")
                .args([
                    "new-session",
                    "-d",
                    "-s",
                    &self.session_name,
                    "-n",
                    "main",
                ])
                .output()?;

            if !output.status.success() {
                return Err(Error::Tmux(format!(
                    "Failed to create tmux session: {}",
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
        }
        Ok(())
    }

    /// Create a new window in the session
    pub fn create_window(&self, name: &str, cwd: &Path) -> Result<()> {
        let output = Command::new("tmux")
            .args([
                "new-window",
                "-t",
                &self.session_name,
                "-n",
                name,
                "-c",
                cwd.to_str().unwrap_or("."),
            ])
            .output()?;

        if !output.status.success() {
            return Err(Error::Tmux(format!(
                "Failed to create tmux window: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(())
    }

    /// Send keys to a window
    pub fn send_keys(&self, window: &str, keys: &str) -> Result<()> {
        let target = format!("{}:{}", self.session_name, window);
        let output = Command::new("tmux")
            .args(["send-keys", "-t", &target, keys, "Enter"])
            .output()?;

        if !output.status.success() {
            return Err(Error::Tmux(format!(
                "Failed to send keys to tmux: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(())
    }

    /// Capture pane output
    pub fn capture_pane(&self, window: &str, lines: usize) -> Result<String> {
        let target = format!("{}:{}", self.session_name, window);
        let output = Command::new("tmux")
            .args([
                "capture-pane",
                "-t",
                &target,
                "-p",
                "-S",
                &format!("-{lines}"),
            ])
            .output()?;

        if !output.status.success() {
            return Err(Error::TmuxWindowNotFound(window.to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Kill a window
    pub fn kill_window(&self, window: &str) -> Result<()> {
        let target = format!("{}:{}", self.session_name, window);
        let output = Command::new("tmux")
            .args(["kill-window", "-t", &target])
            .output()?;

        if !output.status.success() {
            return Err(Error::TmuxWindowNotFound(window.to_string()));
        }
        Ok(())
    }

    /// Attach to the session, optionally selecting a window
    pub fn attach(&self, window: Option<&str>) -> Result<()> {
        let mut args = vec!["attach-session", "-t"];

        let target = match window {
            Some(w) => format!("{}:{}", self.session_name, w),
            None => self.session_name.clone(),
        };
        args.push(&target);

        let status = Command::new("tmux").args(&args).status()?;

        if !status.success() {
            return Err(Error::TmuxSessionNotFound(self.session_name.clone()));
        }
        Ok(())
    }

    /// Create a dashboard with split panes for all windows
    pub fn create_dashboard(&self, windows: &[&str]) -> Result<()> {
        if windows.is_empty() {
            return Err(Error::Tmux("No windows to display".to_string()));
        }

        // Create a new window for the dashboard
        let dashboard_window = "dashboard";

        // If dashboard window exists, kill it first
        let _ = self.kill_window(dashboard_window);

        // Create fresh dashboard window
        let output = Command::new("tmux")
            .args([
                "new-window",
                "-t",
                &self.session_name,
                "-n",
                dashboard_window,
            ])
            .output()?;

        if !output.status.success() {
            return Err(Error::Tmux(format!(
                "Failed to create dashboard window: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let session = &self.session_name;
        let dashboard_target = format!("{session}:{dashboard_window}");

        // Link the first window's pane
        if let Some(first) = windows.first() {
            let first_target = format!("{}:{}", self.session_name, first);
            Command::new("tmux")
                .args([
                    "send-keys",
                    "-t",
                    &dashboard_target,
                    &format!("tmux join-pane -s {first_target} -t {dashboard_target} || true"),
                    "Enter",
                ])
                .output()?;
        }

        // For additional windows, split and link
        for window in windows.iter().skip(1) {
            let target = format!("{}:{}", self.session_name, window);
            Command::new("tmux")
                .args([
                    "split-window",
                    "-t",
                    &dashboard_target,
                    "-h", // horizontal split
                ])
                .output()?;

            Command::new("tmux")
                .args([
                    "send-keys",
                    "-t",
                    &dashboard_target,
                    &format!("tmux join-pane -s {target} -t {dashboard_target} || true"),
                    "Enter",
                ])
                .output()?;
        }

        // Tile the panes evenly
        Command::new("tmux")
            .args(["select-layout", "-t", &dashboard_target, "tiled"])
            .output()?;

        // Attach to the dashboard
        self.attach(Some(dashboard_window))
    }

    pub fn session_name(&self) -> &str {
        &self.session_name
    }
}
