# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Linear CLI is a Rust command-line interface for [Linear](https://linear.app) that provides fast issue management with beautiful terminal output. Key capabilities: issue CRUD, OAuth + API key auth, color-coded tables, TOML configuration with aliases, and shell completions.

## Essential Commands

**Start here**: `make help` shows all available commands.

```bash
# Daily workflow
make dev            # Format, lint, test
make all            # Format, lint, test, build
make dev-setup      # First-time setup

# Testing
make test           # Run all tests
make test-snapshots # Run with snapshot review (use when changing output)
cargo test module_name::test_name  # Run single test

# Running
make run            # Example issues command
make run-debug      # With RUST_LOG=debug
make run-piped      # Test non-TTY output
```

### Cargo Aliases (`.cargo/config.toml`)
```bash
cargo c          # check --workspace
cargo t          # test --workspace
cargo r -- args  # run -p linear-cli -- args
cargo l          # clippy with -D warnings
cargo tsnap      # insta test --review
```

## Development Notes

### Authentication
- **Development**: Use `LINEAR_API_KEY` in `.env` to avoid macOS keychain dialogs
- **Users**: `linear login` for OAuth flow with keychain storage
- API key takes precedence over OAuth when both are present

### Logging
```bash
RUST_LOG=debug linear issues              # All debug logging
RUST_LOG=linear_cli=debug linear issues   # CLI only
RUST_LOG=linear_sdk=debug linear issues   # SDK only
```

## Architecture

### Workspace Structure
```
linear-cli/     # CLI binary - commands, output formatting, config, aliases
linear-sdk/     # API client library - auth, GraphQL, retry logic, error types
xtask/          # Build tools - schema updates
```

### Data Flow
1. **CLI** (`main.rs`) parses args via clap, loads config (`config.rs`), expands aliases
2. **SDK** (`LinearClient`) handles auth, executes GraphQL queries with retry
3. **Output** (`output.rs`) formats response as table/JSON with color via `owo-colors` + `tabled`

### Key SDK Components
- `linear-sdk/src/lib.rs`: GraphQL query definitions via `#[derive(GraphQLQuery)]`
- `linear-sdk/src/error.rs`: `LinearError` enum with retryable/help_text methods
- `linear-sdk/src/retry.rs`: Exponential backoff for transient failures
- `linear-sdk/src/oauth.rs`: OAuth flow with keyring storage (feature-gated)

### Key CLI Components
- `linear-cli/src/cli.rs`: Command definitions via clap derive
- `linear-cli/src/output.rs`: `TableFormatter`, `JsonFormatter` implementing `OutputFormat` trait
- `linear-cli/src/config.rs`: TOML config loading with XDG path hierarchy
- `linear-cli/src/aliases.rs`: Alias expansion with cycle detection

### GraphQL Integration
- Schema: `linear-sdk/graphql/schema.json`
- Queries/Mutations: `linear-sdk/graphql/queries/*.graphql`, `linear-sdk/graphql/mutations/*.graphql`
- Update schema: `cargo run -p xtask -- schema --api-key YOUR_KEY`

## Linear API Notes

- **Auth header**: `Authorization: <API_KEY>` (no Bearer prefix)
- **API URL**: `https://api.linear.app/graphql`
- **OAuth callback**: `http://localhost:8089/callback`

## Testing

- **Snapshot tests**: Use `make test-snapshots` when changing terminal output
- **Mocked HTTP**: Tests use `mockito` for API responses
- **Review carefully**: `.snap` files are excluded from trailing whitespace hooks

## Configuration

Config loaded from (highest precedence first):
1. `./linear-cli.toml` (project)
2. `$XDG_CONFIG_HOME/linear-cli/config.toml`
3. `~/.config/linear-cli/config.toml`

```toml
default_team = "ENG"
default_assignee = "me"

[aliases]
my = ["issues", "--assignee", "me"]
todo = ["issues", "--status", "todo", "--assignee", "me"]
```

## Common Issues

- **Keychain dialogs**: Use `LINEAR_API_KEY` in `.env`
- **TTY detection**: Test with both `make run` and `make run-piped`
- **Snapshot mismatch**: Run `make test-snapshots` to review changes
