# linear-cli

Fast command-line interface for Linear issue tracking.

## Installation

```bash
cargo install linear-cli
```

## Authentication

**Option 1: API Key (Simple)**

Get a Linear API key at https://linear.app/settings/api and set:

```bash
export LINEAR_API_KEY=lin_api_xxxxx
```

**Option 2: OAuth (Interactive)**

```bash
linear login  # Interactive OAuth flow
linear logout
```

## Commands

### List Issues
```bash
linear issues                    # Show recent issues
linear issues --limit 50        # Show more issues
linear issues --assignee me     # Your assigned issues
linear issues --status "done"   # Filter by status
linear issues --team ENG        # Filter by team
linear issues --json            # JSON output
```

### View Issue Details
```bash
linear issue ENG-123             # Full issue details with description
linear issue ENG-123 --json     # JSON output
linear issue ENG-123 --raw      # Plain markdown (no styling)
```

### Create Issues
```bash
# Interactive mode
linear create

# Direct command line
linear create --title "Fix login bug" --team ENG --assignee me --priority 2

# From markdown file with frontmatter
linear create --from-file issue.md

# Dry run (preview without creating)
linear create --title "Test" --team ENG --dry-run
```

#### Creating from Markdown Files

Create `issue.md`:
```markdown
---
title: "Fix authentication race condition"
team: ENG
assignee: me
priority: 1
labels: [bug, auth, urgent]
---

# Problem
Users experiencing login failures when multiple tabs open.

## Steps to Reproduce
1. Open multiple browser tabs
2. Login from each tab simultaneously
3. Some requests fail
```

Then run:
```bash
linear create --from-file issue.md
```

### Other Commands
```bash
linear status                    # Check connection
linear --help                    # Full help
```

## Features

- **Issue Management**: List, view, create, and filter issues
- **Rich Formatting**: Color-coded tables, markdown rendering with syntax highlighting
- **Flexible Input**: Interactive prompts, CLI args, or markdown files with frontmatter
- **Smart Terminal Detection**: Automatic color/formatting based on TTY
- **Authentication**: OAuth with keychain storage or API key
- **Output Formats**: Formatted tables or JSON
- **Image Support**: Inline images in compatible terminals (Kitty, iTerm2, WezTerm)

## Development

```bash
# Setup and quick checks
make dev-setup    # Initial setup
make dev          # Format, lint, test
make run          # Run CLI with sample data

# Build and test
make build        # Debug build
make test         # Run tests
make release      # Release build

# See all commands
make help
```

Project structure:
- `linear-cli/` - Main CLI binary
- `linear-sdk/` - Linear API client library
- `xtask/` - Build tools

## License

MIT
