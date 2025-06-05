// ABOUTME: Interactive prompts for collecting missing command-line arguments
// ABOUTME: Provides user-friendly terminal-based input for the create command

use crate::preferences::{ContextDefaults, PreferencesManager};
use crate::templates::TemplateManager;
use dialoguer::{Confirm, Editor, Input, Select};
use linear_sdk::{LinearClient, LinearError, Result as SdkResult, Team, User};
use std::collections::HashMap;
use std::io::IsTerminal;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Cache for team and user data to improve performance
#[derive(Debug, Clone)]
struct CachedData<T> {
    data: T,
    cached_at: Instant,
}

impl<T> CachedData<T> {
    fn new(data: T) -> Self {
        Self {
            data,
            cached_at: Instant::now(),
        }
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.cached_at.elapsed() > ttl
    }
}

/// Performance cache for API data
#[derive(Debug)]
struct PerformanceCache {
    teams: Arc<RwLock<Option<CachedData<Vec<Team>>>>>,
    users: Arc<RwLock<HashMap<String, CachedData<Vec<User>>>>>, // keyed by search query
    ttl: Duration,
}

impl PerformanceCache {
    fn new() -> Self {
        Self {
            teams: Arc::new(RwLock::new(None)),
            users: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(300), // 5 minutes cache
        }
    }

    fn get_teams(&self) -> Option<Vec<Team>> {
        let cache = self.teams.read().ok()?;
        if let Some(cached) = cache.as_ref() {
            if !cached.is_expired(self.ttl) {
                return Some(cached.data.clone());
            }
        }
        None
    }

    fn set_teams(&self, teams: Vec<Team>) {
        if let Ok(mut cache) = self.teams.write() {
            *cache = Some(CachedData::new(teams));
        }
    }

    fn get_users(&self, query: &str) -> Option<Vec<User>> {
        let cache = self.users.read().ok()?;
        if let Some(cached) = cache.get(query) {
            if !cached.is_expired(self.ttl) {
                return Some(cached.data.clone());
            }
        }
        None
    }

    fn set_users(&self, query: &str, users: Vec<User>) {
        if let Ok(mut cache) = self.users.write() {
            cache.insert(query.to_string(), CachedData::new(users));
        }
    }
}

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
    preferences_manager: PreferencesManager,
    template_manager: TemplateManager,
    cache: PerformanceCache,
}

impl<'a> InteractivePrompter<'a> {
    pub fn new(client: &'a LinearClient) -> SdkResult<Self> {
        let is_tty = std::io::stdin().is_terminal();
        let preferences_manager =
            PreferencesManager::new().map_err(|e| LinearError::InvalidInput {
                message: format!("Failed to initialize preferences: {}", e),
            })?;
        let template_manager = TemplateManager::new();
        let cache = PerformanceCache::new();

        Ok(Self {
            client,
            is_tty,
            preferences_manager,
            template_manager,
            cache,
        })
    }

    /// Check if interactive prompts should be used
    pub fn should_prompt(&self) -> bool {
        self.is_tty && !self.is_in_ci()
    }

    /// Check if running in CI environment
    pub fn is_in_ci(&self) -> bool {
        std::env::var("CI").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
            || std::env::var("JENKINS_URL").is_ok()
            || std::env::var("BUILDKITE").is_ok()
    }

    /// Test helper to override TTY detection
    #[cfg(test)]
    #[allow(dead_code)] // Used in tests to simulate different TTY environments
    pub fn with_tty_override(mut self, is_tty: bool) -> Self {
        self.is_tty = is_tty;
        self
    }

    /// Get teams with caching for improved performance
    async fn get_teams_cached(&self) -> SdkResult<Vec<Team>> {
        // Try cache first
        if let Some(teams) = self.cache.get_teams() {
            log::debug!("Using cached teams data ({} teams)", teams.len());
            return Ok(teams);
        }

        // Cache miss, fetch from API
        log::debug!("Cache miss, fetching teams from API");
        let teams = self.client.list_teams().await?;
        self.cache.set_teams(teams.clone());
        log::debug!("Cached {} teams for future use", teams.len());
        Ok(teams)
    }

