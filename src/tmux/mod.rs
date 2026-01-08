use crate::error::{Error, Result};
use std::path::Path;
use std::process::Command;

const TMUX: &str = "tmux";
const MAIN_WINDOW: &str = "main";
const ERR_CREATE_SESSION: &str = "Failed to create tmux session";
const ERR_CREATE_WINDOW: &str = "Failed to create tmux window";
const ERR_SEND_KEYS: &str = "Failed to send keys to tmux";

pub struct TmuxManager {
    session_name: String,
}

impl TmuxManager {
    pub fn new(session_name: &str) -> Self {
        Self {
            session_name: session_name.to_string(),
        }
    }

    fn target(&self, window: &str) -> String {
        format!("{}:{window}", self.session_name)
    }

    fn run_tmux(&self, args: &[&str]) -> Result<std::process::Output> {
        Command::new(TMUX).args(args).output().map_err(Error::from)
    }

    /// Check if tmux session exists
    fn session_exists(&self) -> bool {
        self.run_tmux(&["has-session", "-t", &self.session_name])
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Ensure the tmux session exists, creating it if necessary
    pub fn ensure_session(&self) -> Result<()> {
        if self.session_exists() {
            return Ok(());
        }

        let output = self.run_tmux(&[
            "new-session",
            "-d",
            "-s",
            &self.session_name,
            "-n",
            MAIN_WINDOW,
        ])?;

        if !output.status.success() {
            return Err(Error::Tmux(format!(
                "{ERR_CREATE_SESSION}: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(())
    }

    /// Create a new window in the session
    pub fn create_window(&self, name: &str, cwd: &Path) -> Result<()> {
        let output = self.run_tmux(&[
            "new-window",
            "-t",
            &self.session_name,
            "-n",
            name,
            "-c",
            cwd.to_str().unwrap_or("."),
        ])?;

        if !output.status.success() {
            return Err(Error::Tmux(format!(
                "{ERR_CREATE_WINDOW}: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(())
    }

    /// Send keys to a window
    pub fn send_keys(&self, window: &str, keys: &str) -> Result<()> {
        let target = self.target(window);
        let output = self.run_tmux(&["send-keys", "-t", &target, keys, "Enter"])?;

        if !output.status.success() {
            return Err(Error::Tmux(format!(
                "{ERR_SEND_KEYS}: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(())
    }

    /// Capture pane output
    pub fn capture_pane(&self, window: &str, lines: usize) -> Result<String> {
        let target = self.target(window);
        let lines_arg = format!("-{lines}");
        let output = self.run_tmux(&["capture-pane", "-t", &target, "-p", "-S", &lines_arg])?;

        if !output.status.success() {
            return Err(Error::TmuxWindowNotFound(window.to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Check if a window exists
    pub fn window_exists(&self, window: &str) -> bool {
        let target = self.target(window);
        self.run_tmux(&["has-session", "-t", &target])
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Kill a window
    pub fn kill_window(&self, window: &str) -> Result<()> {
        let target = self.target(window);
        let output = self.run_tmux(&["kill-window", "-t", &target])?;

        if !output.status.success() {
            return Err(Error::TmuxWindowNotFound(window.to_string()));
        }
        Ok(())
    }

    /// Attach to the session, optionally selecting a window
    pub fn attach(&self, window: Option<&str>) -> Result<()> {
        let target = match window {
            Some(w) => self.target(w),
            None => self.session_name.clone(),
        };

        let status = Command::new(TMUX)
            .args(["attach-session", "-t", &target])
            .status()?;

        if !status.success() {
            return Err(Error::TmuxSessionNotFound(self.session_name.clone()));
        }
        Ok(())
    }
}
