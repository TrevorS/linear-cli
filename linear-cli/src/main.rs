// ABOUTME: Main entry point for the Linear CLI application
// ABOUTME: Provides command-line interface for Linear issue tracking

use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use linear_sdk::{IssueFilters, LinearClient, LinearError, Result};
use owo_colors::OwoColorize;
use std::env;

mod output;
mod types;

use crate::output::{JsonFormatter, OutputFormat, TableFormatter};

fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
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
struct Cli {
    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

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
        #[arg(short, long, default_value = "20")]
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
    let use_color = !cli.no_color
        && env::var("NO_COLOR").is_err()
        && env::var("TERM").unwrap_or_default() != "dumb";

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
            let spinner = create_spinner("Logging out...");
            // We don't need a valid OAuth manager to logout, just need to clear the storage
            match linear_sdk::storage::clear() {
                Ok(_) => {
                    spinner.finish_and_clear();
                    if use_color {
                        println!("{} Successfully logged out!", "✓".green());
                    } else {
                        println!("✓ Successfully logged out!");
                    }
                    Ok(())
                }
                Err(e) => {
                    spinner.finish_and_clear();
                    display_error(&LinearError::from(e), use_color);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            // Continue with async commands
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async move { run_async_commands(cli, use_color).await })
        }
    }
}

async fn run_async_commands(cli: Cli, use_color: bool) -> Result<()> {
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

    let spinner = create_spinner("Connecting to Linear...");

    // Determine if this is an OAuth token (from keychain) or API key
    let is_oauth_token = env::var("LINEAR_API_KEY").is_err();

    let client = if is_oauth_token {
        #[cfg(feature = "oauth")]
        match LinearClient::new_with_oauth_token_and_verbose(auth_token, cli.verbose) {
            Ok(client) => {
                spinner.finish_and_clear();
                client
            }
            Err(e) => {
                spinner.finish_and_clear();
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
                spinner.finish_and_clear();
                client
            }
            Err(e) => {
                spinner.finish_and_clear();
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

            let spinner = create_spinner("Fetching issues...");
            let issues = match client.list_issues_filtered(limit, filters).await {
                Ok(issues) => {
                    spinner.finish_and_clear();
                    issues
                }
                Err(e) => {
                    spinner.finish_and_clear();
                    display_error(&e, use_color);
                    std::process::exit(1);
                }
            };

            if issues.is_empty() && !json {
                println!("No issues found.");
            } else {
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
                    let formatter = TableFormatter::new(use_color);
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
        Commands::Issue { id, json } => {
            let spinner = create_spinner(&format!("Fetching issue {}...", id));
            match client.get_issue(id).await {
                Ok(issue) => {
                    spinner.finish_and_clear();
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
                        let formatter = TableFormatter::new(use_color);
                        match formatter.format_detailed_issue(&issue) {
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
                    spinner.finish_and_clear();
                    display_error(&e, use_color);
                    std::process::exit(1);
                }
            }
        }
        Commands::Status { verbose } => {
            let spinner = create_spinner("Checking Linear connection...");
            match client.execute_viewer_query().await {
                Ok(viewer_data) => {
                    spinner.finish_and_clear();
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
                Err(e) => {
                    spinner.finish_and_clear();
                    if use_color {
                        println!("{} Failed to connect to Linear", "✗".red());
                    } else {
                        println!("✗ Failed to connect to Linear");
                    }
                    println!();
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
            Commands::Issue { id, json } => {
                assert_eq!(id, "ENG-123");
                assert!(!json);
            }
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issue command"),
            _ => panic!("Expected Issue command"),
        }

        // Test issue command with JSON
        let cli = Cli::try_parse_from(["linear", "issue", "ENG-456", "--json"]).unwrap();
        match cli.command {
            Commands::Issue { id, json } => {
                assert_eq!(id, "ENG-456");
                assert!(json);
            }
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issue command"),
            _ => panic!("Expected Issue command"),
        }

        // Test issue command with UUID
        let cli = Cli::try_parse_from(["linear", "issue", "abc-123-def-456"]).unwrap();
        match cli.command {
            Commands::Issue { id, json } => {
                assert_eq!(id, "abc-123-def-456");
                assert!(!json);
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
