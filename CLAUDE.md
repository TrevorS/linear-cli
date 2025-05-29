# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Linear CLI is a command-line interface for Linear (issue tracking/project management tool) built in Rust. The project provides fast issue browsing and management directly from the terminal with beautiful table formatting and color-coded output.

## Build and Development Commands

```bash
# Build the project
cargo build

# Run tests
cargo test --workspace

# Run snapshot tests with review
cargo insta test --review

# Run the CLI
cargo run -p linear-cli

# Run with specific arguments
cargo run -p linear-cli -- [args]

# Example: List issues without color
cargo run -p linear-cli -- --no-color issues

# Build release version
cargo build --release --workspace

# Check code without building
cargo check --workspace

# Format code
cargo fmt --all

# Run linter
cargo clippy --workspace --all-targets -- -D warnings

# Set up pre-commit hooks
uv tool install pre-commit
pre-commit install

# Download Linear GraphQL schema
cargo run -p xtask -- schema --api-key YOUR_API_KEY
```

## Architecture

The project is structured as a Cargo workspace with:
- `linear-cli/`: Main CLI binary crate
- `linear-sdk/`: Reusable Linear API client library
- `xtask/`: Build automation and schema management

### Key Implementation Phases (from docs/plan.md):
1. **Phase 0**: Initial setup and validation spike
2. **Phase 1**: Core SDK functionality (authentication, basic queries)
3. **Phase 2**: CLI functionality (commands, formatting)
4. **Phase 3**: Advanced features (bulk operations, search)
5. **Phase 4**: Polish (shell completions, homebrew formula)

### Technology Stack:
- **Async Runtime**: tokio
- **HTTP Client**: reqwest
- **CLI Parsing**: clap (with derive macros)
- **GraphQL**: graphql_client with code generation
- **Error Handling**: anyhow
- **Terminal UI**: tabled (with ansi feature), owo-colors
- **Snapshot Testing**: insta

## Linear API Integration

The project uses Linear's GraphQL API. Key considerations:
- OAuth flow for authentication (with API key fallback)
- GraphQL schema-driven development
- Type-safe query generation using graphql_client

### OAuth Setup

To use OAuth authentication:

1. Create a Linear OAuth application at https://linear.app/settings/api/applications/new
2. Set the callback URL to: `http://localhost:8089/callback`
3. Configure your client ID using either:
   - Environment variable: `export LINEAR_OAUTH_CLIENT_ID=your-client-id`
   - Command line flag: `linear login --client-id your-client-id`
4. Run `linear login` to authenticate

### API Validation Findings (from linear-api-spike/)
- **Authentication**: Use `Authorization: <API_KEY>` header (no Bearer prefix)
- **Schema**: Introspection is available - see `linear-api-spike/schema.json`
- **Rate Limits**: Headers not observed in testing
- **Error Format**: Standard GraphQL errors array with extensions
- **API URL**: `https://api.linear.app/graphql`

## Testing Strategy

- Unit tests for SDK components
- Integration tests with mocked API responses (using mockito)
- End-to-end tests against Linear's API (behind feature flag)
- Snapshot tests for terminal output formatting (using insta)
- Test coverage for all major functionality

### Snapshot Testing

When working with output formatting, use snapshot tests:
```bash
# Run snapshot tests
cargo insta test

# Review and accept snapshot changes
cargo insta review

# Accept all pending snapshots
cargo insta accept
```

Note: The pre-commit hook excludes `.snap` files from trailing whitespace checks to preserve exact output formatting.

## Important Project Documentation

- `docs/specs.md`: Complete project specification with user stories and technical requirements
- `docs/plan.md`: Detailed 18-prompt implementation roadmap
- GraphQL schema will be stored in `linear-sdk/schema/` (when implemented)
