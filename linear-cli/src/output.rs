// ABOUTME: This module handles output formatting for the Linear CLI
// ABOUTME: It provides different formatters like table formatting with color support

use linear_sdk::{DetailedIssue, Issue, Result};
use owo_colors::OwoColorize;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::types::IssueStatus;

pub trait OutputFormat {
    fn format_issues(&self, issues: &[Issue]) -> Result<String>;
    fn format_detailed_issue(&self, issue: &DetailedIssue) -> Result<String>;
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

    fn format_detailed_assignee(&self, assignee: &Option<linear_sdk::IssueAssignee>) -> String {
        match assignee {
            Some(a) => {
                if self.use_color {
                    format!("{} ({})", a.name.bold(), a.email.dimmed())
                } else {
                    format!("{} ({})", a.name, a.email)
                }
            }
            None => {
                if self.use_color {
                    "Unassigned".dimmed().to_string()
                } else {
                    "Unassigned".to_string()
                }
            }
        }
    }

    fn format_team(&self, team: &Option<linear_sdk::IssueTeam>) -> String {
        match team {
            Some(t) => {
                if self.use_color {
                    format!("{} ({})", t.name.cyan(), t.key.dimmed())
                } else {
                    format!("{} ({})", t.name, t.key)
                }
            }
            None => "No team".to_string(),
        }
    }

    fn format_priority(&self, priority: Option<i64>, priority_label: &Option<String>) -> String {
        if let Some(label) = priority_label {
            if self.use_color {
                match label.as_str() {
                    "Urgent" => label.red().bold().to_string(),
                    "High" => label.red().to_string(),
                    "Medium" => label.yellow().to_string(),
                    "Low" => label.blue().to_string(),
                    _ => label.to_string(),
                }
            } else {
                label.to_string()
            }
        } else if let Some(p) = priority {
            p.to_string()
        } else {
            "None".to_string()
        }
    }