    /// Search users with caching for improved performance
    async fn search_users_cached(&self, query: &str, limit: i32) -> SdkResult<Vec<User>> {
        // Try cache first
        let cache_key = format!("{}:{}", query, limit);
        if let Some(users) = self.cache.get_users(&cache_key) {
            log::debug!(
                "Using cached user search results for '{}' ({} users)",
                query,
                users.len()
            );
            return Ok(users);
        }

        // Cache miss, fetch from API
        log::debug!("Cache miss, searching users via API for query: '{}'", query);
        let users = self.client.search_users(query, limit).await?;
        self.cache.set_users(&cache_key, users.clone());
        log::debug!(
            "Cached {} users for query '{}' for future use",
            users.len(),
            query
        );
        Ok(users)
    }

    /// Create instance with smart defaults integration
    pub fn new_with_defaults(client: &'a LinearClient) -> SdkResult<Self> {
        Self::new(client)
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

        // Get smart defaults and context
        let context_defaults =
            self.preferences_manager
                .get_context_defaults()
                .unwrap_or(ContextDefaults {
                    suggested_team: None,
                    suggested_title_prefix: None,
                    suggested_assignee: None,
                    branch_context: None,
                });

        // Show context if available
        if let Some(branch) = &context_defaults.branch_context {
            println!("📋 Detected Git branch: {}", branch);
        }
        if let Some(prefix) = &context_defaults.suggested_title_prefix {
            println!("💡 Suggested title prefix: {}", prefix);
        }
        println!();

        // Check if user wants to use a template
        let template_result = self.prompt_template_selection()?;

        // Collect title (with smart defaults)
        let title = match options.title {
            Some(title) => title,
            None => self.prompt_title_with_defaults(&context_defaults, &template_result)?,
        };

        // Apply template if selected
        let (final_title, template_description, template_priority) =
            if let Some(template) = template_result {
                let applied = self
                    .template_manager
                    .apply_template(&template, &title, "")
                    .unwrap();
                (
                    applied.title,
                    Some(applied.description),
                    applied.suggested_priority,
                )
            } else {
                (title, None, None)
            };

        // Collect team (with smart defaults)
        let team_id = match options.team {
            Some(team) => self.resolve_team(&team).await?,
            None => self.prompt_team_with_defaults(&context_defaults).await?,
        };

        // Collect description (considering template)
        let description = match options.description {
            Some(desc) => Some(desc),
            None => self.prompt_description_with_template(template_description)?,
        };

        // Collect assignee (with team context and smart defaults)
        let assignee_id = match options.assignee {
            Some(assignee) => self.resolve_assignee(&assignee).await?,
            None => {
                self.prompt_assignee_with_defaults(Some(&team_id), &context_defaults)
                    .await?
            }
        };

        // Collect priority (with template and smart defaults)
        let priority = match options.priority {
            Some(priority) => Some(priority),
            None => self.prompt_priority_with_defaults(template_priority)?,
        };

        // Save preferences for next time
        let _ =
            self.preferences_manager
                .update_last_used(&team_id, assignee_id.as_deref(), priority);

        Ok(InteractiveCreateInput {
            title: final_title,
            description,
            team_id,
            assignee_id,
            priority,
        })
    }

    /// Prompt for template selection
    fn prompt_template_selection(&self) -> SdkResult<Option<String>> {
        let use_template = Confirm::new()
            .with_prompt("Would you like to use an issue template?")
            .default(false)
            .interact()
            .map_err(|e| LinearError::InvalidInput {
                message: format!("Failed to read template choice: {}", e),
            })?;

        if !use_template {
            return Ok(None);
        }

        let templates = self.template_manager.list_templates();
        let mut template_options = vec!["None - continue without template".to_string()];
        template_options.extend(templates.iter().map(|(_, name)| name.clone()));

        let selection = Select::new()
            .with_prompt("Select a template")
            .items(&template_options)
            .default(0)
            .interact()
            .map_err(|e| LinearError::InvalidInput {
                message: format!("Failed to select template: {}", e),
            })?;

        if selection == 0 {
            Ok(None)
        } else {
            let template_key = &templates[selection - 1].0;
            Ok(Some(template_key.clone()))
        }
    }

