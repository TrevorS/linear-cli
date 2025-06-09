// ABOUTME: Command alias expansion system with recursion detection and cycle prevention
// ABOUTME: Expands user-defined command aliases before CLI argument parsing

use crate::config::ConfigAliases;
use anyhow::{Result, anyhow};
use std::collections::HashSet;

const MAX_EXPANSION_DEPTH: usize = 10;

pub struct AliasExpander {
    aliases: ConfigAliases,
}

impl AliasExpander {
    /// Create a new alias expander with the given aliases configuration
    pub fn new(aliases: ConfigAliases) -> Self {
        Self { aliases }
    }

    /// Expand aliases in the command arguments
    ///
    /// Takes a full command line (including program name) and expands any aliases found.
    /// Returns the expanded command line or an error if recursion is detected.
    pub fn expand(&self, mut args: Vec<String>) -> Result<Vec<String>> {
        if args.len() < 2 {
            return Ok(args); // No command to expand
        }

        let mut expansion_history = HashSet::new();
        let mut depth = 0;

        // Keep expanding until no more aliases are found or limits are hit
        loop {
            if depth >= MAX_EXPANSION_DEPTH {
                return Err(anyhow!(
                    "Maximum alias expansion depth exceeded ({})",
                    MAX_EXPANSION_DEPTH
                ));
            }

            let command = &args[1]; // Skip program name

            // Check if this command is an alias
            if let Some(alias_args) = self.aliases.expand(command) {
                // Check for recursion
                if expansion_history.contains(command) {
                    return Err(anyhow!("Recursive alias detected: {}", command));
                }

                expansion_history.insert(command.clone());

                // Replace the command with its expansion
                let mut new_args = vec![args[0].clone()]; // Keep program name
                new_args.extend(alias_args.clone()); // Add alias expansion
                new_args.extend_from_slice(&args[2..]); // Add remaining args

                args = new_args;
                depth += 1;
            } else {
                // No alias found, we're done
                break;
            }
        }

        Ok(args)
    }

    /// Check if a command is an alias
    #[allow(dead_code)]
    pub fn is_alias(&self, command: &str) -> bool {
        self.aliases.expand(command).is_some()
    }

    /// Get the raw alias definition for a command
    #[allow(dead_code)]
    pub fn get_alias(&self, command: &str) -> Option<&Vec<String>> {
        self.aliases.expand(command)
    }
}

/// Expand aliases in command line arguments from environment
///
/// This is the main entry point for alias expansion, taking command line arguments
/// from std::env::args() and returning the expanded version.
#[allow(dead_code)]
pub fn expand_aliases_from_env(aliases: ConfigAliases) -> Result<Vec<String>> {
    let args: Vec<String> = std::env::args().collect();
    let expander = AliasExpander::new(aliases);
    expander.expand(args)
}

/// Expand aliases in a command line string
///
/// Splits the command line and expands aliases, returning the result as a string.
#[allow(dead_code)]
pub fn expand_aliases_from_string(aliases: ConfigAliases, command_line: &str) -> Result<String> {
    let args: Vec<String> = command_line
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    let expander = AliasExpander::new(aliases);
    let expanded = expander.expand(args)?;
    Ok(expanded.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_aliases() -> ConfigAliases {
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
            "todo".to_string(),
            vec![
                "issues".to_string(),
                "--status".to_string(),
                "todo".to_string(),
            ],
        );
        ConfigAliases { commands: aliases }
    }

    #[test]
    fn test_simple_expansion() {
        let aliases = create_test_aliases();
        let expander = AliasExpander::new(aliases);

        let args = vec!["linear".to_string(), "my".to_string()];
        let result = expander.expand(args).unwrap();

        assert_eq!(result, vec!["linear", "issues", "--assignee", "me"]);
    }

    #[test]
    fn test_no_expansion_needed() {
        let aliases = create_test_aliases();
        let expander = AliasExpander::new(aliases);

        let args = vec!["linear".to_string(), "issues".to_string()];
        let result = expander.expand(args).unwrap();

        assert_eq!(result, vec!["linear", "issues"]);
    }

    #[test]
    fn test_is_alias() {
        let aliases = create_test_aliases();
        let expander = AliasExpander::new(aliases);

        assert!(expander.is_alias("my"));
        assert!(expander.is_alias("todo"));
        assert!(!expander.is_alias("issues"));
        assert!(!expander.is_alias("nonexistent"));
    }

    #[test]
    fn test_get_alias() {
        let aliases = create_test_aliases();
        let expander = AliasExpander::new(aliases);

        let alias_def = expander.get_alias("my").unwrap();
        assert_eq!(alias_def, &vec!["issues", "--assignee", "me"]);

        assert!(expander.get_alias("nonexistent").is_none());
    }

    #[test]
    fn test_expand_from_string() {
        let aliases = create_test_aliases();
        let result = expand_aliases_from_string(aliases, "linear my --limit 10").unwrap();
        assert_eq!(result, "linear issues --assignee me --limit 10");
    }
}
