// ABOUTME: Configuration file loading, validation, and hierarchical merging for Linear CLI
// ABOUTME: Supports TOML config files with XDG Base Directory specification compliance

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub default_team: Option<String>,
    #[serde(default)]
    pub default_assignee: Option<String>,
    #[serde(default, deserialize_with = "validate_format")]
    pub preferred_format: Option<String>,
    #[serde(default)]
    pub api_url: Option<String>,
    #[serde(default)]
    pub aliases: Option<ConfigAliases>,
    #[serde(default)]
    pub completions: Option<ConfigCompletions>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ConfigAliases {
    #[serde(flatten)]
    pub commands: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ConfigCompletions {
    #[serde(default, deserialize_with = "validate_duration")]
    pub cache_duration: Option<String>,
    #[serde(default)]
    pub enable_dynamic: Option<bool>,
}

impl Config {
    /// Load configuration from standard XDG-compliant locations
    pub fn load() -> Result<Self> {
        let paths = Self::get_config_paths();
        Self::load_from_paths(&paths.iter().map(|p| p.as_str()).collect::<Vec<_>>())
    }

    /// Load configuration from specific file paths in order of precedence
    pub fn load_from_paths(paths: &[&str]) -> Result<Self> {
        let mut config = Config::default();

        for path in paths {
            // Apply in order - later paths override earlier ones
            if let Ok(file_config) = Self::load_from_file(path) {
                config = config.merge(file_config);
            }
        }

        config.validate()?;
        Ok(config)
    }

    /// Load configuration from a single file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        let config: Config = toml::from_str(&content).with_context(|| {
            format!(
                "Failed to parse TOML config file: {}",
                path.as_ref().display()
            )
        })?;

        config.validate()?;
        Ok(config)
    }

    /// Get standard config file paths in order of precedence (highest first)
    pub fn get_config_paths() -> Vec<String> {
        let mut paths = Vec::new();

        // 1. Project-specific config (highest precedence)
        if let Ok(current_dir) = std::env::current_dir() {
            paths.push(
                current_dir
                    .join("linear-cli.toml")
                    .to_string_lossy()
                    .to_string(),
            );
        }

        // 2. XDG config home
        if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
            let path = PathBuf::from(config_home)
                .join("linear-cli")
                .join("config.toml");
            paths.push(path.to_string_lossy().to_string());
        }

        // 3. User config directory fallback
        if let Some(home_dir) = dirs::home_dir() {
            let path = home_dir
                .join(".config")
                .join("linear-cli")
                .join("config.toml");
            paths.push(path.to_string_lossy().to_string());
        }

        paths
    }

    /// Merge this config with another, giving precedence to the other config
    pub fn merge(self, other: Config) -> Config {
        Config {
            default_team: other.default_team.or(self.default_team),
            default_assignee: other.default_assignee.or(self.default_assignee),
            preferred_format: other.preferred_format.or(self.preferred_format),
            api_url: other.api_url.or(self.api_url),
            aliases: match (self.aliases, other.aliases) {
                (Some(base), Some(other)) => Some(base.merge(other)),
                (Some(base), None) => Some(base),
                (None, Some(other)) => Some(other),
                (None, None) => None,
            },
            completions: other.completions.or(self.completions),
        }
    }

    /// Validate the entire configuration
    pub fn validate(&self) -> Result<()> {
        if let Some(ref aliases) = self.aliases {
            aliases.validate().context("Invalid alias configuration")?;
        }

        Ok(())
    }
}

impl ConfigAliases {
    /// Merge aliases, combining command maps
    pub fn merge(mut self, other: ConfigAliases) -> ConfigAliases {
        self.commands.extend(other.commands);
        self
    }

    /// Validate alias configuration
    pub fn validate(&self) -> Result<()> {
        // Check for recursive aliases
        for (alias_name, command_args) in &self.commands {
            if command_args.first() == Some(alias_name) {
                return Err(anyhow!("Recursive alias detected: {}", alias_name));
            }
        }

        // TODO: More sophisticated cycle detection for indirect recursion
        Ok(())
    }

    /// Expand an alias into its component arguments
    pub fn expand(&self, alias: &str) -> Option<&Vec<String>> {
        self.commands.get(alias)
    }
}

// Custom deserializer for format validation
fn validate_format<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    // Handle the case where the field might not be present
    let value: Option<Option<String>> = Option::deserialize(deserializer).ok();
    let value = value.flatten();

    if let Some(ref format) = value {
        match format.as_str() {
            "table" | "json" | "yaml" => Ok(value),
            _ => Err(D::Error::custom(format!(
                "Invalid format '{}'. Must be one of: table, json, yaml",
                format
            ))),
        }
    } else {
        Ok(None)
    }
}

// Custom deserializer for duration validation
fn validate_duration<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    // Handle the case where the field might not be present
    let value: Option<Option<String>> = Option::deserialize(deserializer).ok();
    let value = value.flatten();

    if let Some(ref duration) = value {
        // Simple validation for common duration formats
        if duration.ends_with('s')
            || duration.ends_with('m')
            || duration.ends_with('h')
            || duration.ends_with('d')
        {
            // Extract the numeric part and validate it's a number
            let numeric_part = &duration[..duration.len() - 1];
            if numeric_part.parse::<u32>().is_ok() {
                Ok(value)
            } else {
                Err(D::Error::custom(format!(
                    "Invalid duration format '{}'. Expected format like '30m', '1h', '2d'",
                    duration
                )))
            }
        } else {
            Err(D::Error::custom(format!(
                "Invalid duration format '{}'. Must end with s, m, h, or d",
                duration
            )))
        }
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.default_team.is_none());
        assert!(config.aliases.is_none());
    }

    #[test]
    fn test_merge_configs() {
        let base = Config {
            default_team: Some("BASE".to_string()),
            default_assignee: Some("base_user".to_string()),
            ..Default::default()
        };

        let override_config = Config {
            default_team: Some("OVERRIDE".to_string()),
            api_url: Some("https://custom.api.com".to_string()),
            ..Default::default()
        };

        let merged = base.merge(override_config);
        assert_eq!(merged.default_team, Some("OVERRIDE".to_string()));
        assert_eq!(merged.default_assignee, Some("base_user".to_string()));
        assert_eq!(merged.api_url, Some("https://custom.api.com".to_string()));
    }
}
