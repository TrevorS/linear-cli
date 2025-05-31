// ABOUTME: Type definitions and enums for the Linear CLI
// ABOUTME: Provides structured types for issue status and other domain models

use crate::constants;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum IssueStatus {
    Todo,
    InProgress,
    Done,
    Backlog,
    Canceled,
    Unknown(String),
}

impl From<&str> for IssueStatus {
    fn from(s: &str) -> Self {
        match s {
            constants::status::TODO => IssueStatus::Todo,
            constants::status::IN_PROGRESS => IssueStatus::InProgress,
            constants::status::DONE => IssueStatus::Done,
            constants::status::BACKLOG => IssueStatus::Backlog,
            constants::status::CANCELED => IssueStatus::Canceled,
            other => IssueStatus::Unknown(other.to_string()),
        }
    }
}

impl From<String> for IssueStatus {
    fn from(s: String) -> Self {
        IssueStatus::from(s.as_str())
    }
}

impl fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueStatus::Todo => write!(f, "{}", constants::status::TODO),
            IssueStatus::InProgress => write!(f, "{}", constants::status::IN_PROGRESS),
            IssueStatus::Done => write!(f, "{}", constants::status::DONE),
            IssueStatus::Backlog => write!(f, "{}", constants::status::BACKLOG),
            IssueStatus::Canceled => write!(f, "{}", constants::status::CANCELED),
            IssueStatus::Unknown(s) => write!(f, "{}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_status_from_str() {
        assert_eq!(IssueStatus::from("Todo"), IssueStatus::Todo);
        assert_eq!(IssueStatus::from("In Progress"), IssueStatus::InProgress);
        assert_eq!(IssueStatus::from("Done"), IssueStatus::Done);
        assert_eq!(IssueStatus::from("Backlog"), IssueStatus::Backlog);
        assert_eq!(IssueStatus::from("Canceled"), IssueStatus::Canceled);
        assert_eq!(
            IssueStatus::from("Custom Status"),
            IssueStatus::Unknown("Custom Status".to_string())
        );
    }

    #[test]
    fn test_issue_status_display() {
        assert_eq!(IssueStatus::Todo.to_string(), "Todo");
        assert_eq!(IssueStatus::InProgress.to_string(), "In Progress");
        assert_eq!(IssueStatus::Done.to_string(), "Done");
        assert_eq!(IssueStatus::Backlog.to_string(), "Backlog");
        assert_eq!(IssueStatus::Canceled.to_string(), "Canceled");
        assert_eq!(
            IssueStatus::Unknown("Custom".to_string()).to_string(),
            "Custom"
        );
    }
}
