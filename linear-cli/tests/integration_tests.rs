// ABOUTME: End-to-end integration tests for config, aliases, and completions features
// ABOUTME: Tests the complete workflow from config loading through alias expansion to command execution

use linear_cli::aliases::AliasExpander;
use linear_cli::completions::{CompletionGenerator, Shell};
use linear_cli::config::Config;
use std::collections::HashMap;
use std::io::Cursor;
use tempfile::TempDir;

#[test]
fn test_config_and_alias_integration() {
    // Test that config loading and alias expansion work together
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let config_path = temp_dir.path().join("config.toml");

    // Create a config file with aliases
    let config_content = r#"
        default_team = "ENG"
        default_assignee = "test-user"
        preferred_format = "json"

        [aliases]
        my = ["issues", "--assignee", "me"]
        todo = ["issues", "--status", "todo"]
        standup = ["issues", "--team", "ENG", "--updated-after", "yesterday"]
    "#;

    std::fs::write(&config_path, config_content).expect("Should write config file");

    // Load config
    let config = Config::load_from_file(&config_path).expect("Should load config");

    // Verify config loaded correctly
    assert_eq!(config.default_team, Some("ENG".to_string()));
    assert_eq!(config.default_assignee, Some("test-user".to_string()));
    assert_eq!(config.preferred_format, Some("json".to_string()));

    // Test alias expansion
    let aliases = config.aliases.expect("Should have aliases");
    let expander = AliasExpander::new(aliases);

    // Test simple alias expansion
    let args = vec!["linear".to_string(), "my".to_string()];
    let expanded = expander.expand(args).expect("Should expand alias");
    assert_eq!(expanded, vec!["linear", "issues", "--assignee", "me"]);

    // Test alias with additional arguments
    let args = vec![
        "linear".to_string(),
        "todo".to_string(),
        "--limit".to_string(),
        "5".to_string(),
    ];
    let expanded = expander
        .expand(args)
        .expect("Should expand alias with args");
    assert_eq!(
        expanded,
        vec!["linear", "issues", "--status", "todo", "--limit", "5"]
    );

    // Test complex alias
    let args = vec!["linear".to_string(), "standup".to_string()];
    let expanded = expander.expand(args).expect("Should expand complex alias");
    assert_eq!(
        expanded,
        vec![
            "linear",
            "issues",
            "--team",
            "ENG",
            "--updated-after",
            "yesterday"
        ]
    );
}

#[test]
fn test_config_defaults_behavior() {
    // Test that config defaults work as expected
    let config_content = r#"
        default_team = "DESIGN"
        preferred_format = "table"
    "#;

    let config: Config = toml::from_str(config_content).expect("Should parse config");

    assert_eq!(config.default_team, Some("DESIGN".to_string()));
    assert_eq!(config.default_assignee, None);
    assert_eq!(config.preferred_format, Some("table".to_string()));
    assert!(config.aliases.is_none());
    assert!(config.completions.is_none());
}

#[test]
fn test_alias_recursion_prevention() {
    // Test that recursive aliases are properly detected
    let mut aliases = HashMap::new();
    aliases.insert("a".to_string(), vec!["b".to_string()]);
    aliases.insert("b".to_string(), vec!["c".to_string()]);
    aliases.insert("c".to_string(), vec!["a".to_string()]);

    let config_aliases = linear_cli::config::ConfigAliases { commands: aliases };
    let expander = AliasExpander::new(config_aliases);

    let args = vec!["linear".to_string(), "a".to_string()];
    let result = expander.expand(args);

    assert!(result.is_err(), "Should detect recursive alias");
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("recursive") || error_msg.contains("Recursive"),
        "Error should mention recursion: {}",
        error_msg
    );
}

