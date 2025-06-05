# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Linear CLI is a command-line interface for Linear (issue tracking/project management tool) built in Rust. The project provides fast issue browsing and management directly from the terminal with beautiful table formatting and color-coded output.

The CLI is fully functional and includes:
- Issue listing with filters (assignee, status, team, labels)
- Interactive OAuth authentication with keychain storage
- Beautiful table output with color coding
- TTY detection for automatic color/formatting adjustments
- Enhanced error handling with retry logic
- Comprehensive test coverage with snapshot testing

## Essential Commands

**Start here**: `make help` shows all available commands organized by category.

### Quick Workflows
```bash
make dev            # Quick development check (fmt, lint, test)
make all            # Full workflow (fmt, lint, test, build)
make dev-setup      # Complete development environment setup
```

### Testing & Debugging
```bash
make test           # Run all tests
make test-snapshots # Run tests with snapshot review
make test-debug     # Run tests with debug output
make run            # Run CLI with example command
make run-debug      # Run CLI with debug logging
make run-piped      # Test CLI output when piped (no TTY)

# Inline images testing
cargo test --features inline-images    # Test with image support
cargo run --features inline-images -p linear-cli -- issue [ID] --force-images
```

### Code Quality
```bash
make fmt            # Format code
make lint           # Run clippy with strict warnings
make check          # CI-style format and lint checks
```

### Advanced
```bash
make debug-deps     # Show dependency tree and check for issues
cargo run -p xtask -- schema --api-key YOUR_API_KEY  # Update GraphQL schema
```

## Development Notes

### First Time Setup
1. Run `make dev-setup` for complete environment setup
2. Copy `.env.example` to `.env` and add your Linear API key
3. VS Code users get automatic configuration and extension recommendations

### Authentication
- **For development**: Use API key (LINEAR_API_KEY in .env) to avoid macOS keychain dialogs
- **For users**: OAuth flow with keychain storage for secure token management
- API key takes precedence over OAuth if both are available
- **Tip**: Use the API key in .env to avoid CLI auth pop ups

### Logging
The CLI uses structured logging via the `log` crate and `env_logger`:
- **Enable debug logging**: `RUST_LOG=debug linear <command>`
- **CLI-specific logging**: `RUST_LOG=linear_cli=debug linear <command>`
- **SDK-specific logging**: `RUST_LOG=linear_sdk=debug linear <command>`
- **Multiple modules**: `RUST_LOG=linear_cli=debug,linear_sdk=info linear <command>`
- **Legacy**: `LINEAR_CLI_VERBOSE` is still displayed in diagnostics for reference but no longer controls debug output

### Inline Images (Optional Feature)
The `inline-images` feature enables displaying images from Linear issues directly in compatible terminals:
- **Supported terminals**: Kitty (primary), Ghostty, WezTerm
- **Automatic detection**: Enabled automatically in supported terminals
- **Manual control**: Use `--force-images` or `--no-images` flags
- **Fallback**: Gracefully falls back to clickable links in unsupported terminals
- **Security**: Only processes images from allowed domains (uploads.linear.app by default)
- **Caching**: Downloaded images are cached locally for performance

### Cargo Aliases
The project includes helpful cargo aliases (`.cargo/config.toml`):
```bash
cargo c          # check --workspace
cargo t          # test --workspace
cargo f          # fmt --all
cargo l          # clippy --workspace --all-targets -- -D warnings
cargo r -- args  # run -p linear-cli -- args
cargo tsnap      # insta test --review
cargo dtree      # tree --workspace
cargo deps       # outdated (if installed)
cargo sec        # audit (if installed)
```

### Testing Philosophy
- **Unit tests**: Core SDK functionality, authentication, error handling
- **Integration tests**: Mocked API responses using mockito
- **Snapshot tests**: Terminal output formatting verification with insta
- **Error scenarios**: Network failures, API errors, retry logic
- Always use `make test-snapshots` when changing output formatting

## Architecture

The project is structured as a Cargo workspace with:
- `linear-cli/`: Main CLI binary crate with commands and output formatting
- `linear-sdk/`: Reusable Linear API client library with authentication and retry logic
- `xtask/`: Build automation and schema management tools

### Current Implementation Status:
- ✅ **Core SDK**: Authentication (OAuth + API key), GraphQL queries, error handling, retry logic
- ✅ **CLI Commands**: Issue listing with comprehensive filtering options
- ✅ **Output Formatting**: Beautiful tables with color coding and TTY detection
- ✅ **Error Handling**: Enhanced error messages with retry and user guidance
- 🚧 **Advanced Features**: Bulk operations, advanced search (future work)
- 🚧 **Polish**: Shell completions, homebrew formula (future work)

### Technology Stack:
- **Async Runtime**: tokio
- **HTTP Client**: reqwest with custom retry implementation
- **CLI Parsing**: clap (derive macros)
- **GraphQL**: graphql_client with build-time code generation
- **Error Handling**: anyhow with custom error types and native retry logic
- **Terminal UI**: tabled (ansi feature), owo-colors with TTY detection
- **Logging**: structured logging with env_logger and log crate
- **Testing**: insta for snapshots, mockito for mocked HTTP responses
- **Authentication**: OAuth with keychain storage, API key fallback

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

## Snapshot Testing

Critical for output formatting changes:
```bash
make test-snapshots       # Run tests with snapshot review (recommended)
cargo insta test          # Run snapshot tests only
cargo insta review        # Review pending changes
cargo insta accept        # Accept all pending snapshots
```

**Important**: Always review snapshot changes carefully. The pre-commit hook excludes `.snap` files from trailing whitespace checks to preserve exact output formatting.

## Debugging Guide

### Common Issues & Solutions
- **macOS Keychain Dialogs**: Use `LINEAR_API_KEY` in .env during development
- **TTY Detection**: Test both `make run` and `make run-piped` for output formatting
- **Snapshot Mismatches**: Use `make test-snapshots` to review and accept changes
- **Dependency Issues**: Use `make debug-deps` to check for problems and security advisories

### Quick Debug Commands
```bash
make run-debug      # Run CLI with debug logging
make test-debug     # Run tests with debug output
make check          # Quick code quality check
```

## Important Project Documentation

- `docs/specs.md`: Complete project specification with user stories and technical requirements
- `docs/plan.md`: Detailed implementation roadmap and decisions
- `linear-api-spike/`: API validation findings and examples
- `linear-sdk/graphql/schema.json`: Current GraphQL schema from Linear
