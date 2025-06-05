// ABOUTME: Issue templates for common types and workflows
// ABOUTME: Provides pre-filled templates to speed up issue creation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTemplate {
    pub name: String,
    pub title_template: String,
    pub description_template: String,
    pub suggested_priority: Option<i64>,
    pub suggested_team: Option<String>,
    pub tags: Vec<String>,
}

pub struct TemplateManager {
    templates: HashMap<String, IssueTemplate>,
}

impl TemplateManager {
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        // Bug report template
        templates.insert(
            "bug".to_string(),
            IssueTemplate {
                name: "Bug Report".to_string(),
                title_template: "[Bug] {title}".to_string(),
                description_template: r#"## Bug Description
{description}

## Steps to Reproduce
1.
2.
3.

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Environment
- Browser/OS:
- Version:

## Additional Context
Any other relevant information"#
                    .to_string(),
                suggested_priority: Some(2), // High priority
                suggested_team: None,
                tags: vec!["bug".to_string()],
            },
        );

        // Feature request template
        templates.insert(
            "feature".to_string(),
            IssueTemplate {
                name: "Feature Request".to_string(),
                title_template: "[Feature] {title}".to_string(),
                description_template: r#"## Feature Description
{description}

## User Story
As a [user type], I want [goal] so that [benefit].

## Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3

## Design Notes
Any design considerations or mockups

## Technical Notes
Implementation details or considerations"#
                    .to_string(),
                suggested_priority: Some(3), // Normal priority
                suggested_team: None,
                tags: vec!["feature".to_string(), "enhancement".to_string()],
            },
        );

        // Task template
        templates.insert(
            "task".to_string(),
            IssueTemplate {
                name: "Task".to_string(),
                title_template: "{title}".to_string(),
                description_template: r#"## Task Description
{description}

## Checklist
- [ ] Task item 1
- [ ] Task item 2
- [ ] Task item 3

## Definition of Done
What constitutes completion of this task

## Dependencies
Any blocking issues or dependencies"#
                    .to_string(),
                suggested_priority: Some(3), // Normal priority
                suggested_team: None,
                tags: vec!["task".to_string()],
            },
        );

        // Improvement/refactoring template
        templates.insert(
            "improvement".to_string(),
            IssueTemplate {
                name: "Improvement".to_string(),
                title_template: "[Improvement] {title}".to_string(),
                description_template: r#"## Current State
Description of what currently exists

## Proposed Improvement
{description}

## Benefits
- Benefit 1
- Benefit 2
- Benefit 3

## Implementation Plan
High-level approach to implementing this improvement

## Success Metrics
How we'll measure the success of this improvement"#
                    .to_string(),
                suggested_priority: Some(3), // Normal priority
                suggested_team: None,
                tags: vec!["improvement".to_string(), "tech-debt".to_string()],
            },
        );

        // Investigation/spike template
        templates.insert(
            "investigation".to_string(),
            IssueTemplate {
                name: "Investigation".to_string(),
                title_template: "[Investigation] {title}".to_string(),
                description_template: r#"## Investigation Goal
{description}

## Questions to Answer
- Question 1?
- Question 2?
- Question 3?

## Research Areas
Areas to investigate and explore

## Success Criteria
What constitutes a successful investigation

## Timeline
Expected duration and key milestones

## Deliverables
- [ ] Research findings document
- [ ] Recommendations
- [ ] Next steps"#
                    .to_string(),
                suggested_priority: Some(3), // Normal priority
                suggested_team: None,
                tags: vec!["investigation".to_string(), "spike".to_string()],
            },
        );

        Self { templates }
    }

    #[allow(dead_code)] // Used in tests and for future template listing functionality
    pub fn get_template_names(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    pub fn get_template(&self, name: &str) -> Option<&IssueTemplate> {
        self.templates.get(name)
    }

    pub fn apply_template(
        &self,
        template_name: &str,
        title: &str,
        description: &str,
    ) -> Option<AppliedTemplate> {
        let template = self.get_template(template_name)?;

        let applied_title = template.title_template.replace("{title}", title);
        let applied_description = template
            .description_template
            .replace("{description}", description);

        Some(AppliedTemplate {
            title: applied_title,
            description: applied_description,
            suggested_priority: template.suggested_priority,
            suggested_team: template.suggested_team.clone(),
            tags: template.tags.clone(),
        })
    }

    pub fn list_templates(&self) -> Vec<(String, String)> {
        self.templates
            .iter()
            .map(|(key, template)| (key.clone(), template.name.clone()))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct AppliedTemplate {
    pub title: String,
    pub description: String,
    pub suggested_priority: Option<i64>,
    #[allow(dead_code)] // Future enhancement: auto-suggest team based on template
    pub suggested_team: Option<String>,
    #[allow(dead_code)] // Future enhancement: categorize templates with tags
    pub tags: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_manager_creation() {
        let manager = TemplateManager::new();
        let names = manager.get_template_names();

        assert!(names.contains(&"bug".to_string()));
        assert!(names.contains(&"feature".to_string()));
        assert!(names.contains(&"task".to_string()));
        assert!(names.contains(&"improvement".to_string()));
        assert!(names.contains(&"investigation".to_string()));
    }

    #[test]
    fn test_get_template() {
        let manager = TemplateManager::new();
        let bug_template = manager.get_template("bug").unwrap();

        assert_eq!(bug_template.name, "Bug Report");
        assert_eq!(bug_template.title_template, "[Bug] {title}");
        assert_eq!(bug_template.suggested_priority, Some(2));
        assert!(bug_template.tags.contains(&"bug".to_string()));
    }

    #[test]
    fn test_apply_template() {
        let manager = TemplateManager::new();
        let applied = manager
            .apply_template("bug", "Login not working", "Users can't sign in")
            .unwrap();

        assert_eq!(applied.title, "[Bug] Login not working");
        assert!(applied.description.contains("Users can't sign in"));
        assert!(applied.description.contains("## Bug Description"));
        assert!(applied.description.contains("## Steps to Reproduce"));
        assert_eq!(applied.suggested_priority, Some(2));
        assert!(applied.tags.contains(&"bug".to_string()));
    }

    #[test]
    fn test_feature_template() {
        let manager = TemplateManager::new();
        let applied = manager
            .apply_template("feature", "Add dark mode", "Support dark theme")
            .unwrap();

        assert_eq!(applied.title, "[Feature] Add dark mode");
        assert!(applied.description.contains("Support dark theme"));
        assert!(applied.description.contains("## User Story"));
        assert!(applied.description.contains("## Acceptance Criteria"));
        assert_eq!(applied.suggested_priority, Some(3));
        assert!(applied.tags.contains(&"feature".to_string()));
        assert!(applied.tags.contains(&"enhancement".to_string()));
    }

    #[test]
    fn test_list_templates() {
        let manager = TemplateManager::new();
        let templates = manager.list_templates();

        assert!(!templates.is_empty());

        // Find the bug template
        let bug_template = templates.iter().find(|(key, _)| key == "bug");
        assert!(bug_template.is_some());
        assert_eq!(bug_template.unwrap().1, "Bug Report");
    }

    #[test]
    fn test_nonexistent_template() {
        let manager = TemplateManager::new();
        assert!(manager.get_template("nonexistent").is_none());
        assert!(
            manager
                .apply_template("nonexistent", "title", "desc")
                .is_none()
        );
    }
}
