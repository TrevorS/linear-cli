// ABOUTME: Comprehensive tests for configuration file loading, validation, and merging
// ABOUTME: Tests TOML parsing, XDG path resolution, and hierarchical config merging

use linear_cli::config::{Config, ConfigAliases, ConfigCompletions};
use std::collections::HashMap;
use tempfile::TempDir;

#[test]
fn test_config_deserialize_complete() {
    let toml_content = r#"
        default_team = "ENG"
        default_assignee = "me"
        preferred_format = "table"
        api_url = "https://api.linear.app/graphql"

        [aliases]
        my = ["issues", "--assignee", "me"]
        todo = ["issues", "--status", "todo", "--assignee", "me"]
        standup = ["issues", "--team", "ENG", "--updated-after", "yesterday"]

        [completions]
        cache_duration = "1h"
        enable_dynamic = true
    "#;

    let config: Config = toml::from_str(toml_content).expect("Should parse valid TOML");

    assert_eq!(config.default_team, Some("ENG".to_string()));
    assert_eq!(config.default_assignee, Some("me".to_string()));
    assert_eq!(config.preferred_format, Some("table".to_string()));
    assert_eq!(
        config.api_url,
        Some("https://api.linear.app/graphql".to_string())
    );

    let aliases = config.aliases.unwrap();
    assert_eq!(
        aliases.commands.get("my"),
        Some(&vec![
            "issues".to_string(),
            "--assignee".to_string(),
            "me".to_string()
        ])
    );
    assert_eq!(
        aliases.commands.get("todo"),
        Some(&vec![
            "issues".to_string(),
            "--status".to_string(),
            "todo".to_string(),
            "--assignee".to_string(),
            "me".to_string()
        ])
    );

    let completions = config.completions.unwrap();
    assert_eq!(completions.cache_duration, Some("1h".to_string()));
    assert_eq!(completions.enable_dynamic, Some(true));
}

#[test]
fn test_config_deserialize_minimal() {
    let toml_content = r#"
        default_team = "ENG"
    "#;

    let config: Config = toml::from_str(toml_content).expect("Should parse minimal TOML");

    assert_eq!(config.default_team, Some("ENG".to_string()));
    assert_eq!(config.default_assignee, None);
    assert_eq!(config.preferred_format, None);
    assert_eq!(config.api_url, None);
    assert!(config.aliases.is_none());
    assert!(config.completions.is_none());
}

#[test]
fn test_config_deserialize_empty() {
    let toml_content = "";

    let config: Config = toml::from_str(toml_content).expect("Should parse empty TOML");

    assert_eq!(config.default_team, None);
    assert_eq!(config.default_assignee, None);
    assert_eq!(config.preferred_format, None);
    assert_eq!(config.api_url, None);
    assert!(config.aliases.is_none());
    assert!(config.completions.is_none());
}

#[test]
fn test_config_validation_errors() {
    // Invalid format value
    let invalid_format = r#"
        preferred_format = "invalid"
    "#;

    let result: Result<Config, _> = toml::from_str(invalid_format);
    assert!(result.is_err(), "Should reject invalid format");

    // Invalid cache duration
    let invalid_duration = r#"
        [completions]
        cache_duration = "invalid"
    "#;

    let result: Result<Config, _> = toml::from_str(invalid_duration);
    assert!(result.is_err(), "Should reject invalid cache duration");
}

