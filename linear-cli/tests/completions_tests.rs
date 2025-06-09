// ABOUTME: Tests for shell completion generation functionality
// ABOUTME: Ensures completions work correctly for all supported shells

use clap::{CommandFactory, Parser, Subcommand};
use linear_cli::completions::{CompletionGenerator, Shell};
use std::io::Cursor;

// Create a mock CLI structure for testing
#[derive(Parser)]
#[command(name = "linear", version = "1.0.0")]
struct MockCli {
    #[command(subcommand)]
    command: MockCommands,
}

#[derive(Subcommand)]
enum MockCommands {
    Issues,
    Issue { id: String },
    Create,
}

#[test]
fn test_completion_shell_parsing() {
    assert_eq!("bash".parse::<Shell>().unwrap(), Shell::Bash);
    assert_eq!("zsh".parse::<Shell>().unwrap(), Shell::Zsh);
    assert_eq!("fish".parse::<Shell>().unwrap(), Shell::Fish);
    assert_eq!("powershell".parse::<Shell>().unwrap(), Shell::PowerShell);

    // Test case insensitive
    assert_eq!("BASH".parse::<Shell>().unwrap(), Shell::Bash);
    assert_eq!("Zsh".parse::<Shell>().unwrap(), Shell::Zsh);

    // Test invalid shell
    assert!("invalid".parse::<Shell>().is_err());
}

#[test]
fn test_bash_completion_generation() {
    let mut output = Cursor::new(Vec::new());
    let generator = CompletionGenerator::new();
    let mut cmd = MockCli::command();

    generator
        .generate(Shell::Bash, &mut cmd, &mut output)
        .expect("Should generate bash completions");

    let output_str = String::from_utf8(output.into_inner()).expect("Should be valid UTF-8");

    // Check for bash-specific completion syntax
    assert!(output_str.contains("_linear"));
    assert!(output_str.contains("complete"));
    assert!(output_str.contains("COMPREPLY"));
}

#[test]
fn test_zsh_completion_generation() {
    let mut output = Cursor::new(Vec::new());
    let generator = CompletionGenerator::new();
    let mut cmd = MockCli::command();

    generator
        .generate(Shell::Zsh, &mut cmd, &mut output)
        .expect("Should generate zsh completions");

    let output_str = String::from_utf8(output.into_inner()).expect("Should be valid UTF-8");

    // Check for zsh-specific completion syntax
    assert!(output_str.contains("#compdef"));
    assert!(output_str.contains("_linear"));
}

#[test]
fn test_fish_completion_generation() {
    let mut output = Cursor::new(Vec::new());
    let generator = CompletionGenerator::new();
    let mut cmd = MockCli::command();

    generator
        .generate(Shell::Fish, &mut cmd, &mut output)
        .expect("Should generate fish completions");

    let output_str = String::from_utf8(output.into_inner()).expect("Should be valid UTF-8");

    // Check for fish-specific completion syntax
    assert!(output_str.contains("complete"));
    assert!(output_str.contains("-c linear"));
}

#[test]
fn test_powershell_completion_generation() {
    let mut output = Cursor::new(Vec::new());
    let generator = CompletionGenerator::new();
    let mut cmd = MockCli::command();

    generator
        .generate(Shell::PowerShell, &mut cmd, &mut output)
        .expect("Should generate powershell completions");

    let output_str = String::from_utf8(output.into_inner()).expect("Should be valid UTF-8");

    // Check for powershell-specific completion syntax
    assert!(output_str.contains("Register-ArgumentCompleter"));
    assert!(output_str.contains("linear"));
}

#[test]
fn test_completion_output_contains_commands() {
    let mut output = Cursor::new(Vec::new());
    let generator = CompletionGenerator::new();
    let mut cmd = MockCli::command();

    generator
        .generate(Shell::Bash, &mut cmd, &mut output)
        .expect("Should generate completions");

    let output_str = String::from_utf8(output.into_inner()).expect("Should be valid UTF-8");

    // Check that mock commands are referenced in completions
    assert!(output_str.contains("issues") || output_str.contains("'issues'"));
    assert!(output_str.contains("issue") || output_str.contains("'issue'"));
    assert!(output_str.contains("create") || output_str.contains("'create'"));
}

#[test]
fn test_completion_output_contains_flags() {
    let mut output = Cursor::new(Vec::new());
    let generator = CompletionGenerator::new();
    let mut cmd = MockCli::command();

    generator
        .generate(Shell::Bash, &mut cmd, &mut output)
        .expect("Should generate completions");

    let output_str = String::from_utf8(output.into_inner()).expect("Should be valid UTF-8");

    // Check that common flags are referenced in completions
    assert!(output_str.contains("--help") || output_str.contains("'--help'"));
    assert!(output_str.contains("--version") || output_str.contains("'--version'"));
}

#[test]
fn test_completion_installation_instructions() {
    let instructions = CompletionGenerator::installation_instructions();

    // Should have instructions for all shells
    assert!(instructions.contains("bash"));
    assert!(instructions.contains("zsh"));
    assert!(instructions.contains("fish"));
    assert!(instructions.contains("powershell"));

    // Should contain file paths
    assert!(instructions.contains("bash-completion"));
    assert!(instructions.contains(".zfunc"));
    assert!(instructions.contains("completions"));

    // Should contain example commands
    assert!(instructions.contains("linear completions"));
}

#[test]
fn test_shell_display() {
    assert_eq!(format!("{}", Shell::Bash), "bash");
    assert_eq!(format!("{}", Shell::Zsh), "zsh");
    assert_eq!(format!("{}", Shell::Fish), "fish");
    assert_eq!(format!("{}", Shell::PowerShell), "powershell");
}

#[test]
fn test_shell_all_variants() {
    let shells = Shell::all();
    assert_eq!(shells.len(), 4);
    assert!(shells.contains(&Shell::Bash));
    assert!(shells.contains(&Shell::Zsh));
    assert!(shells.contains(&Shell::Fish));
    assert!(shells.contains(&Shell::PowerShell));
}

#[test]
fn test_completion_generator_creation() {
    let generator = CompletionGenerator::new();

    // Test that generator can be created multiple times
    let _generator2 = CompletionGenerator::new();

    // Generator should be able to generate for all shells
    for shell in Shell::all() {
        let mut output = Cursor::new(Vec::new());
        let mut cmd = MockCli::command();
        generator
            .generate(shell, &mut cmd, &mut output)
            .expect("Should generate for all shells");

        let output_str = String::from_utf8(output.into_inner()).expect("Should be valid UTF-8");
        assert!(
            !output_str.is_empty(),
            "Output should not be empty for {}",
            shell
        );
    }
}

#[test]
fn test_completion_output_deterministic() {
    let generator = CompletionGenerator::new();

    // Generate completions twice and ensure they're identical
    let mut output1 = Cursor::new(Vec::new());
    let mut output2 = Cursor::new(Vec::new());
    let mut cmd1 = MockCli::command();
    let mut cmd2 = MockCli::command();

    generator
        .generate(Shell::Bash, &mut cmd1, &mut output1)
        .expect("Should generate first time");
    generator
        .generate(Shell::Bash, &mut cmd2, &mut output2)
        .expect("Should generate second time");

    let output1_str = String::from_utf8(output1.into_inner()).expect("Should be valid UTF-8");
    let output2_str = String::from_utf8(output2.into_inner()).expect("Should be valid UTF-8");

    assert_eq!(
        output1_str, output2_str,
        "Completion output should be deterministic"
    );
}
