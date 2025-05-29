# Linear CLI - A Fun Terminal Client for Linear

## 🎯 What Are We Building?

A sleek command-line tool for Linear that lets you manage issues without leaving your terminal. Perfect for those moments when you're deep in code and just need to quickly check what you're supposed to be working on (or update that ticket you forgot about).

**Why?** Because switching to the browser is for people who don't live in tmux.

## 🚀 Project Goals

- **Fast issue browsing** - See your work without context switching
- **Scriptable** - Pipe it, grep it, jq it - make it yours
- **Extensible** - Built to grow with more commands as we need them
- **OAuth support** - Because manually copying API keys is so 2020
- **Just works™** - On macOS (for now)

## 📦 What's In The Box?

### Phase 1: The Basics (MVP)
```bash
# See what you're supposed to be working on
$ linear issues --assignee me --status "In Progress"

# Morning standup helper
$ linear issues --status "Todo" --team ENG

# Quick issue details
$ linear issue ENG-123

# Pretty JSON for your scripts
$ linear issues --json | jq '.[] | select(.priority == 1)'
```

### Phase 2: More Context
```bash
# View comments on an issue
$ linear comments ENG-123

# See all projects
$ linear projects

# List your teams
$ linear teams

# View a project's issues
$ linear project "Mobile App"

# Search everything
$ linear search "bug in login"
```

### Phase 3: Actually Do Things
```bash
# Create an issue
$ linear create --title "Fix that thing" --team ENG --assignee me

# Update status
$ linear update ENG-123 --status "In Review"

# Add a comment
$ linear comment ENG-123 "Fixed in PR #456"

# Quick close
$ linear close ENG-123

# Bulk operations
$ linear bulk update --status "Done" --filter "assignee:me,status:Review"
```

## 🏗 Architecture

### Project Structure
```
linear-cli/
├── Cargo.toml                 # Workspace root
├── linear-cli/                # The CLI binary
│   ├── src/
│   │   ├── main.rs           # Entry point & command router
│   │   ├── cli.rs            # Clap command definitions
│   │   ├── commands/         # One file per command
│   │   │   ├── mod.rs
│   │   │   ├── issues.rs    # List issues
│   │   │   ├── issue.rs     # Show single issue
│   │   │   ├── create.rs    # Create issue
│   │   │   ├── update.rs    # Update issue
│   │   │   └── ...          # More as we add them
│   │   ├── config.rs         # Auth & settings
│   │   ├── output/           # Formatters
│   │   │   ├── mod.rs
│   │   │   ├── table.rs     # Pretty tables
│   │   │   └── json.rs      # JSON output
│   │   └── oauth.rs          # OAuth flow (feature-gated)
│   └── tests/                # Integration tests
└── linear-sdk/               # Reusable Linear API client
    ├── src/
    │   ├── lib.rs            # Public API
    │   ├── client.rs         # HTTP client setup
    │   ├── graphql/          # All the queries
    │   │   ├── mod.rs
    │   │   ├── queries.rs    # Query definitions
    │   │   └── mutations.rs  # Mutations (phase 3)
    │   └── types.rs          # Generated from schema
    └── graphql/
        ├── schema.json       # Linear's GraphQL schema
        └── *.graphql         # Our query files
```

### Tech Stack

We're using the best-in-class Rust crates:

- **clap** - CLI parsing with derive macros (so clean!)
- **tokio** - Async all the things
- **reqwest** - HTTP client that just works
- **graphql_client** - Type-safe GraphQL with code generation
- **tabled** - Beautiful terminal tables
- **owo-colors** - Colors that respect your terminal
- **anyhow** - Error handling without the boilerplate

Optional OAuth support adds:
- **oauth2** - OAuth flows done right
- **keyring** - macOS Keychain integration
- **open** - Launch the browser for auth

### GraphQL Strategy

We're using `graphql_client` which is the de-facto standard for Rust GraphQL clients. It gives us:

- **Compile-time type safety** - If Linear changes their API, our code won't compile
- **Code generation from schema** - No manual type definitions
- **Works great with async** - First-class tokio/reqwest support

