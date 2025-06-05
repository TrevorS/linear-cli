// ABOUTME: Frontmatter parsing for markdown files used in issue creation
// ABOUTME: Handles YAML frontmatter extraction and validation for Linear CLI

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Represents the frontmatter fields that can be specified in a markdown file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IssueFrontmatter {
    /// Issue title (required)
    pub title: String,
    /// Team key or UUID
    pub team: Option<String>,
    /// Assignee (can be "me", user ID, or email)
    pub assignee: Option<String>,
    /// Priority level (1-4: Urgent, High, Normal, Low)
    pub priority: Option<i64>,
    /// Labels for the issue
    pub labels: Option<Vec<String>>,
    /// Project name or ID
    pub project: Option<String>,
}

/// Represents a parsed markdown file with frontmatter and content
#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownFile {
    /// Parsed frontmatter metadata
    pub frontmatter: IssueFrontmatter,
    /// Markdown content (without frontmatter)
    pub content: String,
}

/// Parse a markdown file from a file path
pub fn parse_markdown_file<P: AsRef<Path>>(file_path: P) -> Result<MarkdownFile> {
    let content = fs::read_to_string(&file_path).map_err(|e| {
        anyhow!(
            "Failed to read file '{}': {}",
            file_path.as_ref().display(),
            e
        )
    })?;

    parse_markdown_content(&content)
}

