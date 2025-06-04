// ABOUTME: Main entry point for the Linear CLI application
// ABOUTME: Provides command-line interface for Linear issue tracking

use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use linear_sdk::{CreateIssueInput, IssueFilters, LinearClient, LinearError, Result};
use owo_colors::OwoColorize;
use secrecy::SecretString;
use std::env;
use std::io::IsTerminal;

mod cli_output;
mod constants;
mod output;
mod types;

#[cfg(feature = "inline-images")]
mod image_protocols;

use crate::cli_output::CliOutput;
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
    pb.enable_steady_tick(std::time::Duration::from_millis(
        constants::timeouts::PROGRESS_BAR_TICK_MS,
    ));
    Some(pb)
}

fn display_error(error: &LinearError, use_color: bool) {
    let cli = CliOutput::with_color(use_color);
    cli.error(&error.to_string());

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

        /// Disable inline image display (requires inline-images feature)
        #[cfg(feature = "inline-images")]
        #[arg(long)]
        no_images: bool,

        /// Force inline image display even in unsupported terminals (requires inline-images feature)
        #[cfg(feature = "inline-images")]
        #[arg(long, conflicts_with = "no_images")]
        force_images: bool,
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
    /// Create a new issue
    Create {
        /// Title of the issue
        #[arg(long)]
        title: Option<String>,

        /// Description of the issue
        #[arg(long)]
        description: Option<String>,

        /// Team key (e.g., ENG, DESIGN)
        #[arg(long)]
        team: Option<String>,

        /// Assignee (use "me" for yourself)
        #[arg(long)]
        assignee: Option<String>,

        /// Priority level (1=Urgent, 2=High, 3=Normal, 4=Low)
        #[arg(long, value_parser = clap::value_parser!(i64).range(1..=4))]
        priority: Option<i64>,

        /// Open the created issue in the browser
        #[arg(long)]
        open: bool,

        /// Dry run - validate inputs without creating the issue
        #[arg(long)]
        dry_run: bool,
    },
    /// Manage image cache and diagnostics (requires inline-images feature)
    #[cfg(feature = "inline-images")]
    Images {
        #[command(subcommand)]
        action: ImageAction,
    },
}

#[cfg(feature = "inline-images")]
#[derive(Subcommand)]
enum ImageAction {
    /// Clear the image cache
    Clear,
    /// Show cache statistics and information
    Stats,
    /// Test image protocol support for current terminal
    Test {
        /// Test URL to use (optional, uses a small test image if not provided)
        #[arg(long)]
        url: Option<String>,
    },
    /// Show detailed diagnostics about image capabilities
    Diagnostics,
}

