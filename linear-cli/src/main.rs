// ABOUTME: Main entry point for the Linear CLI application
// ABOUTME: Provides command-line interface for Linear issue tracking

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use linear_sdk::constants::status::{DEFAULT_DONE_STATE, DEFAULT_TODO_STATE};
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
#[cfg(test)]
mod tests;
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

    let help = error.help_text();
    if !help.is_empty() {
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
    estimate: Option<i64>,
    labels: Vec<String>,
    cycle: Option<String>,
    project: Option<String>,
    project_id: Option<String>,
    from_file: Option<String>,
    open: bool,
    dry_run: bool,
}

/// Resolve assignee string to user ID.
/// Handles special values: "me"/"self" -> current user, "unassigned"/"none" -> None
/// UUIDs are passed through as-is.
async fn resolve_assignee_to_id(
    client: &LinearClient,
    assignee: Option<String>,
) -> Result<Option<String>> {
    let Some(assignee) = assignee else {
        return Ok(None);
    };

    if assignee.trim().is_empty()
        || assignee.eq_ignore_ascii_case("unassigned")
        || assignee.eq_ignore_ascii_case("none")
        || assignee.eq_ignore_ascii_case("null")
    {
        return Ok(None);
    }

    if assignee.eq_ignore_ascii_case("me") || assignee.eq_ignore_ascii_case("self") {
        let viewer_data = client.execute_viewer_query().await?;
        return Ok(Some(viewer_data.viewer.id));
    }

    // UUID or email/name - pass through as-is
    Ok(Some(assignee))
}

/// Resolve team string to team ID.
/// If the input looks like a UUID, it's passed through. Otherwise, it's resolved as a team key.
async fn resolve_team_to_id(client: &LinearClient, team: &str) -> Result<String> {
    // Check if it looks like a UUID (hex digits and hyphens, longer than 20 chars)
    if team.chars().all(|c| c.is_ascii_hexdigit() || c == '-') && team.len() > 20 {
        return Ok(team.to_string());
    }

    // Resolve as team key
    client.resolve_team_key_to_id(team).await
}

/// Resolve a team name/key to an ID, exiting with an error message on failure.
async fn resolve_team_or_exit(
    client: &LinearClient,
    cli_output: &CliOutput,
    team: Option<String>,
    required_msg: &str,
) -> String {
    let team = match team {
        Some(t) => t,
        None => {
            cli_output.error(required_msg);
            std::process::exit(1);
        }
    };
    match resolve_team_to_id(client, &team).await {
        Ok(id) => id,
        Err(e) => {
            cli_output.error(&format!("Failed to resolve team '{team}': {e}"));
            std::process::exit(1);
        }
    }
}

/// Resolve an assignee name to an ID, exiting with an error message on failure.
async fn resolve_assignee_or_exit(
    client: &LinearClient,
    cli_output: &CliOutput,
    assignee: Option<String>,
) -> Option<String> {
    let assignee = assignee.unwrap_or_else(|| "me".to_string());
    match resolve_assignee_to_id(client, Some(assignee)).await {
        Ok(id) => id,
        Err(e) => {
            cli_output.error(&format!("Failed to resolve assignee: {e}"));
            std::process::exit(1);
        }
    }
}

/// Fields used for dry-run preview output when creating an issue
struct DryRunPreview<'a> {
    header: &'a str,
    input: &'a crate::interactive::InteractiveCreateInput,
    labels: &'a [String],
    cycle: Option<&'a str>,
    project: Option<&'a str>,
}

fn print_dry_run_preview(cli_output: &CliOutput, preview: &DryRunPreview) {
    cli_output.info("Dry run mode - no issue will be created");
    println!();
    println!("{}:", preview.header);
    println!("  Title: {}", preview.input.title);
    if let Some(desc) = &preview.input.description {
        println!("  Description: {desc}");
    }
    println!("  Team ID: {}", preview.input.team_id);
    if let Some(assignee_id) = &preview.input.assignee_id {
        println!("  Assignee ID: {assignee_id}");
    }
    if let Some(priority) = preview.input.priority {
        println!("  Priority: {priority}");
    }
    if let Some(estimate) = preview.input.estimate {
        println!("  Estimate: {estimate}");
    }
    if !preview.labels.is_empty() {
        println!("  Labels: {}", preview.labels.join(", "));
    }
    if let Some(cycle) = preview.cycle {
        println!("  Cycle: {cycle}");
    }
    if let Some(project) = preview.project {
        println!("  Project: {project}");
    }
}

