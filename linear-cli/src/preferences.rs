// ABOUTME: User preferences and smart defaults for the create command
// ABOUTME: Stores last used settings and provides context-aware defaults

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserPreferences {
    pub last_used_team: Option<String>,
    pub last_used_assignee: Option<String>,
    pub default_priority: Option<i64>,
    pub preferred_templates: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ContextDefaults {
    pub suggested_team: Option<String>,
    pub suggested_title_prefix: Option<String>,
    pub suggested_assignee: Option<String>,
    pub branch_context: Option<String>,
}

pub struct PreferencesManager {
    config_dir: PathBuf,
}

impl PreferencesManager {
    pub fn new() -> anyhow::Result<Self> {
        let config_dir = Self::get_config_dir()?;
        Ok(Self { config_dir })
    }

    fn get_config_dir() -> anyhow::Result<PathBuf> {
        let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
        let config_dir = PathBuf::from(home).join(".linear-cli");

        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
        }

        Ok(config_dir)
    }

    fn preferences_file(&self) -> PathBuf {
        self.config_dir.join("preferences.json")
    }

    pub fn load_preferences(&self) -> anyhow::Result<UserPreferences> {
        let file_path = self.preferences_file();

        if !file_path.exists() {
            return Ok(UserPreferences::default());
        }

        let content = std::fs::read_to_string(file_path)?;
        let preferences: UserPreferences = serde_json::from_str(&content)?;
        Ok(preferences)
    }

    pub fn save_preferences(&self, preferences: &UserPreferences) -> anyhow::Result<()> {
        let file_path = self.preferences_file();
        let content = serde_json::to_string_pretty(preferences)?;
        std::fs::write(file_path, content)?;
        Ok(())
    }

    pub fn update_last_used(
        &self,
        team_id: &str,
        assignee_id: Option<&str>,
        priority: Option<i64>,
    ) -> anyhow::Result<()> {
        let mut preferences = self.load_preferences()?;
        preferences.last_used_team = Some(team_id.to_string());
        preferences.last_used_assignee = assignee_id.map(|s| s.to_string());
        preferences.default_priority = priority;
        self.save_preferences(&preferences)?;
        Ok(())
    }

    pub fn get_context_defaults(&self) -> anyhow::Result<ContextDefaults> {
        let mut context = ContextDefaults {
            suggested_team: None,
            suggested_title_prefix: None,
            suggested_assignee: None,
            branch_context: None,
        };

        // Load user preferences
        if let Ok(preferences) = self.load_preferences() {
            context.suggested_team = preferences.last_used_team;
            context.suggested_assignee = preferences.last_used_assignee;
        }

        // Detect Git context
        if let Ok(branch_info) = self.detect_git_context() {
            context.branch_context = Some(branch_info.branch_name.clone());

            // Suggest title prefix based on branch
            if let Some(prefix) = self.extract_title_prefix(&branch_info.branch_name) {
                context.suggested_title_prefix = Some(prefix);
            }

            // Suggest team based on branch naming convention
            if let Some(team) = self.extract_team_from_branch(&branch_info.branch_name) {
                context.suggested_team = Some(team);
            }
        }

        Ok(context)
    }

    fn detect_git_context(&self) -> anyhow::Result<GitContext> {
        use std::process::Command;

        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Not in a git repository"));
        }

        let branch_name = String::from_utf8(output.stdout)?.trim().to_string();

        Ok(GitContext { branch_name })
    }

    fn extract_title_prefix(&self, branch_name: &str) -> Option<String> {
        // Common patterns: feature/ABC-123-description, bugfix/XYZ-456-fix-something
        if let Some(captures) = regex::Regex::new(r"(?:feature|bugfix|hotfix)/([A-Z]+-\d+)")
            .ok()?
            .captures(branch_name)
        {
            if let Some(ticket) = captures.get(1) {
                return Some(ticket.as_str().to_string());
            }
        }

        // Pattern: issue-123-description
        if let Some(captures) = regex::Regex::new(r"issue-(\d+)")
            .ok()?
            .captures(branch_name)
        {
            if let Some(issue_num) = captures.get(1) {
                return Some(format!("Issue #{}", issue_num.as_str()));
            }
        }

        None
    }

    fn extract_team_from_branch(&self, branch_name: &str) -> Option<String> {
        // Extract team from branch patterns like: feature/ENG-123-description
        if let Some(captures) = regex::Regex::new(r"(?:feature|bugfix|hotfix)/([A-Z]+)-\d+")
            .ok()?
            .captures(branch_name)
        {
            if let Some(team) = captures.get(1) {
                return Some(team.as_str().to_string());
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
struct GitContext {
    branch_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_manager() -> (PreferencesManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let manager = PreferencesManager {
            config_dir: temp_dir.path().to_path_buf(),
        };
        (manager, temp_dir)
    }

    #[test]
    fn test_load_default_preferences() {
        let (manager, _temp) = create_test_manager();
        let preferences = manager.load_preferences().unwrap();
        assert_eq!(preferences.last_used_team, None);
        assert_eq!(preferences.last_used_assignee, None);
        assert_eq!(preferences.default_priority, None);
    }

    #[test]
    fn test_save_and_load_preferences() {
        let (manager, _temp) = create_test_manager();

        let preferences = UserPreferences {
            last_used_team: Some("ENG".to_string()),
            last_used_assignee: Some("user-123".to_string()),
            default_priority: Some(2),
            ..Default::default()
        };

        manager.save_preferences(&preferences).unwrap();

        let loaded = manager.load_preferences().unwrap();
        assert_eq!(loaded.last_used_team, Some("ENG".to_string()));
        assert_eq!(loaded.last_used_assignee, Some("user-123".to_string()));
        assert_eq!(loaded.default_priority, Some(2));
    }

    #[test]
    fn test_update_last_used() {
        let (manager, _temp) = create_test_manager();

        manager
            .update_last_used("team-123", Some("user-456"), Some(3))
            .unwrap();

        let preferences = manager.load_preferences().unwrap();
        assert_eq!(preferences.last_used_team, Some("team-123".to_string()));
        assert_eq!(preferences.last_used_assignee, Some("user-456".to_string()));
        assert_eq!(preferences.default_priority, Some(3));
    }

    #[test]
    fn test_extract_title_prefix() {
        let (manager, _temp) = create_test_manager();

        assert_eq!(
            manager.extract_title_prefix("feature/ENG-123-fix-login"),
            Some("ENG-123".to_string())
        );

        assert_eq!(
            manager.extract_title_prefix("bugfix/DESIGN-456-button-color"),
            Some("DESIGN-456".to_string())
        );

        assert_eq!(
            manager.extract_title_prefix("issue-789-performance"),
            Some("Issue #789".to_string())
        );

        assert_eq!(manager.extract_title_prefix("random-branch-name"), None);
    }

    #[test]
    fn test_extract_team_from_branch() {
        let (manager, _temp) = create_test_manager();

        assert_eq!(
            manager.extract_team_from_branch("feature/ENG-123-fix-login"),
            Some("ENG".to_string())
        );

        assert_eq!(
            manager.extract_team_from_branch("bugfix/DESIGN-456-button-color"),
            Some("DESIGN".to_string())
        );

        assert_eq!(manager.extract_team_from_branch("random-branch-name"), None);
    }
}
