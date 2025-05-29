# Linear CLI Implementation Plan

## Blueprint Overview

### Project Goals
Build a command-line interface for Linear that:
- Provides fast issue browsing without leaving the terminal
- Supports both API key and OAuth authentication
- Offers scriptable output (JSON) and beautiful terminal tables
- Starts with read operations, then adds write capabilities
- Works reliably with Linear's GraphQL API

### Architecture Strategy
- **Validate First**: Confirm all assumptions about Linear's API before building
- **Test Continuously**: Integration tests from day one
- **Iterative Development**: Each step produces working software
- **Fail Fast**: Discover integration issues immediately
- **Progressive Enhancement**: Start simple, add complexity gradually

## Phase Breakdown

### Phase 0: Validation & Exploration (De-risk)
1. API exploration spike
2. Schema validation
3. Authentication verification

### Phase 1: Foundation (Prove the concept)
1. Project setup with real schema
2. Minimal working CLI
3. CI/CD pipeline
4. Basic error handling

### Phase 2: Core Features (Deliver value)
1. Table formatting
2. JSON output
3. Filtering system
4. Single issue view

### Phase 3: Enhanced Features (Improve UX)
1. OAuth authentication
2. Additional queries (projects, teams, comments)
3. Search functionality

### Phase 4: Write Operations (Complete the tool)
1. Create issues
2. Update operations
3. Bulk commands

## Detailed Implementation Steps

After reviewing the complexity and considering safe incremental progress, here are the refined implementation prompts:

---

## Implementation Prompts

### Prompt 1: API Validation Spike

```text
Create a throwaway spike to validate Linear's GraphQL API assumptions before building any production code.

Create a new directory called `linear-api-spike` with a simple Node.js script that:

1. Uses a real Linear API key (from environment variable LINEAR_API_KEY)
2. Makes a raw HTTP POST request to https://api.linear.app/graphql
3. Tests these queries:
   - Introspection query to download the schema
   - Simple viewer query: `{ viewer { id name email } }`
   - Issues query: `{ issues(first: 5) { nodes { id identifier title } } }`

4. Document in a FINDINGS.md file:
   - Exact authentication header format
   - Rate limit headers returned
   - Any unexpected response structures
   - Schema introspection availability
   - Error response formats

5. Save the downloaded schema.json for the next step

This is exploratory code - focus on learning, not code quality. The goal is to validate our assumptions before building the real project.

Expected output: A FINDINGS.md file and schema.json that we'll use in the actual project.
```

### Prompt 2: Workspace Setup with Real Schema

```text
Create the production Linear CLI workspace with proper structure and real schema integration.

1. Create a Rust workspace with this structure:
   ```
   linear-cli/
   ├── Cargo.toml (workspace)
   ├── xtask/
   │   ├── Cargo.toml
   │   └── src/
   │       └── main.rs
   ├── linear-sdk/
   │   ├── Cargo.toml
   │   ├── build.rs
   │   ├── graphql/
   │   │   ├── schema.json (copy from spike)
   │   │   └── queries/
   │   │       └── viewer.graphql
   │   └── src/
   │       ├── lib.rs
   │       └── generated/
   └── linear-cli/
       ├── Cargo.toml
       └── src/
           └── main.rs
   ```

2. Set up xtask for schema management:
   - Command to download fresh schema from Linear
   - Use the introspection query from the spike
   - Save to linear-sdk/graphql/schema.json

3. Configure linear-sdk:
   - Dependencies: tokio, reqwest, serde, graphql_client, anyhow
   - build.rs that generates types from schema.json
   - Simple viewer.graphql query from the spike

4. Configure linear-cli:
   - Dependencies: tokio, clap, anyhow, linear-sdk (path dependency)
   - Feature flags: `default = []`, `integration-tests = []`

5. Make it compile:
   - Basic main.rs that prints "Linear CLI"
   - lib.rs that exposes generated GraphQL types
   - Verify graphql_client generates valid Rust code from the schema

6. Add .gitignore for generated code but commit schema.json

Test: `cargo build` should succeed and `cargo run -p linear-cli` should print "Linear CLI"
```