/// Display a successfully created issue
fn display_created_issue(
    cli_output: &CliOutput,
    issue: &linear_sdk::CreatedIssue,
    open: bool,
    is_interactive: bool,
) {
    if is_interactive {
        cli_output.success(&format!("Created issue: {}", issue.identifier));
        println!("Title: {}", issue.title);
        if let Some(desc) = &issue.description {
            println!("Description: {desc}");
        }
        println!("Status: {}", issue.state.name);
        if let Some(assignee) = &issue.assignee {
            println!("Assignee: {}", assignee.name);
        }
        if let Some(team) = &issue.team {
            println!("Team: {} ({})", team.name, team.key);
        }
        println!("URL: {}", issue.url);

        if open {
            println!();
            cli_output.info(&format!("Issue URL: {}", issue.url));
            cli_output.info("Please open this URL in your browser");
        }
    } else {
        println!("{}", issue.identifier);
    }
}

/// Resolve a project name to its ID using the interactive prompter.
/// Returns project_id from --project-id if --project is not specified.
async fn resolve_project_to_id(
    client: &LinearClient,
    cli_output: &CliOutput,
    project_name: Option<&str>,
    project_id_fallback: Option<String>,
) -> Result<Option<String>> {
    if let Some(name) = project_name {
        use crate::interactive::InteractivePrompter;
        let prompter = InteractivePrompter::new(client).unwrap();
        match prompter.resolve_project(name).await {
            Ok(id) => Ok(id),
            Err(e) => {
                cli_output.error(&format!("Failed to resolve project '{name}': {e}"));
                std::process::exit(1);
            }
        }
    } else {
        Ok(project_id_fallback)
    }
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
    let estimate = args.estimate.or(markdown_file.frontmatter.estimate);
    let labels: Vec<String> = if !args.labels.is_empty() {
        args.labels.clone()
    } else {
        markdown_file.frontmatter.labels.clone().unwrap_or_default()
    };
    let cycle = args
        .cycle
        .as_ref()
        .or(markdown_file.frontmatter.cycle.as_ref())
        .cloned();

    // Validate required fields
    if title.trim().is_empty() {
        cli_output.error("Title is required (specify in frontmatter or use --title)");
        std::process::exit(1);
    }

    let team_id = resolve_team_or_exit(
        client,
        &cli_output,
        team,
        "Team is required (specify in frontmatter or use --team)",
    )
    .await;

    let assignee_id = resolve_assignee_or_exit(client, &cli_output, assignee).await;

    let input = crate::interactive::InteractiveCreateInput {
        title,
        description,
        team_id: team_id.clone(),
        assignee_id,
        priority,
        estimate,
    };

    // Handle dry-run mode
    if args.dry_run {
        print_dry_run_preview(
            &cli_output,
            &DryRunPreview {
                header: &format!("Would create issue from file '{file_path}'"),
                input: &input,
                labels: &labels,
                cycle: cycle.as_deref(),
                project: markdown_file.frontmatter.project.as_deref(),
            },
        );
        return Ok(());
    }

    // Resolve project if provided
    let project_id = resolve_project_to_id(
        client,
        &cli_output,
        args.project.as_deref(),
        args.project_id.clone(),
    )
    .await?;

    // Resolve labels if provided
    let label_ids = if !labels.is_empty() {
        Some(client.resolve_label_names_to_ids(&team_id, &labels).await?)
    } else {
        None
    };

    // Resolve cycle if provided
    let cycle_id = if let Some(cycle_str) = &cycle {
        Some(client.resolve_cycle_to_id(&team_id, cycle_str).await?)
    } else {
        None
    };

    // Build the SDK create input
    let sdk_input = linear_sdk::CreateIssueInput {
        title: input.title,
        description: input.description,
        team_id: Some(input.team_id),
        assignee_id: input.assignee_id,
        priority: input.priority,
        project_id,
        label_ids,
        estimate: input.estimate,
        cycle_id,
    };

    // Create the issue
    let spinner = SpinnerGuard::new("Creating issue from file...", is_interactive);
    match client.create_issue(sdk_input).await {
        Ok(created_issue) => {
            drop(spinner);
            display_created_issue(&cli_output, &created_issue, args.open, is_interactive);
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
        let prompter = match InteractivePrompter::new(client) {
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
            estimate: args.estimate,
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
        let team_id = resolve_team_or_exit(
            client,
            &cli_output,
            args.team,
            "Team is required for issue creation",
        )
        .await;

        let assignee_id = resolve_assignee_or_exit(client, &cli_output, args.assignee).await;

        crate::interactive::InteractiveCreateInput {
            title,
            description: args.description,
            team_id,
            assignee_id,
            priority: args.priority,
            estimate: args.estimate,
        }
    };

    // Handle dry-run mode
    if args.dry_run {
        print_dry_run_preview(
            &cli_output,
            &DryRunPreview {
                header: "Would create issue",
                input: &input,
                labels: &args.labels,
                cycle: args.cycle.as_deref(),
                project: None,
            },
        );
        return Ok(());
    }

    // Resolve project if provided
    let project_id = resolve_project_to_id(
        client,
        &cli_output,
        args.project.as_deref(),
        args.project_id.clone(),
    )
    .await?;

    // Resolve labels if provided
    let label_ids = if !args.labels.is_empty() {
        Some(
            client
                .resolve_label_names_to_ids(&input.team_id, &args.labels)
                .await?,
        )
    } else {
        None
    };

    // Resolve cycle if provided
    let cycle_id = if let Some(cycle_str) = &args.cycle {
        Some(
            client
                .resolve_cycle_to_id(&input.team_id, cycle_str)
                .await?,
        )
    } else {
        None
    };

    // Build the SDK create input
    let sdk_input = linear_sdk::CreateIssueInput {
        title: input.title,
        description: input.description,
        team_id: Some(input.team_id),
        assignee_id: input.assignee_id,
        priority: input.priority,
        project_id,
        label_ids,
        estimate: input.estimate,
        cycle_id,
    };

    // Create the issue
    let spinner = SpinnerGuard::new("Creating issue...", is_interactive);
    match client.create_issue(sdk_input).await {
        Ok(created_issue) => {
            drop(spinner);
            display_created_issue(&cli_output, &created_issue, args.open, is_interactive);
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
    estimate: Option<i64>,
    labels: Vec<String>,
    cycle: Option<String>,
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
        && args.estimate.is_none()
        && args.labels.is_empty()
        && args.cycle.is_none()
        && args.project.is_none()
        && args.project_id.is_none()
    {
        cli_output.error("At least one field must be specified for update");
        eprintln!("Use --title, --description, --assignee, --status, --priority, --estimate, --label, --cycle, --project, or --project-id");
        std::process::exit(1);
    }

    // Resolve assignee if provided (empty string for update means "unassign")
    let assignee_id = match &args.assignee {
        Some(assignee)
            if assignee.trim().is_empty() || assignee.eq_ignore_ascii_case("unassigned") =>
        {
            Some(String::new()) // Empty string to unassign
        }
        Some(_) => match resolve_assignee_to_id(client, args.assignee.clone()).await {
            Ok(id) => id,
            Err(e) => {
                cli_output.error(&format!("Failed to resolve assignee: {e}"));
                std::process::exit(1);
            }
        },
        None => None,
    };

    // Fetch issue to get team_id if any field needs it
    let needs_team = args.status.is_some() || !args.labels.is_empty() || args.cycle.is_some();
    let issue = if needs_team {
        Some(client.get_issue(args.id.clone()).await?)
    } else {
        None
    };
    let team_id = issue
        .as_ref()
        .and_then(|i| i.team.as_ref())
        .map(|t| t.id.clone());

    // Resolve status to state_id if provided
    let state_id = if let Some(ref status_name) = args.status {
        let tid = team_id.as_ref().unwrap();
        Some(client.resolve_status_to_state_id(tid, status_name).await?)
    } else {
        None
    };

    // Resolve labels if provided
    let label_ids = if !args.labels.is_empty() {
        let tid = team_id.as_ref().unwrap();
        Some(client.resolve_label_names_to_ids(tid, &args.labels).await?)
    } else {
        None
    };

    // Resolve cycle if provided
    let cycle_id = if let Some(ref cycle_str) = args.cycle {
        let tid = team_id.as_ref().unwrap();
        Some(client.resolve_cycle_to_id(tid, cycle_str).await?)
    } else {
        None
    };

    // Resolve project if provided
    let project_id = resolve_project_to_id(
        client,
        &cli_output,
        args.project.as_deref(),
        args.project_id.clone(),
    )
    .await?;

    let input = linear_sdk::UpdateIssueInput {
        title: args.title,
        description: args.description,
        assignee_id,
        state_id,
        priority: args.priority,
        project_id,
        label_ids,
        estimate: args.estimate,
        cycle_id,
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
        if let Some(estimate) = input.estimate {
            println!("  Estimate: {estimate}");
        }
        if let Some(ref label_ids) = input.label_ids {
            println!("  Labels: {} label(s)", label_ids.len());
        }
        if let Some(ref cycle_id) = input.cycle_id {
            println!("  Cycle: {cycle_id}");
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

async fn handle_status_change(
    client: &LinearClient,
    id: String,
    action: &str,
    status_name: &str,
    force: bool,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
    let cli_output = CliOutput::with_color(use_color);

    // Show preview unless --force is used
    if !force && is_interactive {
        println!("Would {action} issue: {id}");
        println!();

        if !confirm_action(action, false, true) {
            let cancel_msg = if action == "close" {
                "Close cancelled"
            } else {
                "Reopen cancelled"
            };
            cli_output.info(cancel_msg);
            return Ok(());
        }
    }

    // Get the issue first to determine its team, then resolve the target state ID
    let issue = client.get_issue(id.clone()).await?;
    let team_id = issue.team.as_ref().unwrap().id.clone();

    let target_state_id = client
        .resolve_status_to_state_id(&team_id, status_name)
        .await?;

    let input = linear_sdk::UpdateIssueInput {
        title: None,
        description: None,
        assignee_id: None,
        state_id: Some(target_state_id),
        priority: None,
        project_id: None,
        label_ids: None,
        estimate: None,
        cycle_id: None,
    };

    let (success_msg, spinner_msg) = if action == "close" {
        ("Closed", "Closing issue...")
    } else {
        ("Reopened", "Reopening issue...")
    };

    let spinner = SpinnerGuard::new(spinner_msg, is_interactive);
    match client.update_issue(id.clone(), input).await {
        Ok(updated_issue) => {
            drop(spinner);

            if is_interactive {
                cli_output.success(&format!(
                    "{success_msg} issue: {}",
                    updated_issue.identifier
                ));
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

async fn handle_close_command(
    client: &LinearClient,
    id: String,
    force: bool,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
    handle_status_change(
        client,
        id,
        "close",
        DEFAULT_DONE_STATE,
        force,
        use_color,
        is_interactive,
    )
    .await
}

async fn handle_reopen_command(
    client: &LinearClient,
    id: String,
    force: bool,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
    handle_status_change(
        client,
        id,
        "reopen",
        DEFAULT_TODO_STATE,
        force,
        use_color,
        is_interactive,
    )
    .await
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
        use std::io::IsTerminal as _;
        if std::io::stdin().is_terminal() {
            cli_output.error("No message provided. Use --message TEXT or pipe content to stdin.");
            std::process::exit(1);
        }
        use std::io::Read;
        let mut buffer = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut buffer) {
            cli_output.error(&format!("Failed to read from stdin: {e}"));
            std::process::exit(1);
        }

        if buffer.trim().is_empty() {
            cli_output.error("Comment body cannot be empty");
            cli_output.error("Provide a message argument or pipe content to stdin");
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

async fn handle_attach_command(
    client: &LinearClient,
    id: String,
    url: String,
    title: Option<String>,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
    let cli_output = CliOutput::with_color(use_color);

    // Resolve issue identifier to UUID
    let issue = client.get_issue(id.clone()).await?;

    let input = linear_sdk::CreateAttachmentInput {
        issue_id: issue.id.clone(),
        url: url.clone(),
        title,
    };

    let spinner = SpinnerGuard::new("Attaching URL...", is_interactive);
    match client.create_attachment(input).await {
        Ok(attachment) => {
            drop(spinner);

            if is_interactive {
                cli_output.success(&format!("Attached URL to issue {}", issue.identifier));
                println!("URL: {}", attachment.url);
                println!("Title: {}", attachment.title);
            } else {
                println!("{}", attachment.id);
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

async fn handle_relate_command(
    client: &LinearClient,
    id: String,
    kind: String,
    targets: Vec<String>,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
    use linear_sdk::{CreateIssueRelationInput, IssueRelationKind};

    let cli_output = CliOutput::with_color(use_color);

    // Parse the relation kind. `blocked-by` and `duplicate-of` swap the issue pair.
    let (relation_kind, swap) = match kind.as_str() {
        "blocks" => (IssueRelationKind::Blocks, false),
        "blocked-by" => (IssueRelationKind::Blocks, true),
        "related" => (IssueRelationKind::Related, false),
        "duplicate" => (IssueRelationKind::Duplicate, false),
        "duplicate-of" => (IssueRelationKind::Duplicate, true),
        "similar" => (IssueRelationKind::Similar, false),
        _ => unreachable!("clap restricts the set of allowed values"),
    };

    let source = client.get_issue(id.clone()).await?;

    let mut had_error = false;
    for target_ref in targets {
        let target = match client.get_issue(target_ref.clone()).await {
            Ok(t) => t,
            Err(e) => {
                had_error = true;
                if is_interactive {
                    cli_output.error(&format!("Could not resolve {target_ref}: {e}"));
                } else {
                    eprintln!("{target_ref}: {e}");
                }
                continue;
            }
        };

        let (issue_id, related_issue_id) = if swap {
            (target.id.clone(), source.id.clone())
        } else {
            (source.id.clone(), target.id.clone())
        };

        let spinner = SpinnerGuard::new(
            &format!(
                "Linking {} {} {}...",
                source.identifier, kind, target.identifier
            ),
            is_interactive,
        );
        let input = CreateIssueRelationInput {
            issue_id,
            related_issue_id,
            kind: relation_kind,
        };
        match client.create_issue_relation(input).await {
            Ok(relation) => {
                drop(spinner);
                if is_interactive {
                    cli_output.success(&format!(
                        "{} {} {}",
                        relation.issue_identifier,
                        relation.kind.as_str(),
                        relation.related_issue_identifier
                    ));
                } else {
                    println!("{}", relation.id);
                }
            }
            Err(e) => {
                drop(spinner);
                had_error = true;
                display_error(&e, use_color);
            }
        }
    }

    if had_error {
        std::process::exit(1);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_issues_command(
    client: &LinearClient,
    limit: i32,
    json: bool,
    pretty: bool,
    assignee: Option<String>,
    status: Option<String>,
    team: Option<String>,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
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

    Ok(())
}

async fn handle_issue_detail_command(
    client: &LinearClient,
    id: String,
    json: bool,
    raw: bool,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
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
                let formatter = TableFormatter::new_with_interactive(use_color, is_interactive);
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

    Ok(())
}

async fn handle_status_command(
    client: &LinearClient,
    verbose: bool,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
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

    Ok(())
}

async fn handle_projects_command(
    client: &LinearClient,
    limit: i32,
    json: bool,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
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

    Ok(())
}

async fn handle_teams_command(
    client: &LinearClient,
    json: bool,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
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

    Ok(())
}

async fn handle_comments_command(
    client: &LinearClient,
    id: String,
    limit: i32,
    json: bool,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
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

    Ok(())
}

async fn handle_mywork_command(
    client: &LinearClient,
    limit: i32,
    json: bool,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
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

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_search_command(
    client: &LinearClient,
    query: String,
    issues_only: bool,
    docs_only: bool,
    projects_only: bool,
    limit: i32,
    json: bool,
    pretty: bool,
    include_archived: bool,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
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

    let result = match search(client, options).await {
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
            handle_issues_command(
                &client,
                limit,
                json,
                pretty,
                assignee,
                status,
                team,
                use_color,
                is_interactive,
            )
            .await?;
        }
        Commands::Issue { id, json, raw } => {
            handle_issue_detail_command(&client, id, json, raw, use_color, is_interactive).await?;
        }
        Commands::Create {
            title,
            description,
            team,
            assignee,
            priority,
            estimate,
            labels,
            cycle,
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
                estimate,
                labels,
                cycle,
                project,
                project_id,
                from_file,
                open,
                dry_run,
            };
            handle_create_command(&client, args, use_color, is_interactive).await?;
        }
        Commands::Status { verbose } => {
            handle_status_command(&client, verbose, use_color, is_interactive).await?;
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
            estimate,
            labels,
            cycle,
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
                    estimate,
                    labels,
                    cycle,
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
        Commands::Attach { id, url, title } => {
            handle_attach_command(&client, id, url, title, use_color, is_interactive).await?;
        }
        Commands::Relate { id, kind, targets } => {
            handle_relate_command(&client, id, kind, targets, use_color, is_interactive).await?;
        }
        Commands::Projects {
            limit,
            json,
            pretty: _,
        } => {
            handle_projects_command(&client, limit, json, use_color, is_interactive).await?;
        }
        Commands::Teams { json, pretty: _ } => {
            handle_teams_command(&client, json, use_color, is_interactive).await?;
        }
        Commands::Comments {
            id,
            limit,
            json,
            pretty: _,
        } => {
            handle_comments_command(&client, id, limit, json, use_color, is_interactive).await?;
        }
        Commands::MyWork {
            limit,
            json,
            pretty: _,
        } => {
            handle_mywork_command(&client, limit, json, use_color, is_interactive).await?;
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
            handle_search_command(
                &client,
                query,
                issues_only,
                docs_only,
                projects_only,
                limit,
                json,
                pretty,
                include_archived,
                use_color,
                is_interactive,
            )
            .await?;
        }
        Commands::Completions { .. } => {
            // This should never be reached because completions are handled synchronously above
            unreachable!("Completions command should be handled synchronously")
        }
    }

    Ok(())
}
