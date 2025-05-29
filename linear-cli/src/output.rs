// ABOUTME: This module handles output formatting for the Linear CLI
// ABOUTME: It provides different formatters like table formatting with color support

use anyhow::Result;
use linear_sdk::Issue;
use owo_colors::OwoColorize;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::types::IssueStatus;

pub trait OutputFormat {
    fn format_issues(&self, issues: &[Issue]) -> Result<String>;
}

pub struct TableFormatter {
    use_color: bool,
}

impl TableFormatter {
    pub fn new(use_color: bool) -> Self {
        Self { use_color }
    }

    fn truncate_title(title: &str, max_len: usize) -> String {
        if title.len() <= max_len {
            title.to_string()
        } else {
            format!("{}...", &title[..max_len - 3])
        }
    }

    fn format_status(&self, status_str: &str) -> String {
        let status: IssueStatus = status_str.into();

        if self.use_color {
            match status {
                IssueStatus::Todo | IssueStatus::Backlog => {
                    format!("{}", status.to_string().dimmed())
                }
                IssueStatus::InProgress => format!("{}", status.to_string().yellow()),
                IssueStatus::Done => format!("{}", status.to_string().green()),
                IssueStatus::Canceled => format!("{}", status.to_string().red()),
                IssueStatus::Unknown(_) => status.to_string(),
            }
        } else {
            status.to_string()
        }
    }

    fn format_assignee(&self, assignee: &Option<String>) -> String {
        let text = assignee.as_deref().unwrap_or("Unassigned");

        if self.use_color && assignee.is_none() {
            text.dimmed().to_string()
        } else {
            text.to_string()
        }
    }
}

pub struct JsonFormatter {
    pretty: bool,
}

impl JsonFormatter {
    pub fn new(pretty: bool) -> Self {
        Self { pretty }
    }
}

impl OutputFormat for JsonFormatter {
    fn format_issues(&self, issues: &[Issue]) -> Result<String> {
        if self.pretty {
            Ok(serde_json::to_string_pretty(issues)?)
        } else {
            Ok(serde_json::to_string(issues)?)
        }
    }
}

#[derive(Tabled)]
struct TableRow {
    #[tabled(rename = "Issue")]
    issue: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Assignee")]
    assignee: String,
}