### Prompt 3: Minimal End-to-End Query

```text
Implement the smallest possible working Linear CLI that actually queries the API.

1. In linear-sdk, create a minimal client:
   ```rust
   pub struct LinearClient {
       client: reqwest::Client,
       api_key: String,
   }
   ```
   - Constructor that takes API key
   - Single method: `execute_viewer_query()` using the generated types
   - Proper error handling with anyhow

2. Add a simple integration test in linear-sdk:
   ```rust
   #[test]
   #[ignore] // Run with: cargo test -- --ignored
   fn test_real_api() {
       let api_key = std::env::var("LINEAR_API_KEY").unwrap();
       let client = LinearClient::new(api_key);
       let viewer = client.execute_viewer_query().unwrap();
       println!("Viewer: {:?}", viewer);
   }
   ```

3. In linear-cli, create the simplest possible CLI:
   - No command structure yet, just runs the viewer query
   - Reads API key from LINEAR_API_KEY env var
   - Prints viewer name and email
   - Returns error if no API key

4. Add error handling that shows:
   ```
   Error: No LINEAR_API_KEY environment variable found

   Please set your Linear API key:
   export LINEAR_API_KEY=lin_api_xxxxx

   Get your API key from: https://linear.app/settings/api
   ```

5. Add logging setup:
   - Use `env_logger` for now
   - Log HTTP requests/responses when RUST_LOG=debug

Test manually:
- `LINEAR_API_KEY=xxx cargo run -p linear-cli` should show your Linear user info
- Without API key should show helpful error
```

### Prompt 4: CI Pipeline and Testing Infrastructure

```text
Set up continuous integration and testing infrastructure before adding more features.

1. Create GitHub Actions workflow (.github/workflows/ci.yml):
   - Runs on: push, pull_request
   - Matrix: macOS-latest (primary), ubuntu-latest
   - Steps:
     - Checkout
     - Cache: cargo registry, cargo index, target/
     - Run: cargo fmt -- --check
     - Run: cargo clippy -- -D warnings
     - Run: cargo test
     - Run: cargo test --features integration-tests (using secrets.LINEAR_API_KEY)

2. Add test helpers to linear-sdk:
   ```rust
   #[cfg(test)]
   pub mod test_helpers {
       use mockito;

       pub fn mock_linear_server() -> mockito::ServerGuard {
           // Return configured mock server
       }

       pub fn mock_viewer_response() -> serde_json::Value {
           // Return realistic response
       }
   }
   ```

3. Add mockito-based unit tests:
   - Test successful API calls
   - Test authentication errors (401)
   - Test GraphQL errors
   - Test network timeouts

4. Create fixtures/ directory with sample responses:
   - viewer_response.json
   - issues_response.json
   - error_response.json

5. Add Makefile for common commands:
   ```makefile
   test:
       cargo test

   test-integration:
       cargo test --features integration-tests -- --ignored

   fmt:
       cargo fmt --all

   lint:
       cargo clippy -- -D warnings
   ```

6. Set up dependabot for dependency updates

Verify: Push to GitHub and ensure CI passes. The pipeline should run both unit and integration tests.
```

### Prompt 5: Basic Issues Query

```text
Add the first real feature: listing issues with a proper CLI structure.

1. Create issues query in linear-sdk:
   ```graphql
   query ListIssues($first: Int!) {
     issues(first: $first) {
       nodes {
         id
         identifier
         title
         state { name }
         assignee { name }
       }
     }
   }
   ```

2. Extend LinearClient:
   - Add `list_issues(&self, limit: i32)` method
   - Use graphql_client generated types
   - Return simplified Issue struct (not the generated one)

3. Add CLI structure with clap:
   ```rust
   #[derive(Parser)]
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
   ```

4. Implement basic output:
   - For now, just print one line per issue
   - Format: "ISSUE-ID: Title (Status) - Assignee"
   - Handle unassigned issues gracefully

5. Add tests:
   - Unit test with mocked response
   - Integration test that fetches real issues
   - Test empty results case

6. Update error handling to be command-specific

This creates our first useful feature while establishing patterns for future commands.
```

