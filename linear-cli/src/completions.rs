// ABOUTME: Shell completion generation using clap_complete for all supported shells
// ABOUTME: Provides static completions for bash, zsh, fish, and powershell

use anyhow::{anyhow, Result};
use clap::{Command, ValueEnum};
use clap_complete::{generate, shells};
use std::fmt;
use std::io::Write;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    #[allow(clippy::enum_variant_names)]
    PowerShell,
}

impl Shell {
    /// Get all supported shell variants
    #[allow(dead_code)]
    pub fn all() -> Vec<Shell> {
        vec![Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell]
    }
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shell_str = match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::PowerShell => "powershell",
        };
        write!(f, "{}", shell_str)
    }
}

impl FromStr for Shell {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(Shell::Bash),
            "zsh" => Ok(Shell::Zsh),
            "fish" => Ok(Shell::Fish),
            "powershell" | "pwsh" => Ok(Shell::PowerShell),
            _ => Err(anyhow!(
                "Unsupported shell: {}. Supported shells: bash, zsh, fish, powershell",
                s
            )),
        }
    }
}

pub struct CompletionGenerator {
    #[allow(dead_code)]
    cmd: Option<Command>,
}

impl CompletionGenerator {
    /// Create a new completion generator without a command
    pub fn new() -> Self {
        Self { cmd: None }
    }

    /// Create a completion generator with a specific command
    #[allow(dead_code)]
    pub fn with_command(cmd: Command) -> Self {
        Self { cmd: Some(cmd) }
    }

    /// Generate completion script for the specified shell
    pub fn generate<W: Write>(
        &self,
        shell: Shell,
        cmd: &mut Command,
        writer: &mut W,
    ) -> Result<()> {
        match shell {
            Shell::Bash => {
                generate(shells::Bash, cmd, "linear", writer);
            }
            Shell::Zsh => {
                generate(shells::Zsh, cmd, "linear", writer);
            }
            Shell::Fish => {
                generate(shells::Fish, cmd, "linear", writer);
            }
            Shell::PowerShell => {
                generate(shells::PowerShell, cmd, "linear", writer);
            }
        }

        Ok(())
    }

    /// Get installation instructions for shell completions
    #[allow(dead_code)]
    pub fn installation_instructions() -> String {
        r#"Shell Completion Installation

The completions subcommand outputs shell completion code to stdout. To install:

Bash:
  Linux: linear completions bash > ~/.local/share/bash-completion/completions/linear
  macOS: linear completions bash > $(brew --prefix)/etc/bash_completion.d/linear

Zsh:
  linear completions zsh > ~/.zfunc/_linear
  # Add ~/.zfunc to $fpath in your ~/.zshrc:
  # fpath=(~/.zfunc $fpath)

Fish:
  linear completions fish > ~/.config/fish/completions/linear.fish

PowerShell:
  linear completions powershell > linear_completions.ps1
  # Then source it in your PowerShell profile

Examples:
  linear completions bash                    # Output bash completions
  linear completions zsh > ~/.zfunc/_linear # Install zsh completions
  linear completions --help                 # Show this help

Note: You may need to restart your shell or source the completion file.
"#
        .to_string()
    }
}

impl Default for CompletionGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_shell_parsing() {
        assert_eq!("bash".parse::<Shell>().unwrap(), Shell::Bash);
        assert_eq!("zsh".parse::<Shell>().unwrap(), Shell::Zsh);
        assert_eq!("fish".parse::<Shell>().unwrap(), Shell::Fish);
        assert_eq!("powershell".parse::<Shell>().unwrap(), Shell::PowerShell);
        assert_eq!("pwsh".parse::<Shell>().unwrap(), Shell::PowerShell);

        assert!("invalid".parse::<Shell>().is_err());
    }

    #[test]
    fn test_shell_display() {
        assert_eq!(Shell::Bash.to_string(), "bash");
        assert_eq!(Shell::Zsh.to_string(), "zsh");
        assert_eq!(Shell::Fish.to_string(), "fish");
        assert_eq!(Shell::PowerShell.to_string(), "powershell");
    }

    #[test]
    fn test_shell_all() {
        let shells = Shell::all();
        assert_eq!(shells.len(), 4);
        assert!(shells.contains(&Shell::Bash));
        assert!(shells.contains(&Shell::Zsh));
        assert!(shells.contains(&Shell::Fish));
        assert!(shells.contains(&Shell::PowerShell));
    }

    #[test]
    fn test_completion_generator() {
        use clap::{CommandFactory, Parser, Subcommand};

        #[derive(Parser)]
        #[command(name = "test")]
        struct TestCli {
            #[command(subcommand)]
            command: TestCommands,
        }

        #[derive(Subcommand)]
        enum TestCommands {
            Test,
        }

        let generator = CompletionGenerator::new();

        for shell in Shell::all() {
            let mut output = Cursor::new(Vec::new());
            let mut cmd = TestCli::command();
            generator
                .generate(shell, &mut cmd, &mut output)
                .expect("Should generate completions");

            let output_str = String::from_utf8(output.into_inner()).expect("Should be valid UTF-8");
            assert!(!output_str.is_empty());
        }
    }

    #[test]
    fn test_installation_instructions() {
        let instructions = CompletionGenerator::installation_instructions();
        assert!(instructions.contains("bash"));
        assert!(instructions.contains("zsh"));
        assert!(instructions.contains("fish"));
        assert!(instructions.contains("powershell"));
    }
}
