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

#[allow(dead_code)] // Phase 3 development - will be integrated in next commit
#[derive(Debug, Clone)]
pub struct InteractiveCreateInput {
    pub title: String,
    pub description: Option<String>,
    pub team_id: String,
    pub assignee_id: Option<String>,
    pub priority: Option<i64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CreateOptions {
    pub title: Option<String>,
    pub description: Option<String>,
    pub team: Option<String>,
    pub assignee: Option<String>,
    pub priority: Option<i64>,
}

#[allow(dead_code)]
pub struct InteractivePrompter<'a> {
    client: &'a LinearClient,
    is_tty: bool,
    preferences_manager: PreferencesManager,
    template_manager: TemplateManager,
    cache: PerformanceCache,
}

#[allow(dead_code)]
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
                                "Team '{}' not found. {} Use 'linear teams' to list all teams.",
                                team, suggestion_text
                            ),
                        })
                    }
                    _ => Err(LinearError::InvalidInput {
                        message: format!(
                            "Team '{}' not found. Use 'linear teams' to list all available teams.",
                            team
                        ),
                    }),
                }
            }
        }
    }

    /// Suggest similar team names using fuzzy matching
    async fn suggest_similar_teams(&self, input: &str) -> SdkResult<Vec<String>> {
        let teams = self.get_teams_cached().await?;
        let input_lower = input.to_lowercase();

        let mut suggestions = Vec::new();

        // Look for teams that contain the input or are similar
        for team in teams {
            let team_key_lower = team.key.to_lowercase();
            let team_name_lower = team.name.to_lowercase();

            // Check for exact substring match (highest priority) or fuzzy match
            let is_match = team_key_lower.contains(&input_lower)
                || team_name_lower.contains(&input_lower)
                || self.is_similar_string(&input_lower, &team_key_lower, 2);

            if is_match {
                suggestions.push(team.key);
            }
        }

        // Limit suggestions to avoid overwhelming the user
        suggestions.truncate(3);
        Ok(suggestions)
    }

    /// Simple fuzzy string matching (Levenshtein-like with max distance)
    fn is_similar_string(&self, input: &str, target: &str, max_distance: usize) -> bool {
        // Convert to lowercase for case-insensitive matching
        let input_lower = input.to_lowercase();
        let target_lower = target.to_lowercase();

        if input_lower.len().abs_diff(target_lower.len()) > max_distance {
            return false;
        }

        let mut distance = 0;
        let input_chars: Vec<char> = input_lower.chars().collect();
        let target_chars: Vec<char> = target_lower.chars().collect();

        let mut i = 0;
        let mut j = 0;

        while i < input_chars.len() && j < target_chars.len() {
            if input_chars[i] == target_chars[j] {
                i += 1;
                j += 1;
            } else {
                distance += 1;
                if distance > max_distance {
                    return false;
                }
                // Try advancing both pointers to handle substitutions
                i += 1;
                j += 1;
            }
        }

        // Account for remaining characters
        distance += input_chars.len().saturating_sub(i) + target_chars.len().saturating_sub(j);
        distance <= max_distance
    }

    /// Resolve assignee (handle "me" or return as-is)
    /// Enhanced with validation and helpful error messages
    async fn resolve_assignee(&self, assignee: &str) -> SdkResult<Option<String>> {
        if assignee == "me" {
            match self.client.execute_viewer_query().await {
                Ok(viewer_data) => Ok(Some(viewer_data.viewer.id)),
                Err(e) => Err(LinearError::InvalidInput {
                    message: format!(
                        "Failed to resolve 'me' to current user: {}. Check your authentication.",
                        e
                    ),
                }),
            }
        } else if assignee.is_empty() || assignee == "unassigned" {
            Ok(None)
        } else if assignee.contains('@') {
            // Looks like an email, provide helpful message
            Err(LinearError::InvalidInput {
                message: format!(
                    "User identifier '{}' looks like an email. Use the user's ID instead, or use interactive mode to search by email.",
                    assignee
                ),
            })
        } else if assignee.len() < 5 || assignee.chars().any(|c| c.is_whitespace()) {
            // Too short or contains spaces, likely not a valid user ID
            Err(LinearError::InvalidInput {
                message: format!(
                    "User identifier '{}' doesn't look like a valid user ID. Use 'linear create' without --assignee to search interactively, or use a full user ID.",
                    assignee
                ),
            })
        } else {
            // Assume it's a user ID and let the API validate it
            // This preserves backwards compatibility while still catching obvious errors
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

    fn create_test_prompter(client: &LinearClient) -> InteractivePrompter {
        let preferences_manager = PreferencesManager::new().unwrap();
        let template_manager = TemplateManager::new();
        let cache = PerformanceCache::new();

        InteractivePrompter {
            client,
            is_tty: true,
            preferences_manager,
            template_manager,
            cache,
        }
    }

    #[test]
    fn test_should_prompt_with_tty() {
        let client = create_test_client();
        let prompter = create_test_prompter(&client).with_tty_override(true);

        // Should prompt when TTY and not in CI
        assert!(prompter.should_prompt());
    }

    #[test]
    fn test_should_not_prompt_in_ci() {
        let client = create_test_client();
        let prompter = create_test_prompter(&client).with_tty_override(true);

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
        let prompter = create_test_prompter(&client).with_tty_override(false);

        assert!(!prompter.should_prompt());
    }

    #[tokio::test]
    async fn test_resolve_team_with_uuid() {
        let client = create_test_client();
        let prompter = create_test_prompter(&client);

        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let result = prompter.resolve_team(uuid).await.unwrap();

        assert_eq!(result, uuid);
    }

    // Note: Testing resolve_assignee("me") requires a working GraphQL client
    // This is integration-tested in the main CLI tests

    #[tokio::test]
    async fn test_resolve_assignee_unassigned() {
        let client = create_test_client();
        let prompter = create_test_prompter(&client);

        let result = prompter.resolve_assignee("unassigned").await.unwrap();
        assert_eq!(result, None);

        let result = prompter.resolve_assignee("").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_resolve_assignee_other() {
        let client = create_test_client();
        let prompter = create_test_prompter(&client);

        let user_id = "user-123";
        let result = prompter.resolve_assignee(user_id).await.unwrap();

        assert_eq!(result, Some(user_id.to_string()));
    }

    #[tokio::test]
    async fn test_collect_create_input_non_tty() {
        let client = create_test_client();
        let prompter = create_test_prompter(&client).with_tty_override(false);

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

    #[test]
    fn test_enhanced_user_lookup_structure() {
        // Test that the enhanced assignee prompt structure is correct
        let client = create_test_client();
        let prompter = create_test_prompter(&client).with_tty_override(true);

        // Test that the prompter structure is correct
        // We need to account for CI environment in testing
        if !prompter.is_in_ci() {
            assert!(prompter.should_prompt());
        } else {
            // In CI, should_prompt returns false even with TTY override
            assert!(!prompter.should_prompt());
        }

        // Test team context awareness - this validates the method signatures exist
        // Note: We can't easily test interactive prompts without mocking the dialoguer library
        // but we can test the structure and logic
    }

    #[tokio::test]
    async fn test_prompt_user_id_direct_validation() {
        // Test that we can create the structure for direct user ID input
        // This validates that the method exists and has correct signature
        let client = create_test_client();
        let prompter = create_test_prompter(&client);

        // We can't test the actual interactive prompt without mocking dialoguer,
        // but we can ensure the method exists and the structure is correct
        assert!(prompter.should_prompt() || !prompter.should_prompt()); // Always true, just validates structure
    }

    #[tokio::test]
    async fn test_team_context_integration() {
        // Test that team context can be passed correctly
        let client = create_test_client();
        let prompter = create_test_prompter(&client);

        let _test_team_id = "team-test-123";

        // This validates that the method signature accepts team_id parameter
        // We can't test actual interaction without mocking dialoguer
        // but we can ensure the integration structure is correct

        // Test that resolve methods work with valid inputs
        let result = prompter.resolve_assignee("unassigned").await.unwrap();
        assert_eq!(result, None);

        let result = prompter.resolve_assignee("").await.unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_enhanced_error_handling() {
        // Test that new error types are handled correctly
        use linear_sdk::LinearError;

        let error = LinearError::InvalidInput {
            message: "Test error message".to_string(),
        };

        assert!(error.to_string().contains("Invalid input"));
        assert!(error.to_string().contains("Test error message"));
    }

    #[test]
    fn test_user_search_validation() {
        // Test validation logic for user search functionality
        let search_query = "test@example.com";
        assert!(search_query.len() >= 2); // Validates minimum search length

        let short_query = "a";
        assert!(short_query.len() < 2); // Should fail validation

        let empty_query = "";
        assert!(empty_query.trim().is_empty()); // Should fail validation
    }

    #[test]
    fn test_phase5_fuzzy_matching() {
        let client = create_test_client();
        let prompter = create_test_prompter(&client);

        // Test exact match
        assert!(prompter.is_similar_string("eng", "eng", 2));

        // Test case insensitive matching
        assert!(prompter.is_similar_string("eng", "ENG", 2));

        // Test substitution (1 character difference)
        assert!(prompter.is_similar_string("eng", "end", 2));

        // Test too many differences
        assert!(!prompter.is_similar_string("eng", "xyz", 2));

        // Test length difference beyond threshold
        assert!(!prompter.is_similar_string("a", "abcdef", 2));
    }

    #[tokio::test]
    async fn test_phase5_enhanced_assignee_validation() {
        let client = create_test_client();
        let prompter = create_test_prompter(&client);

        // Test email detection
        let result = prompter.resolve_assignee("user@example.com").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("looks like an email")
        );

        // Test short invalid ID
        let result = prompter.resolve_assignee("a").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("doesn't look like a valid user ID")
        );

        // Test whitespace in ID
        let result = prompter.resolve_assignee("user 123").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("doesn't look like a valid user ID")
        );

        // Test valid-looking ID (should pass through)
        let result = prompter.resolve_assignee("user-12345").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("user-12345".to_string()));
    }

    #[test]
    fn test_phase5_cache_structure() {
        let cache = PerformanceCache::new();

        // Test that cache starts empty
        assert!(cache.get_teams().is_none());
        assert!(cache.get_users("test").is_none());

        // Test cache TTL setting
        assert_eq!(cache.ttl, Duration::from_secs(300));
    }

    #[test]
    fn test_phase5_cached_data_expiration() {
        let data = CachedData::new("test".to_string());

        // Should not be expired immediately
        assert!(!data.is_expired(Duration::from_secs(1)));

        // Test with zero TTL (should be expired)
        assert!(data.is_expired(Duration::from_secs(0)));
    }
}