### Prompt 6: Table Formatting

```text
Add beautiful table output for issues using the tabled crate.

1. Add dependencies to linear-cli:
   - tabled = "0.15"
   - owo-colors = "4"

2. Create output module structure:
   ```rust
   // src/output/mod.rs
   pub trait OutputFormat {
       fn format_issues(&self, issues: &[Issue]) -> Result<String>;
   }

   // src/output/table.rs
   pub struct TableFormatter {
       use_color: bool,
   }
   ```

3. Implement table formatting:
   - Columns: Issue, Title, Status, Assignee
   - Truncate title to 40 chars with "..."
   - Color status based on state (Todo=gray, In Progress=yellow, Done=green)
   - Show "Unassigned" in dimmed text for no assignee

4. Add --no-color flag to CLI globally:
   ```rust
   #[derive(Parser)]
   struct Cli {
       /// Disable colored output
       #[arg(long, global = true)]
       no_color: bool,

       #[command(subcommand)]
       command: Commands,
   }
   ```

5. Respect NO_COLOR and TERM environment variables

6. Add snapshot tests using insta:
   - Snapshot of colored output
   - Snapshot of non-colored output
   - Snapshot with very long titles
   - Snapshot with no issues

Output should look like:
```
 Issue     Title                          Status        Assignee
 ─────────────────────────────────────────────────────────────────
 ENG-123   Fix login race condition       In Progress   John Doe
 ENG-124   Implement OAuth flow           Todo          Unassigned
```
```

### Prompt 7: JSON Output

```text
Add JSON output format for scriptability.

1. Add serde derives to the simplified Issue struct:
   ```rust
   #[derive(Debug, Serialize)]
   #[serde(rename_all = "camelCase")]
   pub struct Issue {
       pub id: String,
       pub identifier: String,
       pub title: String,
       pub status: String,
       pub assignee: Option<String>,
   }
   ```

2. Add JSON formatter:
   ```rust
   // src/output/json.rs
   pub struct JsonFormatter {
       pretty: bool,
   }
   ```

3. Add format flags to issues command:
   ```rust
   Issues {
       #[arg(short, long, default_value = "20")]
       limit: i32,

       /// Output as JSON
       #[arg(long)]
       json: bool,

       /// Pretty print JSON output
       #[arg(long, requires = "json")]
       pretty: bool,
   }
   ```

4. Implement output routing:
   - Default: table format
   - --json: compact JSON array
   - --json --pretty: formatted JSON

5. Ensure JSON fields match Linear's API naming

6. Add tests:
   - Valid JSON output
   - Can pipe to jq
   - Pretty printing works

Example usage:
```bash
# Find high priority issues
linear issues --json | jq '.[] | select(.priority == "High")'

# Count issues by status
linear issues --json | jq 'group_by(.status) | map({status: .[0].status, count: length})'
```
```

### Prompt 8: Query Filters

```text
Add filtering capabilities to the issues command.

1. Extend the GraphQL query to accept filters:
   ```graphql
   query ListIssues($first: Int!, $filter: IssueFilter) {
     issues(first: $first, filter: $filter) {
       nodes {
         id
         identifier
         title
         state { name }
         assignee { id name }
         team { key }
       }
     }
   }
   ```

2. Add filter arguments to CLI:
   ```rust
   Issues {
       #[arg(short, long, default_value = "20")]
       limit: i32,

       /// Filter by assignee (use "me" for yourself)
       #[arg(long)]
       assignee: Option<String>,

       /// Filter by status (case insensitive)
       #[arg(long)]
       status: Option<String>,

       /// Filter by team
       #[arg(long)]
       team: Option<String>,

       // ... existing format flags
   }
   ```

3. Implement special assignee values:
   - "me": Fetch viewer.id and filter by it
   - "unassigned": Filter for null assignee

4. Add status normalization:
   - "todo" → "Todo"
   - "in progress" → "In Progress"
   - "done" → "Done"
   - Show error for unknown statuses

5. Build GraphQL filter object:
   ```rust
   fn build_filter(assignee: Option<String>, status: Option<String>, team: Option<String>) -> Option<IssueFilter> {
       // Construct filter based on provided options
   }
   ```

6. Add tests:
   - Each filter individually
   - Combined filters (AND logic)
   - Special values ("me", case variations)
   - Error cases

7. Cache "me" lookup for performance

Example usage:
```bash
linear issues --assignee me --status "in progress"
linear issues --team ENG --status todo
```
```

