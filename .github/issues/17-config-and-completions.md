## Description

Add configuration file support and shell completions for improved user experience.

## Context

From the implementation plan (Prompt 17), we need to:
- Create configuration file support
- Add command aliases
- Generate shell completions

## Acceptance Criteria

- [ ] Create config module:
  ```rust
  #[derive(Deserialize, Default)]
  pub struct Config {
      pub default_team: Option<String>,
      pub default_assignee: Option<String>,
      pub preferred_format: Option<OutputFormat>,
      pub aliases: HashMap<String, Vec<String>>,
  }
  ```
- [ ] Load config from standard locations:
  - [ ] `~/.config/linear-cli/config.toml`
  - [ ] `$XDG_CONFIG_HOME/linear-cli/config.toml`
  - [ ] `.linear-cli.toml` in current directory
- [ ] Example config file:
  ```toml
  default_team = "ENG"
  default_assignee = "me"
  preferred_format = "table"

  [aliases]
  my = ["issues", "--assignee", "me"]
  todo = ["issues", "--status", "todo", "--assignee", "me"]
  standup = ["issues", "--team", "ENG", "--updated-after", "yesterday"]
  ```
- [ ] Generate shell completions:
  ```rust
  /// Generate shell completions
  Completions {
      /// Shell to generate for
      #[arg(value_enum)]
      shell: clap_complete::Shell,
  }
  ```
- [ ] Add installation instructions:
  ```bash
  # Bash
  linear completions bash > ~/.local/share/bash-completion/completions/linear

  # Zsh
  linear completions zsh > ~/.zfunc/_linear

  # Fish
  linear completions fish > ~/.config/fish/completions/linear.fish
  ```
- [ ] Support config in commands:
  - [ ] Use default_team if --team not specified
  - [ ] Apply aliases before parsing args
  - [ ] Respect preferred_format
- [ ] Add config validation with helpful errors

## Example Usage

```bash
# Use alias
linear my  # expands to: linear issues --assignee me

# Generate completions
linear completions zsh > ~/.zfunc/_linear
```

## Technical Details

- Use directories-rs for platform-specific paths
- Use toml for config parsing
- Use clap_complete for shell completions

## Dependencies

- Depends on: #16 (Bulk Operations)

## Definition of Done

- [ ] Config files load from standard locations
- [ ] Aliases work correctly
- [ ] Defaults apply when not overridden
- [ ] Shell completions generate correctly
- [ ] Installation instructions clear
- [ ] Config errors show helpful messages
