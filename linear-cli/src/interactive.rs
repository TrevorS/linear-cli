// ABOUTME: Interactive prompts for collecting missing command-line arguments
// ABOUTME: Provides user-friendly terminal-based input for the create command

use dialoguer::{Confirm, Editor, Input, Select};
use linear_sdk::{LinearClient, LinearError, Result as SdkResult};
use std::io::IsTerminal;

#[derive(Debug, Clone)]
pub struct InteractiveCreateInput {
    pub title: String,
    pub description: Option<String>,
    pub team_id: String,
    pub assignee_id: Option<String>,
    pub priority: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct CreateOptions {
    pub title: Option<String>,
    pub description: Option<String>,
    pub team: Option<String>,
    pub assignee: Option<String>,
    pub priority: Option<i64>,
}

pub struct InteractivePrompter<'a> {
    client: &'a LinearClient,
    is_tty: bool,
}

impl<'a> InteractivePrompter<'a> {
    pub fn new(client: &'a LinearClient) -> Self {
        let is_tty = std::io::stdin().is_terminal();
        Self { client, is_tty }
    }

    /// Check if interactive prompts should be used
    pub fn should_prompt(&self) -> bool {
        self.is_tty && !self.is_in_ci()
    }

    /// Check if running in CI environment
    fn is_in_ci(&self) -> bool {
        std::env::var("CI").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
            || std::env::var("JENKINS_URL").is_ok()
            || std::env::var("BUILDKITE").is_ok()
    }

    /// Test helper to override TTY detection
    #[cfg(test)]
    pub fn with_tty_override(mut self, is_tty: bool) -> Self {
        self.is_tty = is_tty;
        self
    }

    /// Collect all missing fields interactively
    pub async fn collect_create_input(
        &self,
        options: CreateOptions,
    ) -> SdkResult<InteractiveCreateInput> {
        if !self.should_prompt() {
            return Err(LinearError::InvalidInput {
                message: "Interactive prompts not available in non-TTY environment".to_string(),
            });
        }

        println!("Creating a new Linear issue...\n");

        // Collect title
        let title = match options.title {
            Some(title) => title,
            None => self.prompt_title()?,
        };

        // Collect team
        let team_id = match options.team {
            Some(team) => self.resolve_team(&team).await?,
            None => self.prompt_team().await?,
        };

        // Collect description
        let description = match options.description {
            Some(desc) => Some(desc),
            None => self.prompt_description()?,
        };

        // Collect assignee
        let assignee_id = match options.assignee {
            Some(assignee) => self.resolve_assignee(&assignee).await?,
            None => self.prompt_assignee().await?,
        };

        // Collect priority
        let priority = match options.priority {
            Some(priority) => Some(priority),
            None => self.prompt_priority()?,
        };

        Ok(InteractiveCreateInput {
            title,
            description,
            team_id,
            assignee_id,
            priority,
        })
    }

    /// Prompt for issue title
    fn prompt_title(&self) -> SdkResult<String> {
        let title: String = Input::new()
            .with_prompt("Title")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.trim().is_empty() {
                    Err("Title cannot be empty")
                } else if input.len() > 255 {
                    Err("Title must be 255 characters or less")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .map_err(|e| LinearError::InvalidInput {
                message: format!("Failed to read title: {}", e),
            })?;

        Ok(title.trim().to_string())
    }

    /// Prompt for issue description
    fn prompt_description(&self) -> SdkResult<Option<String>> {
        let use_editor = Confirm::new()
            .with_prompt("Would you like to write a multi-line description?")
            .default(false)
            .interact()
            .map_err(|e| LinearError::InvalidInput {
                message: format!("Failed to read description choice: {}", e),
            })?;

        if use_editor {
            let description = Editor::new()
                .edit("Enter your issue description here...")
                .map_err(|e| LinearError::InvalidInput {
                    message: format!("Failed to open editor: {}", e),
                })?;

            Ok(description.and_then(|d| {
                let trimmed = d.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            }))
        } else {
            let description: String = Input::new()
                .with_prompt("Description (optional)")
                .allow_empty(true)
                .interact_text()
                .map_err(|e| LinearError::InvalidInput {
                    message: format!("Failed to read description: {}", e),
                })?;

            let trimmed = description.trim();
            Ok(if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            })
        }
    }

    /// Prompt for team selection
    async fn prompt_team(&self) -> SdkResult<String> {
        let teams = self.client.list_teams().await?;

        if teams.is_empty() {
            return Err(LinearError::InvalidInput {
                message: "No teams found in your workspace".to_string(),
            });
        }

        let team_names: Vec<String> = teams
            .iter()
            .map(|team| format!("{} ({})", team.name, team.key))
            .collect();

        let selection = Select::new()
            .with_prompt("Select a team")
            .items(&team_names)
            .interact()
            .map_err(|e| LinearError::InvalidInput {
                message: format!("Failed to select team: {}", e),
            })?;

        Ok(teams[selection].id.clone())
    }