### Prompt 9: Single Issue View

```text
Implement detailed view for a single issue.

1. Create detailed issue query:
   ```graphql
   query GetIssue($id: String!) {
     issue(id: $id) {
       id
       identifier
       title
       description
       state { name type }
       assignee { name email }
       team { key name }
       project { name }
       labels { nodes { name color } }
       priority
       priorityLabel
       createdAt
       updatedAt
       url
     }
   }
   ```

2. Add issue command to CLI:
   ```rust
   #[derive(Subcommand)]
   enum Commands {
       Issues { /* ... */ },

       /// Show details for a single issue
       Issue {
           /// Issue identifier (e.g., ENG-123)
           id: String,

           /// Output as JSON
           #[arg(long)]
           json: bool,
       },
   }
   ```

3. Create detailed view formatter:
   ```
   ─────────────────────────────────────────
   ENG-123: Fix login race condition
   ─────────────────────────────────────────
   Status:    In Progress
   Assignee:  John Doe (john@example.com)
   Team:      Engineering (ENG)
   Project:   Web App
   Priority:  High

   Description:
   Users are experiencing race conditions when logging in
   simultaneously from multiple devices.

   Labels: bug, authentication

   Created: 2024-01-15 10:30 AM
   Updated: 2024-01-16 2:45 PM

   View in Linear: https://linear.app/...
   ```

4. Handle issue not found:
   - Clear error message
   - Suggest checking the ID format

5. Support both formats:
   - Issue key (ENG-123)
   - Issue ID (UUID)

6. Add tests:
   - Found issue display
   - Not found error
   - JSON output format
   - Various field combinations
```

### Prompt 10: Error Handling and UX Polish

```text
Establish consistent error handling and improve user experience.

1. Create custom error types:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum LinearError {
       #[error("Authentication failed. Check your LINEAR_API_KEY")]
       Auth,

       #[error("Issue {0} not found")]
       IssueNotFound(String),

       #[error("Network error: {0}")]
       Network(String),

       #[error("GraphQL error: {0}")]
       GraphQL(String),
   }
   ```

2. Add user-friendly error display:
   ```rust
   fn print_error(err: &LinearError) {
       eprintln!("{}: {}", "Error".red().bold(), err);

       match err {
           LinearError::Auth => {
               eprintln!("\nGet your API key from: {}",
                   "https://linear.app/settings/api".cyan());
           }
           // ... helpful context for each error type
       }
   }
   ```

3. Add progress indicators:
   - Use indicatif for spinners
   - "Fetching issues..." while loading
   - "Connecting to Linear..." for initial request

4. Add --verbose flag globally:
   - Show HTTP requests/responses
   - Display timing information
   - Include GraphQL query details

5. Implement retry logic in SDK:
   - Retry network errors (not auth)
   - Exponential backoff
   - Max 3 retries
   - Show retry attempts in verbose mode

6. Add connection check command:
   ```rust
   /// Check connection to Linear
   Status {
       /// Show detailed connection info
       #[arg(long)]
       verbose: bool,
   }
   ```

7. Create integration tests for error scenarios

The goal is helpful errors that guide users to solutions.
```

### Prompt 11: OAuth Authentication

```text
Add OAuth authentication as an optional feature.

1. Add OAuth dependencies behind feature flag:
   ```toml
   [features]
   default = []
   oauth = ["oauth2", "keyring", "open", "tiny_http"]
   ```

2. Create OAuth module (conditional compilation):
   ```rust
   #[cfg(feature = "oauth")]
   mod oauth {
       pub async fn login() -> Result<String> {
           // Implement OAuth2 flow
       }
   }
   ```

