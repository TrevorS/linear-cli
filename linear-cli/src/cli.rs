// ABOUTME: CLI argument definitions for Linear CLI application
// ABOUTME: Defines the command-line interface structure using clap derive macros

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "linear")]
#[command(about = "A CLI for Linear", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Force colored output even when piped
    #[arg(long, global = true, conflicts_with = "no_color")]
    pub force_color: bool,

    /// Enable verbose output for debugging
    #[arg(long, short, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
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
    /// Create a new issue
    Create {
        /// Issue title
        #[arg(long)]
        title: Option<String>,

        /// Issue description
        #[arg(long)]
        description: Option<String>,

        /// Team key (e.g., ENG) or UUID
        #[arg(long)]
        team: Option<String>,

        /// Assignee (use "me" for yourself)
        #[arg(long)]
        assignee: Option<String>,

        /// Priority (1=Urgent, 2=High, 3=Normal, 4=Low)
        #[arg(long, value_parser = clap::value_parser!(i64).range(1..=4))]
        priority: Option<i64>,

        /// Project name to assign the issue to
        #[arg(long)]
        project: Option<String>,

        /// Project ID to assign the issue to (alternative to --project)
        #[arg(long, conflicts_with = "project")]
        project_id: Option<String>,

        /// Create issue from markdown file with frontmatter
        #[arg(long, short = 'f', value_name = "FILE")]
        from_file: Option<String>,

        /// Open the created issue in browser
        #[arg(long)]
        open: bool,

        /// Show what would be created without actually creating it
        #[arg(long)]
        dry_run: bool,
    },
    /// Update an existing issue
    Update {
        /// Issue identifier (e.g., ENG-123)
        id: String,

        /// New title for the issue
        #[arg(long)]
        title: Option<String>,

        /// New description for the issue
        #[arg(long)]
        description: Option<String>,

        /// New assignee (use "me" for yourself, "unassigned" to unassign)
        #[arg(long)]
        assignee: Option<String>,

        /// New status/state for the issue
        #[arg(long)]
        status: Option<String>,

        /// New priority (1=Urgent, 2=High, 3=Normal, 4=Low)
        #[arg(long, value_parser = clap::value_parser!(i64).range(1..=4))]
        priority: Option<i64>,

        /// Project name to assign the issue to (use "none" to remove)
        #[arg(long)]
        project: Option<String>,

        /// Project ID to assign the issue to (alternative to --project)
        #[arg(long, conflicts_with = "project")]
        project_id: Option<String>,

        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Close an issue (convenience command)
    Close {
        /// Issue identifier (e.g., ENG-123)
        id: String,

        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Reopen an issue (convenience command)
    Reopen {
        /// Issue identifier (e.g., ENG-123)
        id: String,

        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Add a comment to an issue
    Comment {
        /// Issue identifier (e.g., ENG-123)
        id: String,

        /// Comment text (if not provided, will read from stdin)
        message: Option<String>,
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
    /// List projects
    Projects {
        /// Maximum number of projects to fetch
        #[arg(short, long, default_value = "20", value_parser = clap::value_parser!(i32).range(1..))]
        limit: i32,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Pretty print JSON output
        #[arg(long, requires = "json")]
        pretty: bool,
    },
    /// List teams
    Teams {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Pretty print JSON output
        #[arg(long, requires = "json")]
        pretty: bool,
    },
    /// Show comments for an issue
    Comments {
        /// Issue identifier (e.g., ENG-123)
        id: String,

        /// Maximum number of comments to fetch
        #[arg(short, long, default_value = "20", value_parser = clap::value_parser!(i32).range(1..))]
        limit: i32,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Pretty print JSON output
        #[arg(long, requires = "json")]
        pretty: bool,
    },
    /// Show your assigned and created issues
    MyWork {
        /// Maximum number of issues to fetch per category
        #[arg(short, long, default_value = "20", value_parser = clap::value_parser!(i32).range(1..))]
        limit: i32,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Pretty print JSON output
        #[arg(long, requires = "json")]
        pretty: bool,
    },
    /// Search across issues, projects, and comments
    Search {
        /// Search query string
        query: String,

        /// Search only in issues (default: search all types)
        #[arg(long)]
        issues_only: bool,

        /// Search only in documents
        #[arg(long)]
        docs_only: bool,

        /// Search only in projects
        #[arg(long)]
        projects_only: bool,

        /// Maximum number of results per type
        #[arg(short, long, default_value = "10", value_parser = clap::value_parser!(i32).range(1..=100))]
        limit: i32,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Pretty print JSON output
        #[arg(long, requires = "json")]
        pretty: bool,

        /// Include archived results
        #[arg(long)]
        include_archived: bool,
    },
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: crate::completions::Shell,
    },
}
