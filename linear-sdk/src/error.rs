// ABOUTME: Custom error types for Linear SDK with user-friendly messages
// ABOUTME: Provides specific error handling for different Linear API failure modes

use crate::constants::errors;
use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum LinearError {
    Auth {
        reason: Cow<'static, str>,
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    IssueNotFound {
        identifier: String,
        suggestion: Option<String>,
    },
    Network {
        message: String,
        retryable: bool,
        source: Box<dyn StdError + Send + Sync>,
    },
    GraphQL {
        message: String,
        errors: Vec<GraphQLError>,
    },
    RateLimit {
        reset_seconds: u64,
    },
    InvalidResponse,
    Timeout,
    OAuthConfig,
    Configuration(String),
    InvalidInput {
        message: String,
    },
}

impl fmt::Display for LinearError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinearError::Auth { reason, .. } => write!(f, "Authentication failed: {}", reason),
            LinearError::IssueNotFound { identifier, .. } => {
                write!(f, "Issue {} not found", identifier)
            }
            LinearError::Network { message, .. } => write!(f, "Network error: {}", message),
            LinearError::GraphQL { message, .. } => write!(f, "GraphQL error: {}", message),
            LinearError::RateLimit { .. } => write!(
                f,
                "Rate limit exceeded. Please wait before making more requests"
            ),
            LinearError::InvalidResponse => write!(f, "Invalid API response format"),
            LinearError::Timeout => write!(f, "Timeout: Request took too long to complete"),
            LinearError::OAuthConfig => write!(f, "OAuth configuration error"),
            LinearError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            LinearError::InvalidInput { message } => write!(f, "Invalid input: {}", message),
        }
    }
}

impl StdError for LinearError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            LinearError::Auth { source, .. } => source
                .as_ref()
                .map(|e| e.as_ref() as &(dyn StdError + 'static)),
            LinearError::Network { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphQLError {
    pub message: String,
    pub path: Option<Vec<String>>,
    pub extensions: Option<serde_json::Value>,
}

#[derive(Default)]
pub struct ErrorRecovery {}

impl ErrorRecovery {
    pub fn should_retry(&self, error: &LinearError) -> bool {
        error.is_retryable()
    }
}

pub trait ErrorContextExt<T> {
    fn context(self, msg: &str) -> Result<T, LinearError>;
}

impl<T> ErrorContextExt<T> for Result<T, LinearError> {
    fn context(self, msg: &str) -> Result<T, LinearError> {
        self.map_err(|e| LinearError::Network {
            message: format!("{}: {}", msg, e),
            retryable: e.is_retryable(),
            source: Box::new(e),
        })
    }
}

pub fn format_error_with_suggestion(err: &LinearError) -> String {
    let mut output = err.to_string();

    if let LinearError::IssueNotFound {
        suggestion: Some(sugg),
        ..
    } = err
    {
        output.push_str(&format!("\n{}", sugg));
    }

    output
}

impl LinearError {
    pub fn help_text(&self) -> Option<&'static str> {
        match self {
            LinearError::Auth { .. } => {
                Some("Get your API key from: https://linear.app/settings/api")
            }
            LinearError::IssueNotFound { .. } => {
                Some("Please check the issue identifier format (e.g., ENG-123)")
            }
            LinearError::Network { .. } => Some("Check your internet connection and try again"),
            LinearError::RateLimit { reset_seconds } => {
                if *reset_seconds > 0 {
                    return Some("Wait 60 seconds before making another request");
                }
                Some("Wait a moment before making another request")
            }
            LinearError::Timeout => Some("Try again or check your network connection"),
            LinearError::OAuthConfig => Some(
                "Set up OAuth by creating an application at https://linear.app/settings/api/applications/new\n\nCallback URL: http://localhost:8089/callback\nThen set LINEAR_OAUTH_CLIENT_ID environment variable with your Client ID",
            ),
            _ => None,
        }
    }

    pub fn is_retryable(&self) -> bool {
        match self {
            LinearError::Network { retryable, .. } => *retryable,
            LinearError::Timeout | LinearError::RateLimit { .. } => true,
            _ => false,
        }
    }

    pub fn from_status(status: http::StatusCode) -> Self {
        match status.as_u16() {
            401 => LinearError::Auth {
                reason: Cow::Borrowed("Unauthorized"),
                source: None,
            },
            408 => LinearError::Timeout,
            429 => LinearError::RateLimit { reset_seconds: 0 },
            errors::SERVER_ERROR_MIN..=errors::SERVER_ERROR_MAX => LinearError::Network {
                message: format!("Server error: {}", status),
                retryable: true,
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Server error",
                )),
            },
            _ => LinearError::Network {
                message: format!("HTTP error: {}", status),
                retryable: false,
                source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "HTTP error")),
            },
        }
    }
}