3. Add login command (only with oauth feature):
   ```rust
   #[cfg(feature = "oauth")]
   Login {
       /// Force new login even if token exists
       #[arg(long)]
       force: bool,
   }
   ```

4. Implement OAuth2 flow:
   - Start local server on port 8089
   - Open browser to Linear's OAuth URL
   - Handle callback with authorization code
   - Exchange for access token
   - Store in macOS Keychain

5. Update config to check authentication sources:
   1. Command line --api-key
   2. LINEAR_API_KEY env var
   3. OAuth token from keychain (if feature enabled)

6. Add logout command to clear keychain

7. Security considerations:
   - Use PKCE for OAuth
   - Set restrictive keychain access
   - Clear sensitive data after use

8. Add tests:
   - Mock OAuth flow
   - Keychain storage (with mock)
   - Feature flag compilation

Note: Real OAuth testing requires manual browser interaction.
```

### Prompt 12: Additional Query Commands

```text
Expand the CLI with more read operations for Phase 2.

1. Add new GraphQL queries:
   ```graphql
   query ListProjects {
     projects {
       nodes {
         id
         name
         description
         state
         issueCount
       }
     }
   }

   query ListTeams {
     teams {
       nodes {
         id
         key
         name
         memberCount
       }
     }
   }

   query GetComments($issueId: String!) {
     issue(id: $issueId) {
       comments {
         nodes {
           id
           body
           user { name }
           createdAt
         }
       }
     }
   }
   ```

2. Add commands to CLI:
   ```rust
   /// List all projects
   Projects {
       #[arg(long)]
       json: bool,
   },

   /// List all teams
   Teams {
       #[arg(long)]
       json: bool,
   },

   /// Show comments on an issue
   Comments {
       /// Issue identifier
       issue_id: String,

       #[arg(long)]
       json: bool,
   },
   ```

3. Create formatters for each type:
   - Projects: Table with name, state, issue count
   - Teams: Table with key, name, member count
   - Comments: Threaded view with timestamp and author

4. Add caching for teams/projects:
   - Cache for session duration
   - Use for assignee validation later

5. Implement "my-work" command:
   - Combines multiple queries
   - Shows assigned issues, created issues, commented issues
   - Unified view of your current work

6. Add tests for each new command

These commands reuse all the infrastructure we've built.
```

### Prompt 13: Search Functionality

```text
Implement full-text search across Linear.

1. Add search query:
   ```graphql
   query Search($query: String!) {
     searchIssues(query: $query) {
       nodes {
         id
         identifier
         title
         state { name }
         assignee { name }
       }
     }
   }
   ```

2. Add search command:
   ```rust
   /// Search issues, projects, and comments
   Search {
       /// Search query
       query: String,

       /// Limit results per type
       #[arg(long, default_value = "10")]
       limit: i32,

       #[arg(long)]
       json: bool,
   }
   ```

3. Implement multi-type search:
   - Search issues (primary)
   - Search projects
   - Search within comments
   - Group results by type

4. Create search results formatter:
   ```
   Issues (5 results):
   ─────────────────────
   ENG-123  Fix login race condition     In Progress
   ENG-125  Login timeout issues          Todo

   Projects (1 result):
   ──────────────────
   Login System Refactor

   Comments (2 results):
   ──────────────────
   In ENG-120: "The login fix should address..."
   In ENG-118: "Login is working better now..."
   ```

5. Add search operators:
   - Exact phrase: "exact match"
   - Exclude: -term
   - Field search: assignee:john

6. Performance optimization:
   - Parallel queries for different types
   - Limit initial results
   - Provide "show more" option

7. Add tests for search functionality
```

### Prompt 14: Create Issue Command

```text
Begin Phase 3 with the first write operation.

1. Add create mutation:
   ```graphql
   mutation CreateIssue($input: IssueCreateInput!) {
     issueCreate(input: $input) {
       success
       issue {
         id
         identifier
         url
       }
     }
   }
   ```

