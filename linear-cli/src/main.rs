// ABOUTME: Main entry point for the Linear CLI application
// ABOUTME: Provides command-line interface for Linear issue tracking

use anyhow::Result;
use clap::{Parser, Subcommand};
use linear_sdk::{IssueFilters, LinearClient};
use std::env;

mod output;
mod types;

use crate::output::{JsonFormatter, OutputFormat, TableFormatter};

#[derive(Parser)]
#[command(name = "linear")]
#[command(about = "A CLI for Linear", long_about = None)]
struct Cli {
    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

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
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let api_key = match env::var("LINEAR_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("Error: No LINEAR_API_KEY environment variable found");
            eprintln!();
            eprintln!("Please set your Linear API key:");
            eprintln!("export LINEAR_API_KEY=lin_api_xxxxx");
            eprintln!();
            eprintln!("Get your API key from: https://linear.app/settings/api");
            std::process::exit(1);
        }
    };

    let cli = Cli::parse();
    let client = LinearClient::new(api_key)?;

    // Determine if color should be used
    let use_color = !cli.no_color
        && env::var("NO_COLOR").is_err()
        && env::var("TERM").unwrap_or_default() != "dumb";

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
                if !json {
                    eprintln!("⚠️  Note: Filters are not yet implemented. Showing all issues.");
                    eprintln!();
                }
                Some(IssueFilters {
                    assignee,
                    status,
                    team,
                })
            } else {
                None
            };

            let issues = client.list_issues_filtered(limit, filters).await?;

            if issues.is_empty() && !json {
                println!("No issues found.");
            } else {
                let output = if json {
                    let formatter = JsonFormatter::new(pretty);
                    formatter.format_issues(&issues)?
                } else {
                    let formatter = TableFormatter::new(use_color);
                    formatter.format_issues(&issues)?
                };
                println!("{}", output);
            }
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

        // Check the limit argument
        let limit_arg = issues_cmd
            .get_arguments()
            .find(|arg| arg.get_id() == "limit")
            .expect("limit argument should exist");
        assert!(!limit_arg.is_required_set());
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
        }

        // Test pretty flag requires json (should fail)
        let result = Cli::try_parse_from(["linear", "issues", "--pretty"]);
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
