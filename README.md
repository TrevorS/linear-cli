# linear-cli

A command-line interface for Linear.

## Installation

```bash
cargo install linear-cli
```

## Usage

First, set your Linear API key:

```bash
export LINEAR_API_KEY=lin_api_xxxxx
```

For development, you can also create a `.env` file:
```bash
echo 'LINEAR_API_KEY=your_api_key_here' > .env
source scripts/setup-env.sh  # Load the environment
```

Then use the CLI:

```bash
# List issues in a formatted table
linear issues

# List issues without color
linear --no-color issues

# List a specific number of issues
linear issues --limit 10

# Get help
linear --help
```

### Features

- **Beautiful table output**: Issues are displayed in a clean, formatted table
- **Color-coded status**: Todo (gray), In Progress (yellow), Done (green)
- **Smart truncation**: Long titles are truncated to fit the terminal
- **Color control**: Use `--no-color` flag or set `NO_COLOR` environment variable

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
