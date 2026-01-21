// ABOUTME: Main entry point for the Linear CLI application
// ABOUTME: Provides command-line interface for Linear issue tracking

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use linear_sdk::{IssueFilters, LinearClient, LinearError, Result};
use owo_colors::OwoColorize;
use secrecy::SecretString;
use std::env;
use std::io::IsTerminal;

mod aliases;
mod cli;
mod cli_output;
mod completions;
mod config;
mod constants;
mod frontmatter;
mod interactive;
mod output;
mod preferences;
mod search;
mod templates;
mod types;

use crate::aliases::AliasExpander;
use crate::cli::{Cli, Commands};
use crate::cli_output::CliOutput;
use crate::config::Config;
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

/// RAII guard for spinner that auto-clears on drop
struct SpinnerGuard(Option<ProgressBar>);

impl SpinnerGuard {
    fn new(message: &str, is_interactive: bool) -> Self {
        if !is_interactive {
            return Self(None);
        }

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner:.blue} {msg}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(
            constants::timeouts::PROGRESS_BAR_TICK_MS,
        ));
        Self(Some(pb))
    }
}

impl Drop for SpinnerGuard {
    fn drop(&mut self) {
        if let Some(ref pb) = self.0 {
            pb.finish_and_clear();
        }
    }
}

fn display_error(error: &LinearError, use_color: bool) {
    let cli = CliOutput::with_color(use_color);
    cli.error(&error.to_string());

    if let Some(help) = error.help_text() {
        eprintln!();
        eprintln!("{help}");
    }
}

/// Prompt the user for yes/no confirmation. Returns true if user confirms.
/// Skips prompt and returns true if `force` is set or not interactive.
fn confirm_action(action: &str, force: bool, is_interactive: bool) -> bool {
    if force || !is_interactive {
        return true;
    }

    print!("Continue with {action}? [y/N]: ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    let mut response = String::new();
    std::io::stdin().read_line(&mut response).unwrap();
    let response = response.trim().to_lowercase();

    response == "y" || response == "yes"
}

struct CreateCommandArgs {
    title: Option<String>,
    description: Option<String>,
    team: Option<String>,
    assignee: Option<String>,
    priority: Option<i64>,
    project: Option<String>,
    project_id: Option<String>,
    from_file: Option<String>,
    open: bool,
    dry_run: bool,
}

fn main() -> Result<()> {
    env_logger::init();

    // Load configuration first to get aliases
    let config = Config::load().unwrap_or_default();

    // Expand aliases in command line arguments before parsing
    let args = if let Some(ref aliases) = config.aliases {
        let original_args: Vec<String> = std::env::args().collect();
        let expander = AliasExpander::new(aliases.clone());
        match expander.expand(original_args) {
            Ok(expanded_args) => expanded_args,
            Err(e) => {
                eprintln!("Error expanding aliases: {e}");
                std::process::exit(1);
            }
        }
    } else {
        std::env::args().collect()
    };

    // Parse the expanded arguments
    let cli = Cli::parse_from(args);

    // Determine if color should be used and if we're interactive
    let is_interactive = std::io::stdout().is_terminal();
    let use_color = determine_use_color(cli.no_color, cli.force_color, is_interactive);

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
            let spinner = SpinnerGuard::new("Logging out...", use_color);
            // We don't need a valid OAuth manager to logout, just need to clear the storage
            match linear_sdk::storage::clear() {
                Ok(_) => {
                    drop(spinner);
                    let cli = CliOutput::with_color(use_color);
                    cli.success("Successfully logged out!");
                    Ok(())
                }
                Err(e) => {
                    drop(spinner);
                    display_error(&LinearError::from(e), use_color);
                    std::process::exit(1);
                }
            }
        }
        Commands::Completions { shell } => {
            use crate::completions::CompletionGenerator;
            use clap::CommandFactory;
            let generator = CompletionGenerator::new();
            let mut cmd = Cli::command();
            generator
                .generate(*shell, &mut cmd, &mut std::io::stdout())
                .map_err(|e| LinearError::Configuration(e.to_string()))?;
            Ok(())
        }
        _ => {
            // Continue with async commands
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async move {
                run_async_commands(cli, config, use_color, is_interactive).await
            })
        }
    }
}

