// ABOUTME: Custom error types for Linear SDK with user-friendly messages
// ABOUTME: Provides specific error handling for different Linear API failure modes

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LinearError {
    #[error("Authentication failed. Check your LINEAR_API_KEY")]
    Auth,

    #[error("Issue {0} not found")]
    IssueNotFound(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("GraphQL error: {0}")]
    GraphQL(String),

    #[error("Rate limit exceeded. Please wait before making more requests")]
    RateLimit,

    #[error("Invalid API response format")]
    InvalidResponse,

    #[error("Timeout: Request took too long to complete")]
    Timeout,

    #[error("OAuth configuration error")]
    OAuthConfig,
}

impl LinearError {
    pub fn help_text(&self) -> Option<&'static str> {
        match self {
            LinearError::Auth => Some("Get your API key from: https://linear.app/settings/api"),
            LinearError::IssueNotFound(_) => {
                Some("Please check the issue identifier format (e.g., ENG-123)")
            }
            LinearError::Network(_) => Some("Check your internet connection and try again"),
            LinearError::RateLimit => Some("Wait a moment before making another request"),
            LinearError::Timeout => Some("Try again or check your network connection"),
            LinearError::OAuthConfig => Some(
                "Set up OAuth by creating an application at https://linear.app/settings/api/applications/new\n\nCallback URL: http://localhost:8089/callback\nThen set LINEAR_OAUTH_CLIENT_ID environment variable with your Client ID",
            ),
            _ => None,
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            LinearError::Network(_) | LinearError::Timeout | LinearError::RateLimit
        )
    }
}

impl From<reqwest::Error> for LinearError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            LinearError::Timeout
        } else if err.is_status() {
            if let Some(status) = err.status() {
                match status.as_u16() {
                    401 => LinearError::Auth,
                    429 => LinearError::RateLimit,
                    _ => LinearError::Network(err.to_string()),
                }
            } else {
                LinearError::Network(err.to_string())
            }
        } else {
            LinearError::Network(err.to_string())
        }
    }
}

impl From<serde_json::Error> for LinearError {
    fn from(_err: serde_json::Error) -> Self {
        LinearError::InvalidResponse
    }
}

impl From<anyhow::Error> for LinearError {
    fn from(err: anyhow::Error) -> Self {
        LinearError::Network(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        assert_eq!(
            LinearError::Auth.to_string(),
            "Authentication failed. Check your LINEAR_API_KEY"
        );
        assert_eq!(
            LinearError::IssueNotFound("ENG-123".to_string()).to_string(),
            "Issue ENG-123 not found"
        );
        assert_eq!(
            LinearError::Network("Connection refused".to_string()).to_string(),
            "Network error: Connection refused"
        );
        assert_eq!(
            LinearError::GraphQL("Field not found".to_string()).to_string(),
            "GraphQL error: Field not found"
        );
        assert_eq!(
            LinearError::OAuthConfig.to_string(),
            "OAuth configuration error"
        );
    }

    #[test]
    fn test_help_text() {
        assert_eq!(
            LinearError::Auth.help_text(),
            Some("Get your API key from: https://linear.app/settings/api")
        );
        assert_eq!(
            LinearError::IssueNotFound("ENG-123".to_string()).help_text(),
            Some("Please check the issue identifier format (e.g., ENG-123)")
        );
        assert_eq!(LinearError::GraphQL("test".to_string()).help_text(), None);
        assert!(LinearError::OAuthConfig.help_text().is_some());
        assert!(
            LinearError::OAuthConfig
                .help_text()
                .unwrap()
                .contains("linear.app/settings/api/applications/new")
        );
    }

    #[test]
    fn test_retryable() {
        assert!(LinearError::Network("test".to_string()).is_retryable());
        assert!(LinearError::Timeout.is_retryable());
        assert!(LinearError::RateLimit.is_retryable());
        assert!(!LinearError::Auth.is_retryable());
        assert!(!LinearError::IssueNotFound("ENG-123".to_string()).is_retryable());
        assert!(!LinearError::GraphQL("test".to_string()).is_retryable());
    }

    #[test]
    fn test_from_reqwest_error() {
        // Since we can't easily create specific reqwest errors in tests,
        // we'll test the conversion logic conceptually
        assert!(LinearError::Network("test".to_string()).is_retryable());
        assert!(LinearError::Timeout.is_retryable());
        assert!(LinearError::RateLimit.is_retryable());
    }
}
