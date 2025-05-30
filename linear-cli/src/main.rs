// ABOUTME: Main entry point for the Linear CLI application
// ABOUTME: Provides command-line interface for Linear issue tracking

use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use linear_sdk::{IssueFilters, LinearClient, LinearError, Result};
use owo_colors::OwoColorize;
use std::env;
use std::io::IsTerminal;

mod output;
mod types;

use crate::output::{JsonFormatter, OutputFormat, TableFormatter};

fn determine_use_color(no_color_flag: bool, force_color_flag: bool, is_tty: bool) -> bool {
    if force_color_flag {
        return true;
    }

    !no_color_flag
        && env::var("NO_COLOR").is_err()
        && env::var("TERM").unwrap_or_default() != "dumb"
        && is_tty
}

fn create_spinner(message: &str, is_interactive: bool) -> Option<ProgressBar> {
    if !is_interactive {
        return None;
    }

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    Some(pb)
}

fn display_error(error: &LinearError, use_color: bool) {
    if use_color {
        eprintln!("{} {}", "Error:".red().bold(), error);
    } else {
        eprintln!("Error: {}", error);
    }

    if let Some(help) = error.help_text() {
        eprintln!();
        eprintln!("{}", help);
    }
}

#[derive(Parser)]
#[command(name = "linear")]
#[command(about = "A CLI for Linear", long_about = None)]
#[command(version)]
struct Cli {
    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// Force colored output even when piped
    #[arg(long, global = true, conflicts_with = "no_color")]
    force_color: bool,

    /// Enable verbose output for debugging
    #[arg(long, short, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List issues
    Issues {
        /// Maximum number of issues to fetch
        #[arg(short, long, default_value = "20", value_parser = clap::value_parser!(i32).range(1..))]
        limit: i32,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Pretty print JSON output
        #[arg(long, requires = "json")]
        pretty: bool,

        /// Filter by assignee (use "me" for yourself)
        #[arg(long)]
        assignee: Option<String>,

        /// Filter by status (case insensitive)
        #[arg(long)]
        status: Option<String>,

        /// Filter by team
        #[arg(long)]
        team: Option<String>,
    },
    /// Show details for a single issue
    Issue {
        /// Issue identifier (e.g., ENG-123)
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Force raw markdown output (skip rich formatting)
        #[arg(long)]
        raw: bool,
    },
    /// Check connection to Linear
    Status {
        /// Show detailed connection info
        #[arg(long)]
        verbose: bool,
    },
    /// Login using OAuth (requires oauth feature)
    #[cfg(feature = "oauth")]
    Login {
        /// Force new login even if token exists
        #[arg(long)]
        force: bool,
        /// OAuth Client ID (can also be set via LINEAR_OAUTH_CLIENT_ID env var)
        #[arg(long)]
        client_id: Option<String>,
    },
    /// Logout and clear stored credentials (requires oauth feature)
    #[cfg(feature = "oauth")]
    Logout,
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    // Determine if color should be used
    let use_color = determine_use_color(
        cli.no_color,
        cli.force_color,
        std::io::stdout().is_terminal(),
    );