/// Parse markdown content string with frontmatter
pub fn parse_markdown_content(content: &str) -> Result<MarkdownFile> {
    // Check if content starts with frontmatter delimiter
    if !content.starts_with("---\n") && !content.starts_with("---\r\n") {
        return Err(anyhow!(
            "No frontmatter found. Markdown files must start with '---' delimiter"
        ));
    }

    // Find the end of frontmatter
    let content_after_start = if let Some(stripped) = content.strip_prefix("---\r\n") {
        stripped
    } else {
        &content[4..] // Skip "---\n"
    };

    let end_delimiter_pos = content_after_start
        .find("\n---\n")
        .or_else(|| content_after_start.find("\r\n---\r\n"))
        .or_else(|| content_after_start.find("\n---\r\n"))
        .or_else(|| content_after_start.find("\r\n---\n"))
        .ok_or_else(|| anyhow!("Frontmatter closing delimiter '---' not found"))?;

    // Extract frontmatter YAML
    let frontmatter_yaml = &content_after_start[..end_delimiter_pos];

    // Extract markdown content (everything after the closing delimiter)
    let delimiter_end = if content_after_start[end_delimiter_pos..].starts_with("\r\n---\r\n") {
        end_delimiter_pos + 7
    } else if content_after_start[end_delimiter_pos..].starts_with("\n---\r\n")
        || content_after_start[end_delimiter_pos..].starts_with("\r\n---\n")
    {
        end_delimiter_pos + 6
    } else {
        end_delimiter_pos + 5 // "\n---\n"
    };

    let markdown_content = content_after_start[delimiter_end..]
        .trim_start()
        .to_string();

    // Parse YAML frontmatter
    let frontmatter: IssueFrontmatter = serde_yaml::from_str(frontmatter_yaml)
        .map_err(|e| anyhow!("Failed to parse frontmatter YAML: {}", e))?;

    // Validate required fields
    if frontmatter.title.trim().is_empty() {
        return Err(anyhow!("Title is required in frontmatter"));
    }

    // Validate priority if specified
    if let Some(priority) = frontmatter.priority {
        if !(1..=4).contains(&priority) {
            return Err(anyhow!(
                "Priority must be between 1 and 4 (1=Urgent, 2=High, 3=Normal, 4=Low), got: {}",
                priority
            ));
        }
    }

    Ok(MarkdownFile {
        frontmatter,
        content: markdown_content,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_basic_frontmatter() {
        let content = r#"---
title: "Test Issue"
team: ENG
assignee: me
priority: 2
---

# Issue Description

This is a test issue with basic frontmatter.
"#;

        let result = parse_markdown_content(content).unwrap();

        assert_eq!(result.frontmatter.title, "Test Issue");
        assert_eq!(result.frontmatter.team, Some("ENG".to_string()));
        assert_eq!(result.frontmatter.assignee, Some("me".to_string()));
        assert_eq!(result.frontmatter.priority, Some(2));
        assert_eq!(result.frontmatter.labels, None);
        assert_eq!(result.frontmatter.project, None);
        assert_eq!(
            result.content,
            "# Issue Description\n\nThis is a test issue with basic frontmatter.\n"
        );
    }

    #[test]
    fn test_parse_frontmatter_with_labels() {
        let content = r#"---
title: "Bug Fix"
team: ENG
labels:
  - bug
  - high-priority
  - authentication
project: "Web App Stability"
---

Found a race condition in authentication.
"#;

        let result = parse_markdown_content(content).unwrap();

        assert_eq!(result.frontmatter.title, "Bug Fix");
        assert_eq!(result.frontmatter.team, Some("ENG".to_string()));
        assert_eq!(result.frontmatter.assignee, None);
        assert_eq!(result.frontmatter.priority, None);
        assert_eq!(
            result.frontmatter.labels,
            Some(vec![
                "bug".to_string(),
                "high-priority".to_string(),
                "authentication".to_string()
            ])
        );
        assert_eq!(
            result.frontmatter.project,
            Some("Web App Stability".to_string())
        );
        assert_eq!(
            result.content,
            "Found a race condition in authentication.\n"
        );
    }

    #[test]
    fn test_parse_minimal_frontmatter() {
        let content = r#"---
title: "Minimal Issue"
---

Just a title.
"#;

        let result = parse_markdown_content(content).unwrap();

        assert_eq!(result.frontmatter.title, "Minimal Issue");
        assert_eq!(result.frontmatter.team, None);
        assert_eq!(result.frontmatter.assignee, None);
        assert_eq!(result.frontmatter.priority, None);
        assert_eq!(result.frontmatter.labels, None);
        assert_eq!(result.frontmatter.project, None);
        assert_eq!(result.content, "Just a title.\n");
    }

    #[test]
    fn test_parse_empty_markdown_content() {
        let content = r#"---
title: "Empty Content"
team: ENG
---
"#;

        let result = parse_markdown_content(content).unwrap();

        assert_eq!(result.frontmatter.title, "Empty Content");
        assert_eq!(result.frontmatter.team, Some("ENG".to_string()));
        assert_eq!(result.content, "");
    }

    #[test]
    fn test_parse_frontmatter_with_crlf() {
        let content = "---\r\ntitle: \"Windows Style\"\r\nteam: ENG\r\n---\r\n\r\nContent with CRLF line endings.";

        let result = parse_markdown_content(content).unwrap();

        assert_eq!(result.frontmatter.title, "Windows Style");
        assert_eq!(result.frontmatter.team, Some("ENG".to_string()));
        assert_eq!(result.content, "Content with CRLF line endings.");
    }

    #[test]
    fn test_error_no_frontmatter() {
        let content = "# Just a markdown file\n\nNo frontmatter here.";

        let result = parse_markdown_content(content);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No frontmatter found")
        );
    }

    #[test]
    fn test_error_missing_closing_delimiter() {
        let content = r#"---
title: "Missing closing delimiter"
team: ENG

# Content without closing ---
"#;

        let result = parse_markdown_content(content);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("closing delimiter")
        );
    }

    #[test]
    fn test_error_invalid_yaml() {
        let content = r#"---
title: "Invalid YAML"
team: ENG
invalid: [unclosed array
---

Content here.
"#;

        let result = parse_markdown_content(content);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to parse frontmatter YAML")
        );
    }

    #[test]
    fn test_error_missing_title() {
        let content = r#"---
team: ENG
assignee: me
---

Content without title.
"#;

        let result = parse_markdown_content(content);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("missing field `title`")
        );
    }

    #[test]
    fn test_error_empty_title() {
        let content = r#"---
title: ""
team: ENG
---

Content with empty title.
"#;

        let result = parse_markdown_content(content);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Title is required")
        );
    }

    #[test]
    fn test_error_whitespace_only_title() {
        let content = r#"---
title: "   "
team: ENG
---

Content with whitespace-only title.
"#;

        let result = parse_markdown_content(content);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Title is required")
        );
    }

    #[test]
    fn test_error_invalid_priority() {
        let content = r#"---
title: "Invalid Priority"
team: ENG
priority: 5
---

Priority should be 1-4.
"#;

        let result = parse_markdown_content(content);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Priority must be between 1 and 4")
        );
    }

    #[test]
    fn test_error_negative_priority() {
        let content = r#"---
title: "Negative Priority"
team: ENG
priority: -1
---

Negative priority should be invalid.
"#;

        let result = parse_markdown_content(content);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Priority must be between 1 and 4")
        );
    }

    #[test]
    fn test_parse_all_priority_levels() {
        for priority in 1..=4 {
            let content = format!(
                r#"---
title: "Priority {}"
team: ENG
priority: {}
---

Testing priority level {}.
"#,
                priority, priority, priority
            );

            let result = parse_markdown_content(&content).unwrap();
            assert_eq!(result.frontmatter.priority, Some(priority));
        }
    }

    #[test]
    fn test_parse_file_from_disk() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = r#"---
