# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2025-02-05

### Added
- `--estimate` flag on `create` and `update` commands for setting issue estimate points
- `--label` flag (repeatable) on `create` and `update` for assigning labels by name
- `--cycle` flag on `create` and `update` for assigning issues to cycles ("current", cycle number, or name)
- `attach` command for attaching URLs (e.g., pull requests) to issues
- Estimate, labels, and cycle support in markdown frontmatter for `--from-file` creation
- Label name resolution with case-insensitive matching and error suggestions
- Cycle resolution supporting "current"/"active" keyword, cycle numbers, and name matching

### Changed
- Update command now fetches issue team in a single call when resolving status, labels, or cycles
- Improved CLAUDE.md with architectural patterns and new command checklist

## [0.2.0] - 2025-01-15

### Added
- Comprehensive README with usage examples and configuration guide
- GitHub Actions workflow for automated cross-platform releases
- Shell completions for bash, zsh, fish, and PowerShell
- MIT license file

### Changed
- Improved package metadata for better discoverability

## [0.1.0] - 2024-12-11

### Added
- Initial release of Linear CLI
- Issue management: list, view, create, update, close, reopen
- Rich terminal output with color-coded tables
- Markdown rendering with syntax highlighting
- OAuth and API key authentication
- Configuration system with TOML files and aliases
- Multiple output formats (table, JSON, YAML)
- Interactive issue creation
- Markdown file-based issue creation with frontmatter
- Project and team browsing
- Issue commenting
- Search functionality
- Cross-platform support (Linux, macOS, Windows)
- Comprehensive test suite with snapshot testing
- Performance optimizations for fast startup
- TTY detection for smart formatting
- Image display support in compatible terminals
- Command aliases and configuration hierarchy
- Extensive filtering options for issues
- Development tooling with Make-based workflows