    // Handle OAuth commands first (synchronous commands)
    match &cli.command {
        #[cfg(feature = "oauth")]
        Commands::Login { force, client_id } => {
            let oauth_manager = match client_id {
                Some(id) => match linear_sdk::oauth::OAuthManager::new(id.to_string()) {
                    Ok(manager) => manager,
                    Err(e) => {
                        display_error(&e, use_color);
                        std::process::exit(1);
                    }
                },
                None => match linear_sdk::oauth::OAuthManager::from_env() {
                    Ok(manager) => manager,
                    Err(e) => {
                        display_error(&e, use_color);
                        std::process::exit(1);
                    }
                },
            };

            // Check if we need to force login
            if !force {
                if let Ok(_token) = oauth_manager.get_token() {
                    if use_color {
                        println!(
                            "{} Already logged in! Use --force to login again.",
                            "ℹ".blue()
                        );
                    } else {
                        println!("ℹ Already logged in! Use --force to login again.");
                    }
                    return Ok(());
                }
            }

            match oauth_manager.login() {
                Ok(_) => Ok(()),
                Err(e) => {
                    display_error(&e, use_color);
                    std::process::exit(1);
                }
            }
        }
        #[cfg(feature = "oauth")]
        Commands::Logout => {
            let spinner = create_spinner("Logging out...", use_color);
            // We don't need a valid OAuth manager to logout, just need to clear the storage
            match linear_sdk::storage::clear() {
                Ok(_) => {
                    if let Some(s) = spinner {
                        s.finish_and_clear();
                    }
                    if use_color {
                        println!("{} Successfully logged out!", "✓".green());
                    } else {
                        println!("✓ Successfully logged out!");
                    }
                    Ok(())
                }
                Err(e) => {
                    if let Some(s) = spinner {
                        s.finish_and_clear();
                    }
                    display_error(&LinearError::from(e), use_color);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            // Continue with async commands
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async move { run_async_commands(cli, use_color, use_color).await })
        }
    }
}

async fn run_async_commands(cli: Cli, use_color: bool, is_interactive: bool) -> Result<()> {
    // Authentication priority:
    // 1. Command line --api-key (not implemented yet)
    // 2. LINEAR_API_KEY env var
    // 3. OAuth token from keychain (if feature enabled)
    let auth_token = match env::var("LINEAR_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            #[cfg(feature = "oauth")]
            {
                // Try to get OAuth token from keychain
                match linear_sdk::storage::load() {
                    Ok(token) => token,
                    Err(_) => {
                        eprintln!(
                            "No authentication found. Use 'linear login' to authenticate with OAuth or set LINEAR_API_KEY environment variable."
                        );
                        std::process::exit(1);
                    }
                }
            }
            #[cfg(not(feature = "oauth"))]
            {
                display_error(&LinearError::Auth, use_color);
                std::process::exit(1);
            }
        }
    };

    let spinner = create_spinner("Connecting to Linear...", is_interactive);

    // Determine if this is an OAuth token (from keychain) or API key
    let is_oauth_token = env::var("LINEAR_API_KEY").is_err();

    let client = if is_oauth_token {
        #[cfg(feature = "oauth")]
        match LinearClient::new_with_oauth_token_and_verbose(auth_token, cli.verbose) {
            Ok(client) => {
                if let Some(s) = spinner {
                    s.finish_and_clear();
                }
                client
            }
            Err(e) => {
                if let Some(s) = spinner {
                    s.finish_and_clear();
                }
                display_error(&e, use_color);
                std::process::exit(1);
            }
        }
        #[cfg(not(feature = "oauth"))]
        {
            // This should never happen because we check for oauth feature above
            unreachable!()
        }
    } else {
        match LinearClient::new_with_verbose(auth_token, cli.verbose) {
            Ok(client) => {
                if let Some(s) = spinner {
                    s.finish_and_clear();
                }
                client
            }
            Err(e) => {
                if let Some(s) = spinner {
                    s.finish_and_clear();
                }
                display_error(&e, use_color);
                std::process::exit(1);
            }
        }
    };