    /// Prompt for assignee
    async fn prompt_assignee(&self) -> SdkResult<Option<String>> {
        let options = vec!["Assign to me", "Leave unassigned", "Other (enter user ID)"];

        let selection = Select::new()
            .with_prompt("Assignee")
            .items(&options)
            .default(1) // Default to unassigned
            .interact()
            .map_err(|e| LinearError::InvalidInput {
                message: format!("Failed to select assignee: {}", e),
            })?;

        match selection {
            0 => {
                // Assign to current user
                let viewer_data = self.client.execute_viewer_query().await?;
                Ok(Some(viewer_data.viewer.id))
            }
            1 => {
                // Leave unassigned
                Ok(None)
            }
            2 => {
                // Other user ID
                let user_id: String = Input::new()
                    .with_prompt("User ID")
                    .validate_with(|input: &String| -> Result<(), &str> {
                        if input.trim().is_empty() {
                            Err("User ID cannot be empty")
                        } else {
                            Ok(())
                        }
                    })
                    .interact_text()
                    .map_err(|e| LinearError::InvalidInput {
                        message: format!("Failed to read user ID: {}", e),
                    })?;

                Ok(Some(user_id.trim().to_string()))
            }
            _ => unreachable!(),
        }
    }

    /// Prompt for priority
    fn prompt_priority(&self) -> SdkResult<Option<i64>> {
        let priorities = vec!["None", "1 - Urgent", "2 - High", "3 - Normal", "4 - Low"];

        let selection = Select::new()
            .with_prompt("Priority")
            .items(&priorities)
            .default(0) // Default to None
            .interact()
            .map_err(|e| LinearError::InvalidInput {
                message: format!("Failed to select priority: {}", e),
            })?;

        Ok(match selection {
            0 => None,
            1 => Some(1),
            2 => Some(2),
            3 => Some(3),
            4 => Some(4),
            _ => unreachable!(),
        })
    }

    /// Resolve team key or return team ID as-is if it's already a UUID
    async fn resolve_team(&self, team: &str) -> SdkResult<String> {
        // Check if it looks like a UUID
        if team.chars().all(|c| c.is_ascii_hexdigit() || c == '-') && team.len() > 20 {
            Ok(team.to_string())
        } else {
            self.client.resolve_team_key_to_id(team).await
        }
    }

    /// Resolve assignee (handle "me" or return as-is)
    async fn resolve_assignee(&self, assignee: &str) -> SdkResult<Option<String>> {
        if assignee == "me" {
            let viewer_data = self.client.execute_viewer_query().await?;
            Ok(Some(viewer_data.viewer.id))
        } else if assignee.is_empty() || assignee == "unassigned" {
            Ok(None)
        } else {
            Ok(Some(assignee.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::SecretString;

    fn create_test_client() -> LinearClient {
        LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .build()
            .unwrap()
    }

    #[test]
    fn test_should_prompt_with_tty() {
        let client = create_test_client();
        let prompter = InteractivePrompter::new(&client).with_tty_override(true);

        // Should prompt when TTY and not in CI
        assert!(prompter.should_prompt());
    }

    #[test]
    fn test_should_not_prompt_in_ci() {
        let client = create_test_client();
        let prompter = InteractivePrompter::new(&client).with_tty_override(true);

        // Set CI environment variable
        unsafe {
            std::env::set_var("CI", "true");
        }

        // Should not prompt in CI even with TTY
        assert!(!prompter.should_prompt());

        // Clean up
        unsafe {
            std::env::remove_var("CI");
        }
    }

    #[test]
    fn test_should_not_prompt_without_tty() {
        let client = create_test_client();
        let prompter = InteractivePrompter::new(&client).with_tty_override(false);

        assert!(!prompter.should_prompt());
    }

    #[tokio::test]
    async fn test_resolve_team_with_uuid() {
        let client = create_test_client();
        let prompter = InteractivePrompter::new(&client);

        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let result = prompter.resolve_team(uuid).await.unwrap();

        assert_eq!(result, uuid);
    }

    // Note: Testing resolve_assignee("me") requires a working GraphQL client
    // This is integration-tested in the main CLI tests

    #[tokio::test]
    async fn test_resolve_assignee_unassigned() {
        let client = create_test_client();
        let prompter = InteractivePrompter::new(&client);

        let result = prompter.resolve_assignee("unassigned").await.unwrap();
        assert_eq!(result, None);

        let result = prompter.resolve_assignee("").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_resolve_assignee_other() {
        let client = create_test_client();
        let prompter = InteractivePrompter::new(&client);

        let user_id = "user-123";
        let result = prompter.resolve_assignee(user_id).await.unwrap();

        assert_eq!(result, Some(user_id.to_string()));
    }

    #[tokio::test]
    async fn test_collect_create_input_non_tty() {
        let client = create_test_client();
        let prompter = InteractivePrompter::new(&client).with_tty_override(false);

        let options = CreateOptions {
            title: Some("Test".to_string()),
            description: None,
            team: Some("ENG".to_string()),
            assignee: None,
            priority: None,
        };

        let result = prompter.collect_create_input(options).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("non-TTY"));
    }
}
