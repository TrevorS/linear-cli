# Implementation Plan: Issue #17 - Configuration and Completions

## Overview
Add configuration file support and shell completions to Linear CLI for enhanced user experience.

## Requirements Analysis

### Configuration File Support
- TOML config files in standard locations (XDG spec compliance)
- Default values for team, assignee, format
- Command aliases support
- Validation with helpful error messages

### Shell Completions
- `linear completions <shell>` command
- Support for bash, zsh, fish, powershell
- Dynamic completions for issue IDs, teams, users
- Installation instructions

## Architecture Decisions

### Configuration Module (`linear-cli/src/config.rs`)
- Use `serde` and `toml` for parsing
- Follow XDG Base Directory specification
- Hierarchical config loading (system → user → project)
- Validation using custom serde deserializers

### Completions Module (`linear-cli/src/completions.rs`)
- Use `clap_complete` for static completions
- Cache Linear API data for dynamic completions
- Async completion context loading

### Dependencies to Add
```toml
# In linear-cli/Cargo.toml
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
clap_complete = "4.4"
dirs = "5.0"  # For XDG directories
```

## Implementation Plan (TDD Approach)

### Phase 1: Configuration Foundation
**Target: Basic config file loading**

1. **Create config types and validation tests**
   - `tests/config_tests.rs` - Test config parsing, validation, merging
   - Test TOML parsing edge cases and error messages

2. **Implement config module**
   - `linear-cli/src/config.rs` - Core config types and loading logic
   - XDG-compliant path resolution
   - Hierarchical config merging

3. **Integration with CLI**
   - Update `main.rs` to load and apply config
   - Override config with CLI arguments

### Phase 2: Command Aliases
**Target: Custom command aliases working**

1. **Write alias tests**
   - Test alias expansion and validation
   - Test recursive alias detection
   - Test alias with argument substitution

2. **Implement alias system**
   - Expand aliases before clap parsing
   - Validate alias definitions
   - Prevent infinite recursion

### Phase 3: Shell Completions Foundation
**Target: Basic static completions**

1. **Create completions command tests**
   - Test completion generation for each shell
   - Test output format and structure

2. **Implement static completions**
   - Add `completions` subcommand using clap_complete
   - Generate shell-specific completion scripts

### Phase 4: Dynamic Completions
**Target: Context-aware completions**

1. **Write dynamic completion tests**
   - Mock Linear API responses for team/user data
   - Test completion caching behavior
   - Test completion performance

2. **Implement dynamic completions**
   - Cache team and user data
   - Provide issue ID, team, and assignee completions
   - Handle API failures gracefully

### Phase 5: Documentation and Polish
**Target: Complete feature with docs**

1. **Update documentation**
   - Add config examples to README
   - Shell completion installation instructions
   - Update help text and error messages

2. **Integration testing**
   - End-to-end tests with real config files
   - Test completions in actual shell environments

## File Structure Changes

### New Files
```
linear-cli/src/
├── config.rs           # Configuration loading and validation
├── completions.rs      # Shell completion generation
└── aliases.rs          # Command alias expansion

linear-cli/tests/
├── config_tests.rs     # Unit tests for config functionality
├── completions_tests.rs # Unit tests for completions
└── integration_tests.rs # End-to-end config+completions tests

docs/
└── configuration.md    # User documentation for config and completions
```

### Modified Files
```
linear-cli/src/
├── main.rs             # Integrate config loading and alias expansion
├── cli.rs              # Add completions subcommand
└── lib.rs              # Export new modules

linear-cli/Cargo.toml   # Add new dependencies
README.md               # Update with config and completion docs
```

## Configuration File Locations (Priority Order)
1. `./linear-cli.toml` (project-specific)
2. `$XDG_CONFIG_HOME/linear-cli/config.toml` (user config)
3. `~/.config/linear-cli/config.toml` (fallback user config)

## Example Configuration Structure
```toml
# Default values for commands
default_team = "ENG"
default_assignee = "me"
preferred_format = "table"
api_url = "https://api.linear.app/graphql"

# Command aliases
[aliases]
my = ["issues", "--assignee", "me"]
todo = ["issues", "--status", "todo", "--assignee", "me"]
standup = ["issues", "--team", "ENG", "--updated-after", "yesterday"]

# Shell completion settings
[completions]
cache_duration = "1h"
enable_dynamic = true
```

## Testing Strategy

### Unit Tests
- Config parsing with various TOML inputs
- Alias expansion with edge cases
- Completion generation for all shells
- Error handling and validation

### Integration Tests
- Full config loading with file system
- End-to-end alias execution
- Completion installation and usage

### Performance Tests
- Config loading time
- Completion response time
- API data caching efficiency

## Risk Mitigation

### Breaking Changes
- All new features are opt-in
- Existing CLI behavior unchanged without config
- Graceful fallback when config files missing

### Error Handling
- Detailed validation errors with suggestions
- Graceful degradation when Linear API unavailable
- Clear installation instructions for completions

### Security
- Config file permission validation
- No sensitive data in config files (OAuth tokens in keychain)
- Validate alias commands to prevent injection

## Success Criteria

### Configuration
- [ ] Config files load from correct locations
- [ ] Default values apply correctly
- [ ] Aliases expand and execute properly
- [ ] Validation errors are helpful
- [ ] Performance impact minimal

### Completions
- [ ] All shells supported (bash, zsh, fish, powershell)
- [ ] Dynamic completions work for teams/users
- [ ] Installation instructions clear
- [ ] Completion caching improves performance
- [ ] Graceful fallback when API unavailable

## Implementation Timeline
- **Phase 1-2**: Configuration foundation and aliases (~2-3 days)
- **Phase 3**: Static completions (~1 day)
- **Phase 4**: Dynamic completions (~2 days)
- **Phase 5**: Documentation and polish (~1 day)

**Total Estimated Time**: 6-7 days

## Next Steps
1. Review this plan with team
2. Set up development branch
3. Begin Phase 1 with config parsing tests
4. Iterative development following TDD principles