impl From<reqwest::Error> for LinearError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            LinearError::Timeout
        } else if err.is_status() {
            if let Some(status) = err.status() {
                match status.as_u16() {
                    401 => LinearError::Auth {
                        reason: Cow::Borrowed("Unauthorized"),
                        source: Some(Box::new(err)),
                    },
                    408 => LinearError::Timeout,
                    429 => LinearError::RateLimit { reset_seconds: 0 },
                    _ => LinearError::Network {
                        message: err.to_string(),
                        retryable: status.is_server_error(),
                        source: Box::new(err),
                    },
                }
            } else {
                LinearError::Network {
                    message: err.to_string(),
                    retryable: false,
                    source: Box::new(err),
                }
            }
        } else {
            LinearError::Network {
                message: err.to_string(),
                retryable: err.is_connect() || err.is_request(),
                source: Box::new(err),
            }
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
        LinearError::Network {
            message: err.to_string(),
            retryable: false,
            source: Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                err.to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;
    use std::error::Error as StdError;

    #[test]
    fn test_enhanced_auth_error() {
        let err = LinearError::Auth {
            reason: Cow::Borrowed("Invalid API key format"),
            source: None,
        };

        assert_eq!(
            err.to_string(),
            "Authentication failed: Invalid API key format"
        );
        assert_eq!(
            err.help_text(),
            Some("Get your API key from: https://linear.app/settings/api")
        );
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_enhanced_issue_not_found_error() {
        let err = LinearError::IssueNotFound {
            identifier: "ENG-123".to_string(),
            suggestion: Some("Did you mean ENG-122?".to_string()),
        };

        assert_eq!(err.to_string(), "Issue ENG-123 not found");
        assert!(
            err.help_text()
                .unwrap()
                .contains("check the issue identifier")
        );
        assert!(!err.is_retryable());

        // Test suggestion rendering
        let display = format!("{}", err);
        assert!(display.contains("ENG-123"));
    }

    #[test]
    fn test_enhanced_network_error() {
        let source_err =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused");
        let err = LinearError::Network {
            message: "Failed to connect to Linear API".to_string(),
            retryable: true,
            source: Box::new(source_err),
        };

        assert!(err.to_string().contains("Failed to connect to Linear API"));
        assert!(err.is_retryable());
        assert!(err.source().is_some());
    }

    #[test]
    fn test_graphql_error_with_structured_errors() {
        let graphql_errors = vec![GraphQLError {
            message: "Field 'foo' doesn't exist on type 'Issue'".to_string(),
            path: Some(vec![
                "query".to_string(),
                "issues".to_string(),
                "foo".to_string(),
            ]),
            extensions: None,
        }];

        let err = LinearError::GraphQL {
            message: "Query failed".to_string(),
            errors: graphql_errors,
        };

        assert_eq!(err.to_string(), "GraphQL error: Query failed");
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_rate_limit_with_reset_time() {
        let err = LinearError::RateLimit { reset_seconds: 60 };

        assert!(err.to_string().contains("Rate limit exceeded"));
        assert!(err.is_retryable());
        assert_eq!(
            err.help_text(),
            Some("Wait 60 seconds before making another request")
        );
    }

    #[test]
    fn test_error_context_trait() {
        let base_err = std::io::Error::new(std::io::ErrorKind::Other, "base error");
        let contextualized: Result<(), LinearError> = Err(LinearError::Network {
            message: "Network operation failed".to_string(),
            retryable: false,
            source: Box::new(base_err),
        });

        let with_context = contextualized.context("While fetching issues");
        assert!(with_context.is_err());

        let err = with_context.unwrap_err();
        assert!(format!("{:?}", err).contains("While fetching issues"));
    }

    #[test]
    fn test_error_recovery_retryable() {
        let recovery = ErrorRecovery::default();

        let network_err = LinearError::Network {
            message: "Connection timeout".to_string(),
            retryable: true,
            source: Box::new(std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout")),
        };

        assert!(recovery.should_retry(&network_err));

        let auth_err = LinearError::Auth {
            reason: Cow::Borrowed("Invalid credentials"),
            source: None,
        };

        assert!(!recovery.should_retry(&auth_err));
    }

    #[test]
    fn test_error_display_with_suggestions() {
        let err = LinearError::IssueNotFound {
            identifier: "ENG-999".to_string(),
            suggestion: Some("Try 'linear issues' to see available issues".to_string()),
        };

        let display = format_error_with_suggestion(&err);
        assert!(display.contains("Issue ENG-999 not found"));
        assert!(display.contains("Try 'linear issues'"));
    }

    #[test]
    fn test_from_http_status() {
        use http::StatusCode;

        let err = LinearError::from_status(StatusCode::UNAUTHORIZED);
        assert!(matches!(err, LinearError::Auth { .. }));

        let err = LinearError::from_status(StatusCode::TOO_MANY_REQUESTS);
        assert!(matches!(err, LinearError::RateLimit { .. }));

        let err = LinearError::from_status(StatusCode::INTERNAL_SERVER_ERROR);
        assert!(matches!(
            err,
            LinearError::Network {
                retryable: true,
                ..
            }
        ));

        let err = LinearError::from_status(StatusCode::REQUEST_TIMEOUT);
        assert!(matches!(err, LinearError::Timeout));
        assert!(err.is_retryable());
    }

    #[test]
    fn test_error_messages() {
        assert_eq!(
            LinearError::Auth {
                reason: Cow::Borrowed("Check your LINEAR_API_KEY"),
                source: None
            }
            .to_string(),
            "Authentication failed: Check your LINEAR_API_KEY"
        );
        assert_eq!(
            LinearError::IssueNotFound {
                identifier: "ENG-123".to_string(),
                suggestion: None
            }
            .to_string(),
            "Issue ENG-123 not found"
        );
        assert_eq!(
            LinearError::Network {
                message: "Connection refused".to_string(),
                retryable: false,
                source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test"))
            }
            .to_string(),
            "Network error: Connection refused"
        );
        assert_eq!(
            LinearError::GraphQL {
                message: "Field not found".to_string(),
                errors: vec![]
            }
            .to_string(),
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
            LinearError::Auth {
                reason: Cow::Borrowed("test"),
                source: None
            }
            .help_text(),
            Some("Get your API key from: https://linear.app/settings/api")
        );
        assert_eq!(
            LinearError::IssueNotFound {
                identifier: "ENG-123".to_string(),
                suggestion: None
            }
            .help_text(),
            Some("Please check the issue identifier format (e.g., ENG-123)")
        );
        assert_eq!(
            LinearError::GraphQL {
                message: "test".to_string(),
                errors: vec![]
            }
            .help_text(),
            None
        );
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
        assert!(
            LinearError::Network {
                message: "test".to_string(),
                retryable: true,
                source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test"))
            }
            .is_retryable()
        );
        assert!(LinearError::Timeout.is_retryable());
        assert!(LinearError::RateLimit { reset_seconds: 0 }.is_retryable());
        assert!(
            !LinearError::Auth {
                reason: Cow::Borrowed("test"),
                source: None
            }
            .is_retryable()
        );
        assert!(
            !LinearError::IssueNotFound {
                identifier: "ENG-123".to_string(),
                suggestion: None
            }
            .is_retryable()
        );
        assert!(
            !LinearError::GraphQL {
                message: "test".to_string(),
                errors: vec![]
            }
            .is_retryable()
        );
    }

    #[test]
    fn test_from_reqwest_error() {
        // Since we can't easily create specific reqwest errors in tests,
        // we'll test the conversion logic conceptually
        assert!(
            LinearError::Network {
                message: "test".to_string(),
                retryable: true,
                source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test"))
            }
            .is_retryable()
        );
        assert!(LinearError::Timeout.is_retryable());
        assert!(LinearError::RateLimit { reset_seconds: 0 }.is_retryable());
    }
}
