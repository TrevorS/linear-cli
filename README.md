# Linear CLI

```text
    __    _                       ________    ____
   / /   (_)___  ___  ____  _____/ ____/ /   /  _/
  / /   / / __ \/ _ \/ __ `/ ___/ /   / /    / /
 / /___/ / / / /  __/ /_/ / /  / /___/ /____/ /
/_____/_/_/ /_/\___/\__,_/_/   \____/_____/___/
```

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.88.0+-blue.svg)](https://www.rust-lang.org)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/TrevorS/linear-cli)

> Fast, scriptable Linear issue management from your terminal

A command-line interface for [Linear](https://linear.app) that lets you manage issues, projects, and teams without leaving your terminal. Built in Rust for speed and reliability.

## Table of Contents

- [Quick Start](#quick-start)
- [Installation](#installation)
- [Authentication](#authentication)
- [Features](#features)
- [Usage](#usage)
- [Configuration](#configuration)
- [Shell Completions](#shell-completions)
- [Scripting and Integration](#scripting-and-integration)
- [Development](#development)
- [Performance](#performance)
- [License](#license)

## Quick Start

```bash
# Install with image support (recommended)
git clone https://github.com/TrevorS/linear-cli.git
cd linear-cli
make install-images

# Authenticate
linear login

# Start using
linear issues --assignee me
```

## Installation

### From Source

```bash
git clone https://github.com/TrevorS/linear-cli.git
cd linear-cli

# Install with image support (recommended)
make install-images

# Or install without image support
make install
```

## Authentication

### OAuth

```bash
linear login   # Interactive browser-based authentication
linear logout  # Clear stored credentials
```

### API Key

Get a Linear API key at https://linear.app/settings/api:

```bash
export LINEAR_API_KEY=lin_api_xxxxx
```

For development, add to `.env`:
```bash
LINEAR_API_KEY=lin_api_xxxxx
```

## Features

- **Issue Management**: List, view, create, update, close, and reopen issues
- **Rich Terminal Output**: Color-coded tables with syntax-highlighted markdown
- **Flexible Input**: CLI arguments, interactive prompts, or markdown files with frontmatter
- **Smart Terminal Detection**: Automatic color/formatting based on TTY capabilities
- **Multiple Output Formats**: Formatted tables, JSON, or YAML
- **Configuration System**: TOML configs with aliases and XDG compliance
- **Shell Integration**: Completions for bash, zsh, fish, and PowerShell
- **Inline Images**: Display images from issues directly in compatible terminals (Kitty, Ghostty, WezTerm)
- **Cross-Platform**: Linux, macOS, and Windows support

## Usage

### List Issues

```bash
# Recent issues
linear issues

# Filter by assignee
linear issues --assignee me
linear issues --assignee alice
linear issues --assignee unassigned

# Filter by status
linear issues --status "In Progress"
linear issues --status done

# Filter by team
linear issues --team ENG

# Combine filters
linear issues --assignee me --status todo --team ENG

# JSON output for scripting
linear issues --json | jq '.[] | select(.priority == 1)'

# Pretty printed JSON
linear issues --json --pretty
```

### View Issue Details

```bash
linear issue ENG-123              # Full details with description
linear issue ENG-123 --json       # JSON output
linear issue ENG-123 --raw        # Plain markdown

# Image display options (compatible terminals)
linear issue ENG-123 --force-images  # Force display images
linear issue ENG-123 --no-images     # Disable image display
```

### Create Issues

```bash
# Interactive mode
linear create

# Command line
linear create \
  --title "Fix authentication timeout" \
  --team ENG \
  --assignee me \
  --priority 1

# From markdown file
linear create --from-file issue.md

# Dry run (preview without creating)
linear create --title "Test issue" --team ENG --dry-run

# Open created issue in browser
linear create --title "Test issue" --team ENG --open
```

#### Creating from Markdown Files

Create `issue.md`:
```markdown
---
title: "Fix authentication race condition"
team: ENG
assignee: me
priority: 1
---

