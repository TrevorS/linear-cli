// ABOUTME: Centralized constants for the Linear CLI application
// ABOUTME: Contains limits, timeouts, URLs, terminal sequences, and status mappings

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Default limits for issue queries
#[allow(dead_code)] // Reserved for future query functionality
pub mod limits {
    pub const DEFAULT_ISSUE_LIMIT: i32 = 20;
    pub const MAX_ISSUE_LIMIT: i32 = 100;
}

/// Timeout configurations for various operations
pub mod timeouts {
    use std::time::Duration;

    /// Default timeout for HTTP requests
    #[allow(dead_code)] // Reserved for future timeout configuration
    pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

    /// Progress bar tick interval for smooth animation
    pub const PROGRESS_BAR_TICK_MS: u64 = 80;
}

/// Linear API URLs
#[allow(dead_code)] // Reserved for SDK constants alignment
pub mod urls {
    /// Base URL for Linear API GraphQL endpoint
    pub const LINEAR_API_BASE: &str = "https://api.linear.app";

    /// Linear OAuth authorization URL
    pub const LINEAR_OAUTH_AUTHORIZE: &str = "https://linear.app/oauth/authorize";

    /// Linear OAuth token exchange URL
    pub const LINEAR_OAUTH_TOKEN: &str = "https://api.linear.app/oauth/token";

    /// Local OAuth callback base URL
    pub const OAUTH_CALLBACK_BASE: &str = "http://localhost";
}

/// UI and formatting constants
pub mod ui {
    /// Border line length for issue display formatting
    pub const BORDER_LINE_LENGTH: usize = 50;

    /// OSC 8 hyperlink escape sequence format
    #[allow(dead_code)] // Reserved for future terminal enhancement
    pub const OSC8_HYPERLINK_FORMAT: &str = "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\";
}

/// Status name aliases for improved UX
#[allow(dead_code)] // Reserved for future CLI enhancement
pub static STATUS_ALIASES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // Common aliases for status names
    m.insert("todo", "Todo");
    m.insert("in-progress", "In Progress");
    m.insert("inprogress", "In Progress");
    m.insert("in_progress", "In Progress");
    m.insert("done", "Done");
    m.insert("completed", "Done");
    m.insert("backlog", "Backlog");
    m.insert("canceled", "Canceled");
    m.insert("cancelled", "Canceled");
    m
});

/// Status display names
pub mod status {
    pub const TODO: &str = "Todo";
    pub const IN_PROGRESS: &str = "In Progress";
    pub const DONE: &str = "Done";
    pub const BACKLOG: &str = "Backlog";
    pub const CANCELED: &str = "Canceled";

    /// All valid status names
    #[allow(dead_code)] // Reserved for status validation
    pub const ALL_STATUSES: &[&str] = &[TODO, IN_PROGRESS, DONE, BACKLOG, CANCELED];
}

/// Error messages
#[allow(dead_code)] // Reserved for enhanced error handling
pub mod errors {
    pub const RATE_LIMIT_WAIT_SECONDS: u64 = 60;
    pub const SERVER_ERROR_MIN: u16 = 500;
    pub const SERVER_ERROR_MAX: u16 = 599;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_limits() {
        assert_eq!(limits::DEFAULT_ISSUE_LIMIT, 20);
        assert_eq!(limits::MAX_ISSUE_LIMIT, 100);
        assert_ne!(limits::DEFAULT_ISSUE_LIMIT, limits::MAX_ISSUE_LIMIT);
    }

    #[test]
    fn test_timeouts() {
        assert_eq!(timeouts::DEFAULT_REQUEST_TIMEOUT, Duration::from_secs(30));
        assert_eq!(timeouts::PROGRESS_BAR_TICK_MS, 80);
    }

    #[test]
    fn test_urls() {
        assert!(urls::LINEAR_API_BASE.starts_with("https://"));
        assert!(urls::LINEAR_OAUTH_AUTHORIZE.contains("linear.app"));
        assert!(urls::LINEAR_OAUTH_TOKEN.contains("api.linear.app"));
        assert_eq!(urls::OAUTH_CALLBACK_BASE, "http://localhost");
    }

    #[test]
    fn test_ui_constants() {
        assert_eq!(ui::BORDER_LINE_LENGTH, 50);
        assert!(ui::OSC8_HYPERLINK_FORMAT.contains("\x1b"));
    }

    #[test]
    fn test_status_aliases() {
        // Test case insensitive aliases
        assert_eq!(STATUS_ALIASES.get("todo"), Some(&"Todo"));
        assert_eq!(STATUS_ALIASES.get("in-progress"), Some(&"In Progress"));
        assert_eq!(STATUS_ALIASES.get("in_progress"), Some(&"In Progress"));
        assert_eq!(STATUS_ALIASES.get("done"), Some(&"Done"));
        assert_eq!(STATUS_ALIASES.get("completed"), Some(&"Done"));
        assert_eq!(STATUS_ALIASES.get("cancelled"), Some(&"Canceled"));
    }

    #[test]
    fn test_status_constants() {
        assert_eq!(status::TODO, "Todo");
        assert_eq!(status::IN_PROGRESS, "In Progress");
        assert_eq!(status::DONE, "Done");
        assert_eq!(status::BACKLOG, "Backlog");
        assert_eq!(status::CANCELED, "Canceled");
        assert_eq!(status::ALL_STATUSES.len(), 5);
    }

    #[test]
    fn test_error_constants() {
        assert_eq!(errors::RATE_LIMIT_WAIT_SECONDS, 60);
        assert_eq!(errors::SERVER_ERROR_MIN, 500);
        assert_eq!(errors::SERVER_ERROR_MAX, 599);
    }
}
