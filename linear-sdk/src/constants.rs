// ABOUTME: Centralized constants for the Linear SDK
// ABOUTME: Contains retry configuration, timeouts, and error handling constants

/// Retry configuration constants
pub mod retry {
    use std::time::Duration;

    /// Maximum number of retry attempts
    pub const MAX_RETRIES: u32 = 3;

    /// Initial delay before first retry
    pub const INITIAL_DELAY: Duration = Duration::from_millis(100);

    /// Maximum delay between retries
    pub const MAX_DELAY: Duration = Duration::from_secs(10);

    /// Backoff multiplier for exponential backoff
    pub const BACKOFF_MULTIPLIER: f64 = 2.0;
}

/// HTTP and request timeouts
pub mod timeouts {
    use std::time::Duration;

    /// Default timeout for HTTP requests
    pub const HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
}

/// Linear API URLs
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

/// Error handling constants
pub mod errors {
    /// Default wait time for rate limit errors (in seconds)
    pub const RATE_LIMIT_WAIT_SECONDS: u64 = 60;

    /// Server error status code range
    pub const SERVER_ERROR_MIN: u16 = 500;
    pub const SERVER_ERROR_MAX: u16 = 599;

    /// OAuth callback URL for error messages
    pub const OAUTH_CALLBACK_URL: &str = "http://localhost:8089/callback";
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_retry_constants() {
        assert_eq!(retry::MAX_RETRIES, 3);
        assert_eq!(retry::INITIAL_DELAY, Duration::from_millis(100));
        assert_eq!(retry::MAX_DELAY, Duration::from_secs(10));
        assert_eq!(retry::BACKOFF_MULTIPLIER, 2.0);
    }

    #[test]
    fn test_timeout_constants() {
        assert_eq!(timeouts::HTTP_REQUEST_TIMEOUT, Duration::from_secs(30));
    }

    #[test]
    fn test_url_constants() {
        assert!(urls::LINEAR_API_BASE.starts_with("https://"));
        assert!(urls::LINEAR_OAUTH_AUTHORIZE.contains("linear.app"));
        assert!(urls::LINEAR_OAUTH_TOKEN.contains("api.linear.app"));
        assert_eq!(urls::OAUTH_CALLBACK_BASE, "http://localhost");
    }

    #[test]
    fn test_error_constants() {
        assert_eq!(errors::RATE_LIMIT_WAIT_SECONDS, 60);
        assert_eq!(errors::SERVER_ERROR_MIN, 500);
        assert_eq!(errors::SERVER_ERROR_MAX, 599);
        assert!(errors::OAUTH_CALLBACK_URL.contains("8089"));
    }
}
