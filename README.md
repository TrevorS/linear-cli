# linear-cli

A command-line interface for Linear.

## Installation

```bash
cargo install linear-cli
```

## Usage

```bash
linear-cli --help
```

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
