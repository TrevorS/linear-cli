// ABOUTME: This module handles output formatting for the Linear CLI
// ABOUTME: It provides different formatters like table formatting with color support

use linear_sdk::{DetailedIssue, Issue, Result};
use owo_colors::OwoColorize;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use std::io::Write;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::types::IssueStatus;

pub trait OutputFormat {
    fn format_issues(&self, issues: &[Issue]) -> Result<String>;
    fn format_detailed_issue(&self, issue: &DetailedIssue) -> Result<String>;
    fn format_detailed_issue_rich(
        &self,
        issue: &DetailedIssue,
        is_interactive: bool,
    ) -> Result<String>;
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
            if self.use_color {
                return "None".dimmed().to_string();
            } else {
                return "None".to_string();
            }
        }

        labels
            .iter()
            .enumerate()
            .map(|(i, l)| {
                if self.use_color {
                    // Use different colors for visual variety
                    let colored_name = match i % 4 {
                        0 => l.name.cyan().to_string(),
                        1 => l.name.purple().to_string(),
                        2 => l.name.green().to_string(),
                        _ => l.name.yellow().to_string(),
                    };
                    format!("●{}", colored_name.bold())
                } else {
                    format!("● {}", l.name)
                }
            })
            .collect::<Vec<_>>()
            .join("  ")
    }

    fn format_datetime(&self, datetime: &str) -> String {
        if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(datetime) {
            parsed.format("%Y-%m-%d %H:%M").to_string()
        } else {
            datetime.to_string()
        }
    }

    fn supports_osc8() -> bool {
        // Check for specific terminal programs that support OSC-8
        if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
            match term_program.as_str() {
                "iTerm.app" | "kitty" | "ghostty" | "WezTerm" => return true,
                _ => {}
            }
        }

        // Check for Windows Terminal
        if std::env::var("WT_SESSION").is_ok() {
            return true;
        }

        // Check for VTE-based terminals (GNOME Terminal, etc.)
        if std::env::var("VTE_VERSION").is_ok() {
            return true;
        }

        // Check for Alacritty
        if let Ok(term) = std::env::var("TERM") {
            if term.contains("alacritty") {
                return true;
            }
        }

        false
    }

    fn format_hyperlink(&self, url: &str, text: &str, supports_osc8: bool) -> String {
        if supports_osc8 && self.use_color {
            // OSC-8 hyperlink format: ESC]8;;URL\TEXT\ESC]8;;\
            format!(
                "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
                url,
                text.cyan().underline()
            )
        } else if self.use_color {
            // Fallback to colored text without hyperlink
            format!("{}", text.cyan().underline())
        } else {
            // Plain text fallback
            text.to_string()
        }
    }

    fn render_markdown_to_terminal(&self, markdown: &str) -> anyhow::Result<String> {
        let mut output = Vec::new();
        let parser = Parser::new(markdown);

        let mut current_text = String::new();
        let mut in_code_block = false;
        let mut is_heading = false;
        let mut heading_level = None;
        let mut in_link = false;
        let mut current_link_url = String::new();
        let mut in_strong = false;
        let mut in_emphasis = false;

        let supports_osc8 = Self::supports_osc8();

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    is_heading = true;
                    heading_level = Some(level);
                    current_text.clear();
                    writeln!(output)?; // Add newline before heading
                }
                Event::End(TagEnd::Heading(_)) => {
                    if is_heading {
                        let heading_text = if self.use_color {
                            match heading_level {
                                Some(pulldown_cmark::HeadingLevel::H1) => {
                                    current_text.bold().underline().to_string()
                                }
                                _ => current_text.bold().to_string(),
                            }
                        } else {
                            current_text.clone()
                        };
                        writeln!(output, "{}", heading_text)?;
                        writeln!(output)?; // Add newline after heading
                        current_text.clear();
                        is_heading = false;
                        heading_level = None;
                    }
                }
                Event::Start(Tag::CodeBlock(_)) => {
                    in_code_block = true;
                    writeln!(output)?; // Add newline before code block
                }
                Event::End(TagEnd::CodeBlock) => {
                    if in_code_block {
                        // Process code content
                        for line in current_text.lines() {
                            if self.use_color {
                                writeln!(output, "{}", line.on_black().white())?;
                            } else {
                                writeln!(output, "{}", line)?;
                            }
                        }
                        writeln!(output)?; // Add newline after code block
                        current_text.clear();
                        in_code_block = false;
                    }
                }
                Event::Text(text) => {
                    if in_code_block || in_link {
                        current_text.push_str(&text);
                    } else if is_heading {
                        // For headings, build formatted content in current_text
                        if in_strong && self.use_color {
                            current_text.push_str(&text.bold().to_string());
                        } else if in_emphasis && self.use_color {
                            current_text.push_str(&text.italic().to_string());
                        } else {
                            current_text.push_str(&text);
                        }
                    } else if in_strong && self.use_color {
                        write!(output, "{}", text.bold())?;
                    } else if in_emphasis && self.use_color {
                        write!(output, "{}", text.italic())?;
                    } else {
                        write!(output, "{}", text)?;
                    }
                }
                Event::Code(code) => {
                    if is_heading {
                        // For headings, append formatted code to current_text
                        if self.use_color {
                            current_text.push_str(&code.on_black().white().to_string());
                        } else {
                            current_text.push_str(&format!("`{}`", code));
                        }
                    } else if self.use_color {
                        write!(output, "{}", code.on_black().white())?;
                    } else {
                        write!(output, "`{}`", code)?;
                    }
                }
                Event::Start(Tag::Emphasis) => {
                    in_emphasis = true;
                }
                Event::End(TagEnd::Emphasis) => {
                    in_emphasis = false;
                }
                Event::Start(Tag::Strong) => {
                    in_strong = true;
                }
                Event::End(TagEnd::Strong) => {
                    in_strong = false;
                }
                Event::Start(Tag::List(_)) => {
                    writeln!(output)?;
                }
                Event::End(TagEnd::List(_)) => {
                    writeln!(output)?;
                }
                Event::Start(Tag::Item) => {
                    write!(output, "• ")?;
                }
                Event::End(TagEnd::Item) => {
                    writeln!(output)?;
                }
                Event::Start(Tag::BlockQuote(_)) => {
                    if self.use_color {
                        write!(output, "{}", "│ ".dimmed())?;
                    } else {
                        write!(output, "│ ")?;
                    }
                }
                Event::End(TagEnd::BlockQuote(_)) => {
                    writeln!(output)?;
                }
                Event::Start(Tag::Paragraph) => {
                    writeln!(output)?;
                }
                Event::End(TagEnd::Paragraph) => {
                    writeln!(output)?;
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    // For links, we'll capture the text and check for media
                    in_link = true;
                    current_link_url = dest_url.to_string();
                    current_text.clear();
                }
                Event::End(TagEnd::Link) => {
                    // Check if this is a media link
                    if current_link_url.contains("uploads.linear.app") {
                        // This is embedded media - show with clickable URL
                        if self.use_color {
                            write!(
                                output,
                                "{}{}{}\n{}{}{}",
                                "📎 ".white(),
                                "Media: ".cyan().bold(),
                                current_text.white(),
                                "   ".dimmed(),
                                "🔗 ".dimmed(),
                                self.format_hyperlink(
                                    &current_link_url,
                                    &current_link_url,
                                    supports_osc8
                                )
                            )?;
                        } else {
                            write!(
                                output,
                                "📎 Media: {}\n   🔗 {}",
                                current_text, current_link_url
                            )?;
                        }
                    } else {
                        // Regular link - show the link text as a clickable hyperlink
                        write!(
                            output,
                            "{}",
                            self.format_hyperlink(&current_link_url, &current_text, supports_osc8)
                        )?;
                    }
                    current_text.clear();
                    current_link_url.clear();
                    in_link = false;
                }
                Event::SoftBreak => {
                    write!(output, " ")?;
                }
                Event::HardBreak => {
                    writeln!(output)?;
                }
                _ => {
                    // Handle other events with basic text output
                }
            }
        }

        Ok(String::from_utf8(output)?)
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

    fn format_detailed_issue_rich(
        &self,
        issue: &DetailedIssue,
        _is_interactive: bool,
    ) -> Result<String> {
        // JSON format doesn't change based on interactivity
        self.format_detailed_issue(issue)
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
            if self.use_color {
                output.push(format!("{}{}", "📝 ".white(), "Description:".cyan()));
            } else {
                output.push("📝 Description:".to_string());
            }
            output.push(description.clone());
        }

        // Metadata section with enhanced formatting
        output.push(String::new());
        if self.use_color {
            output.push(format!(
                "{}{}{}",
                "🏷️ ".white(),
                "Labels: ".cyan(),
                self.format_labels(&issue.labels)
            ));
        } else {
            output.push(format!("🏷️ Labels: {}", self.format_labels(&issue.labels)));
        }

        output.push(String::new());
        if self.use_color {
            output.push(format!(
                "{}{} {}    {} {}",
                "📅 ".white(),
                "Created:".cyan(),
                self.format_datetime(&issue.created_at).white(),
                "Updated:".cyan(),
                self.format_datetime(&issue.updated_at).white()
            ));
        } else {
            output.push(format!(
                "📅 Created: {}    Updated: {}",
                self.format_datetime(&issue.created_at),
                self.format_datetime(&issue.updated_at)
            ));
        }

        output.push(String::new());
        let supports_osc8 = Self::supports_osc8();
        if self.use_color {
            output.push(format!(
                "{}{}\n   {}",
                "🔗 ".white(),
                "View in Linear:".cyan(),
                self.format_hyperlink(&issue.url, &issue.url, supports_osc8)
            ));
        } else {
            output.push(format!("🔗 View in Linear:\n   {}", issue.url));
        }

        Ok(output.join("\n"))
    }

    fn format_detailed_issue_rich(
        &self,
        issue: &DetailedIssue,
        is_interactive: bool,
    ) -> Result<String> {
        // If not interactive (piped/redirected), use raw markdown
        if !is_interactive {
            return self.format_detailed_issue(issue);
        }

        // For interactive terminals, render markdown if present in description
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

        // Enhanced markdown rendering for description in interactive mode
        if let Some(description) = &issue.description {
            output.push(String::new());
            if self.use_color {
                output.push(format!("{}{}", "📝 ".white(), "Description:".cyan()));
            } else {
                output.push("📝 Description:".to_string());
            }

            // Render markdown to terminal if interactive
            match self.render_markdown_to_terminal(description) {
                Ok(rendered) => output.push(rendered),
                Err(_) => {
                    // Fallback to raw markdown if rendering fails
                    output.push(description.clone());
                }
            }
        }

        // Metadata section with enhanced formatting
        output.push(String::new());
        if self.use_color {
            output.push(format!(
                "{}{}{}",
                "🏷️ ".white(),
                "Labels: ".cyan(),
                self.format_labels(&issue.labels)
            ));
        } else {
            output.push(format!("🏷️ Labels: {}", self.format_labels(&issue.labels)));
        }

        output.push(String::new());
        if self.use_color {
            output.push(format!(
                "{}{} {}    {} {}",
                "📅 ".white(),
                "Created:".cyan(),
                self.format_datetime(&issue.created_at).white(),
                "Updated:".cyan(),
                self.format_datetime(&issue.updated_at).white()
            ));
        } else {
            output.push(format!(
                "📅 Created: {}    Updated: {}",
                self.format_datetime(&issue.created_at),
                self.format_datetime(&issue.updated_at)
            ));
        }

        output.push(String::new());
        let supports_osc8 = Self::supports_osc8();
        if self.use_color {
            output.push(format!(
                "{}{}\n   {}",
                "🔗 ".white(),
                "View in Linear:".cyan(),
                self.format_hyperlink(&issue.url, &issue.url, supports_osc8)
            ));
        } else {
            output.push(format!("🔗 View in Linear:\n   {}", issue.url));
        }

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
        assert!(result.contains("🏷️ Labels: ● bug  ● authentication"));
        assert!(result.contains("Created:"));
        assert!(result.contains("Updated:"));
        assert!(result.contains("View in Linear:"));
        assert!(result.contains("https://linear.app/test/issue/ENG-123"));
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

    fn create_markdown_test_issue() -> linear_sdk::DetailedIssue {
        use linear_sdk::*;

        DetailedIssue {
            id: "issue-id-markdown".to_string(),
            identifier: "ENG-456".to_string(),
            title: "Test markdown formatting".to_string(),
            description: Some(
                r#"# Markdown Test Issue

This is a test issue with **markdown content** to verify rich formatting.

## Features to test:
- [x] Headers (H1-H6)
- [ ] Code blocks with syntax highlighting
- [ ] Lists (ordered and unordered)
- [ ] Links and emphasis

### Code Example:

```rust
fn main() {
    println!("Hello, World!");
}
```

### Links and Text Formatting:

Check out [Linear](https://linear.app) for more details.

Some text with *italics* and **bold** formatting.

> This is a blockquote
> spanning multiple lines

Final paragraph with normal text."#
                    .to_string(),
            ),
            state: IssueState {
                name: "In Progress".to_string(),
                type_: "started".to_string(),
            },
            assignee: Some(IssueAssignee {
                name: "Test User".to_string(),
                email: "test@example.com".to_string(),
            }),
            team: Some(IssueTeam {
                key: "ENG".to_string(),
                name: "Engineering".to_string(),
            }),
            project: Some(IssueProject {
                name: "Markdown Test".to_string(),
            }),
            labels: vec![IssueLabel {
                name: "documentation".to_string(),
                color: "#0066CC".to_string(),
            }],
            priority: Some(3),
            priority_label: Some("Normal".to_string()),
            created_at: "2024-01-15T10:30:00Z".to_string(),
            updated_at: "2024-01-16T14:45:00Z".to_string(),
            url: "https://linear.app/test/issue/ENG-456".to_string(),
        }
    }

    #[test]
    fn test_markdown_formatting_with_tty_detection() {
        let formatter = TableFormatter::new(false);
        let issue = create_markdown_test_issue();

        let result = formatter.format_detailed_issue(&issue).unwrap();

        // This test should currently pass (raw markdown), but will be updated
        // to test rich formatting when TTY detection is implemented
        assert!(result.contains("# Markdown Test Issue"));
        assert!(result.contains("**markdown content**"));
        assert!(result.contains("```rust"));
        assert!(result.contains("fn main()"));
        assert!(result.contains("[Linear](https://linear.app)"));
    }

    #[test]
    fn test_rich_markdown_formatting_interactive() {
        let formatter = TableFormatter::new(true);
        let issue = create_markdown_test_issue();

        // TODO: This test will fail until we implement markdown parsing
        // When implemented, this should render rich formatted output
        let result = formatter.format_detailed_issue_rich(&issue, true).unwrap();

        // Test that headers are properly formatted (will fail initially)
        assert!(result.contains("Markdown Test Issue")); // H1 should be rendered without #

        // Test that code blocks are syntax highlighted (will fail initially)
        assert!(!result.contains("```rust")); // Raw markdown should be replaced
        assert!(result.contains("fn main()")); // Code content should remain

        // Test that emphasis is rendered (will fail initially)
        assert!(!result.contains("**markdown content**")); // Raw markdown should be replaced

        // Test that links are rendered (will fail initially)
        assert!(!result.contains("[Linear](https://linear.app)")); // Raw markdown should be replaced
        assert!(result.contains("Linear")); // Link text should remain
    }

    #[test]
    fn test_raw_markdown_when_piped() {
        let formatter = TableFormatter::new(false);
        let issue = create_markdown_test_issue();

        // When output is piped (not interactive), should preserve raw markdown
        let result = formatter.format_detailed_issue_rich(&issue, false).unwrap();

        // Should contain raw markdown (same as current behavior)
        assert!(result.contains("# Markdown Test Issue"));
        assert!(result.contains("**markdown content**"));
        assert!(result.contains("```rust"));
        assert!(result.contains("[Linear](https://linear.app)"));
    }

    #[test]
    fn test_osc8_terminal_detection() {
        // Test various terminal environments

        // Save original env vars
        let original_term_program = std::env::var("TERM_PROGRAM").ok();
        let original_wt_session = std::env::var("WT_SESSION").ok();
        let original_vte_version = std::env::var("VTE_VERSION").ok();
        let original_term = std::env::var("TERM").ok();

        unsafe {
            // Test iTerm2
            std::env::set_var("TERM_PROGRAM", "iTerm.app");
            assert!(TableFormatter::supports_osc8());

            // Test kitty
            std::env::set_var("TERM_PROGRAM", "kitty");
            assert!(TableFormatter::supports_osc8());

            // Test Ghostty
            std::env::set_var("TERM_PROGRAM", "ghostty");
            assert!(TableFormatter::supports_osc8());

            // Test WezTerm
            std::env::set_var("TERM_PROGRAM", "WezTerm");
            assert!(TableFormatter::supports_osc8());

            // Test unknown terminal
            std::env::set_var("TERM_PROGRAM", "unknown");
            assert!(!TableFormatter::supports_osc8());
            std::env::remove_var("TERM_PROGRAM");

            // Test Windows Terminal
            std::env::set_var("WT_SESSION", "some-session-id");
            assert!(TableFormatter::supports_osc8());
            std::env::remove_var("WT_SESSION");

            // Test VTE-based terminal
            std::env::set_var("VTE_VERSION", "6200");
            assert!(TableFormatter::supports_osc8());
            std::env::remove_var("VTE_VERSION");

            // Test Alacritty
            std::env::set_var("TERM", "alacritty");
            assert!(TableFormatter::supports_osc8());

            // Test terminal without alacritty in name
            std::env::set_var("TERM", "xterm-256color");
            assert!(!TableFormatter::supports_osc8());

            // Restore original env vars
            if let Some(val) = original_term_program {
                std::env::set_var("TERM_PROGRAM", val);
            } else {
                std::env::remove_var("TERM_PROGRAM");
            }
            if let Some(val) = original_wt_session {
                std::env::set_var("WT_SESSION", val);
            } else {
                std::env::remove_var("WT_SESSION");
            }
            if let Some(val) = original_vte_version {
                std::env::set_var("VTE_VERSION", val);
            } else {
                std::env::remove_var("VTE_VERSION");
            }
            if let Some(val) = original_term {
                std::env::set_var("TERM", val);
            } else {
                std::env::remove_var("TERM");
            }
        }
    }

    #[test]
    fn test_format_hyperlink() {
        let formatter = TableFormatter::new(true);
        let url = "https://linear.app/test/issue/ENG-123";
        let text = "ENG-123";

        // Test with OSC-8 support
        let result = formatter.format_hyperlink(url, text, true);
        assert!(result.contains("\x1b]8;;"));
        assert!(result.contains(url));
        assert!(result.contains(text));
        assert!(result.contains("\x1b\\"));

        // Test without OSC-8 support (colored)
        let result = formatter.format_hyperlink(url, text, false);
        assert!(!result.contains("\x1b]8;;"));
        assert!(result.contains(text));

        // Test without color
        let formatter_no_color = TableFormatter::new(false);
        let result = formatter_no_color.format_hyperlink(url, text, true);
        assert_eq!(result, text);

        let result = formatter_no_color.format_hyperlink(url, text, false);
        assert_eq!(result, text);
    }

    #[test]
    fn test_markdown_links_with_osc8() {
        let formatter = TableFormatter::new(true);

        // Save original env
        let original_term_program = std::env::var("TERM_PROGRAM").ok();

        unsafe {
            // Set terminal to support OSC-8
            std::env::set_var("TERM_PROGRAM", "ghostty");

            let markdown = "Check out [Linear](https://linear.app) for more details.";
            let result = formatter.render_markdown_to_terminal(markdown).unwrap();

            // Should contain OSC-8 sequence
            assert!(result.contains("\x1b]8;;https://linear.app\x1b\\"));
            assert!(result.contains("Linear"));

            // Test media link - images are processed as text, not links in markdown
            let markdown_media =
                "Check out this [media](https://uploads.linear.app/test.png) file.";
            let result = formatter
                .render_markdown_to_terminal(markdown_media)
                .unwrap();
            // Media links should show with the media icon and clickable URL
            assert!(result.contains("\x1b]8;;https://uploads.linear.app/test.png\x1b\\"));
            assert!(result.contains("📎"));
            assert!(result.contains("Media:"));

            // Restore env
            if let Some(val) = original_term_program {
                std::env::set_var("TERM_PROGRAM", val);
            } else {
                std::env::remove_var("TERM_PROGRAM");
            }
        }
    }

    #[test]
    fn test_view_in_linear_with_osc8() {
        let formatter = TableFormatter::new(true);
        let issue = create_test_detailed_issue();

        // Save original env
        let original_term_program = std::env::var("TERM_PROGRAM").ok();

        unsafe {
            // Set terminal to support OSC-8
            std::env::set_var("TERM_PROGRAM", "iTerm.app");

            let result = formatter.format_detailed_issue(&issue).unwrap();

            // Should contain OSC-8 sequence for the Linear URL
            assert!(result.contains("\x1b]8;;https://linear.app/test/issue/ENG-123\x1b\\"));
            assert!(result.contains("View in Linear:"));

            // Restore env
            if let Some(val) = original_term_program {
                std::env::set_var("TERM_PROGRAM", val);
            } else {
                std::env::remove_var("TERM_PROGRAM");
            }
        }
    }

    #[test]
    fn test_osc8_disabled_when_no_color() {
        let formatter = TableFormatter::new(false); // No color = no OSC-8
        let issue = create_test_detailed_issue();

        // Save original env
        let original_term_program = std::env::var("TERM_PROGRAM").ok();

        unsafe {
            // Set terminal to support OSC-8
            std::env::set_var("TERM_PROGRAM", "ghostty");

            let result = formatter.format_detailed_issue(&issue).unwrap();

            // Should NOT contain OSC-8 sequences when color is disabled
            assert!(!result.contains("\x1b]8;;"));
            assert!(result.contains("https://linear.app/test/issue/ENG-123"));

            // Restore env
            if let Some(val) = original_term_program {
                std::env::set_var("TERM_PROGRAM", val);
            } else {
                std::env::remove_var("TERM_PROGRAM");
            }
        }
    }

    #[test]
    fn test_osc8_disabled_in_unsupported_terminal() {
        let formatter = TableFormatter::new(true);
        let issue = create_test_detailed_issue();

        // Save original env
        let original_term_program = std::env::var("TERM_PROGRAM").ok();
        let original_term = std::env::var("TERM").ok();
        let original_wt_session = std::env::var("WT_SESSION").ok();
        let original_vte_version = std::env::var("VTE_VERSION").ok();

        unsafe {
            // Set terminal to not support OSC-8
            std::env::remove_var("TERM_PROGRAM");
            std::env::remove_var("WT_SESSION");
            std::env::remove_var("VTE_VERSION");
            std::env::set_var("TERM", "xterm");

            let result = formatter.format_detailed_issue(&issue).unwrap();

            // Should NOT contain OSC-8 sequences in unsupported terminal
            assert!(!result.contains("\x1b]8;;"));
            // But should still have colored/underlined URLs
            assert!(result.contains("https://linear.app/test/issue/ENG-123"));

            // Restore env
            if let Some(val) = original_term_program {
                std::env::set_var("TERM_PROGRAM", val);
            } else {
                std::env::remove_var("TERM_PROGRAM");
            }
            if let Some(val) = original_wt_session {
                std::env::set_var("WT_SESSION", val);
            } else {
                std::env::remove_var("WT_SESSION");
            }
            if let Some(val) = original_vte_version {
                std::env::set_var("VTE_VERSION", val);
            } else {
                std::env::remove_var("VTE_VERSION");
            }
            if let Some(val) = original_term {
                std::env::set_var("TERM", val);
            } else {
                std::env::remove_var("TERM");
            }
        }
    }

    #[test]
    fn test_markdown_edge_cases() {
        use linear_sdk::*;

        let edge_case_issue = DetailedIssue {
            id: "issue-edge".to_string(),
            identifier: "ENG-999".to_string(),
            title: "Edge case test".to_string(),
            description: Some(
                r#"
# Empty lines and special characters

Test with:
- Unicode: 🚀 ✨ 💻
- Special chars: < > & " '
- Empty code block:
```

```

- Mixed formatting: **bold _italic_** text
"#
                .to_string(),
            ),
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
            url: "https://linear.app/test/issue/ENG-999".to_string(),
        };

        let formatter = TableFormatter::new(true);

        // TODO: This will fail until markdown rendering is implemented
        let result = formatter
            .format_detailed_issue_rich(&edge_case_issue, true)
            .unwrap();

        // Test that special characters are handled properly
        assert!(result.contains("🚀 ✨ 💻"));
        assert!(result.contains("< > & \" '"));
    }
}