Example query:
```graphql
query GetIssue($id: String!) {
  issue(id: $id) {
    id
    identifier
    title
    description
    state { name type color }
    assignee { name email avatarUrl }
    project { name }
    team { key name }
    comments {
      nodes {
        body
        user { name }
        createdAt
      }
    }
    priority
    priorityLabel
    createdAt
    updatedAt
  }
}
```

## 🎨 User Experience

### Beautiful by Default
```
$ linear issues --assignee me

 Issue     Title                          Status        Project         
 ─────────────────────────────────────────────────────────────────────
 ENG-123   Fix login race condition       In Progress   Web App        
 ENG-124   Implement OAuth flow           Todo          Web App        
 MOB-89    Crash on iOS 17.2              In Review     Mobile         
```

### Smart Filtering
```bash
# Natural language status
$ linear issues --status todo  # matches "Todo", "To Do", etc.

# Special assignee values
$ linear issues --assignee me
$ linear issues --assignee unassigned

# Combine filters
$ linear issues --team ENG --status "In Progress" --priority high
```

### Helpful Errors
```bash
$ linear issues
Error: No Linear API key found

To authenticate, either:
  1. Set the LINEAR_API_KEY environment variable
  2. Run 'linear login' for OAuth authentication
  
Get your API key from: https://linear.app/settings/api
```

## 🔑 Authentication

### Option 1: API Key (Simple)
```bash
export LINEAR_API_KEY=lin_api_xxxxx
linear issues
```

### Option 2: OAuth (Fancy)
```bash
$ linear login
Opening browser for Linear authentication...
✓ Authentication successful! Token saved to macOS Keychain.
```

OAuth tokens are stored securely in the macOS Keychain. No plain text files, no leaked secrets in your dotfiles repo.

## 🛠 Development Setup

```bash
# Clone it
git clone https://github.com/yourusername/linear-cli
cd linear-cli

# Build it
cargo build

# Run it
cargo run -- issues --assignee me

# Test it
cargo test
cargo nextest run  # if you're fancy

# Install it
cargo install --path linear-cli
```

### Schema Updates
```bash
# Download latest Linear GraphQL schema
LINEAR_API_KEY=xxx cargo xtask update-schema

# Or use the committed schema for offline builds
OFFLINE=1 cargo build
```

## 🧪 Testing Approach

- **Unit tests** for the important bits (filter mapping, output formatting)
- **Integration tests** with mocked Linear API responses
- **Snapshot tests** for CLI output using `trycmd`
- **Real API tests** behind a feature flag for the brave

## 🗺 Command Roadmap

### Now (Phase 1)
- [x] `issues` - List with filters
- [x] `issue <id>` - Show details
- [x] `login` - OAuth flow

### Next (Phase 2)
- [ ] `projects` - List all projects
- [ ] `teams` - List all teams  
- [ ] `comments <id>` - Show issue comments
- [ ] `search <query>` - Full text search
- [ ] `my work` - Smart view of your current work

### Later (Phase 3)
- [ ] `create` - New issue with interactive prompts
- [ ] `update <id>` - Change fields
- [ ] `comment <id>` - Add comment
- [ ] `close/reopen <id>` - Quick status changes
- [ ] `bulk` - Update multiple issues
- [ ] `watch <id>` - Follow issue changes

### Maybe Someday
- [ ] `linear tui` - Full TUI interface
- [ ] `linear sync` - Offline support
- [ ] `linear hook` - Git hooks integration
- [ ] `linear alfred` - Alfred workflow generator

## 🤝 Contributing

This is a fun side project! Feel free to:

- Add a new command (just add a file in `commands/`)
- Improve the output formatting
- Add more filter options
- Make error messages even friendlier
- Add Linux/Windows support if you're motivated

The codebase is structured to make adding new commands super easy. Just implement the `Command` trait and add it to the CLI enum.

## 📝 License

MIT - Use it, fork it, make it yours!

---

Built with ❤️ and lots of ☕ by developers who prefer terminals over browsers.