// ABOUTME: Tests for command alias expansion and validation functionality
// ABOUTME: Ensures aliases work correctly and prevents infinite recursion

use linear_cli::aliases::AliasExpander;
use linear_cli::config::ConfigAliases;
use std::collections::HashMap;

#[test]
fn test_alias_expansion_simple() {
    let mut aliases = HashMap::new();
    aliases.insert(
        "my".to_string(),
        vec![
            "issues".to_string(),
            "--assignee".to_string(),
            "me".to_string(),
        ],
    );

    let config_aliases = ConfigAliases { commands: aliases };
    let expander = AliasExpander::new(config_aliases);

    let input = vec!["linear".to_string(), "my".to_string()];
    let result = expander.expand(input).expect("Should expand alias");

    assert_eq!(result, vec!["linear", "issues", "--assignee", "me"]);
}

#[test]
fn test_alias_expansion_with_args() {
    let mut aliases = HashMap::new();
    aliases.insert(
        "todo".to_string(),
        vec![
            "issues".to_string(),
            "--status".to_string(),
            "todo".to_string(),
        ],
    );

    let config_aliases = ConfigAliases { commands: aliases };
    let expander = AliasExpander::new(config_aliases);

    let input = vec![
        "linear".to_string(),
        "todo".to_string(),
        "--team".to_string(),
        "ENG".to_string(),
    ];
    let result = expander.expand(input).expect("Should expand alias");

    assert_eq!(
        result,
        vec!["linear", "issues", "--status", "todo", "--team", "ENG"]
    );
}

#[test]
fn test_alias_expansion_multiple_levels() {
    let mut aliases = HashMap::new();
    aliases.insert(
        "my".to_string(),
        vec![
            "issues".to_string(),
            "--assignee".to_string(),
            "me".to_string(),
        ],
    );
    aliases.insert(
        "mytodo".to_string(),
        vec!["my".to_string(), "--status".to_string(), "todo".to_string()],
    );

    let config_aliases = ConfigAliases { commands: aliases };
    let expander = AliasExpander::new(config_aliases);

    let input = vec!["linear".to_string(), "mytodo".to_string()];
    let result = expander
        .expand(input)
        .expect("Should expand nested aliases");

    assert_eq!(
        result,
        vec!["linear", "issues", "--assignee", "me", "--status", "todo"]
    );
}

#[test]
fn test_alias_expansion_no_alias() {
    let aliases = HashMap::new();
    let config_aliases = ConfigAliases { commands: aliases };
    let expander = AliasExpander::new(config_aliases);

    let input = vec![
        "linear".to_string(),
        "issues".to_string(),
        "--assignee".to_string(),
        "me".to_string(),
    ];
    let result = expander
        .expand(input)
        .expect("Should handle non-alias commands");

    assert_eq!(result, vec!["linear", "issues", "--assignee", "me"]);
}

#[test]
fn test_alias_recursive_detection() {
    let mut aliases = HashMap::new();
    aliases.insert("recursive".to_string(), vec!["recursive".to_string()]);

    let config_aliases = ConfigAliases { commands: aliases };
    let expander = AliasExpander::new(config_aliases);

    let input = vec!["linear".to_string(), "recursive".to_string()];
    let result = expander.expand(input);

    assert!(result.is_err(), "Should detect recursive alias");
    assert!(result.unwrap_err().to_string().contains("recursive"));
}

#[test]
fn test_alias_indirect_recursion() {
    let mut aliases = HashMap::new();
    aliases.insert("a".to_string(), vec!["b".to_string()]);
    aliases.insert("b".to_string(), vec!["c".to_string()]);
    aliases.insert("c".to_string(), vec!["a".to_string()]);

    let config_aliases = ConfigAliases { commands: aliases };
    let expander = AliasExpander::new(config_aliases);

    let input = vec!["linear".to_string(), "a".to_string()];
    let result = expander.expand(input);

    assert!(result.is_err(), "Should detect indirect recursive alias");
}

#[test]
fn test_alias_max_depth_limit() {
    let mut aliases = HashMap::new();
    // Create a long chain that doesn't recurse but exceeds reasonable depth
    for i in 0..20 {
        aliases.insert(format!("alias{}", i), vec![format!("alias{}", i + 1)]);
    }
    aliases.insert("alias20".to_string(), vec!["issues".to_string()]);

    let config_aliases = ConfigAliases { commands: aliases };
    let expander = AliasExpander::new(config_aliases);

    let input = vec!["linear".to_string(), "alias0".to_string()];
    let result = expander.expand(input);

    assert!(result.is_err(), "Should limit maximum expansion depth");
}

#[test]
fn test_alias_expansion_preserves_order() {
    let mut aliases = HashMap::new();
    aliases.insert(
        "standup".to_string(),
        vec![
            "issues".to_string(),
            "--team".to_string(),
            "ENG".to_string(),
            "--updated-after".to_string(),
            "yesterday".to_string(),
        ],
    );

    let config_aliases = ConfigAliases { commands: aliases };
    let expander = AliasExpander::new(config_aliases);

    let input = vec![
        "linear".to_string(),
        "standup".to_string(),
        "--limit".to_string(),
        "10".to_string(),
    ];
    let result = expander.expand(input).expect("Should expand alias");

    assert_eq!(
        result,
        vec![
            "linear",
            "issues",
            "--team",
            "ENG",
            "--updated-after",
            "yesterday",
            "--limit",
            "10"
        ]
    );
}

#[test]
fn test_alias_expansion_empty_alias() {
    let mut aliases = HashMap::new();
    aliases.insert("empty".to_string(), vec![]);

    let config_aliases = ConfigAliases { commands: aliases };
    let expander = AliasExpander::new(config_aliases);

    let input = vec![
        "linear".to_string(),
        "empty".to_string(),
        "issues".to_string(),
    ];
    let result = expander.expand(input).expect("Should handle empty alias");

    assert_eq!(result, vec!["linear", "issues"]);
}

#[test]
fn test_alias_expansion_with_special_characters() {
    let mut aliases = HashMap::new();
    aliases.insert(
        "search".to_string(),
        vec![
            "issues".to_string(),
            "--search".to_string(),
            "bug OR critical".to_string(),
        ],
    );

    let config_aliases = ConfigAliases { commands: aliases };
    let expander = AliasExpander::new(config_aliases);

    let input = vec!["linear".to_string(), "search".to_string()];
    let result = expander
        .expand(input)
        .expect("Should handle special characters");

    assert_eq!(
        result,
        vec!["linear", "issues", "--search", "bug OR critical"]
    );
}

#[test]
fn test_alias_case_sensitivity() {
    let mut aliases = HashMap::new();
    aliases.insert(
        "My".to_string(),
        vec![
            "issues".to_string(),
            "--assignee".to_string(),
            "me".to_string(),
        ],
    );

    let config_aliases = ConfigAliases { commands: aliases };
    let expander = AliasExpander::new(config_aliases);

    // Test exact case match
    let input1 = vec!["linear".to_string(), "My".to_string()];
    let result1 = expander
        .expand(input1)
        .expect("Should expand exact case match");
    assert_eq!(result1, vec!["linear", "issues", "--assignee", "me"]);

    // Test different case - should not match
    let input2 = vec!["linear".to_string(), "my".to_string()];
    let result2 = expander
        .expand(input2)
        .expect("Should not expand different case");
    assert_eq!(result2, vec!["linear", "my"]);
}