#[cfg(feature = "inline-images")]
async fn handle_images_command(
    action: ImageAction,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
    use crate::image_protocols::{ImageManager, TerminalCapabilities};

    match action {
        ImageAction::Clear => {
            let spinner = create_spinner("Clearing image cache...", is_interactive);

            match ImageManager::new() {
                Ok(manager) => match manager.clear_cache().await {
                    Ok(_) => {
                        if let Some(s) = spinner {
                            s.finish_and_clear();
                        }
                        let cli = CliOutput::with_color(use_color);
                        cli.success("Image cache cleared successfully!");
                    }
                    Err(e) => {
                        if let Some(s) = spinner {
                            s.finish_and_clear();
                        }
                        let cli = CliOutput::with_color(use_color);
                        cli.error(&format!("Failed to clear cache: {}", e));
                        std::process::exit(1);
                    }
                },
                Err(e) => {
                    if let Some(s) = spinner {
                        s.finish_and_clear();
                    }
                    if use_color {
                        eprintln!("{} Failed to initialize image manager: {}", "✗".red(), e);
                    } else {
                        eprintln!("✗ Failed to initialize image manager: {}", e);
                    }
                    std::process::exit(1);
                }
            }
        }

        ImageAction::Stats => {
            let spinner = create_spinner("Gathering cache statistics...", is_interactive);

            match ImageManager::new() {
                Ok(manager) => {
                    match manager.cache_stats().await {
                        Ok(stats) => {
                            if let Some(s) = spinner {
                                s.finish_and_clear();
                            }
                            if use_color {
                                println!("{} Image Cache Statistics", "📊".blue());
                            } else {
                                println!("📊 Image Cache Statistics");
                            }
                            println!();
                            println!("{}", stats);

                            // Show terminal capabilities
                            let caps = manager.capabilities();
                            println!();
                            if use_color {
                                println!("{} Terminal Capabilities", "🖥️".cyan());
                            } else {
                                println!("🖥️ Terminal Capabilities");
                            }
                            println!("Terminal: {}", caps.terminal_name);
                            println!(
                                "Kitty Protocol: {}",
                                if caps.supports_kitty_images {
                                    "✓"
                                } else {
                                    "✗"
                                }
                            );
                            println!(
                                "iTerm2 Protocol: {}",
                                if caps.supports_iterm2_images {
                                    "✓"
                                } else {
                                    "✗"
                                }
                            );
                            println!(
                                "Sixel Protocol: {}",
                                if caps.supports_sixel { "✓" } else { "✗" }
                            );
                            println!(
                                "Image Support: {}",
                                if caps.supports_inline_images() {
                                    "✓ Enabled"
                                } else {
                                    "✗ Disabled"
                                }
                            );

                            if let Some(protocol) = caps.preferred_protocol() {
                                println!("Preferred Protocol: {}", protocol);
                            }
                        }
                        Err(e) => {
                            if let Some(s) = spinner {
                                s.finish_and_clear();
                            }
                            if use_color {
                                eprintln!("{} Failed to get cache stats: {}", "✗".red(), e);
                            } else {
                                eprintln!("✗ Failed to get cache stats: {}", e);
                            }
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    if let Some(s) = spinner {
                        s.finish_and_clear();
                    }
                    if use_color {
                        eprintln!("{} Failed to initialize image manager: {}", "✗".red(), e);
                    } else {
                        eprintln!("✗ Failed to initialize image manager: {}", e);
                    }
                    std::process::exit(1);
                }
            }
        }

        ImageAction::Test { url } => {
            let test_url = url.unwrap_or_else(|| {
                // Use a small test PNG (1x1 pixel)
                "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string()
            });

            let spinner = create_spinner("Testing image protocol support...", is_interactive);

            match ImageManager::new() {
                Ok(manager) => {
                    if !manager.is_enabled() {
                        if let Some(s) = spinner {
                            s.finish_and_clear();
                        }
                        if use_color {
                            println!(
                                "{} Image support is disabled in this terminal",
                                "ℹ".yellow()
                            );
                        } else {
                            println!("ℹ Image support is disabled in this terminal");
                        }
                        println!();

                        let caps = manager.capabilities();
                        println!("Terminal: {}", caps.terminal_name);
                        println!("Detected support: None");
                        println!();
                        println!(
                            "Try setting LINEAR_CLI_FORCE_PROTOCOL=kitty or LINEAR_CLI_FORCE_PROTOCOL=iterm2 to test protocols manually."
                        );
                        return Ok(());
                    }

                    let result = manager.process_image(&test_url, "Test image").await;

                    if let Some(s) = spinner {
                        s.finish_and_clear();
                    }

                    match result {
                        crate::image_protocols::ImageRenderResult::Rendered(output) => {
                            if use_color {
                                println!("{} Image protocol test successful!", "✓".green());
                            } else {
                                println!("✓ Image protocol test successful!");
                            }
                            println!();
                            println!("Terminal: {}", manager.capabilities().terminal_name);
                            println!(
                                "Protocol: {}",
                                manager
                                    .capabilities()
                                    .preferred_protocol()
                                    .unwrap_or("unknown")
                            );

                            if test_url.starts_with("data:") {
                                println!();
                                println!("Test image rendered below:");
                                println!("{}", output);
                            } else {
                                println!();
                                println!("Downloaded and rendered image from: {}", test_url);
                                println!("{}", output);
                            }
                        }
                        crate::image_protocols::ImageRenderResult::Fallback(link) => {
                            if use_color {
                                println!(
                                    "{} Image protocol not supported, falling back to link",
                                    "ℹ".yellow()
                                );
                            } else {
                                println!("ℹ Image protocol not supported, falling back to link");
                            }
                            println!();
                            println!("Fallback output: {}", link);
                        }
                        crate::image_protocols::ImageRenderResult::Disabled => {
                            if use_color {
                                println!("{} Image processing is disabled", "ℹ".yellow());
                            } else {
                                println!("ℹ Image processing is disabled");
                            }
                        }
                    }
                }
                Err(e) => {
                    if let Some(s) = spinner {
                        s.finish_and_clear();
                    }
                    if use_color {
                        eprintln!("{} Failed to test image protocol: {}", "✗".red(), e);
                    } else {
                        eprintln!("✗ Failed to test image protocol: {}", e);
                    }
                    std::process::exit(1);
                }
            }
        }

        ImageAction::Diagnostics => {
            if use_color {
                println!("{} Linear CLI Image Diagnostics", "🔍".magenta());
            } else {
                println!("🔍 Linear CLI Image Diagnostics");
            }
            println!();

            // Terminal Detection
            let caps = TerminalCapabilities::detect();
            println!("=== Terminal Detection ===");
            println!("Terminal Name: {}", caps.terminal_name);
            println!(
                "TERM_PROGRAM: {}",
                std::env::var("TERM_PROGRAM").unwrap_or("(not set)".to_string())
            );
            println!(
                "TERM: {}",
                std::env::var("TERM").unwrap_or("(not set)".to_string())
            );
            println!(
                "KITTY_WINDOW_ID: {}",
                std::env::var("KITTY_WINDOW_ID").unwrap_or("(not set)".to_string())
            );
            println!(
                "WEZTERM_EXECUTABLE: {}",
                std::env::var("WEZTERM_EXECUTABLE").unwrap_or("(not set)".to_string())
            );
            println!();

            // Protocol Support
            println!("=== Protocol Support ===");
            println!(
                "Kitty Graphics: {} {}",
                if caps.supports_kitty_images {
                    "✓"
                } else {
                    "✗"
                },
                if caps.supports_kitty_images {
                    "(Supported)"
                } else {
                    "(Not supported)"
                }
            );
            println!(
                "iTerm2 Inline: {} {}",
                if caps.supports_iterm2_images {
                    "✓"
                } else {
                    "✗"
                },
                if caps.supports_iterm2_images {
                    "(Supported)"
                } else {
                    "(Not supported)"
                }
            );
            println!(
                "Sixel Graphics: {} {}",
                if caps.supports_sixel { "✓" } else { "✗" },
                if caps.supports_sixel {
                    "(Supported)"
                } else {
                    "(Not supported)"
                }
            );
            println!();

            if caps.supports_inline_images() {
                println!(
                    "Preferred Protocol: {}",
                    caps.preferred_protocol().unwrap_or("none")
                );
            } else {
                println!("No image protocols supported in this terminal");
            }
            println!();

            // Environment Variables
            println!("=== Configuration ===");
            println!(
                "LINEAR_CLI_FORCE_PROTOCOL: {}",
                std::env::var("LINEAR_CLI_FORCE_PROTOCOL").unwrap_or("(not set)".to_string())
            );
            println!(
                "LINEAR_CLI_ALLOWED_IMAGE_DOMAINS: {}",
                std::env::var("LINEAR_CLI_ALLOWED_IMAGE_DOMAINS")
                    .unwrap_or("uploads.linear.app (default)".to_string())
            );
            println!(
                "LINEAR_CLI_MAX_IMAGE_SIZE: {}",
                std::env::var("LINEAR_CLI_MAX_IMAGE_SIZE").unwrap_or("10MB (default)".to_string())
            );
            println!(
                "LINEAR_CLI_VERBOSE: {}",
                if std::env::var("LINEAR_CLI_VERBOSE").is_ok() {
                    "enabled"
                } else {
                    "disabled"
                }
            );
            println!(
                "LINEAR_CLI_QUIET: {}",
                if std::env::var("LINEAR_CLI_QUIET").is_ok() {
                    "enabled"
                } else {
                    "disabled"
                }
            );
            println!();

            // Image Manager Status
            match ImageManager::new() {
                Ok(manager) => {
                    println!("=== Image Manager Status ===");
                    println!(
                        "Manager Enabled: {}",
                        if manager.is_enabled() {
                            "✓ Yes"
                        } else {
                            "✗ No"
                        }
                    );

                    if let Ok(stats) = manager.cache_stats().await {
                        println!("Cache: {}", stats);
                    } else {
                        println!("Cache: Error reading cache information");
                    }

                    // Terminal dimensions (if available)
                    if let Ok(scaler) = crate::image_protocols::scaling::ImageScaler::new() {
                        if let Some(dims) = scaler.get_terminal_dimensions() {
                            println!("Terminal Size: {}x{} characters", dims.width, dims.height);
                            println!(
                                "Estimated Pixel Size: {}x{} pixels",
                                dims.width * dims.char_width,
                                dims.height * dims.char_height
                            );
                        } else {
                            println!("Terminal Size: Could not detect");
                        }
                    }
                }
                Err(e) => {
                    println!("=== Image Manager Status ===");
                    println!("Manager: ✗ Failed to initialize ({})", e);
                }
            }

            println!();
            println!("=== Recommendations ===");
            if !caps.supports_inline_images() {
                println!("• This terminal does not support inline images");
                println!("• Try using iTerm2, Kitty, WezTerm, or Ghostty for image support");
                println!("• You can override detection with: LINEAR_CLI_FORCE_PROTOCOL=kitty");
            } else {
                println!("• Image support is available in this terminal");
                println!("• Use 'linear images test' to verify functionality");
            }

            if std::env::var("RUST_LOG").is_err() {
                println!("• Enable debug logging with RUST_LOG=debug for detailed processing info");
            }
        }
    }

    Ok(())
}

struct CreateCommandArgs {
    title: Option<String>,
    description: Option<String>,
    team: Option<String>,
    assignee: Option<String>,
    priority: Option<i64>,
    open: bool,
    dry_run: bool,
}

async fn handle_create_command(
    args: CreateCommandArgs,
    client: &LinearClient,
    use_color: bool,
    is_interactive: bool,
) -> Result<()> {
    // TODO: Implement interactive prompts and issue creation
    // For now, just handle basic validation

    let cli = CliOutput::with_color(use_color);

    if args.dry_run {
        cli.info("Dry run mode - would create issue with:");
        if let Some(title) = &args.title {
            println!("  Title: {}", title);
        }
        if let Some(description) = &args.description {
            println!("  Description: {}", description);
        }
        if let Some(team) = &args.team {
            println!("  Team: {}", team);
        }
        if let Some(assignee) = &args.assignee {
            println!("  Assignee: {}", assignee);
        }
        if let Some(priority) = args.priority {
            println!("  Priority: {}", priority);
        }
        return Ok(());
    }

    // For now, require title and team
    let title = args.title.ok_or_else(|| LinearError::InvalidInput {
        message: "Title is required".to_string(),
    })?;

    let team = args.team.ok_or_else(|| LinearError::InvalidInput {
        message: "Team is required".to_string(),
    })?;

    // Convert team to team_id (simplified for now - assume team is already team key)
    let team_id = team.clone();

    // Handle assignee_id conversion if needed
    let assignee_id = if let Some(assignee) = args.assignee {
        if assignee == "me" {
            // Get current user ID
            let viewer_data = client.execute_viewer_query().await?;
            Some(viewer_data.viewer.id)
        } else {
            // For now, assume it's already a user ID
            Some(assignee)
        }
    } else {
        None
    };

    let spinner = create_spinner("Creating issue...", is_interactive);

    let input = CreateIssueInput {
        title,
        description: args.description,
        team_id,
        assignee_id,
        priority: args.priority,
    };

    match client.create_issue(input).await {
        Ok(issue) => {
            if let Some(s) = spinner {
                s.finish_and_clear();
            }

            cli.success(&format!("Created issue: {}", issue.identifier));
            println!("Title: {}", issue.title);
            println!("URL: {}", issue.url);

            if args.open {
                if let Err(e) = webbrowser::open(&issue.url) {
                    cli.warning(&format!("Failed to open browser: {}", e));
                }
            }
        }
        Err(e) => {
            if let Some(s) = spinner {
                s.finish_and_clear();
            }
            display_error(&e, use_color);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

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
            let spinner = create_spinner("Logging out...", use_color);
            // We don't need a valid OAuth manager to logout, just need to clear the storage
            match linear_sdk::storage::clear() {
                Ok(_) => {
                    if let Some(s) = spinner {
                        s.finish_and_clear();
                    }
                    let cli = CliOutput::with_color(use_color);
                    cli.success("Successfully logged out!");
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
            runtime
                .block_on(async move { run_async_commands(cli, use_color, is_interactive).await })
        }
    }
}

async fn run_async_commands(cli: Cli, use_color: bool, is_interactive: bool) -> Result<()> {
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

    let spinner = create_spinner("Connecting to Linear...", is_interactive);

    // Determine if this is an OAuth token (from keychain) or API key
    let is_oauth_token = env::var("LINEAR_API_KEY").is_err();

    let client = if is_oauth_token {
        #[cfg(feature = "oauth")]
        {
            // OAuth tokens need "Bearer " prefix
            let bearer_token = format!("Bearer {}", auth_token);
            match LinearClient::builder()
                .auth_token(SecretString::new(bearer_token.into_boxed_str()))
                .verbose(cli.verbose)
                .build()
            {
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
        }
        #[cfg(not(feature = "oauth"))]
        {
            // This should never happen because we check for oauth feature above
            unreachable!()
        }
    } else {
        match LinearClient::builder()
            .auth_token(SecretString::new(auth_token.into_boxed_str()))
            .verbose(cli.verbose)
            .build()
        {
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
        Commands::Issue {
            id,
            json,
            raw,
            #[cfg(feature = "inline-images")]
            no_images,
            #[cfg(feature = "inline-images")]
            force_images,
        } => {
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

                        // Determine if images should be enabled based on correct logic:
                        // - Interactive (TTY) + no --no-images = Enable images
                        // - Non-interactive + --force-images = Enable images
                        // - Non-interactive + no --force-images = Disable images
                        // - Any case + --no-images = Disable images
                        #[cfg(feature = "inline-images")]
                        {
                            let should_enable_images = {
                                if no_images {
                                    // --no-images flag always disables
                                    false
                                } else if force_images {
                                    // --force-images enables even when non-interactive
                                    true
                                } else {
                                    // Default: enable only when interactive (TTY) and rich formatting
                                    use_rich_formatting
                                }
                            };

                            if should_enable_images {
                                log::debug!("Creating image manager for issue processing...");

                                // Create image manager and use async image processing
                                match crate::image_protocols::ImageManager::new() {
                                    Ok(mut image_manager) => {
                                        // Enable the image manager (it auto-detects terminal capabilities)
                                        image_manager.set_enabled(true);

                                        log::debug!(
                                            "Image manager enabled: {}",
                                            image_manager.is_enabled()
                                        );

                                        match formatter
                                            .format_detailed_issue_with_image_manager_async(
                                                &issue,
                                                use_rich_formatting,
                                                &image_manager,
                                            )
                                            .await
                                        {
                                            Ok(output) => output,
                                            Err(e) => {
                                                log::debug!(
                                                    "Image processing failed, falling back to regular formatting: {}",
                                                    e
                                                );
                                                // Fallback to regular formatting
                                                match formatter.format_detailed_issue_rich(
                                                    &issue,
                                                    use_rich_formatting,
                                                ) {
                                                    Ok(output) => output,
                                                    Err(e) => {
                                                        display_error(&e, use_color);
                                                        std::process::exit(1);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::debug!("Failed to create image manager: {}", e);
                                        // Fallback to regular formatting
                                        match formatter
                                            .format_detailed_issue_rich(&issue, use_rich_formatting)
                                        {
                                            Ok(output) => output,
                                            Err(e) => {
                                                display_error(&e, use_color);
                                                std::process::exit(1);
                                            }
                                        }
                                    }
                                }
                            } else {
                                // Images disabled - use regular rich formatting
                                match formatter
                                    .format_detailed_issue_rich(&issue, use_rich_formatting)
                                {
                                    Ok(output) => output,
                                    Err(e) => {
                                        display_error(&e, use_color);
                                        std::process::exit(1);
                                    }
                                }
                            }
                        }

                        #[cfg(not(feature = "inline-images"))]
                        {
                            // Images not compiled in - use regular rich formatting
                            match formatter.format_detailed_issue_rich(&issue, use_rich_formatting)
                            {
                                Ok(output) => output,
                                Err(e) => {
                                    display_error(&e, use_color);
                                    std::process::exit(1);
                                }
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
        Commands::Create {
            title,
            description,
            team,
            assignee,
            priority,
            open,
            dry_run,
        } => {
            handle_create_command(
                CreateCommandArgs {
                    title,
                    description,
                    team,
                    assignee,
                    priority,
                    open,
                    dry_run,
                },
                &client,
                use_color,
                is_interactive,
            )
            .await?
        }
        #[cfg(feature = "oauth")]
        Commands::Login { .. } | Commands::Logout => {
            // These commands are handled earlier, this should never be reached
            unreachable!()
        }
        #[cfg(feature = "inline-images")]
        Commands::Images { action } => {
            handle_images_command(action, use_color, is_interactive).await?
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
            Commands::Create { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issues command"),
            #[cfg(feature = "inline-images")]
            Commands::Images { .. } => panic!("Expected Issues command"),
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
            Commands::Create { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issues command"),
            #[cfg(feature = "inline-images")]
            Commands::Images { .. } => panic!("Expected Issues command"),
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
            Commands::Create { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issues command"),
            #[cfg(feature = "inline-images")]
            Commands::Images { .. } => panic!("Expected Issues command"),
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
            Commands::Create { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issues command"),
            #[cfg(feature = "inline-images")]
            Commands::Images { .. } => panic!("Expected Issues command"),
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
            Commands::Create { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issues command"),
            #[cfg(feature = "inline-images")]
            Commands::Images { .. } => panic!("Expected Issues command"),
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
            Commands::Create { .. } => panic!("Expected Issue command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issue command"),
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
            Commands::Create { .. } => panic!("Expected Issue command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issue command"),
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
            Commands::Create { .. } => panic!("Expected Issue command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issue command"),
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
            Commands::Create { .. } => panic!("Expected Issue command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issue command"),
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
            Commands::Create { .. } => panic!("Expected Issue command"),
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
            Commands::Create { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issues command"),
            #[cfg(feature = "inline-images")]
            Commands::Images { .. } => panic!("Expected Issues command"),
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
            Commands::Create { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issues command"),
            #[cfg(feature = "inline-images")]
            Commands::Images { .. } => panic!("Expected Issues command"),
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
            Commands::Create { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issues command"),
            #[cfg(feature = "inline-images")]
            Commands::Images { .. } => panic!("Expected Issues command"),
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
            Commands::Create { .. } => panic!("Expected Issues command"),
            #[cfg(feature = "oauth")]
            Commands::Login { .. } | Commands::Logout => panic!("Expected Issues command"),
            #[cfg(feature = "inline-images")]
            Commands::Images { .. } => panic!("Expected Issues command"),
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

    #[test]
    fn test_parse_create_command() {
        use clap::Parser;

        // Test basic create command with title and team
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

        // Test create command with all options
        let cli = Cli::try_parse_from([
            "linear",
            "create",
            "--title",
            "Full Test Issue",
            "--description",
            "This is a test description",
            "--team",
            "ENG",
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
            } => {
                assert_eq!(title, Some("Full Test Issue".to_string()));
                assert_eq!(description, Some("This is a test description".to_string()));
                assert_eq!(team, Some("ENG".to_string()));
                assert_eq!(assignee, Some("me".to_string()));
                assert_eq!(priority, Some(2));
                assert!(open);
                assert!(dry_run);
            }
            _ => panic!("Expected Create command"),
        }
    }

    #[test]
    fn test_create_command_priority_validation() {
        use clap::Parser;

        // Test valid priority values
        for priority in 1..=4 {
            let cli = Cli::try_parse_from([
                "linear",
                "create",
                "--title",
                "Test",
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

        // Test invalid priority values (should fail)
        let result = Cli::try_parse_from([
            "linear",
            "create",
            "--title",
            "Test",
            "--team",
            "ENG",
            "--priority",
            "0",
        ]);
        assert!(result.is_err());

        let result = Cli::try_parse_from([
            "linear",
            "create",
            "--title",
            "Test",
            "--team",
            "ENG",
            "--priority",
            "5",
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_command_help() {
        use clap::CommandFactory;

        let cmd = Cli::command();
        let create_cmd = cmd
            .find_subcommand("create")
            .expect("create command should exist");
        assert_eq!(create_cmd.get_name(), "create");

        // Check that all expected arguments are present
        let args: Vec<_> = create_cmd.get_arguments().map(|arg| arg.get_id()).collect();
        assert!(args.iter().any(|&id| id == "title"));
        assert!(args.iter().any(|&id| id == "description"));
        assert!(args.iter().any(|&id| id == "team"));
        assert!(args.iter().any(|&id| id == "assignee"));
        assert!(args.iter().any(|&id| id == "priority"));
        assert!(args.iter().any(|&id| id == "open"));
        assert!(args.iter().any(|&id| id == "dry_run"));
    }
}
