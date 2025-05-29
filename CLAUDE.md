# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Linear CLI is a command-line interface for Linear (issue tracking/project management tool) built in Rust. The project provides fast issue browsing and management directly from the terminal.

## Build and Development Commands

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run the CLI
cargo run

# Run with specific arguments
cargo run -- [args]

# Build release version
cargo build --release

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Architecture

The project is structured as a Cargo workspace (planned) with:
- `linear-cli`: Main CLI binary crate
- `linear-sdk`: Reusable Linear API client library (planned)

### Key Implementation Phases (from docs/plan.md):
1. **Phase 0**: Initial setup and validation spike
2. **Phase 1**: Core SDK functionality (authentication, basic queries)
3. **Phase 2**: CLI functionality (commands, formatting)
4. **Phase 3**: Advanced features (bulk operations, search)
5. **Phase 4**: Polish (shell completions, homebrew formula)

### Technology Stack:
- **Async Runtime**: tokio
- **HTTP Client**: reqwest
- **CLI Parsing**: clap
- **GraphQL**: graphql_client with code generation
- **Error Handling**: anyhow
- **Terminal UI**: tabled, owo-colors

## Linear API Integration

The project uses Linear's GraphQL API. Key considerations:
- OAuth flow for authentication (with API key fallback)
- GraphQL schema-driven development
- Type-safe query generation using graphql_client

### API Validation Findings (from linear-api-spike/)
- **Authentication**: Use `Authorization: <API_KEY>` header (no Bearer prefix)
- **Schema**: Introspection is available - see `linear-api-spike/schema.json`
- **Rate Limits**: Headers not observed in testing
- **Error Format**: Standard GraphQL errors array with extensions
- **API URL**: `https://api.linear.app/graphql`

## Testing Strategy

- Unit tests for SDK components
- Integration tests with mocked API responses
- End-to-end tests against Linear's API (behind feature flag)
- Test coverage for all major functionality

## Important Project Documentation

- `docs/specs.md`: Complete project specification with user stories and technical requirements
- `docs/plan.md`: Detailed 18-prompt implementation roadmap
- GraphQL schema will be stored in `linear-sdk/schema/` (when implemented)