impl OutputFormat for TableFormatter {
    fn format_issues(&self, issues: &[Issue]) -> Result<String> {
        let rows: Vec<TableRow> = issues
            .iter()
            .map(|issue| TableRow {
                issue: issue.identifier.clone(),
                title: Self::truncate_title(&issue.title, 40),
                status: self.format_status(&issue.status),
                assignee: self.format_assignee(&issue.assignee),
            })
            .collect();

        let mut table = Table::new(rows);
        table.with(Style::psql());
        Ok(table.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_issue(
        identifier: &str,
        title: &str,
        status: &str,
        assignee: Option<String>,
    ) -> Issue {
        Issue {
            id: format!("id-{}", identifier),
            identifier: identifier.to_string(),
            title: title.to_string(),
            status: status.to_string(),
            assignee: assignee.clone(),
            assignee_id: assignee.map(|_| format!("user-{}", identifier)),
            team: Some("TEST".to_string()),
        }
    }

    #[test]
    fn test_table_formatter_with_color() {
        let formatter = TableFormatter::new(true);
        let issues = vec![
            create_test_issue(
                "ENG-123",
                "Fix login race condition",
                "In Progress",
                Some("John Doe".to_string()),
            ),
            create_test_issue("ENG-124", "Implement OAuth flow", "Todo", None),
        ];

        let result = formatter.format_issues(&issues).unwrap();

        // Basic structure tests - will be more specific once implemented
        assert!(result.contains("ENG-123"));
        assert!(result.contains("Fix login race condition"));
        assert!(result.contains("John Doe"));
        assert!(result.contains("ENG-124"));
        assert!(result.contains("Implement OAuth flow"));
        assert!(result.contains("Unassigned"));
    }

    #[test]
    fn test_table_formatter_without_color() {
        let formatter = TableFormatter::new(false);
        let issues = vec![create_test_issue(
            "ENG-125",
            "A very long title that should be truncated because it exceeds the maximum allowed length",
            "Done",
            None,
        )];

        let result = formatter.format_issues(&issues).unwrap();

        // Test truncation
        assert!(result.contains("A very long title that should be trun..."));
        assert!(!result.contains("exceeds the maximum"));
    }

    #[test]
    fn test_empty_issues() {
        let formatter = TableFormatter::new(false);
        let issues = vec![];

        let result = formatter.format_issues(&issues).unwrap();

        // Should still have headers
        assert!(result.contains("Issue"));
        assert!(result.contains("Title"));
        assert!(result.contains("Status"));
        assert!(result.contains("Assignee"));
    }

    #[test]
    fn test_snapshot_colored_output() {
        let formatter = TableFormatter::new(true);
        let issues = vec![
            create_test_issue(
                "ENG-100",
                "Fix authentication bug",
                "Todo",
                Some("Alice Smith".to_string()),
            ),
            create_test_issue(
                "ENG-101",
                "Implement user profile page",
                "In Progress",
                None,
            ),
            create_test_issue(
                "ENG-102",
                "Update documentation",
                "Done",
                Some("Bob Johnson".to_string()),
            ),
        ];

        let result = formatter.format_issues(&issues).unwrap();
        insta::assert_snapshot!(result);
    }

    #[test]
    fn test_snapshot_noncolored_output() {
        let formatter = TableFormatter::new(false);
        let issues = vec![
            create_test_issue(
                "ENG-100",
                "Fix authentication bug",
                "Todo",
                Some("Alice Smith".to_string()),
            ),
            create_test_issue(
                "ENG-101",
                "Implement user profile page",
                "In Progress",
                None,
            ),
            create_test_issue(
                "ENG-102",
                "Update documentation",
                "Done",
                Some("Bob Johnson".to_string()),
            ),
        ];

        let result = formatter.format_issues(&issues).unwrap();
        insta::assert_snapshot!(result);
    }

    #[test]
    fn test_snapshot_long_titles() {
        let formatter = TableFormatter::new(false);
        let issues = vec![
            create_test_issue(
                "ENG-200",
                "This is a very long title that definitely exceeds the maximum character limit and should be truncated",
                "Todo",
                Some("Very Long Assignee Name That Also Gets Displayed".to_string()),
            ),
            create_test_issue("ENG-201", "Short title", "Done", None),
        ];

        let result = formatter.format_issues(&issues).unwrap();
        insta::assert_snapshot!(result);
    }

    #[test]
    fn test_snapshot_empty_results() {
        let formatter = TableFormatter::new(false);
        let issues = vec![];

        let result = formatter.format_issues(&issues).unwrap();
        insta::assert_snapshot!(result);
    }

    #[test]
    fn test_json_formatter_compact() {
        let formatter = JsonFormatter::new(false);
        let issues = vec![
            create_test_issue(
                "ENG-123",
                "Fix login race condition",
                "In Progress",
                Some("John Doe".to_string()),
            ),
            create_test_issue("ENG-124", "Implement OAuth flow", "Todo", None),
        ];

        let result = formatter.format_issues(&issues).unwrap();

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 2);

        // Verify fields are present and correctly named (camelCase)
        let first_issue = &parsed[0];
        assert!(first_issue["id"].is_string());
        assert!(first_issue["identifier"].is_string());
        assert!(first_issue["title"].is_string());
        assert!(first_issue["status"].is_string());
        assert!(first_issue["assignee"].is_string());

        assert_eq!(first_issue["identifier"], "ENG-123");
        assert_eq!(first_issue["title"], "Fix login race condition");
        assert_eq!(first_issue["status"], "In Progress");
        assert_eq!(first_issue["assignee"], "John Doe");
    }

    #[test]
    fn test_json_formatter_pretty() {
        let formatter = JsonFormatter::new(true);
        let issues = vec![create_test_issue("ENG-125", "Test issue", "Done", None)];

        let result = formatter.format_issues(&issues).unwrap();

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_array());

        // Pretty printed JSON should contain newlines and indentation
        assert!(result.contains('\n'));
        assert!(result.contains("  "));

        // Verify null handling for assignee
        let first_issue = &parsed[0];
        assert!(first_issue["assignee"].is_null());
    }

    #[test]
    fn test_json_formatter_empty() {
        let formatter = JsonFormatter::new(false);
        let issues = vec![];

        let result = formatter.format_issues(&issues).unwrap();

        // Should be empty array
        assert_eq!(result, "[]");

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 0);
    }
}