2. Add create command:
   ```rust
   /// Create a new issue
   Create {
       /// Issue title
       #[arg(short, long)]
       title: Option<String>,

       /// Team key (e.g., ENG)
       #[arg(short = 'T', long)]
       team: Option<String>,

       /// Description
       #[arg(short, long)]
       description: Option<String>,

       /// Assignee email or "me"
       #[arg(short, long)]
       assignee: Option<String>,

       /// Priority (urgent, high, medium, low)
       #[arg(short, long)]
       priority: Option<String>,

       /// Open in browser after creation
       #[arg(long)]
       open: bool,
   }
   ```

3. Add interactive mode with dialoguer:
   - If no args provided, enter interactive mode
   - Fetch and show team list for selection
   - Fetch team members for assignee selection
   - Multi-line description editor

4. Implement validation:
   - Team exists (use cached data)
   - Assignee is in selected team
   - Required fields are present

5. Show success result:
   ```
   ✓ Created issue ENG-126

   Title: Implement user settings
   URL: https://linear.app/company/issue/ENG-126

   Opening in browser...
   ```

6. Add dry-run mode:
   - --dry-run flag
   - Shows what would be created
   - Validates without submitting

7. Tests:
   - Successful creation
   - Validation failures
   - Interactive mode flow
```

### Prompt 15: Update Operations

```text
Complete write operations with update functionality.

1. Add update mutation:
   ```graphql
   mutation UpdateIssue($id: String!, $input: IssueUpdateInput!) {
     issueUpdate(id: $id, input: $input) {
       success
       issue {
         id
         identifier
         state { name }
       }
     }
   }
   ```

2. Add update command:
   ```rust
   /// Update an issue
   Update {
       /// Issue identifier
       id: String,

       /// New status
       #[arg(long)]
       status: Option<String>,

       /// New assignee
       #[arg(long)]
       assignee: Option<String>,

       /// New priority
       #[arg(long)]
       priority: Option<String>,

       /// New title
       #[arg(long)]
       title: Option<String>,
   }
   ```

3. Add convenience commands:
   ```rust
   /// Close an issue
   Close {
       /// Issue identifier
       id: String,
   },

   /// Reopen an issue
   Reopen {
       /// Issue identifier
       id: String,
   },
   ```

4. Add comment command:
   ```rust
   /// Add a comment to an issue
   Comment {
       /// Issue identifier
       id: String,

       /// Comment text (or read from stdin)
       message: Option<String>,
   }
   ```

5. Support stdin for comments:
   ```bash
   echo "Fixed in PR #123" | linear comment ENG-126
   ```

6. Show before/after for updates:
   ```
   Updating ENG-126:
   Status: In Progress → Done
   Assignee: John Doe → Jane Smith

   Confirm? [y/N]
   ```

7. Add tests for all update scenarios
```

### Prompt 16: Bulk Operations

```text
Add bulk operations for efficiency.

1. Implement bulk update command:
   ```rust
   /// Update multiple issues at once
   Bulk {
       #[command(subcommand)]
       action: BulkAction,
   }

   #[derive(Subcommand)]
   enum BulkAction {
       /// Update issues matching filters
       Update {
           // Same filters as 'issues' command
           #[arg(long)]
           assignee: Option<String>,

           #[arg(long)]
           status: Option<String>,

           #[arg(long)]
           team: Option<String>,

           // Updates to apply
           #[arg(long)]
           set_status: Option<String>,

           #[arg(long)]
           set_assignee: Option<String>,

           /// Skip confirmation
           #[arg(long)]
           force: bool,
       },
   }
   ```

2. Implement preview mode:
   ```
   Found 5 issues matching filters:

   ENG-123  Fix login race condition
   ENG-124  Implement OAuth flow
   ENG-125  Add user preferences
   ENG-126  Refactor auth module
   ENG-127  Update documentation

   Will update:
   - Status: Todo → In Progress

   Continue? [y/N]
   ```

3. Add progress bar for bulk operations:
   - Show current issue being updated
   - Display success/failure count
   - Allow cancellation with Ctrl+C

4. Implement transaction safety:
   - Collect all changes first
   - Apply in batch
   - Rollback on failure (if possible)

5. Add bulk close/reopen:
   ```bash
   linear bulk update --status done --set-status cancelled --force
   linear bulk update --assignee me --set-status "in progress"
   ```