#[test]
fn test_completions_integration() {
    // Test that completions work for all shells
    use clap::{CommandFactory, Parser, Subcommand};

    #[derive(Parser)]
    #[command(name = "linear", version = "1.0.0")]
    struct TestCli {
        #[command(subcommand)]
        command: TestCommands,
    }

    #[derive(Subcommand)]
    enum TestCommands {
        Issues,
        Completions {
            #[arg(value_enum)]
            shell: Shell,
        },
    }

    let generator = CompletionGenerator::new();

    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
        let mut output = Cursor::new(Vec::new());
        let mut cmd = TestCli::command();

        generator
            .generate(shell, &mut cmd, &mut output)
            .expect("Should generate completions for all shells");

        let output_str = String::from_utf8(output.into_inner()).expect("Should be valid UTF-8");

        assert!(
            !output_str.is_empty(),
            "Output should not be empty for {}",
            shell
        );

        // Each shell should have its specific syntax
        match shell {
            Shell::Bash => {
                assert!(output_str.contains("COMPREPLY") && output_str.contains("complete"));
            }
            Shell::Zsh => {
                assert!(output_str.contains("#compdef"));
            }
            Shell::Fish => {
                assert!(output_str.contains("complete -c"));
            }
            Shell::PowerShell => {
                assert!(output_str.contains("Register-ArgumentCompleter"));
            }
        }
    }
}

#[test]
fn test_config_hierarchical_loading() {
    // Test that config files are loaded in the correct precedence order
    let temp_dir = TempDir::new().expect("Should create temp dir");

    // Create base config
    let base_config_path = temp_dir.path().join("base.toml");
    std::fs::write(
        &base_config_path,
        r#"
        default_team = "BASE"
        default_assignee = "base-user"
        preferred_format = "json"
    "#,
    )
    .expect("Should write base config");

    // Create override config
    let override_config_path = temp_dir.path().join("override.toml");
    std::fs::write(
        &override_config_path,
        r#"
        default_team = "OVERRIDE"
        api_url = "https://custom.api.com"
    "#,
    )
    .expect("Should write override config");

    // Load in precedence order (base first, override second)
    let config = Config::load_from_paths(&[
        base_config_path.to_str().unwrap(),
        override_config_path.to_str().unwrap(),
    ])
    .expect("Should load config hierarchy");

    // Override values should take precedence
    assert_eq!(config.default_team, Some("OVERRIDE".to_string()));
    assert_eq!(config.api_url, Some("https://custom.api.com".to_string()));

    // Base values should be preserved when not overridden
    assert_eq!(config.default_assignee, Some("base-user".to_string()));
    assert_eq!(config.preferred_format, Some("json".to_string()));
}

#[test]
fn test_alias_with_special_characters() {
    // Test that aliases work with special characters and spaces
    let config_content = r#"
        [aliases]
        search-bugs = ["issues", "--search", "bug OR critical"]
        my-urgent = ["issues", "--assignee", "me", "--priority", "1"]
    "#;

    let config: Config = toml::from_str(config_content).expect("Should parse config");
    let aliases = config.aliases.expect("Should have aliases");
    let expander = AliasExpander::new(aliases);

    // Test alias with special characters in arguments
    let args = vec!["linear".to_string(), "search-bugs".to_string()];
    let expanded = expander
        .expand(args)
        .expect("Should expand alias with special chars");
    assert_eq!(
        expanded,
        vec!["linear", "issues", "--search", "bug OR critical"]
    );

    // Test alias with hyphenated name
    let args = vec!["linear".to_string(), "my-urgent".to_string()];
    let expanded = expander
        .expand(args)
        .expect("Should expand hyphenated alias");
    assert_eq!(
        expanded,
        vec!["linear", "issues", "--assignee", "me", "--priority", "1"]
    );
}

#[test]
fn test_config_validation_errors() {
    // Test various config validation scenarios

    // Invalid format
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

    // Invalid TOML syntax
    let invalid_toml = r#"
        default_team = "ENG"
        [invalid
    "#;
    let result: Result<Config, _> = toml::from_str(invalid_toml);
    assert!(result.is_err(), "Should reject invalid TOML");
}

#[test]
fn test_empty_config_handling() {
    // Test that empty or minimal configs work correctly
    let empty_config = "";
    let config: Config = toml::from_str(empty_config).expect("Should parse empty config");

    assert!(config.default_team.is_none());
    assert!(config.default_assignee.is_none());
    assert!(config.preferred_format.is_none());
    assert!(config.api_url.is_none());
    assert!(config.aliases.is_none());
    assert!(config.completions.is_none());

    // Test with only aliases
    let alias_only_config = r#"
        [aliases]
        my = ["issues", "--assignee", "me"]
    "#;
    let config: Config = toml::from_str(alias_only_config).expect("Should parse alias-only config");

    assert!(config.aliases.is_some());
    assert!(config.default_team.is_none());
}