title: "File Test"
team: ENG
assignee: me
---

# Test Issue

This issue was loaded from a file.
"#;

        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = parse_markdown_file(temp_file.path()).unwrap();

        assert_eq!(result.frontmatter.title, "File Test");
        assert_eq!(result.frontmatter.team, Some("ENG".to_string()));
        assert_eq!(result.frontmatter.assignee, Some("me".to_string()));
        assert_eq!(
            result.content,
            "# Test Issue\n\nThis issue was loaded from a file.\n"
        );
    }

    #[test]
    fn test_error_file_not_found() {
        let result = parse_markdown_file("/nonexistent/file.md");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to read file")
        );
    }

    #[test]
    fn test_frontmatter_serialization() {
        let frontmatter = IssueFrontmatter {
            title: "Test Serialization".to_string(),
            team: Some("ENG".to_string()),
            assignee: Some("me".to_string()),
            priority: Some(2),
            labels: Some(vec!["test".to_string(), "serialization".to_string()]),
            project: Some("Test Project".to_string()),
        };

        // Test that we can serialize and deserialize
        let yaml = serde_yaml::to_string(&frontmatter).unwrap();
        let deserialized: IssueFrontmatter = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(frontmatter, deserialized);
    }

    #[test]
    fn test_markdown_file_equality() {
        let file1 = MarkdownFile {
            frontmatter: IssueFrontmatter {
                title: "Test".to_string(),
                team: Some("ENG".to_string()),
                assignee: None,
                priority: None,
                labels: None,
                project: None,
            },
            content: "Test content".to_string(),
        };

        let file2 = MarkdownFile {
            frontmatter: IssueFrontmatter {
                title: "Test".to_string(),
                team: Some("ENG".to_string()),
                assignee: None,
                priority: None,
                labels: None,
                project: None,
            },
            content: "Test content".to_string(),
        };

        assert_eq!(file1, file2);
    }

    #[test]
    fn test_complex_markdown_content() {
        let content = r#"---
title: "Complex Issue with Code"
team: ENG
priority: 1
labels:
  - bug
  - authentication
  - urgent
---

# Authentication Race Condition

## Description

Users are experiencing intermittent login failures when multiple requests are made simultaneously.

## Steps to Reproduce

1. Open multiple browser tabs
2. Attempt to log in simultaneously from each tab
3. Observe that some requests fail with "Invalid session" error

## Code Sample

```rust
// Problematic code in session_manager.rs:45
if !self.sessions.contains_key(&user_id) {
    self.sessions.insert(user_id, new_session); // Race condition here
}
```

## Acceptance Criteria

- [ ] Multiple simultaneous login attempts work correctly
- [ ] No "Session conflict detected" errors occur
- [ ] Existing sessions are properly preserved

**Priority**: High - affecting user onboarding
"#;

        let result = parse_markdown_content(content).unwrap();

        assert_eq!(result.frontmatter.title, "Complex Issue with Code");
        assert_eq!(result.frontmatter.team, Some("ENG".to_string()));
        assert_eq!(result.frontmatter.priority, Some(1));
        assert_eq!(
            result.frontmatter.labels,
            Some(vec![
                "bug".to_string(),
                "authentication".to_string(),
                "urgent".to_string()
            ])
        );

        // Verify the content preserves formatting and code blocks
        assert!(result.content.contains("# Authentication Race Condition"));
        assert!(result.content.contains("```rust"));
        assert!(result.content.contains("session_manager.rs:45"));
        assert!(result.content.contains("- [ ] Multiple simultaneous"));
    }
}
