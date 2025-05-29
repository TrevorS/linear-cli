// ABOUTME: Main entry point for the Linear CLI application
// ABOUTME: Provides command-line interface for Linear issue tracking

use anyhow::Result;
use clap::{Parser, Subcommand};
use linear_sdk::LinearClient;
use std::env;

#[derive(Parser)]
#[command(name = "linear")]
#[command(about = "A CLI for Linear", long_about = None)]
struct Cli {
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

    match cli.command {
        Commands::Issues { limit } => {
            let issues = client.list_issues(limit).await?;

            if issues.is_empty() {
                println!("No issues found.");
            } else {
                for issue in issues {
                    let assignee = issue.assignee.unwrap_or_else(|| "Unassigned".to_string());
                    println!(
                        "{}: {} ({}) - {}",
                        issue.identifier, issue.title, issue.status, assignee
                    );
                }
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
            Commands::Issues { limit } => {
                assert_eq!(limit, 20);
            }
        }

        // Test custom limit
        let cli = Cli::try_parse_from(["linear", "issues", "--limit", "5"]).unwrap();
        match cli.command {
            Commands::Issues { limit } => {
                assert_eq!(limit, 5);
            }
        }

        // Test short form
        let cli = Cli::try_parse_from(["linear", "issues", "-l", "10"]).unwrap();
        match cli.command {
            Commands::Issues { limit } => {
                assert_eq!(limit, 10);
            }
        }
    }
}