#[test]
fn test_config_merge_precedence() {
    let base_config = Config {
        default_team: Some("BASE".to_string()),
        default_assignee: Some("base_user".to_string()),
        preferred_format: Some("json".to_string()),
        api_url: None,
        aliases: Some(ConfigAliases {
            commands: {
                let mut map = HashMap::new();
                map.insert("base".to_string(), vec!["issues".to_string()]);
                map
            },
        }),
        completions: None,
    };

    let override_config = Config {
        default_team: Some("OVERRIDE".to_string()),
        default_assignee: None,
        preferred_format: None,
        api_url: Some("https://custom.api.com".to_string()),
        aliases: Some(ConfigAliases {
            commands: {
                let mut map = HashMap::new();
                map.insert("override".to_string(), vec!["my-issues".to_string()]);
                map
            },
        }),
        completions: Some(ConfigCompletions {
            cache_duration: Some("30m".to_string()),
            enable_dynamic: Some(false),
        }),
    };

    let merged = base_config.merge(override_config);

    // Override values should take precedence
    assert_eq!(merged.default_team, Some("OVERRIDE".to_string()));
    assert_eq!(merged.api_url, Some("https://custom.api.com".to_string()));

    // Base values should be preserved when not overridden
    assert_eq!(merged.default_assignee, Some("base_user".to_string()));
    assert_eq!(merged.preferred_format, Some("json".to_string()));

    // Aliases should be merged, not replaced
    let aliases = merged.aliases.unwrap();
    assert_eq!(aliases.commands.len(), 2);
    assert!(aliases.commands.contains_key("base"));
    assert!(aliases.commands.contains_key("override"));

    // Completions from override should be present
    let completions = merged.completions.unwrap();
    assert_eq!(completions.cache_duration, Some("30m".to_string()));
    assert_eq!(completions.enable_dynamic, Some(false));
}

#[test]
fn test_config_load_hierarchy() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let config_dir = temp_dir.path().join(".config").join("linear-cli");
    std::fs::create_dir_all(&config_dir).expect("Should create config dir");

    // Create user config
    let user_config_path = config_dir.join("config.toml");
    std::fs::write(
        &user_config_path,
        r#"
        default_team = "USER_TEAM"
        default_assignee = "user"
        preferred_format = "table"
    "#,
    )
    .expect("Should write user config");

    // Create project config
    let project_config_path = temp_dir.path().join("linear-cli.toml");
    std::fs::write(
        &project_config_path,
        r#"
        default_team = "PROJECT_TEAM"
        api_url = "https://project.api.com"
    "#,
    )
    .expect("Should write project config");

    let config = Config::load_from_paths(&[
        user_config_path.to_str().unwrap(),
        project_config_path.to_str().unwrap(),
    ])
    .expect("Should load config hierarchy");

    // Project config should override user config
    assert_eq!(config.default_team, Some("PROJECT_TEAM".to_string()));
    assert_eq!(config.api_url, Some("https://project.api.com".to_string()));

    // User config values should be preserved when not overridden
    assert_eq!(config.default_assignee, Some("user".to_string()));
    assert_eq!(config.preferred_format, Some("table".to_string()));
}

#[test]
fn test_config_alias_validation() {
    // Valid aliases
    let valid_aliases = r#"
        [aliases]
        my = ["issues", "--assignee", "me"]
        todo = ["issues", "--status", "todo"]
    "#;

    let config: Config = toml::from_str(valid_aliases).expect("Should parse valid aliases");
    let aliases = config.aliases.unwrap();

    assert!(
        aliases.validate().is_ok(),
        "Valid aliases should pass validation"
    );

    // Invalid alias (recursive)
    let mut recursive_aliases = HashMap::new();
    recursive_aliases.insert("recursive".to_string(), vec!["recursive".to_string()]);

    let invalid_config = ConfigAliases {
        commands: recursive_aliases,
    };

    assert!(
        invalid_config.validate().is_err(),
        "Recursive aliases should fail validation"
    );
}

#[test]
fn test_config_xdg_paths() {
    let paths = Config::get_config_paths();

    // Should include project config as highest priority
    assert!(paths.iter().any(|p| p.ends_with("linear-cli.toml")));

    // Should include XDG config home
    assert!(
        paths
            .iter()
            .any(|p| p.contains("linear-cli") && p.ends_with("config.toml"))
    );
}

#[test]
fn test_config_error_messages() {
    let invalid_toml = r#"
        default_team = "ENG"
        [invalid
    "#;

    let result: Result<Config, _> = toml::from_str(invalid_toml);
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("TOML"),
        "Error should mention TOML format issue"
    );
}
