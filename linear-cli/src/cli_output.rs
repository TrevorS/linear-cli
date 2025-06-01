// ABOUTME: Centralized CLI output utilities for consistent user-facing messages
// ABOUTME: Provides standardized formatting for errors, warnings, info and success messages

use owo_colors::OwoColorize;
use std::io::IsTerminal;

/// Centralized CLI output utilities for consistent formatting
pub struct CliOutput {
    use_color: bool,
}

#[allow(dead_code)]
impl CliOutput {
    /// Create new CLI output utility with TTY detection
    pub fn new() -> Self {
        Self {
            use_color: std::io::stderr().is_terminal(),
        }
    }

    /// Create CLI output utility with explicit color setting
    pub fn with_color(use_color: bool) -> Self {
        Self { use_color }
    }

    /// Display an error message
    pub fn error(&self, message: &str) {
        if self.use_color {
            eprintln!("{} {}", "error:".red().bold(), message);
        } else {
            eprintln!("error: {}", message);
        }
    }

    /// Display a warning message
    pub fn warning(&self, message: &str) {
        if self.use_color {
            eprintln!("{} {}", "warning:".yellow().bold(), message);
        } else {
            eprintln!("warning: {}", message);
        }
    }

    /// Display an informational message
    pub fn info(&self, message: &str) {
        if self.use_color {
            eprintln!("{} {}", "info:".blue().bold(), message);
        } else {
            eprintln!("info: {}", message);
        }
    }

    /// Display a success message
    pub fn success(&self, message: &str) {
        if self.use_color {
            eprintln!("{} {}", "success:".green().bold(), message);
        } else {
            eprintln!("success: {}", message);
        }
    }

    /// Display a progress/status message with an icon
    pub fn status(&self, icon: &str, message: &str) {
        if self.use_color {
            eprintln!("{} {}", icon.dimmed(), message);
        } else {
            eprintln!("{} {}", icon, message);
        }
    }

    /// Display a debug message (only when RUST_LOG enables debug logging)
    pub fn debug(&self, message: &str) {
        log::debug!("{}", message);
    }
}

impl Default for CliOutput {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_output_creation() {
        let _cli = CliOutput::new();

        let cli_color = CliOutput::with_color(true);
        assert!(cli_color.use_color);

        let cli_no_color = CliOutput::with_color(false);
        assert!(!cli_no_color.use_color);
    }

    #[test]
    fn test_message_formatting() {
        let cli = CliOutput::with_color(false);

        // These tests don't capture output, but verify methods can be called
        cli.error("test error");
        cli.warning("test warning");
        cli.info("test info");
        cli.success("test success");
        cli.status("🔄", "processing");
        cli.debug("test debug");
    }

    #[test]
    fn test_default_trait() {
        let _cli: CliOutput = Default::default();
        // Test just verifies the Default trait works
    }
}