async fn handle_create_from_file(
    client: &LinearClient,
    args: &CreateCommandArgs,
    file_path: &str,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
    let cli_output = CliOutput::with_color(use_color);

    // Parse the markdown file
    let markdown_file = match crate::frontmatter::parse_markdown_file(file_path) {
        Ok(file) => file,
        Err(e) => {
            cli_output.error(&format!("Failed to parse markdown file '{file_path}': {e}"));
            std::process::exit(1);
        }
    };

    // CLI arguments override frontmatter values
    let title = args
        .title
        .as_ref()
        .unwrap_or(&markdown_file.frontmatter.title)
        .clone();
    let description = args
        .description
        .as_ref()
        .or(if markdown_file.content.trim().is_empty() {
            None
        } else {
            Some(&markdown_file.content)
        })
        .map(|s| s.to_string());
    let team = args
        .team
        .as_ref()
        .or(markdown_file.frontmatter.team.as_ref())
        .cloned();
    let assignee = args
        .assignee
        .as_ref()
        .or(markdown_file.frontmatter.assignee.as_ref())
        .cloned();
    let priority = args.priority.or(markdown_file.frontmatter.priority);

    // Validate required fields
    if title.trim().is_empty() {
        cli_output.error("Title is required (specify in frontmatter or use --title)");
        std::process::exit(1);
    }

    let team_id = match team {
        Some(team) => {
            // Enhanced team resolution - detect UUID vs team key
            if team.chars().all(|c| c.is_ascii_hexdigit() || c == '-') && team.len() > 20 {
                // Looks like a UUID
                team
            } else {
                // Assume it's a team key, resolve it
                match client.resolve_team_key_to_id(&team).await {
                    Ok(team_id) => team_id,
                    Err(e) => {
                        cli_output.error(&format!("Failed to resolve team '{team}': {e}"));
                        std::process::exit(1);
                    }
                }
            }
        }
        None => {
            cli_output.error("Team is required (specify in frontmatter or use --team)");
            std::process::exit(1);
        }
    };

    // Enhanced assignee resolution
    let assignee_id = if let Some(assignee) = assignee {
        if assignee.trim().is_empty() || assignee.eq_ignore_ascii_case("unassigned") {
            None
        } else if assignee.eq_ignore_ascii_case("me") {
            let viewer_data = client.execute_viewer_query().await?;
            Some(viewer_data.viewer.id)
        } else {
            // Could be UUID or email/name - pass as-is for now
            Some(assignee)
        }
    } else {
        None
    };

    let input = crate::interactive::InteractiveCreateInput {
        title,
        description,
        team_id,
        assignee_id,
        priority,
    };

    // Handle dry-run mode
    if args.dry_run {
        cli_output.info("Dry run mode - no issue will be created");
        println!();
        println!("Would create issue from file '{file_path}':");
        println!("  Title: {}", input.title);
        if let Some(desc) = &input.description {
            println!("  Description: {desc}");
        }
        println!("  Team ID: {}", input.team_id);
        if let Some(assignee_id) = &input.assignee_id {
            println!("  Assignee ID: {assignee_id}");
        }
        if let Some(priority) = input.priority {
            println!("  Priority: {priority}");
        }
        if let Some(labels) = &markdown_file.frontmatter.labels {
            println!("  Labels: {labels:?} (not yet supported)");
        }
        if let Some(project) = &markdown_file.frontmatter.project {
            println!("  Project: {project} (not yet supported)");
        }
        return Ok(());
    }

    // Resolve project if provided
    let project_id = if let Some(project_name) = &args.project {
        // Use project name - need to resolve to ID
        use crate::interactive::InteractivePrompter;
        let prompter = InteractivePrompter::new_with_defaults(client).unwrap();
        match prompter.resolve_project(project_name).await {
            Ok(id) => id,
            Err(e) => {
                cli_output.error(&format!("Failed to resolve project '{project_name}': {e}"));
                std::process::exit(1);
            }
        }
    } else {
        args.project_id.clone()
    };

    // Build the SDK create input
    let sdk_input = linear_sdk::CreateIssueInput {
        title: input.title,
        description: input.description,
        team_id: Some(input.team_id),
        assignee_id: input.assignee_id,
        priority: input.priority,
        project_id,
        label_ids: None, // Future enhancement - need to resolve label names to IDs
    };

    // Create the issue
    let spinner = SpinnerGuard::new("Creating issue from file...", is_interactive);
    match client.create_issue(sdk_input).await {
        Ok(created_issue) => {
            drop(spinner);

            if is_interactive {
                cli_output.success(&format!("Created issue: {}", created_issue.identifier));
                println!("Title: {}", created_issue.title);
                if let Some(desc) = &created_issue.description {
                    println!("Description: {desc}");
                }
                println!("Status: {}", created_issue.state.name);
                if let Some(assignee) = &created_issue.assignee {
                    println!("Assignee: {}", assignee.name);
                }
                if let Some(team) = &created_issue.team {
                    println!("Team: {} ({})", team.name, team.key);
                }
                println!("URL: {}", created_issue.url);

                // Handle --open flag
                if args.open {
                    println!();
                    cli_output.info(&format!("Issue URL: {}", created_issue.url));
                    cli_output.info("Please open this URL in your browser");
                }
            } else {
                // Non-interactive: just print the identifier for scripting
                println!("{}", created_issue.identifier);
            }
        }
        Err(e) => {
            drop(spinner);
            display_error(&e, use_color);
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn handle_create_command(
    client: &LinearClient,
    args: CreateCommandArgs,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
    use crate::interactive::{CreateOptions, InteractivePrompter};

    let cli_output = CliOutput::with_color(use_color);

    // Handle file-based input first
    if let Some(file_path) = &args.from_file {
        return handle_create_from_file(client, &args, file_path, use_color, is_interactive).await;
    }

    // Check if we need to collect input interactively
    let needs_prompts = args.title.is_none() || args.team.is_none();

    let input = if needs_prompts && is_interactive {
        // Interactive mode with Phase 5 smart defaults and templates
        let prompter = match InteractivePrompter::new_with_defaults(client) {
            Ok(prompter) => prompter,
            Err(e) => {
                cli_output.error(&format!("Failed to initialize interactive prompter: {e}"));
                std::process::exit(1);
            }
        };

        if !prompter.should_prompt() {
            cli_output.error("Interactive prompts are not available in this environment");
            eprintln!("Please provide --title and --team arguments explicitly");
            std::process::exit(1);
        }

        let options = CreateOptions {
            title: args.title,
            description: args.description,
            team: args.team,
            assignee: args.assignee,
            priority: args.priority,
        };

        match prompter.collect_create_input(options).await {
            Ok(input) => input,
            Err(e) => {
                cli_output.error(&format!("Failed to collect issue details: {e}"));
                std::process::exit(1);
            }
        }
    } else {
        // Non-interactive mode with validation
        let title = match args.title {
            Some(title) => title,
            None => {
                cli_output.error("Title is required for issue creation");
                eprintln!(
                    "Use --title \"Your issue title\" or run without arguments for interactive mode"
                );
                std::process::exit(1);
            }
        };

        let team_id = match args.team {
            Some(team) => {
                // Enhanced team resolution - detect UUID vs team key
                if team.chars().all(|c| c.is_ascii_hexdigit() || c == '-') && team.len() > 20 {
                    // Looks like a UUID
                    team
                } else {
                    // Assume it's a team key, resolve it
                    match client.resolve_team_key_to_id(&team).await {
                        Ok(team_id) => team_id,
                        Err(e) => {
                            cli_output.error(&format!("Failed to resolve team '{team}': {e}"));
                            std::process::exit(1);
                        }
                    }
                }
            }
            None => {
                cli_output.error("Team is required for issue creation");
                eprintln!("Use --team TEAM_KEY or run without arguments for interactive mode");
                std::process::exit(1);
            }
        };

        // Enhanced assignee resolution
        let assignee_id = if let Some(assignee) = args.assignee {
            if assignee.trim().is_empty() || assignee.eq_ignore_ascii_case("unassigned") {
                None
            } else if assignee.eq_ignore_ascii_case("me") {
                let viewer_data = client.execute_viewer_query().await?;
                Some(viewer_data.viewer.id)
            } else {
                // Could be UUID or email/name - pass as-is for now
                Some(assignee)
            }
        } else {
            None
        };

        crate::interactive::InteractiveCreateInput {
            title,
            description: args.description,
            team_id,
            assignee_id,
            priority: args.priority,
        }
    };

    // Handle dry-run mode
    if args.dry_run {
        cli_output.info("Dry run mode - no issue will be created");
        println!();
        println!("Would create issue:");
        println!("  Title: {}", input.title);
        if let Some(desc) = &input.description {
            println!("  Description: {desc}");
        }
        println!("  Team ID: {}", input.team_id);
        if let Some(assignee_id) = &input.assignee_id {
            println!("  Assignee ID: {assignee_id}");
        }
        if let Some(priority) = input.priority {
            println!("  Priority: {priority}");
        }
        return Ok(());
    }

    // Resolve project if provided
    let project_id = if let Some(project_name) = &args.project {
        // Use project name - need to resolve to ID
        use crate::interactive::InteractivePrompter;
        let prompter = InteractivePrompter::new_with_defaults(client).unwrap();
        match prompter.resolve_project(project_name).await {
            Ok(id) => id,
            Err(e) => {
                cli_output.error(&format!("Failed to resolve project '{project_name}': {e}"));
                std::process::exit(1);
            }
        }
    } else {
        args.project_id.clone()
    };

    // Build the SDK create input
    let sdk_input = linear_sdk::CreateIssueInput {
        title: input.title,
        description: input.description,
        team_id: Some(input.team_id),
        assignee_id: input.assignee_id,
        priority: input.priority,
        project_id,
        label_ids: None, // Future enhancement
    };

    // Create the issue
    let spinner = SpinnerGuard::new("Creating issue...", is_interactive);
    match client.create_issue(sdk_input).await {
        Ok(created_issue) => {
            drop(spinner);

            if is_interactive {
                cli_output.success(&format!("Created issue: {}", created_issue.identifier));
                println!("Title: {}", created_issue.title);
                if let Some(desc) = &created_issue.description {
                    println!("Description: {desc}");
                }
                println!("Status: {}", created_issue.state.name);
                if let Some(assignee) = &created_issue.assignee {
                    println!("Assignee: {}", assignee.name);
                }
                if let Some(team) = &created_issue.team {
                    println!("Team: {} ({})", team.name, team.key);
                }
                println!("URL: {}", created_issue.url);

                // Handle --open flag
                if args.open {
                    println!();
                    cli_output.info(&format!("Issue URL: {}", created_issue.url));
                    cli_output.info("Please open this URL in your browser");
                }
            } else {
                // Non-interactive: just print the identifier for scripting
                println!("{}", created_issue.identifier);
            }
        }
        Err(e) => {
            drop(spinner);
            display_error(&e, use_color);
            std::process::exit(1);
        }
    }

    Ok(())
}

struct UpdateCommandArgs {
    id: String,
    title: Option<String>,
    description: Option<String>,
    assignee: Option<String>,
    status: Option<String>,
    priority: Option<i64>,
    project: Option<String>,
    project_id: Option<String>,
    force: bool,
}

async fn handle_update_command(
    client: &LinearClient,
    args: UpdateCommandArgs,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
    let cli_output = CliOutput::with_color(use_color);

    // Validate that at least one field is being updated
    if args.title.is_none()
        && args.description.is_none()
        && args.assignee.is_none()
        && args.status.is_none()
        && args.priority.is_none()
        && args.project.is_none()
        && args.project_id.is_none()
    {
        cli_output.error("At least one field must be specified for update");
        eprintln!("Use --title, --description, --assignee, --status, --priority, --project, or --project-id");
        std::process::exit(1);
    }

    // Resolve assignee if provided
    let assignee_id = if let Some(assignee_value) = args.assignee {
        if assignee_value.trim().is_empty() || assignee_value.eq_ignore_ascii_case("unassigned") {
            Some(String::new()) // Empty string to unassign
        } else if assignee_value.eq_ignore_ascii_case("me") {
            let viewer_data = client.execute_viewer_query().await?;
            Some(viewer_data.viewer.id)
        } else {
            // Could be UUID or email/name - pass as-is for now
            Some(assignee_value)
        }
    } else {
        None
    };

    // Resolve status to state_id if provided
    let state_id = if let Some(ref status_name) = args.status {
        // Get the issue first to determine its team
        let issue = client.get_issue(args.id.clone()).await?;
        let team_id = issue.team.as_ref().unwrap().id.clone();

        // Resolve status name to actual state ID for this team
        Some(
            client
                .resolve_status_to_state_id(&team_id, status_name)
                .await?,
        )
    } else {
        None
    };

    // Resolve project if provided
    let project_id = if let Some(project_name) = &args.project {
        // Use project name - need to resolve to ID
        use crate::interactive::InteractivePrompter;
        let prompter = InteractivePrompter::new_with_defaults(client).unwrap();
        match prompter.resolve_project(project_name).await {
            Ok(id) => id,
            Err(e) => {
                cli_output.error(&format!("Failed to resolve project '{project_name}': {e}"));
                std::process::exit(1);
            }
        }
    } else {
        args.project_id.clone()
    };

    let input = linear_sdk::UpdateIssueInput {
        title: args.title,
        description: args.description,
        assignee_id,
        state_id,
        priority: args.priority,
        project_id,
        label_ids: None, // Future enhancement
    };

    // Show preview unless --force is used
    if !args.force && is_interactive {
        println!("Would update issue {}:", args.id);
        if let Some(ref title) = input.title {
            println!("  Title: {title}");
        }
        if let Some(ref description) = input.description {
            println!("  Description: {description}");
        }
        if let Some(ref assignee_id) = input.assignee_id {
            if assignee_id.is_empty() {
                println!("  Assignee: Unassigned");
            } else {
                println!("  Assignee: {assignee_id}");
            }
        }
        if let Some(ref _state_id) = input.state_id {
            if let Some(ref status_name) = args.status {
                println!("  Status: {status_name}");
            }
        }
        if let Some(priority) = input.priority {
            println!("  Priority: {priority}");
        }
        println!();

        if !confirm_action("update", false, true) {
            cli_output.info("Update cancelled");
            return Ok(());
        }
    }

    let spinner = SpinnerGuard::new("Updating issue...", is_interactive);
    match client.update_issue(args.id.clone(), input).await {
        Ok(updated_issue) => {
            drop(spinner);

            if is_interactive {
                cli_output.success(&format!("Updated issue: {}", updated_issue.identifier));
                println!("Title: {}", updated_issue.title);
                if let Some(desc) = &updated_issue.description {
                    println!("Description: {desc}");
                }
                println!("Status: {}", updated_issue.state.name);
                if let Some(assignee) = &updated_issue.assignee {
                    println!("Assignee: {}", assignee.name);
                }
                if let Some(team) = &updated_issue.team {
                    println!("Team: {} ({})", team.name, team.key);
                }
                println!("URL: {}", updated_issue.url);
            } else {
                // Non-interactive: just print the identifier for scripting
                println!("{}", updated_issue.identifier);
            }
        }
        Err(e) => {
            drop(spinner);
            display_error(&e, use_color);
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn handle_close_command(
    client: &LinearClient,
    id: String,
    force: bool,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
    let cli_output = CliOutput::with_color(use_color);

    // Show preview unless --force is used
    if !force && is_interactive {
        println!("Would close issue: {id}");
        println!();

        if !confirm_action("close", false, true) {
            cli_output.info("Close cancelled");
            return Ok(());
        }
    }

    // Get the issue first to determine its team, then resolve the "Done" state ID
    let issue = client.get_issue(id.clone()).await?;
    let team_id = issue.team.as_ref().unwrap().id.clone();

    // Resolve "Done" status to the actual state ID for this team
    let done_state_id = client.resolve_status_to_state_id(&team_id, "Done").await?;

    let input = linear_sdk::UpdateIssueInput {
        title: None,
        description: None,
        assignee_id: None,
        state_id: Some(done_state_id),
        priority: None,
        project_id: None,
        label_ids: None,
    };

    let spinner = SpinnerGuard::new("Closing issue...", is_interactive);
    match client.update_issue(id.clone(), input).await {
        Ok(updated_issue) => {
            drop(spinner);

            if is_interactive {
                cli_output.success(&format!("Closed issue: {}", updated_issue.identifier));
                println!("Status: {}", updated_issue.state.name);
                println!("URL: {}", updated_issue.url);
            } else {
                println!("{}", updated_issue.identifier);
            }
        }
        Err(e) => {
            drop(spinner);
            display_error(&e, use_color);
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn handle_reopen_command(
    client: &LinearClient,
    id: String,
    force: bool,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
    let cli_output = CliOutput::with_color(use_color);

    // Show preview unless --force is used
    if !force && is_interactive {
        println!("Would reopen issue: {id}");
        println!();

        if !confirm_action("reopen", false, true) {
            cli_output.info("Reopen cancelled");
            return Ok(());
        }
    }

    // Get the issue first to determine its team, then resolve the "Todo" state ID
    let issue = client.get_issue(id.clone()).await?;
    let team_id = issue.team.as_ref().unwrap().id.clone();

    // Resolve "Todo" status to the actual state ID for this team (uses default issue state)
    let todo_state_id = client.resolve_status_to_state_id(&team_id, "Todo").await?;

    let input = linear_sdk::UpdateIssueInput {
        title: None,
        description: None,
        assignee_id: None,
        state_id: Some(todo_state_id),
        priority: None,
        project_id: None,
        label_ids: None,
    };

    let spinner = SpinnerGuard::new("Reopening issue...", is_interactive);
    match client.update_issue(id.clone(), input).await {
        Ok(updated_issue) => {
            drop(spinner);

            if is_interactive {
                cli_output.success(&format!("Reopened issue: {}", updated_issue.identifier));
                println!("Status: {}", updated_issue.state.name);
                println!("URL: {}", updated_issue.url);
            } else {
                println!("{}", updated_issue.identifier);
            }
        }
        Err(e) => {
            drop(spinner);
            display_error(&e, use_color);
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn handle_comment_command(
    client: &LinearClient,
    id: String,
    message: Option<String>,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
    let cli_output = CliOutput::with_color(use_color);

    // Get comment body from argument or stdin
    let body = if let Some(msg) = message {
        msg
    } else {
        // Read from stdin
        use std::io::Read;
        let mut buffer = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut buffer) {
            cli_output.error(&format!("Failed to read from stdin: {e}"));
            std::process::exit(1);
        }

        if buffer.trim().is_empty() {
            cli_output.error("Comment body cannot be empty");
            eprintln!("Provide a message argument or pipe content to stdin");
            std::process::exit(1);
        }

        buffer.trim().to_string()
    };

    let input = linear_sdk::CreateCommentInput { body, issue_id: id };

    let spinner = SpinnerGuard::new("Adding comment...", is_interactive);
    match client.create_comment(input).await {
        Ok(created_comment) => {
            drop(spinner);

            if is_interactive {
                cli_output.success(&format!(
                    "Added comment to issue: {}",
                    created_comment.issue.identifier
                ));
                println!("Comment: {}", created_comment.body);
                println!("Author: {}", created_comment.user.name);
                println!(
                    "Issue: {} - {}",
                    created_comment.issue.identifier, created_comment.issue.title
                );
            } else {
                // Non-interactive: just print the comment ID for scripting
                println!("{}", created_comment.id);
            }
        }
        Err(e) => {
            drop(spinner);
            display_error(&e, use_color);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Apply configuration defaults to CLI arguments where not explicitly provided
fn apply_config_defaults(cli: &mut Cli, config: &Config) {
    match &mut cli.command {
        Commands::Issues {
            assignee,
            team,
            json,
            ..
        } => {
            // Apply default assignee if not specified
            if assignee.is_none() {
                *assignee = config.default_assignee.clone();
            }

            // Apply default team if not specified
            if team.is_none() {
                *team = config.default_team.clone();
            }

            // Apply preferred format if not specified and format is table->json
            if !*json {
                if let Some(ref format) = config.preferred_format {
                    if format == "json" {
                        *json = true;
                    }
                }
            }
        }
        Commands::Issue { json, .. } => {
            // Apply preferred format if not specified
            if !*json {
                if let Some(ref format) = config.preferred_format {
                    if format == "json" {
                        *json = true;
                    }
                }
            }
        }
        Commands::Create { team, assignee, .. } => {
            // Apply default team if not specified
            if team.is_none() {
                *team = config.default_team.clone();
            }

            // Apply default assignee if not specified
            if assignee.is_none() {
                *assignee = config.default_assignee.clone();
            }
        }
        Commands::Update { assignee, .. } => {
            // Apply default assignee if not specified and not setting to unassigned
            if assignee.is_none() {
                *assignee = config.default_assignee.clone();
            }
        }
        _ => {
            // Other commands don't use configurable defaults
        }
    }
}

async fn run_async_commands(
    mut cli: Cli,
    config: Config,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
    // Apply config defaults to CLI arguments
    apply_config_defaults(&mut cli, &config);

    // Authentication priority:
    // 1. LINEAR_API_KEY env var
    // 2. OAuth token from keychain (if feature enabled)
    // Note: Command line --api-key flag not implemented (use env var instead)
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

    let spinner = SpinnerGuard::new("Connecting to Linear...", is_interactive);

    // Determine if this is an OAuth token (from keychain) or API key
    let is_oauth_token = env::var("LINEAR_API_KEY").is_err();

    let client = if is_oauth_token {
        #[cfg(feature = "oauth")]
        {
            // OAuth tokens need "Bearer " prefix
            let bearer_token = format!("Bearer {auth_token}");
            match LinearClient::builder()
                .auth_token(SecretString::new(bearer_token.into_boxed_str()))
                .base_url(config.api_url.clone())
                .verbose(cli.verbose)
                .build()
            {
                Ok(client) => {
                    drop(spinner);
                    client
                }
                Err(e) => {
                    drop(spinner);
                    display_error(&e, use_color);
                    std::process::exit(1);
                }
            }
        }
        #[cfg(not(feature = "oauth"))]
        {
            // This should never happen because we check for oauth feature above
            unreachable!()
        }
    } else {
        match LinearClient::builder()
            .auth_token(SecretString::new(auth_token.into_boxed_str()))
            .base_url(config.api_url.clone())
            .verbose(cli.verbose)
            .build()
        {
            Ok(client) => {
                drop(spinner);
                client
            }
            Err(e) => {
                drop(spinner);
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

            let spinner = SpinnerGuard::new("Fetching issues...", is_interactive);
            let issues = match client.list_issues_filtered(limit, filters).await {
                Ok(issues) => {
                    drop(spinner);
                    issues
                }
                Err(e) => {
                    drop(spinner);
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
                println!("{output}");
            }
        }
        Commands::Issue { id, json, raw } => {
            let spinner = SpinnerGuard::new(&format!("Fetching issue {id}..."), is_interactive);
            match client.get_issue(id).await {
                Ok(issue) => {
                    drop(spinner);
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
                    println!("{output}");
                }
                Err(e) => {
                    drop(spinner);
                    display_error(&e, use_color);
                    std::process::exit(1);
                }
            }
        }
        Commands::Create {
            title,
            description,
            team,
            assignee,
            priority,
            project,
            project_id,
            from_file,
            open,
            dry_run,
        } => {
            let args = CreateCommandArgs {
                title,
                description,
                team,
                assignee,
                priority,
                project,
                project_id,
                from_file,
                open,
                dry_run,
            };
            handle_create_command(&client, args, use_color, is_interactive).await?;
        }
        Commands::Status { verbose } => {
            let spinner = SpinnerGuard::new("Checking Linear connection...", is_interactive);
            match client.execute_viewer_query().await {
                Ok(viewer_data) => {
                    drop(spinner);
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
                    drop(spinner);
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
        Commands::Update {
            id,
            title,
            description,
            assignee,
            status,
            priority,
            project,
            project_id,
            force,
        } => {
            handle_update_command(
                &client,
                UpdateCommandArgs {
                    id,
                    title,
                    description,
                    assignee,
                    status,
                    priority,
                    project,
                    project_id,
                    force,
                },
                use_color,
                is_interactive,
            )
            .await?;
        }
        Commands::Close { id, force } => {
            handle_close_command(&client, id, force, use_color, is_interactive).await?;
        }
        Commands::Reopen { id, force } => {
            handle_reopen_command(&client, id, force, use_color, is_interactive).await?;
        }
        Commands::Comment { id, message } => {
            handle_comment_command(&client, id, message, use_color, is_interactive).await?;
        }
        Commands::Projects {
            limit,
            json,
            pretty: _,
        } => {
            let spinner = SpinnerGuard::new("Fetching projects...", is_interactive);
            let projects = match client.list_projects(limit).await {
                Ok(projects) => {
                    drop(spinner);
                    projects
                }
                Err(e) => {
                    drop(spinner);
                    display_error(&e, use_color);
                    std::process::exit(1);
                }
            };

            if projects.is_empty() && !json && is_interactive {
                println!("No projects found.");
            } else if !projects.is_empty() {
                let output = if json {
                    match serde_json::to_string_pretty(&projects) {
                        Ok(output) => output,
                        Err(e) => {
                            display_error(&LinearError::from(e), use_color);
                            std::process::exit(1);
                        }
                    }
                } else {
                    projects
                        .iter()
                        .map(|p| format!("{}: {} ({})", p.id, p.name, p.state))
                        .collect::<Vec<_>>()
                        .join("\n")
                };
                println!("{output}");
            }
        }
        Commands::Teams { json, pretty: _ } => {
            let spinner = SpinnerGuard::new("Fetching teams...", is_interactive);
            let teams = match client.list_teams().await {
                Ok(teams) => {
                    drop(spinner);
                    teams
                }
                Err(e) => {
                    drop(spinner);
                    display_error(&e, use_color);
                    std::process::exit(1);
                }
            };

            if teams.is_empty() && !json && is_interactive {
                println!("No teams found.");
            } else if !teams.is_empty() {
                let output = if json {
                    match serde_json::to_string_pretty(&teams) {
                        Ok(output) => output,
                        Err(e) => {
                            display_error(&LinearError::from(e), use_color);
                            std::process::exit(1);
                        }
                    }
                } else {
                    teams
                        .iter()
                        .map(|t| format!("{}: {} ({} members)", t.key, t.name, t.members.len()))
                        .collect::<Vec<_>>()
                        .join("\n")
                };
                println!("{output}");
            }
        }
        Commands::Comments {
            id,
            limit,
            json,
            pretty: _,
        } => {
            let spinner = SpinnerGuard::new("Fetching comments...", is_interactive);
            let issue_with_comments = match client.get_issue_comments(&id, limit).await {
                Ok(result) => {
                    drop(spinner);
                    result
                }
                Err(e) => {
                    drop(spinner);
                    display_error(&e, use_color);
                    std::process::exit(1);
                }
            };

            if issue_with_comments.comments.is_empty() && !json && is_interactive {
                println!("No comments found for issue {id}.");
            } else {
                let output = if json {
                    match serde_json::to_string_pretty(&issue_with_comments) {
                        Ok(output) => output,
                        Err(e) => {
                            display_error(&LinearError::from(e), use_color);
                            std::process::exit(1);
                        }
                    }
                } else {
                    format!(
                        "Issue: {} - {}\n\nComments:\n{}",
                        issue_with_comments.identifier,
                        issue_with_comments.title,
                        issue_with_comments
                            .comments
                            .iter()
                            .map(|c| format!("{}: {}", c.user.name, c.body))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                };
                println!("{output}");
            }
        }
        Commands::MyWork {
            limit,
            json,
            pretty: _,
        } => {
            let spinner = SpinnerGuard::new("Fetching your work...", is_interactive);
            let my_work = match client.get_my_work(limit).await {
                Ok(work) => {
                    drop(spinner);
                    work
                }
                Err(e) => {
                    drop(spinner);
                    display_error(&e, use_color);
                    std::process::exit(1);
                }
            };

            let output = if json {
                match serde_json::to_string_pretty(&my_work) {
                    Ok(output) => output,
                    Err(e) => {
                        display_error(&LinearError::from(e), use_color);
                        std::process::exit(1);
                    }
                }
            } else {
                format!(
                    "Assigned to you:\n{}\n\nCreated by you:\n{}",
                    my_work
                        .assigned_issues
                        .iter()
                        .map(|i| format!("{}: {}", i.identifier, i.title))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    my_work
                        .created_issues
                        .iter()
                        .map(|i| format!("{}: {}", i.identifier, i.title))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            };
            println!("{output}");
        }
        Commands::Search {
            query,
            issues_only,
            docs_only,
            projects_only,
            limit,
            json,
            pretty,
            include_archived,
        } => {
            use crate::search::{search, SearchOptions};

            let spinner = SpinnerGuard::new("Searching...", is_interactive);

            let options = SearchOptions {
                query,
                issues_only,
                docs_only,
                projects_only,
                limit,
                include_archived,
            };

            let result = match search(&client, options).await {
                Ok(result) => {
                    drop(spinner);
                    result
                }
                Err(e) => {
                    drop(spinner);
                    display_error(&e, use_color);
                    std::process::exit(1);
                }
            };

            if json {
                let output = if pretty {
                    match serde_json::to_string_pretty(&result) {
                        Ok(output) => output,
                        Err(e) => {
                            display_error(&LinearError::from(e), use_color);
                            std::process::exit(1);
                        }
                    }
                } else {
                    match serde_json::to_string(&result) {
                        Ok(output) => output,
                        Err(e) => {
                            display_error(&LinearError::from(e), use_color);
                            std::process::exit(1);
                        }
                    }
                };
                println!("{output}");
            } else {
                // Display grouped results
                let has_results = !result.issues.is_empty()
                    || !result.documents.is_empty()
                    || !result.projects.is_empty();

                if !has_results {
                    if is_interactive {
                        println!("No results found.");
                    }
                } else {
                    if !result.issues.is_empty() {
                        println!("Issues:");
                        for issue in &result.issues {
                            println!("  {}: {}", issue.identifier, issue.title);
                        }
                        if !result.documents.is_empty() || !result.projects.is_empty() {
                            println!();
                        }
                    }

                    if !result.documents.is_empty() {
                        println!("Documents:");
                        for doc in &result.documents {
                            println!("  {}: {}", doc.title, doc.url);
                        }
                        if !result.projects.is_empty() {
                            println!();
                        }
                    }

                    if !result.projects.is_empty() {
                        println!("Projects:");
                        for project in &result.projects {
                            println!("  {}: {}", project.name, project.url);
                        }
                    }
                }
            }
        }
        Commands::Completions { .. } => {
            // This should never be reached because completions are handled synchronously above
            unreachable!("Completions command should be handled synchronously")
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
            Commands::Create { .. } => panic!("Expected Issues command"),
            Commands::Update { .. } => panic!("Expected Issues command"),
            Commands::Close { .. } => panic!("Expected Issues command"),
            Commands::Reopen { .. } => panic!("Expected Issues command"),
            Commands::Comment { .. } => panic!("Expected Issues command"),
            Commands::Projects { .. } => panic!("Expected Issues command"),
            Commands::Teams { .. } => panic!("Expected Issues command"),
            Commands::Comments { .. } => panic!("Expected Issues command"),
            Commands::MyWork { .. } => panic!("Expected Issues command"),
            Commands::Search { .. } => panic!("Expected Issues command"),
            Commands::Status { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
                panic!("Expected Issues command")
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
            Commands::Issue { .. } => panic!("Expected Issues command"),
            Commands::Create { .. } => panic!("Expected Issues command"),
            Commands::Update { .. } => panic!("Expected Issues command"),
            Commands::Close { .. } => panic!("Expected Issues command"),
            Commands::Reopen { .. } => panic!("Expected Issues command"),
            Commands::Comment { .. } => panic!("Expected Issues command"),
            Commands::Projects { .. } => panic!("Expected Issues command"),
            Commands::Teams { .. } => panic!("Expected Issues command"),
            Commands::Comments { .. } => panic!("Expected Issues command"),
            Commands::MyWork { .. } => panic!("Expected Issues command"),
            Commands::Search { .. } => panic!("Expected Issues command"),
            Commands::Status { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
                panic!("Expected Issues command")
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
            Commands::Issue { .. } => panic!("Expected Issues command"),
            Commands::Create { .. } => panic!("Expected Issues command"),
            Commands::Update { .. } => panic!("Expected Issues command"),
            Commands::Close { .. } => panic!("Expected Issues command"),
            Commands::Reopen { .. } => panic!("Expected Issues command"),
            Commands::Comment { .. } => panic!("Expected Issues command"),
            Commands::Projects { .. } => panic!("Expected Issues command"),
            Commands::Teams { .. } => panic!("Expected Issues command"),
            Commands::Comments { .. } => panic!("Expected Issues command"),
            Commands::MyWork { .. } => panic!("Expected Issues command"),
            Commands::Search { .. } => panic!("Expected Issues command"),
            Commands::Status { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
                panic!("Expected Issues command")
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
            Commands::Issue { .. } => panic!("Expected Issues command"),
            Commands::Create { .. } => panic!("Expected Issues command"),
            Commands::Update { .. } => panic!("Expected Issues command"),
            Commands::Close { .. } => panic!("Expected Issues command"),
            Commands::Reopen { .. } => panic!("Expected Issues command"),
            Commands::Comment { .. } => panic!("Expected Issues command"),
            Commands::Projects { .. } => panic!("Expected Issues command"),
            Commands::Teams { .. } => panic!("Expected Issues command"),
            Commands::Comments { .. } => panic!("Expected Issues command"),
            Commands::MyWork { .. } => panic!("Expected Issues command"),
            Commands::Search { .. } => panic!("Expected Issues command"),
            Commands::Status { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
                panic!("Expected Issues command")
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
            Commands::Issue { .. } => panic!("Expected Issues command"),
            Commands::Create { .. } => panic!("Expected Issues command"),
            Commands::Update { .. } => panic!("Expected Issues command"),
            Commands::Close { .. } => panic!("Expected Issues command"),
            Commands::Reopen { .. } => panic!("Expected Issues command"),
            Commands::Comment { .. } => panic!("Expected Issues command"),
            Commands::Projects { .. } => panic!("Expected Issues command"),
            Commands::Teams { .. } => panic!("Expected Issues command"),
            Commands::Comments { .. } => panic!("Expected Issues command"),
            Commands::MyWork { .. } => panic!("Expected Issues command"),
            Commands::Search { .. } => panic!("Expected Issues command"),
            Commands::Status { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
                panic!("Expected Issues command")
            }
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
            Commands::Issue { id, json, raw, .. } => {
                assert_eq!(id, "ENG-123");
                assert!(!json);
                assert!(!raw);
            }
            #[cfg(feature = "oauth")]
            Commands::Projects { .. } => panic!("Expected Issue command"),
            Commands::Teams { .. } => panic!("Expected Issue command"),
            Commands::Comments { .. } => panic!("Expected Issue command"),
            Commands::MyWork { .. } => panic!("Expected Issue command"),
            Commands::Search { .. } => panic!("Expected Issue command"),
            Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
                panic!("Expected Issue command")
            }
            _ => panic!("Expected Issue command"),
        }

        // Test issue command with JSON
        let cli = Cli::try_parse_from(["linear", "issue", "ENG-456", "--json"]).unwrap();
        match cli.command {
            Commands::Issue { id, json, raw, .. } => {
                assert_eq!(id, "ENG-456");
                assert!(json);
                assert!(!raw);
            }
            #[cfg(feature = "oauth")]
            Commands::Projects { .. } => panic!("Expected Issue command"),
            Commands::Teams { .. } => panic!("Expected Issue command"),
            Commands::Comments { .. } => panic!("Expected Issue command"),
            Commands::MyWork { .. } => panic!("Expected Issue command"),
            Commands::Search { .. } => panic!("Expected Issue command"),
            Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
                panic!("Expected Issue command")
            }
            _ => panic!("Expected Issue command"),
        }

        // Test issue command with UUID
        let cli = Cli::try_parse_from(["linear", "issue", "abc-123-def-456"]).unwrap();
        match cli.command {
            Commands::Issue { id, json, raw, .. } => {
                assert_eq!(id, "abc-123-def-456");
                assert!(!json);
                assert!(!raw);
            }
            #[cfg(feature = "oauth")]
            Commands::Projects { .. } => panic!("Expected Issue command"),
            Commands::Teams { .. } => panic!("Expected Issue command"),
            Commands::Comments { .. } => panic!("Expected Issue command"),
            Commands::MyWork { .. } => panic!("Expected Issue command"),
            Commands::Search { .. } => panic!("Expected Issue command"),
            Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
                panic!("Expected Issue command")
            }
            _ => panic!("Expected Issue command"),
        }

        // Test issue command with --raw flag
        let cli = Cli::try_parse_from(["linear", "issue", "ENG-789", "--raw"]).unwrap();
        match cli.command {
            Commands::Issue { id, json, raw, .. } => {
                assert_eq!(id, "ENG-789");
                assert!(!json);
                assert!(raw);
            }
            #[cfg(feature = "oauth")]
            Commands::Projects { .. } => panic!("Expected Issue command"),
            Commands::Teams { .. } => panic!("Expected Issue command"),
            Commands::Comments { .. } => panic!("Expected Issue command"),
            Commands::MyWork { .. } => panic!("Expected Issue command"),
            Commands::Search { .. } => panic!("Expected Issue command"),
            Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
                panic!("Expected Issue command")
            }
            _ => panic!("Expected Issue command"),
        }

        // Test issue command with both --json and --raw flags
        let cli = Cli::try_parse_from(["linear", "issue", "ENG-999", "--json", "--raw"]).unwrap();
        match cli.command {
            Commands::Issue { id, json, raw, .. } => {
                assert_eq!(id, "ENG-999");
                assert!(json);
                assert!(raw);
            }
            #[cfg(feature = "oauth")]
            Commands::Projects { .. } => panic!("Expected Issue command"),
            Commands::Teams { .. } => panic!("Expected Issue command"),
            Commands::Comments { .. } => panic!("Expected Issue command"),
            Commands::MyWork { .. } => panic!("Expected Issue command"),
            Commands::Search { .. } => panic!("Expected Issue command"),
            Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
                panic!("Expected Issue command")
            }
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
            Commands::Create { .. } => panic!("Expected Issues command"),
            Commands::Update { .. } => panic!("Expected Issues command"),
            Commands::Close { .. } => panic!("Expected Issues command"),
            Commands::Reopen { .. } => panic!("Expected Issues command"),
            Commands::Comment { .. } => panic!("Expected Issues command"),
            Commands::Projects { .. } => panic!("Expected Issues command"),
            Commands::Teams { .. } => panic!("Expected Issues command"),
            Commands::Comments { .. } => panic!("Expected Issues command"),
            Commands::MyWork { .. } => panic!("Expected Issues command"),
            Commands::Search { .. } => panic!("Expected Issues command"),
            Commands::Status { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
                panic!("Expected Issues command")
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
            Commands::Issue { .. } => panic!("Expected Issues command"),
            Commands::Create { .. } => panic!("Expected Issues command"),
            Commands::Update { .. } => panic!("Expected Issues command"),
            Commands::Close { .. } => panic!("Expected Issues command"),
            Commands::Reopen { .. } => panic!("Expected Issues command"),
            Commands::Comment { .. } => panic!("Expected Issues command"),
            Commands::Projects { .. } => panic!("Expected Issues command"),
            Commands::Teams { .. } => panic!("Expected Issues command"),
            Commands::Comments { .. } => panic!("Expected Issues command"),
            Commands::MyWork { .. } => panic!("Expected Issues command"),
            Commands::Search { .. } => panic!("Expected Issues command"),
            Commands::Status { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
                panic!("Expected Issues command")
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
            Commands::Issue { .. } => panic!("Expected Issues command"),
            Commands::Create { .. } => panic!("Expected Issues command"),
            Commands::Update { .. } => panic!("Expected Issues command"),
            Commands::Close { .. } => panic!("Expected Issues command"),
            Commands::Reopen { .. } => panic!("Expected Issues command"),
            Commands::Comment { .. } => panic!("Expected Issues command"),
            Commands::Projects { .. } => panic!("Expected Issues command"),
            Commands::Teams { .. } => panic!("Expected Issues command"),
            Commands::Comments { .. } => panic!("Expected Issues command"),
            Commands::MyWork { .. } => panic!("Expected Issues command"),
            Commands::Search { .. } => panic!("Expected Issues command"),
            Commands::Status { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
                panic!("Expected Issues command")
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
            Commands::Issue { .. } => panic!("Expected Issues command"),
            Commands::Create { .. } => panic!("Expected Issues command"),
            Commands::Update { .. } => panic!("Expected Issues command"),
            Commands::Close { .. } => panic!("Expected Issues command"),
            Commands::Reopen { .. } => panic!("Expected Issues command"),
            Commands::Comment { .. } => panic!("Expected Issues command"),
            Commands::Projects { .. } => panic!("Expected Issues command"),
            Commands::Teams { .. } => panic!("Expected Issues command"),
            Commands::Comments { .. } => panic!("Expected Issues command"),
            Commands::MyWork { .. } => panic!("Expected Issues command"),
            Commands::Search { .. } => panic!("Expected Issues command"),
            Commands::Status { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout | Commands::Completions { .. } => {
                panic!("Expected Issues command")
            }
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

        let cli = Cli::try_parse_from(["linear", "create", "--title", "Test"]).unwrap();
        match cli.command {
            Commands::Create { .. } => {} // Success - parsing works without auth
            _ => panic!("Expected Create command"),
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
                state_id: "state-todo-123".to_string(),
                assignee: Some("Alice".to_string()),
                assignee_id: Some("user-1".to_string()),
                team: Some("ENG".to_string()),
                team_id: "team-eng-123".to_string(),
            },
            Issue {
                id: "2".to_string(),
                identifier: "ENG-124".to_string(),
                title: "Another issue".to_string(),
                status: "Done".to_string(),
                state_id: "state-done-124".to_string(),
                assignee: None,
                assignee_id: None,
                team: Some("ENG".to_string()),
                team_id: "team-eng-124".to_string(),
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

    // CREATE COMMAND TESTS - Testing CLI parsing and validation

    #[test]
    fn test_parse_create_command_minimal() {
        use clap::Parser;

        // Test minimal create command with just title and team
        let cli =
            Cli::try_parse_from(["linear", "create", "--title", "Test Issue", "--team", "ENG"])
                .unwrap();

        match cli.command {
            Commands::Create {
                title,
                description,
                team,
                assignee,
                priority,
                open,
                dry_run,
                ..
            } => {
                assert_eq!(title, Some("Test Issue".to_string()));
                assert_eq!(description, None);
                assert_eq!(team, Some("ENG".to_string()));
                assert_eq!(assignee, None);
                assert_eq!(priority, None);
                assert!(!open);
                assert!(!dry_run);
            }
            _ => panic!("Expected Create command"),
        }
    }

    #[test]
    fn test_parse_create_command_all_fields() {
        use clap::Parser;

        // Test create command with all possible arguments
        let cli = Cli::try_parse_from([
            "linear",
            "create",
            "--title",
            "Complete Test Issue",
            "--description",
            "A complete test description",
            "--team",
            "DESIGN",
            "--assignee",
            "me",
            "--priority",
            "2",
            "--open",
            "--dry-run",
        ])
        .unwrap();

        match cli.command {
            Commands::Create {
                title,
                description,
                team,
                assignee,
                priority,
                open,
                dry_run,
                ..
            } => {
                assert_eq!(title, Some("Complete Test Issue".to_string()));
                assert_eq!(description, Some("A complete test description".to_string()));
                assert_eq!(team, Some("DESIGN".to_string()));
                assert_eq!(assignee, Some("me".to_string()));
                assert_eq!(priority, Some(2));
                assert!(open);
                assert!(dry_run);
            }
            _ => panic!("Expected Create command"),
        }
    }

    #[test]
    fn test_parse_create_command_short_flags() {
        use clap::Parser;

        // Test create command with short flag aliases where available
        let cli = Cli::try_parse_from([
            "linear",
            "create",
            "--title",
            "Short Flag Test",
            "--team",
            "ENG",
            "--priority",
            "1",
        ])
        .unwrap();

        match cli.command {
            Commands::Create {
                title,
                description: _,
                team,
                assignee: _,
                priority,
                open: _,
                dry_run: _,
                ..
            } => {
                assert_eq!(title, Some("Short Flag Test".to_string()));
                assert_eq!(team, Some("ENG".to_string()));
                assert_eq!(priority, Some(1));
            }
            _ => panic!("Expected Create command"),
        }
    }

    #[test]
    fn test_parse_create_command_interactive_mode() {
        use clap::Parser;

        // Test create command without any arguments (should trigger interactive mode)
        let cli = Cli::try_parse_from(["linear", "create"]).unwrap();

        match cli.command {
            Commands::Create {
                title,
                description,
                team,
                assignee,
                priority,
                open,
                dry_run,
                ..
            } => {
                assert_eq!(title, None);
                assert_eq!(description, None);
                assert_eq!(team, None);
                assert_eq!(assignee, None);
                assert_eq!(priority, None);
                assert!(!open);
                assert!(!dry_run);
            }
            _ => panic!("Expected Create command"),
        }
    }

    #[test]
    fn test_parse_create_command_priority_validation() {
        use clap::Parser;

        // Test valid priority values
        for priority in 1..=4 {
            let cli = Cli::try_parse_from([
                "linear",
                "create",
                "--title",
                "Priority Test",
                "--team",
                "ENG",
                "--priority",
                &priority.to_string(),
            ])
            .unwrap();

            match cli.command {
                Commands::Create { priority: p, .. } => {
                    assert_eq!(p, Some(priority));
                }
                _ => panic!("Expected Create command"),
            }
        }
    }

    #[test]
    fn test_parse_create_command_invalid_priority() {
        use clap::Parser;

        // Test invalid priority values (should fail parsing)
        for invalid_priority in [0, 5, 10] {
            let result = Cli::try_parse_from([
                "linear",
                "create",
                "--title",
                "Priority Test",
                "--team",
                "ENG",
                "--priority",
                &invalid_priority.to_string(),
            ]);

            assert!(
                result.is_err(),
                "Priority {invalid_priority} should be invalid"
            );
        }
    }

    #[test]
    fn test_parse_create_command_special_assignees() {
        use clap::Parser;

        // Test special assignee values
        let special_assignees = ["me", "unassigned"];

        for assignee in &special_assignees {
            let cli = Cli::try_parse_from([
                "linear",
                "create",
                "--title",
                "Assignee Test",
                "--team",
                "ENG",
                "--assignee",
                assignee,
            ])
            .unwrap();

            match cli.command {
                Commands::Create { assignee: a, .. } => {
                    assert_eq!(a, Some(assignee.to_string()));
                }
                _ => panic!("Expected Create command"),
            }
        }
    }

    #[test]
    fn test_create_command_args_struct() {
        // Test the CreateCommandArgs structure used internally
        let args = CreateCommandArgs {
            title: Some("Test Title".to_string()),
            description: Some("Test Description".to_string()),
            team: Some("ENG".to_string()),
            assignee: Some("me".to_string()),
            priority: Some(2),
            project: None,
            project_id: None,
            from_file: None,
            open: true,
            dry_run: false,
        };

        assert_eq!(args.title, Some("Test Title".to_string()));
        assert_eq!(args.description, Some("Test Description".to_string()));
        assert_eq!(args.team, Some("ENG".to_string()));
        assert_eq!(args.assignee, Some("me".to_string()));
        assert_eq!(args.priority, Some(2));
        assert_eq!(args.from_file, None);
        assert!(args.open);
        assert!(!args.dry_run);
    }

    #[test]
    fn test_parse_create_command_whitespace_handling() {
        use clap::Parser;

        // Test that whitespace in arguments is properly handled
        let cli = Cli::try_parse_from([
            "linear",
            "create",
            "--title",
            "  Title with spaces  ",
            "--description",
            "  Description with\nmultiple lines  ",
            "--team",
            " ENG ",
            "--assignee",
            " test@example.com ",
        ])
        .unwrap();

        match cli.command {
            Commands::Create {
                title,
                description,
                team,
                assignee,
                ..
            } => {
                // Arguments should preserve whitespace as-is (trimming is handled later)
                assert_eq!(title, Some("  Title with spaces  ".to_string()));
                assert_eq!(
                    description,
                    Some("  Description with\nmultiple lines  ".to_string())
                );
                assert_eq!(team, Some(" ENG ".to_string()));
                assert_eq!(assignee, Some(" test@example.com ".to_string()));
            }
            _ => panic!("Expected Create command"),
        }
    }

    #[test]
    fn test_parse_create_command_from_file() {
        use clap::Parser;

        // Test create command with --from-file flag
        let cli = Cli::try_parse_from(["linear", "create", "--from-file", "issue.md"]).unwrap();

        match cli.command {
            Commands::Create { from_file, .. } => {
                assert_eq!(from_file, Some("issue.md".to_string()));
            }
            _ => panic!("Expected Create command"),
        }
    }

    #[test]
    fn test_parse_create_command_from_file_short() {
        use clap::Parser;

        // Test create command with -f short flag
        let cli = Cli::try_parse_from(["linear", "create", "-f", "/path/to/issue.md"]).unwrap();

        match cli.command {
            Commands::Create { from_file, .. } => {
                assert_eq!(from_file, Some("/path/to/issue.md".to_string()));
            }
            _ => panic!("Expected Create command"),
        }
    }

    #[test]
    fn test_parse_create_command_from_file_with_other_args() {
        use clap::Parser;

        // Test create command with --from-file and other arguments (CLI args should override)
        let cli = Cli::try_parse_from([
            "linear",
            "create",
            "--from-file",
            "issue.md",
            "--title",
            "Override Title",
            "--team",
            "OVERRIDE",
            "--dry-run",
        ])
        .unwrap();

        match cli.command {
            Commands::Create {
                from_file,
                title,
                team,
                dry_run,
                ..
            } => {
                assert_eq!(from_file, Some("issue.md".to_string()));
                assert_eq!(title, Some("Override Title".to_string()));
                assert_eq!(team, Some("OVERRIDE".to_string()));
                assert!(dry_run);
            }
            _ => panic!("Expected Create command"),
        }
    }

    #[test]
    fn test_parse_create_command_empty_values() {
        use clap::Parser;

        // Test parsing with empty string values
        let cli = Cli::try_parse_from([
            "linear",
            "create",
            "--title",
            "",
            "--description",
            "",
            "--team",
            "",
            "--assignee",
            "",
        ])
        .unwrap();

        match cli.command {
            Commands::Create {
                title,
                description,
                team,
                assignee,
                ..
            } => {
                // Empty strings should still be parsed as Some("")
                assert_eq!(title, Some("".to_string()));
                assert_eq!(description, Some("".to_string()));
                assert_eq!(team, Some("".to_string()));
                assert_eq!(assignee, Some("".to_string()));
            }
            _ => panic!("Expected Create command"),
        }
    }
}