    match cli.command {
        Commands::Issues {
            limit,
            json,
            pretty,
            assignee,
            status,
            team,
        } => {
            let filters = if assignee.is_some() || status.is_some() || team.is_some() {
                Some(IssueFilters {
                    assignee,
                    status,
                    team,
                })
            } else {
                None
            };

            let spinner = create_spinner("Fetching issues...", is_interactive);
            let issues = match client.list_issues_filtered(limit, filters).await {
                Ok(issues) => {
                    if let Some(s) = spinner {
                        s.finish_and_clear();
                    }
                    issues
                }
                Err(e) => {
                    if let Some(s) = spinner {
                        s.finish_and_clear();
                    }
                    display_error(&e, use_color);
                    std::process::exit(1);
                }
            };

            if issues.is_empty() && !json && is_interactive {
                println!("No issues found.");
            } else if !issues.is_empty() {
                let output = if json {
                    let formatter = JsonFormatter::new(pretty);
                    match formatter.format_issues(&issues) {
                        Ok(output) => output,
                        Err(e) => {
                            display_error(&e, use_color);
                            std::process::exit(1);
                        }
                    }
                } else {
                    let formatter = TableFormatter::new_with_interactive(use_color, is_interactive);
                    match formatter.format_issues(&issues) {
                        Ok(output) => output,
                        Err(e) => {
                            display_error(&e, use_color);
                            std::process::exit(1);
                        }
                    }
                };
                println!("{}", output);
            }
        }
        Commands::Issue { id, json, raw } => {
            let spinner = create_spinner(&format!("Fetching issue {}...", id), is_interactive);
            match client.get_issue(id).await {
                Ok(issue) => {
                    if let Some(s) = spinner {
                        s.finish_and_clear();
                    }
                    let output = if json {
                        let formatter = JsonFormatter::new(false);
                        match formatter.format_detailed_issue(&issue) {
                            Ok(output) => output,
                            Err(e) => {
                                display_error(&e, use_color);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        let formatter =
                            TableFormatter::new_with_interactive(use_color, is_interactive);
                        // Use TTY detection for rich formatting, allow --raw to override
                        let use_rich_formatting = is_interactive && !raw;
                        match formatter.format_detailed_issue_rich(&issue, use_rich_formatting) {
                            Ok(output) => output,
                            Err(e) => {
                                display_error(&e, use_color);
                                std::process::exit(1);
                            }
                        }
                    };
                    println!("{}", output);
                }
                Err(e) => {
                    if let Some(s) = spinner {
                        s.finish_and_clear();
                    }
                    display_error(&e, use_color);
                    std::process::exit(1);
                }
            }
        }
        Commands::Status { verbose } => {
            let spinner = create_spinner("Checking Linear connection...", is_interactive);
            match client.execute_viewer_query().await {
                Ok(viewer_data) => {
                    if let Some(s) = spinner {
                        s.finish_and_clear();
                    }
                    if is_interactive {
                        if use_color {
                            println!("{} Connected to Linear", "✓".green());
                        } else {
                            println!("✓ Connected to Linear");
                        }

                        if verbose {
                            println!();
                            println!(
                                "User: {} ({})",
                                viewer_data.viewer.name, viewer_data.viewer.email
                            );
                            println!("User ID: {}", viewer_data.viewer.id);
                        }
                    }
                }
                Err(e) => {
                    if let Some(s) = spinner {
                        s.finish_and_clear();
                    }
                    if is_interactive {
                        if use_color {
                            println!("{} Failed to connect to Linear", "✗".red());
                        } else {
                            println!("✗ Failed to connect to Linear");
                        }
                        println!();
                    }
                    display_error(&e, use_color);
                    std::process::exit(1);
                }
            }
        }
        #[cfg(feature = "oauth")]
        Commands::Login { .. } | Commands::Logout => {
            // These commands are handled earlier, this should never be reached
            unreachable!()
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use serial_test::serial;

    #[test]
    fn test_cli_structure() {
        let cli = Cli::command();

        // Verify basic structure
        assert_eq!(cli.get_name(), "linear");

        // Check that we have the issues subcommand
        let issues_cmd = cli
            .find_subcommand("issues")
            .expect("issues command should exist");
        assert_eq!(issues_cmd.get_name(), "issues");

        // Check that we have the issue subcommand
        let issue_cmd = cli
            .find_subcommand("issue")
            .expect("issue command should exist");
        assert_eq!(issue_cmd.get_name(), "issue");

        // Check the limit argument
        let limit_arg = issues_cmd
            .get_arguments()
            .find(|arg| arg.get_id() == "limit")
            .expect("limit argument should exist");
        assert!(!limit_arg.is_required_set());

        // Check the id argument for issue command
        let id_arg = issue_cmd
            .get_arguments()
            .find(|arg| arg.get_id() == "id")
            .expect("id argument should exist");
        assert!(id_arg.is_required_set());
    }

    #[test]
    fn test_parse_issues_command() {
        use clap::Parser;

        // Test default limit
        let cli = Cli::try_parse_from(["linear", "issues"]).unwrap();
        match cli.command {
            Commands::Issues {
                limit,
                json,
                pretty,
                assignee: _,
                status: _,
                team: _,
            } => {
                assert_eq!(limit, 20);
                assert!(!json);
                assert!(!pretty);
            }
            Commands::Issue { .. } => panic!("Expected Issues command"),
            Commands::Status { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issues command"),
        }

        // Test custom limit
        let cli = Cli::try_parse_from(["linear", "issues", "--limit", "5"]).unwrap();
        match cli.command {
            Commands::Issues {
                limit,
                json,
                pretty,
                assignee: _,
                status: _,
                team: _,
            } => {
                assert_eq!(limit, 5);
                assert!(!json);
                assert!(!pretty);
            }
            Commands::Issue { .. } => panic!("Expected Issues command"),
            Commands::Status { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issues command"),
        }

        // Test short form
        let cli = Cli::try_parse_from(["linear", "issues", "-l", "10"]).unwrap();
        match cli.command {
            Commands::Issues {
                limit,
                json,
                pretty,
                assignee: _,
                status: _,
                team: _,
            } => {
                assert_eq!(limit, 10);
                assert!(!json);
                assert!(!pretty);
            }
            Commands::Issue { .. } => panic!("Expected Issues command"),
            Commands::Status { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issues command"),
        }

        // Test JSON flag
        let cli = Cli::try_parse_from(["linear", "issues", "--json"]).unwrap();
        match cli.command {
            Commands::Issues {
                limit,
                json,
                pretty,
                assignee: _,
                status: _,
                team: _,
            } => {
                assert_eq!(limit, 20);
                assert!(json);
                assert!(!pretty);
            }
            Commands::Issue { .. } => panic!("Expected Issues command"),
            Commands::Status { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issues command"),
        }

        // Test JSON with pretty flag
        let cli = Cli::try_parse_from(["linear", "issues", "--json", "--pretty"]).unwrap();
        match cli.command {
            Commands::Issues {
                limit,
                json,
                pretty,
                assignee: _,
                status: _,
                team: _,
            } => {
                assert_eq!(limit, 20);
                assert!(json);
                assert!(pretty);
            }
            Commands::Issue { .. } => panic!("Expected Issues command"),
            Commands::Status { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issues command"),
        }

        // Test pretty flag requires json (should fail)
        let result = Cli::try_parse_from(["linear", "issues", "--pretty"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_issue_command() {
        use clap::Parser;

        // Test basic issue command
        let cli = Cli::try_parse_from(["linear", "issue", "ENG-123"]).unwrap();
        match cli.command {
            Commands::Issue { id, json, raw } => {
                assert_eq!(id, "ENG-123");
                assert!(!json);
                assert!(!raw);
            }
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issue command"),
            _ => panic!("Expected Issue command"),
        }

        // Test issue command with JSON
        let cli = Cli::try_parse_from(["linear", "issue", "ENG-456", "--json"]).unwrap();
        match cli.command {
            Commands::Issue { id, json, raw } => {
                assert_eq!(id, "ENG-456");
                assert!(json);
                assert!(!raw);
            }
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issue command"),
            _ => panic!("Expected Issue command"),
        }

        // Test issue command with UUID
        let cli = Cli::try_parse_from(["linear", "issue", "abc-123-def-456"]).unwrap();
        match cli.command {
            Commands::Issue { id, json, raw } => {
                assert_eq!(id, "abc-123-def-456");
                assert!(!json);
                assert!(!raw);
            }
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issue command"),
            _ => panic!("Expected Issue command"),
        }

        // Test issue command with --raw flag
        let cli = Cli::try_parse_from(["linear", "issue", "ENG-789", "--raw"]).unwrap();
        match cli.command {
            Commands::Issue { id, json, raw } => {
                assert_eq!(id, "ENG-789");
                assert!(!json);
                assert!(raw);
            }
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issue command"),
            _ => panic!("Expected Issue command"),
        }

        // Test issue command with both --json and --raw flags
        let cli = Cli::try_parse_from(["linear", "issue", "ENG-999", "--json", "--raw"]).unwrap();
        match cli.command {
            Commands::Issue { id, json, raw } => {
                assert_eq!(id, "ENG-999");
                assert!(json);
                assert!(raw);
            }
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issue command"),
            _ => panic!("Expected Issue command"),
        }

        // Test issue command without ID (should fail)
        let result = Cli::try_parse_from(["linear", "issue"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_filter_arguments() {
        use clap::Parser;

        // Test assignee filter
        let cli = Cli::try_parse_from(["linear", "issues", "--assignee", "me"]).unwrap();
        match cli.command {
            Commands::Issues {
                assignee,
                status,
                team,
                ..
            } => {
                assert_eq!(assignee, Some("me".to_string()));
                assert_eq!(status, None);
                assert_eq!(team, None);
            }
            Commands::Issue { .. } => panic!("Expected Issues command"),
            Commands::Status { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issues command"),
        }

        // Test status filter
        let cli = Cli::try_parse_from(["linear", "issues", "--status", "done"]).unwrap();
        match cli.command {
            Commands::Issues {
                assignee,
                status,
                team,
                ..
            } => {
                assert_eq!(assignee, None);
                assert_eq!(status, Some("done".to_string()));
                assert_eq!(team, None);
            }
            Commands::Issue { .. } => panic!("Expected Issues command"),
            Commands::Status { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issues command"),
        }

        // Test team filter
        let cli = Cli::try_parse_from(["linear", "issues", "--team", "ENG"]).unwrap();
        match cli.command {
            Commands::Issues {
                assignee,
                status,
                team,
                ..
            } => {
                assert_eq!(assignee, None);
                assert_eq!(status, None);
                assert_eq!(team, Some("ENG".to_string()));
            }
            Commands::Issue { .. } => panic!("Expected Issues command"),
            Commands::Status { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issues command"),
        }

        // Test combined filters
        let cli = Cli::try_parse_from([
            "linear",
            "issues",
            "--assignee",
            "me",
            "--status",
            "in progress",
            "--team",
            "ENG",
        ])
        .unwrap();
        match cli.command {
            Commands::Issues {
                assignee,
                status,
                team,
                ..
            } => {
                assert_eq!(assignee, Some("me".to_string()));
                assert_eq!(status, Some("in progress".to_string()));
                assert_eq!(team, Some("ENG".to_string()));
            }
            Commands::Issue { .. } => panic!("Expected Issues command"),
            Commands::Status { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issues command"),
        }
    }

    #[test]
    fn test_version_command() {
        use clap::Parser;
        use std::process::Command;

        // Test that version command parses correctly
        let result = Cli::try_parse_from(["linear", "--version"]);
        // This will fail because --version is handled by clap before we get the result
        // But we can test that it doesn't conflict with other parsing
        assert!(result.is_err()); // clap exits early for --version

        // Test that version is included in help output
        let output = Command::new("cargo")
            .args(["run", "-p", "linear-cli", "--", "--help"])
            .output()
            .unwrap();
        let help_output = std::str::from_utf8(&output.stdout).unwrap();
        assert!(help_output.contains("--version") || help_output.contains("-V"));
    }

    #[test]
    fn test_deferred_authentication() {
        use clap::Parser;

        // Test that help commands can be parsed without requiring authentication
        // (Authentication is only checked at runtime when making API calls)

        // Help command should parse successfully
        let cli = Cli::try_parse_from(["linear", "--help"]);
        assert!(cli.is_err()); // clap exits early for --help

        // Subcommand help should parse successfully
        let cli = Cli::try_parse_from(["linear", "issues", "--help"]);
        assert!(cli.is_err()); // clap exits early for --help

        // Regular commands should parse successfully (auth checked later)
        let cli = Cli::try_parse_from(["linear", "issues"]).unwrap();
        match cli.command {
            Commands::Issues { .. } => {} // Success - parsing works without auth
            _ => panic!("Expected Issues command"),
        }

        let cli = Cli::try_parse_from(["linear", "status"]).unwrap();
        match cli.command {
            Commands::Status { .. } => {} // Success - parsing works without auth
            _ => panic!("Expected Status command"),
        }
    }

    #[test]
    #[serial]
    fn test_determine_use_color_with_tty() {
        // Save original env vars
        let original_no_color = std::env::var("NO_COLOR").ok();
        let original_term = std::env::var("TERM").ok();

        unsafe {
            // Clear env vars to test default behavior
            std::env::remove_var("NO_COLOR");
            std::env::remove_var("TERM");

            // Mock a TTY scenario (we can't directly mock IsTerminal)
            // but we can test the logic separately
            let use_color = determine_use_color(false, false, true);
            assert!(use_color);

            // Test with no-color flag
            let use_color = determine_use_color(true, false, true);
            assert!(!use_color);

            // Test with NO_COLOR env var
            std::env::set_var("NO_COLOR", "1");
            let use_color = determine_use_color(false, false, true);
            assert!(!use_color);
            std::env::remove_var("NO_COLOR");

            // Test with TERM=dumb
            std::env::set_var("TERM", "dumb");
            let use_color = determine_use_color(false, false, true);
            assert!(!use_color);

            // Test non-TTY (piped/redirected)
            std::env::remove_var("TERM");
            let use_color = determine_use_color(false, false, false);
            assert!(!use_color);

            // Test force-color overrides non-TTY
            let use_color = determine_use_color(false, true, false);
            assert!(use_color);

            // Restore original env vars
            if let Some(val) = original_no_color {
                std::env::set_var("NO_COLOR", val);
            } else {
                std::env::remove_var("NO_COLOR");
            }
            if let Some(val) = original_term {
                std::env::set_var("TERM", val);
            } else {
                std::env::remove_var("TERM");
            }
        }
    }

    #[test]
    #[serial]
    fn test_determine_use_color_priority() {
        // Test that flags take precedence over env vars and TTY detection
        let original_no_color = std::env::var("NO_COLOR").ok();
        let original_term = std::env::var("TERM").ok();

        unsafe {
            // Test flag overrides everything
            std::env::remove_var("NO_COLOR");
            std::env::remove_var("TERM");
            let use_color = determine_use_color(true, false, true);
            assert!(!use_color, "--no-color flag should override TTY detection");

            // Test NO_COLOR env var overrides TTY
            std::env::set_var("NO_COLOR", "1");
            let use_color = determine_use_color(false, false, true);
            assert!(!use_color, "NO_COLOR env should override TTY detection");

            // Test TERM=dumb overrides TTY
            std::env::remove_var("NO_COLOR");
            std::env::set_var("TERM", "dumb");
            let use_color = determine_use_color(false, false, true);
            assert!(!use_color, "TERM=dumb should override TTY detection");

            // Test force-color overrides everything
            std::env::set_var("NO_COLOR", "1");
            std::env::set_var("TERM", "dumb");
            let use_color = determine_use_color(false, true, false);
            assert!(use_color, "--force-color should override everything");

            // Restore env vars
            if let Some(val) = original_no_color {
                std::env::set_var("NO_COLOR", val);
            } else {
                std::env::remove_var("NO_COLOR");
            }
            if let Some(val) = original_term {
                std::env::set_var("TERM", val);
            } else {
                std::env::remove_var("TERM");
            }
        }
    }

    #[test]
    fn test_force_color_flag() {
        use clap::Parser;

        // Test force-color flag
        let cli = Cli::try_parse_from(["linear", "--force-color", "issues"]).unwrap();
        assert!(cli.force_color);
        assert!(!cli.no_color);

        // Test that force-color and no-color conflict
        let result = Cli::try_parse_from(["linear", "--force-color", "--no-color", "issues"]);
        assert!(result.is_err());

        // Test no-color flag still works
        let cli = Cli::try_parse_from(["linear", "--no-color", "issues"]).unwrap();
        assert!(!cli.force_color);
        assert!(cli.no_color);
    }

    #[test]
    fn test_limit_validation() {
        use clap::Parser;

        // Test that limit must be at least 1
        let result = Cli::try_parse_from(["linear", "issues", "--limit", "0"]);
        assert!(result.is_err());

        // Test that negative limits are rejected
        let result = Cli::try_parse_from(["linear", "issues", "--limit=-5"]);
        assert!(result.is_err());

        // Test that valid limits work
        let cli = Cli::try_parse_from(["linear", "issues", "--limit", "1"]).unwrap();
        match cli.command {
            Commands::Issues { limit, .. } => {
                assert_eq!(limit, 1);
            }
            _ => panic!("Expected Issues command"),
        }
    }

    #[test]
    fn test_json_output_can_be_parsed() {
        use crate::output::{JsonFormatter, OutputFormat};
        use linear_sdk::Issue;

        // Create test issues
        let issues = vec![
            Issue {
                id: "1".to_string(),
                identifier: "ENG-123".to_string(),
                title: "Test issue".to_string(),
                status: "Todo".to_string(),
                assignee: Some("Alice".to_string()),
                assignee_id: Some("user-1".to_string()),
                team: Some("ENG".to_string()),
            },
            Issue {
                id: "2".to_string(),
                identifier: "ENG-124".to_string(),
                title: "Another issue".to_string(),
                status: "Done".to_string(),
                assignee: None,
                assignee_id: None,
                team: Some("ENG".to_string()),
            },
        ];

        // Test compact JSON
        let formatter = JsonFormatter::new(false);
        let output = formatter.format_issues(&issues).unwrap();

        // Verify it can be parsed back
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["identifier"], "ENG-123");
        assert_eq!(parsed[1]["identifier"], "ENG-124");

        // Test pretty JSON
        let formatter = JsonFormatter::new(true);
        let output = formatter.format_issues(&issues).unwrap();

        // Verify pretty JSON can also be parsed back
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed.len(), 2);

        // Verify it's actually pretty printed (contains newlines)
        assert!(output.contains('\n'));
    }
}
