## Description

Add beautiful table output for issues using the tabled crate. This significantly improves the user experience.

## Context

From the implementation plan (Prompt 6), we need to:
- Add table formatting with tabled
- Support colored output
- Implement output format abstraction

## Acceptance Criteria

- [ ] Add dependencies to linear-cli:
  - [ ] tabled = "0.15"
  - [ ] owo-colors = "4"
- [ ] Create output module structure:
  ```rust
  pub trait OutputFormat {
      fn format_issues(&self, issues: &[Issue]) -> Result<String>;
  }
  
  pub struct TableFormatter {
      use_color: bool,
  }
  ```
- [ ] Implement table formatting:
  - [ ] Columns: Issue, Title, Status, Assignee
  - [ ] Truncate title to 40 chars with "..."
  - [ ] Color status: Todo=gray, In Progress=yellow, Done=green
  - [ ] Show "Unassigned" in dimmed text
- [ ] Add `--no-color` flag globally:
  ```rust
  #[derive(Parser)]
  struct Cli {
      /// Disable colored output
      #[arg(long, global = true)]
      no_color: bool,
      
      #[command(subcommand)]
      command: Commands,
  }
  ```
- [ ] Respect `NO_COLOR` and `TERM` environment variables
- [ ] Add snapshot tests using insta:
  - [ ] Colored output snapshot
  - [ ] Non-colored output snapshot
  - [ ] Long titles snapshot
  - [ ] Empty results snapshot

## Expected Output

```
 Issue     Title                          Status        Assignee
 ─────────────────────────────────────────────────────────────────
 ENG-123   Fix login race condition       In Progress   John Doe
 ENG-124   Implement OAuth flow           Todo          Unassigned
```

## Technical Details

- Use tabled's built-in themes
- Detect terminal capabilities for color support
- Ensure output is properly aligned

## Dependencies

- Depends on: #5 (Issues List Command)

## Definition of Done

- [ ] Issues display in a formatted table
- [ ] Colors work when supported
- [ ] --no-color flag disables colors
- [ ] Long titles are truncated cleanly
- [ ] Snapshot tests pass