    fn format_labels(&self, labels: &[linear_sdk::IssueLabel]) -> String {
        if labels.is_empty() {
            return "None".to_string();
        }

        labels
            .iter()
            .map(|l| {
                if self.use_color {
                    // For now, just use a simple colored format
                    format!("{}", l.name.cyan())
                } else {
                    l.name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_datetime(&self, datetime: &str) -> String {
        if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(datetime) {
            parsed.format("%Y-%m-%d %H:%M").to_string()
        } else {
            datetime.to_string()
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

    fn format_detailed_issue(&self, issue: &DetailedIssue) -> Result<String> {
        if self.pretty {
            Ok(serde_json::to_string_pretty(issue)?)
        } else {
            Ok(serde_json::to_string(issue)?)
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

    fn format_detailed_issue(&self, issue: &DetailedIssue) -> Result<String> {
        let border_line = "─".repeat(50);
        let title_line = if self.use_color {
            format!("{}: {}", issue.identifier.bold().blue(), issue.title.bold())
        } else {
            format!("{}: {}", issue.identifier, issue.title)
        };

        let mut output = vec![
            border_line.clone(),
            title_line,
            border_line,
            format!("Status:     {}", self.format_status(&issue.state.name)),
            format!(
                "Assignee:   {}",
                self.format_detailed_assignee(&issue.assignee)
            ),
            format!("Team:       {}", self.format_team(&issue.team)),
        ];

        if let Some(project) = &issue.project {
            output.push(format!("Project:    {}", project.name));
        }

        output.push(format!(
            "Priority:   {}",
            self.format_priority(issue.priority, &issue.priority_label)
        ));

        if let Some(description) = &issue.description {
            output.push(String::new());
            output.push("Description:".to_string());
            output.push(description.clone());
        }

        output.push(String::new());
        output.push(format!("Labels: {}", self.format_labels(&issue.labels)));

        output.push(String::new());
        output.push(format!(
            "Created: {}",
            self.format_datetime(&issue.created_at)
        ));
        output.push(format!(
            "Updated: {}",
            self.format_datetime(&issue.updated_at)
        ));

        output.push(String::new());
        output.push(format!("View in Linear: {}", issue.url));

        Ok(output.join("\n"))
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

    fn create_test_detailed_issue() -> DetailedIssue {
        use linear_sdk::*;

        DetailedIssue {
            id: "issue-id-123".to_string(),
            identifier: "ENG-123".to_string(),
            title: "Fix login race condition".to_string(),
            description: Some("Users are experiencing race conditions when logging in simultaneously from multiple devices.".to_string()),
            state: IssueState {
                name: "In Progress".to_string(),
                type_: "started".to_string(),
            },
            assignee: Some(IssueAssignee {
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
            }),
            team: Some(IssueTeam {
                key: "ENG".to_string(),
                name: "Engineering".to_string(),
            }),
            project: Some(IssueProject {
                name: "Web App".to_string(),
            }),
            labels: vec![
                IssueLabel {
                    name: "bug".to_string(),
                    color: "#ff0000".to_string(),
                },
                IssueLabel {
                    name: "authentication".to_string(),
                    color: "#00ff00".to_string(),
                },
            ],
            priority: Some(2),
            priority_label: Some("High".to_string()),
            created_at: "2024-01-15T10:30:00Z".to_string(),
            updated_at: "2024-01-16T14:45:00Z".to_string(),
            url: "https://linear.app/test/issue/ENG-123".to_string(),
        }
    }

    #[test]
    fn test_detailed_issue_table_format() {
        let formatter = TableFormatter::new(false);
        let issue = create_test_detailed_issue();

        let result = formatter.format_detailed_issue(&issue).unwrap();

        // Check that key elements are present
        assert!(result.contains("ENG-123: Fix login race condition"));
        assert!(result.contains("Status:     In Progress"));
        assert!(result.contains("Assignee:   John Doe (john@example.com)"));
        assert!(result.contains("Team:       Engineering (ENG)"));
        assert!(result.contains("Project:    Web App"));
        assert!(result.contains("Priority:   High"));
        assert!(result.contains("Description:"));
        assert!(result.contains("Users are experiencing race conditions"));
        assert!(result.contains("Labels: bug, authentication"));
        assert!(result.contains("Created:"));
        assert!(result.contains("Updated:"));
        assert!(result.contains("View in Linear: https://linear.app/test/issue/ENG-123"));
    }

    #[test]
    fn test_detailed_issue_table_format_colored() {
        let formatter = TableFormatter::new(true);
        let issue = create_test_detailed_issue();

        let result = formatter.format_detailed_issue(&issue).unwrap();

        // Should still contain the basic text content
        assert!(result.contains("ENG-123"));
        assert!(result.contains("Fix login race condition"));
        assert!(result.contains("John Doe"));
        assert!(result.contains("Engineering"));
    }

    #[test]
    fn test_detailed_issue_json_format() {
        let formatter = JsonFormatter::new(false);
        let issue = create_test_detailed_issue();

        let result = formatter.format_detailed_issue(&issue).unwrap();

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_object());

        // Check key fields
        assert_eq!(parsed["identifier"], "ENG-123");
        assert_eq!(parsed["title"], "Fix login race condition");
        assert_eq!(parsed["state"]["name"], "In Progress");
        assert_eq!(parsed["assignee"]["name"], "John Doe");
        assert_eq!(parsed["team"]["name"], "Engineering");
        assert_eq!(parsed["priorityLabel"], "High");
        assert!(parsed["labels"].is_array());
        assert_eq!(parsed["labels"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_detailed_issue_minimal_fields() {
        use linear_sdk::*;

        let minimal_issue = DetailedIssue {
            id: "issue-id-456".to_string(),
            identifier: "ENG-456".to_string(),
            title: "Simple issue".to_string(),
            description: None,
            state: IssueState {
                name: "Todo".to_string(),
                type_: "unstarted".to_string(),
            },
            assignee: None,
            team: None,
            project: None,
            labels: vec![],
            priority: None,
            priority_label: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            url: "https://linear.app/test/issue/ENG-456".to_string(),
        };

        let formatter = TableFormatter::new(false);
        let result = formatter.format_detailed_issue(&minimal_issue).unwrap();

        // Check that it handles missing fields gracefully
        assert!(result.contains("ENG-456: Simple issue"));
        assert!(result.contains("Assignee:   Unassigned"));
        assert!(result.contains("Team:       No team"));
        assert!(result.contains("Priority:   None"));
        assert!(result.contains("Labels: None"));
        assert!(!result.contains("Project:"));
        assert!(!result.contains("Description:"));
    }
}