6. Safety features:
   - Max 50 issues without --force
   - Dry run by default for >10 issues
   - Clear summary of changes

7. Tests for bulk operations
```

### Prompt 17: Configuration and Completions

```text
Add configuration file support and shell completions.

1. Create config module:
   ```rust
   #[derive(Deserialize, Default)]
   pub struct Config {
       pub default_team: Option<String>,
       pub default_assignee: Option<String>,
       pub preferred_format: Option<OutputFormat>,
       pub aliases: HashMap<String, Vec<String>>,
   }
   ```

2. Load config from standard locations:
   - `~/.config/linear-cli/config.toml`
   - `$XDG_CONFIG_HOME/linear-cli/config.toml`
   - `.linear-cli.toml` in current directory

3. Example config file:
   ```toml
   default_team = "ENG"
   default_assignee = "me"
   preferred_format = "table"

   [aliases]
   my = ["issues", "--assignee", "me"]
   todo = ["issues", "--status", "todo", "--assignee", "me"]
   standup = ["issues", "--team", "ENG", "--updated-after", "yesterday"]
   ```

4. Generate shell completions:
   ```rust
   /// Generate shell completions
   Completions {
       /// Shell to generate for
       #[arg(value_enum)]
       shell: clap_complete::Shell,
   }
   ```

5. Add installation instructions:
   ```bash
   # Bash
   linear completions bash > ~/.local/share/bash-completion/completions/linear

   # Zsh
   linear completions zsh > ~/.zfunc/_linear

   # Fish
   linear completions fish > ~/.config/fish/completions/linear.fish
   ```

6. Support config in commands:
   - Use default_team if --team not specified
   - Apply aliases before parsing args

7. Add config validation and helpful errors
```

### Prompt 18: Performance and Distribution

```text
Optimize performance and prepare for distribution.

1. Performance optimizations:
   - Lazy load large dependencies
   - Use `once_cell` for caching
   - Profile with cargo-flamegraph
   - Minimize allocations in hot paths

2. Binary size optimization:
   ```toml
   [profile.release]
   opt-level = "z"
   lto = true
   codegen-units = 1
   strip = true
   ```

3. Add build-time feature detection:
   - Detect if schema.json exists
   - Provide helpful error if missing
   - Support offline builds

4. Create release workflow:
   ```yaml
   name: Release
   on:
     push:
       tags: ['v*']

   jobs:
     release:
       # Build for macOS (Intel and Apple Silicon)
       # Create GitHub release
       # Upload binaries
   ```

5. Add Homebrew formula template:
   ```ruby
   class LinearCli < Formula
     desc "Fast command-line client for Linear"
     homepage "https://github.com/user/linear-cli"
     # Auto-updated by release process
   end
   ```

6. Create man page generation:
   - Use clap_mangen
   - Include in releases
   - Install with Homebrew

7. Final checklist:
   - README with GIFs/screenshots
   - CHANGELOG.md
   - LICENSE file
   - Security policy
   - Contributing guidelines

8. Performance benchmarks:
   - Startup time < 50ms
   - Issues list < 200ms
   - JSON parsing benchmark

This completes the Linear CLI with professional polish.
```

---

## Implementation Strategy

### Key Principles
1. **Always have working software** - Each step produces a usable result
2. **Test continuously** - Both unit and integration tests from the start
3. **Fail fast** - Discover integration issues immediately
4. **User-focused** - Helpful errors and beautiful output
5. **Progressive enhancement** - Start simple, add complexity gradually

### Testing Strategy
- Unit tests with mocked API responses (fast, offline)
- Integration tests with real API (thorough, requires connection)
- Feature flag `integration-tests` to separate test types
- Snapshot tests for output formatting
- Manual tests documented for OAuth flow

### Risk Mitigation
- API validation before building (Prompt 1)
- Real schema from day one (Prompt 2)
- Working end-to-end in Prompt 3
- CI pipeline in Prompt 4
- Progressive feature addition with tests

Each prompt builds directly on previous work with no orphaned code. The progression ensures we validate assumptions early and maintain momentum throughout development.