# Problem

Users experiencing login failures when multiple tabs are open.

## Steps to Reproduce

1. Open multiple browser tabs
2. Login from each tab simultaneously
3. Some requests fail with timeout errors

## Expected Behavior

All login attempts should succeed or fail gracefully.
```

Then:
```bash
linear create --from-file issue.md
```

### Manage Issues

```bash
# Update issue status
linear update ENG-123 --status "In Progress"

# Close issue
linear close ENG-123

# Reopen issue
linear reopen ENG-123

# Add comment
linear comment ENG-123 "Fixed in PR #456"
```

### Browse Projects and Teams

```bash
# List projects
linear projects

# List teams
linear teams

# View comments on an issue
linear comments ENG-123

# Search across issues
linear search "authentication bug"
```

### Your Work

```bash
# See your assigned and created issues
linear my-work

# Morning standup helper
linear issues --assignee me --status "In Progress"
```

## Example Output

```
┌─────────┬───────────────────────────────────┬──────────┬─────────────┬──────────┐
│ ID      │ Title                             │ Assignee │ Status      │ Priority │
├─────────┼───────────────────────────────────┼──────────┼─────────────┼──────────┤
│ ENG-123 │ Fix authentication timeout        │ alice    │ Todo        │ High     │
│ ENG-124 │ Add user preferences UI           │ bob      │ In Progress │ Medium   │
│ ENG-125 │ Optimize database queries         │ carol    │ In Review   │ Low      │
│ ENG-126 │ Update API documentation          │ dave     │ Done        │ Medium   │
└─────────┴───────────────────────────────────┴──────────┴─────────────┴──────────┘
```

## Configuration

Linear CLI supports TOML configuration files:

### Config Locations

1. `./linear-cli.toml` (project-specific)
2. `$XDG_CONFIG_HOME/linear-cli/config.toml` (user config)
3. `~/.config/linear-cli/config.toml` (fallback)

### Example Configuration

```toml
# Default values
default_team = "ENG"
default_assignee = "me"
preferred_format = "table"

# Command aliases
[aliases]
my = ["issues", "--assignee", "me"]
todo = ["issues", "--status", "todo", "--assignee", "me"]
standup = ["issues", "--team", "ENG", "--updated-after", "yesterday"]
```

### Using Aliases

```bash
linear my          # Expands to: linear issues --assignee me
linear todo        # Expands to: linear issues --status todo --assignee me
linear standup     # Show team's recent activity
```

## Shell Completions

Generate and install completions for your shell:

```bash
# Generate completions
linear completions bash > ~/.local/share/bash-completion/completions/linear
linear completions zsh > ~/.zfunc/_linear
linear completions fish > ~/.config/fish/completions/linear.fish
linear completions powershell > linear_completions.ps1
```

Restart your shell or source the completion file.

## Scripting and Integration

### JSON Output

All commands support `--json` for machine-readable output:

```bash
# Get high-priority issues
linear issues --json | jq '.[] | select(.priority == 1) | .title'

# Count issues by status
linear issues --json | jq 'group_by(.status) | map({status: .[0].status, count: length})'

# Export team's work
linear issues --team ENG --json > team-issues.json
```

### Exit Codes

- `0`: Success
- `1`: General error
- `2`: Authentication error
- `3`: Network error
- `4`: Not found error

## Development

The project uses a Make-based workflow:

```bash
# Setup development environment
make dev-setup

# Quick development check
make dev          # Format, lint, test

# Run with sample data
make run

# See all available commands
make help
```

### Project Structure

```
linear-cli/
├── linear-cli/    # Main CLI binary
├── linear-sdk/    # Reusable Linear API client
└── xtask/         # Build automation tools
```

See [CLAUDE.md](./CLAUDE.md) for detailed development documentation.

## License

MIT
