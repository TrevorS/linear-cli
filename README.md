# linear-cli

A fast, feature-rich command-line interface for Linear with beautiful terminal output.

## Installation

```bash
cargo install linear-cli
```

## Usage

### Authentication

You can authenticate using either OAuth (recommended) or API key:

#### OAuth Authentication (Recommended)

To use OAuth authentication, you'll need to create a Linear OAuth application:

1. Go to https://linear.app/settings/api/applications/new
2. Set the callback URL to: `http://localhost:8089/callback`
3. Save your client ID
4. Configure the CLI with your client ID:

```bash
# Login with OAuth
linear login

# Login with a custom OAuth client ID
linear login --client-id your-client-id

# Or set it as an environment variable
export LINEAR_OAUTH_CLIENT_ID=your-client-id
linear login

# Logout
linear logout
```

Credentials are securely stored in your system's keychain.

#### API Key Authentication
```bash
export LINEAR_API_KEY=lin_api_xxxxx
```

For development, you can also create a `.env` file:
```bash
echo 'LINEAR_API_KEY=your_api_key_here' > .env
source scripts/setup-env.sh  # Load the environment
```

### Commands

```bash
# List issues in a formatted table
linear issues

# List issues without color
linear --no-color issues

# List a specific number of issues
linear issues --limit 10

# Output as JSON
linear issues --format json

# Pretty-print JSON output
linear issues --format json --pretty

# View detailed information about a specific issue
linear issue ENG-123

# Get current user information
linear me

# Get help
linear --help
```

### Features

- **🔐 OAuth Authentication**: Secure OAuth flow with system keychain integration
- **📊 Beautiful Table Output**: Issues displayed in clean, formatted tables with smart column sizing
- **🎨 Rich Terminal Formatting**:
  - Color-coded status indicators: Todo (gray), In Progress (yellow), Done (green)
  - Colored labels with matching backgrounds
  - Priority indicators with visual cues
  - Smart truncation for long titles
- **📝 Markdown Rendering**: Full markdown support in issue descriptions with:
  - Syntax highlighting for code blocks
  - Formatted lists, blockquotes, and emphasis
  - Proper heading hierarchy
  - Media attachments with visual indicators
- **🔗 Clickable Links**: OSC-8 hyperlink support for compatible terminals (iTerm2, Ghostty, WezTerm, etc.)
- **📋 Multiple Output Formats**: Table (default) or JSON with optional pretty-printing
- **🎯 Detailed Issue Views**: Complete issue information including:
  - Description with full markdown rendering
  - Assignee, team, and project details
  - Labels with color indicators
  - Priority levels
  - Timestamps
  - Direct Linear links
- **⚡ Fast and Efficient**: Built with Rust for optimal performance
- **🛡️ Robust Error Handling**: Automatic retries with exponential backoff
- **🎮 Color Control**: Respects `--no-color` flag and `NO_COLOR` environment variable
- **🧪 Well-tested**: Comprehensive test suite with snapshot testing

## Development

This project uses a Cargo workspace structure:

- `linear-cli/` - Main CLI binary
- `linear-sdk/` - Reusable Linear API client library
- `xtask/` - Build automation and schema management

### Getting Started

```bash
# Build the workspace
cargo build

# Run tests
cargo test --workspace

# Run the CLI
cargo run -p linear-cli

# Run linting and formatting
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

### Pre-commit Hooks

Set up pre-commit hooks to ensure code quality:

```bash
# Option 1: Using pre-commit framework (recommended)
uv tool install pre-commit
pre-commit install

# Option 2: Manual git hook
cp scripts/pre-commit-hook.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

See `docs/pre-commit.md` for detailed setup instructions.

### Schema Management

Download the latest Linear GraphQL schema:

```bash
cargo run -p xtask -- schema --api-key YOUR_API_KEY
```

## License

MIT
