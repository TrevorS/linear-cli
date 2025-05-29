## Description

Add JSON output format for scriptability, enabling users to process Linear data programmatically.

## Context

From the implementation plan (Prompt 7), we need to:
- Add JSON serialization for issues
- Support both compact and pretty-printed JSON
- Enable piping to tools like jq

## Acceptance Criteria

- [ ] Add serde derives to Issue struct:
  ```rust
  #[derive(Debug, Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Issue {
      pub id: String,
      pub identifier: String,
      pub title: String,
      pub status: String,
      pub assignee: Option<String>,
  }
  ```
- [ ] Add JSON formatter:
  ```rust
  pub struct JsonFormatter {
      pretty: bool,
  }
  ```
- [ ] Add format flags to issues command:
  ```rust
  Issues {
      #[arg(short, long, default_value = "20")]
      limit: i32,
      
      /// Output as JSON
      #[arg(long)]
      json: bool,
      
      /// Pretty print JSON output
      #[arg(long, requires = "json")]
      pretty: bool,
  }
  ```
- [ ] Implement output routing:
  - [ ] Default: table format
  - [ ] `--json`: compact JSON array
  - [ ] `--json --pretty`: formatted JSON
- [ ] Ensure JSON fields match Linear's API naming conventions
- [ ] Add tests:
  - [ ] Valid JSON output
  - [ ] Can pipe to jq successfully
  - [ ] Pretty printing works correctly

## Example Usage

```bash
# Find high priority issues
linear issues --json | jq '.[] | select(.priority == "High")'

# Count issues by status
linear issues --json | jq 'group_by(.status) | map({status: .[0].status, count: length})'
```

## Technical Details

- Use serde_json for serialization
- Ensure consistent field naming with Linear's API
- Output to stdout for easy piping

## Dependencies

- Depends on: #6 (Table Formatting)

## Definition of Done

- [ ] `linear issues --json` outputs valid JSON
- [ ] `linear issues --json --pretty` outputs formatted JSON
- [ ] JSON can be piped to jq and other tools
- [ ] Field names match Linear's conventions
- [ ] Tests verify JSON validity