    /// Prompt for title with smart defaults
    fn prompt_title_with_defaults(
        &self,
        context_defaults: &ContextDefaults,
        _template: &Option<String>,
    ) -> SdkResult<String> {
        let mut prompt = Input::new().with_prompt("Title");

        // Set default based on context
        if let Some(prefix) = &context_defaults.suggested_title_prefix {
            prompt = prompt.default(prefix.clone());
        }

        let title: String = prompt
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

    /// Prompt for team with smart defaults
    async fn prompt_team_with_defaults(
        &self,
        context_defaults: &ContextDefaults,
    ) -> SdkResult<String> {
        let teams = self.get_teams_cached().await?;

        if teams.is_empty() {
            return Err(LinearError::InvalidInput {
                message: "No teams found in your workspace".to_string(),
            });
        }

        let team_names: Vec<String> = teams
            .iter()
            .map(|team| format!("{} ({})", team.name, team.key))
            .collect();

        // Find default selection based on context
        let mut default_selection = 0;
        if let Some(suggested_team) = &context_defaults.suggested_team {
            for (i, team) in teams.iter().enumerate() {
                if team.key.eq_ignore_ascii_case(suggested_team) || team.id == *suggested_team {
                    default_selection = i;
                    break;
                }
            }
        }

        let mut prompt_msg = "Select a team".to_string();
        if default_selection > 0 {
            prompt_msg = format!("Select a team (default: {})", teams[default_selection].key);
        }

        let selection = Select::new()
            .with_prompt(&prompt_msg)
            .items(&team_names)
            .default(default_selection)
            .interact()
            .map_err(|e| LinearError::InvalidInput {
                message: format!("Failed to select team: {}", e),
            })?;

        Ok(teams[selection].id.clone())
    }

    /// Prompt for description with template support
    fn prompt_description_with_template(
        &self,
        template_description: Option<String>,
    ) -> SdkResult<Option<String>> {
        if let Some(template_desc) = template_description {
            let use_template_desc = Confirm::new()
                .with_prompt("Use template description?")
                .default(true)
                .interact()
                .map_err(|e| LinearError::InvalidInput {
                    message: format!("Failed to read template description choice: {}", e),
                })?;

            if use_template_desc {
                let description =
                    Editor::new()
                        .edit(&template_desc)
                        .map_err(|e| LinearError::InvalidInput {
                            message: format!("Failed to open editor: {}", e),
                        })?;

                return Ok(description.and_then(|d| {
                    let trimmed = d.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                }));
            }
        }

        // Fall back to regular description prompt
        self.prompt_description()
    }

    /// Prompt for assignee with smart defaults
    async fn prompt_assignee_with_defaults(
        &self,
        team_id: Option<&str>,
        context_defaults: &ContextDefaults,
    ) -> SdkResult<Option<String>> {
        let mut options = vec!["Assign to me", "Leave unassigned"];

        // Add suggestion from context if available
        if context_defaults.suggested_assignee.is_some() {
            options.insert(1, "Use suggested assignee");
        }

        options.extend(["Search for user by name/email"]);

        // Add team-based suggestions if team is available
        if team_id.is_some() {
            options.insert(options.len() - 1, "Show team members");
        }

        options.push("Enter user ID directly");

        let selection = Select::new()
            .with_prompt("Assignee")
            .items(&options)
            .default(1) // Default to unassigned or suggested
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
                if context_defaults.suggested_assignee.is_some() {
                    // Use suggested assignee
                    Ok(context_defaults.suggested_assignee.clone())
                } else {
                    // Leave unassigned
                    Ok(None)
                }
            }
            _ => {
                // Handle other options (search, team members, direct input)
                self.prompt_assignee_with_team_context(team_id).await
            }
        }
    }

    /// Prompt for priority with smart defaults
    fn prompt_priority_with_defaults(
        &self,
        template_priority: Option<i64>,
    ) -> SdkResult<Option<i64>> {
        let priorities = vec!["None", "1 - Urgent", "2 - High", "3 - Normal", "4 - Low"];

        // Determine default based on template or preferences
        let mut default_selection = 0;
        if let Some(priority) = template_priority {
            default_selection = priority as usize;
        } else if let Ok(preferences) = self.preferences_manager.load_preferences() {
            if let Some(priority) = preferences.default_priority {
                default_selection = priority as usize;
            }
        }

        let mut prompt_msg = "Priority".to_string();
        if default_selection > 0 && default_selection < priorities.len() {
            prompt_msg = format!("Priority (default: {})", priorities[default_selection]);
        }

        let selection = Select::new()
            .with_prompt(&prompt_msg)
            .items(&priorities)
            .default(default_selection)
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

    /// Prompt for issue title
    #[allow(dead_code)] // Individual prompt methods for future modular prompting
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
    #[allow(dead_code)] // Individual prompt methods for future modular prompting
    async fn prompt_team(&self) -> SdkResult<String> {
        let teams = self.get_teams_cached().await?;

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

    /// Enhanced prompt for assignee with user search and team filtering
    #[allow(dead_code)] // Individual prompt methods for future modular prompting
    async fn prompt_assignee(&self) -> SdkResult<Option<String>> {
        self.prompt_assignee_with_team_context(None).await
    }

    /// Enhanced prompt for assignee with optional team context for filtering
    async fn prompt_assignee_with_team_context(
        &self,
        team_id: Option<&str>,
    ) -> SdkResult<Option<String>> {
        let mut options = vec![
            "Assign to me",
            "Leave unassigned",
            "Search for user by name/email",
        ];

        // Add team-based suggestions if team is available
        if team_id.is_some() {
            options.insert(2, "Show team members");
        }

        options.push("Enter user ID directly");

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
                if team_id.is_some() {
                    // Show team members
                    self.prompt_team_members(team_id.unwrap()).await
                } else {
                    // Search for user by name/email
                    self.prompt_user_search().await
                }
            }
            3 => {
                if team_id.is_some() {
                    // Search for user by name/email (when team option is present)
                    self.prompt_user_search().await
                } else {
                    // Enter user ID directly (when no team option)
                    self.prompt_user_id_direct().await
                }
            }
            4 => {
                // Enter user ID directly (when team option is present)
                self.prompt_user_id_direct().await
            }
            _ => unreachable!(),
        }
    }

    /// Prompt for user search with autocomplete-like functionality
    async fn prompt_user_search(&self) -> SdkResult<Option<String>> {
        let search_query: String = Input::new()
            .with_prompt("Search users by name or email")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.trim().len() < 2 {
                    Err("Search query must be at least 2 characters")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .map_err(|e| LinearError::InvalidInput {
                message: format!("Failed to read search query: {}", e),
            })?;

        let search_query = search_query.trim();

        // Search for users
        let users = self
            .search_users_cached(search_query, 10)
            .await
            .map_err(|e| LinearError::InvalidInput {
                message: format!("Failed to search users: {}", e),
            })?;

        if users.is_empty() {
            println!(
                "No users found matching '{}'. Try a different search term.",
                search_query
            );
            return Ok(None);
        }

        // Display users for selection
        let mut user_options = vec!["Cancel - don't assign".to_string()];
        user_options.extend(users.iter().map(|user| {
            format!(
                "{} ({}) - {}",
                user.name,
                user.email,
                if user.active { "Active" } else { "Inactive" }
            )
        }));

        let selection = Select::new()
            .with_prompt(format!(
                "Found {} user(s) matching '{}'. Select one:",
                users.len(),
                search_query
            ))
            .items(&user_options)
            .default(0)
            .interact()
            .map_err(|e| LinearError::InvalidInput {
                message: format!("Failed to select user: {}", e),
            })?;

        if selection == 0 {
            Ok(None) // Cancelled
        } else {
            let selected_user = &users[selection - 1];
            if !selected_user.active {
                println!(
                    "Warning: Selected user '{}' is inactive.",
                    selected_user.name
                );
            }
            Ok(Some(selected_user.id.clone()))
        }
    }

    /// Prompt for team member selection
    async fn prompt_team_members(&self, team_id: &str) -> SdkResult<Option<String>> {
        // Get teams to find the one with the specified ID
        let teams = self.get_teams_cached().await?;
        let team =
            teams
                .iter()
                .find(|t| t.id == team_id)
                .ok_or_else(|| LinearError::InvalidInput {
                    message: format!("Team with ID '{}' not found", team_id),
                })?;

        if team.members.is_empty() {
            println!("No members found in team '{}'.", team.name);
            return Ok(None);
        }

        // Display team members for selection
        let mut member_options = vec!["Cancel - don't assign".to_string()];
        member_options.extend(team.members.iter().map(|member| {
            format!(
                "{} ({}) - {}",
                member.name,
                member.email,
                if member.active { "Active" } else { "Inactive" }
            )
        }));

        let selection = Select::new()
            .with_prompt(format!("Team '{}' members - Select assignee:", team.name))
            .items(&member_options)
            .default(0)
            .interact()
            .map_err(|e| LinearError::InvalidInput {
                message: format!("Failed to select team member: {}", e),
            })?;

        if selection == 0 {
            Ok(None) // Cancelled
        } else {
            let selected_member = &team.members[selection - 1];
            if !selected_member.active {
                println!(
                    "Warning: Selected user '{}' is inactive.",
                    selected_member.name
                );
            }
            Ok(Some(selected_member.id.clone()))
        }
    }

    /// Prompt for direct user ID input
    async fn prompt_user_id_direct(&self) -> SdkResult<Option<String>> {
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

    /// Prompt for priority
    #[allow(dead_code)] // Individual prompt methods for future modular prompting
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
    /// Enhanced with fuzzy matching and helpful error suggestions
    async fn resolve_team(&self, team: &str) -> SdkResult<String> {
        // Check if it looks like a UUID
        if team.chars().all(|c| c.is_ascii_hexdigit() || c == '-') && team.len() > 20 {
            return Ok(team.to_string());
        }

        // Try exact resolution first
        match self.client.resolve_team_key_to_id(team).await {
            Ok(team_id) => Ok(team_id),
            Err(_) => {
                // Enhanced error handling with fuzzy matching suggestions
                match self.suggest_similar_teams(team).await {
                    Ok(suggestions) if !suggestions.is_empty() => {
                        let suggestion_text = if suggestions.len() == 1 {
                            format!("Did you mean '{}'?", suggestions[0])
                        } else {
                            format!("Did you mean one of: {}?", suggestions.join(", "))
                        };

                        Err(LinearError::InvalidInput {
                            message: format!(
                                "Team '{}' not found. {} Use 'linear teams' to see all available teams.",
                                team, suggestion_text
                            ),
                        })
                    }
                    Ok(_) => Err(LinearError::InvalidInput {
                        message: format!(
                            "Team '{}' not found. Use 'linear teams' to see all available teams.",
                            team
                        ),
                    }),
                    Err(_) => Err(LinearError::InvalidInput {
                        message: format!(
                            "Team '{}' not found and unable to fetch team suggestions.",
                            team
                        ),
                    }),
                }
            }
        }
    }

    /// Suggest similar team names for improved UX
    async fn suggest_similar_teams(&self, input: &str) -> SdkResult<Vec<String>> {
        let teams = self.get_teams_cached().await?;
        let mut suggestions = Vec::new();

        let input_lower = input.to_lowercase();

        // Look for exact key matches (case insensitive)
        for team in &teams {
            if team.key.to_lowercase() == input_lower {
                return Ok(vec![team.key.clone()]);
            }
        }

        // Look for partial matches in team keys
        for team in &teams {
            if team.key.to_lowercase().contains(&input_lower) {
                suggestions.push(team.key.clone());
            }
        }

        // Look for partial matches in team names
        if suggestions.is_empty() {
            for team in &teams {
                if team.name.to_lowercase().contains(&input_lower) {
                    suggestions.push(team.key.clone());
                }
            }
        }

        // Limit suggestions to avoid overwhelming the user
        suggestions.truncate(3);

        Ok(suggestions)
    }

    /// Resolve assignee (user) input to user ID
    async fn resolve_assignee(&self, assignee: &str) -> SdkResult<Option<String>> {
        // Handle special cases
        if assignee.eq_ignore_ascii_case("me") || assignee.eq_ignore_ascii_case("self") {
            let viewer_data = self.client.execute_viewer_query().await?;
            return Ok(Some(viewer_data.viewer.id));
        }

        if assignee.eq_ignore_ascii_case("none")
            || assignee.eq_ignore_ascii_case("unassigned")
            || assignee.eq_ignore_ascii_case("null")
        {
            return Ok(None);
        }

        // Check if it looks like a UUID
        if assignee.chars().all(|c| c.is_ascii_hexdigit() || c == '-') && assignee.len() > 20 {
            return Ok(Some(assignee.to_string()));
        }

        // Try searching for the user
        let users = self.search_users_cached(assignee, 5).await?;

        if users.is_empty() {
            return Err(LinearError::InvalidInput {
                message: format!(
                    "User '{}' not found. Use 'linear users' to see available users.",
                    assignee
                ),
            });
        }

        // If we have exactly one match, use it
        if users.len() == 1 {
            let user = &users[0];
            if !user.active {
                log::warn!("Selected user '{}' is inactive", user.name);
            }
            return Ok(Some(user.id.clone()));
        }

        // Multiple matches - look for exact match first
        for user in &users {
            if user.email.eq_ignore_ascii_case(assignee) || user.name.eq_ignore_ascii_case(assignee)
            {
                if !user.active {
                    log::warn!("Selected user '{}' is inactive", user.name);
                }
                return Ok(Some(user.id.clone()));
            }
        }

        // Multiple matches, no exact match
        let user_list: Vec<String> = users
            .iter()
            .map(|u| format!("{} ({})", u.name, u.email))
            .collect();

        Err(LinearError::InvalidInput {
            message: format!(
                "Multiple users found matching '{}': {}. Please be more specific.",
                assignee,
                user_list.join(", ")
            ),
        })
    }
}
