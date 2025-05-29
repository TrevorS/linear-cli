## Description

Add the first real feature: listing issues with a proper CLI structure. This establishes patterns for all future commands.

## Context

From the implementation plan (Prompt 5), we need to:
- Create GraphQL query for listing issues
- Add CLI command structure with clap
- Implement basic output formatting

## Acceptance Criteria

- [ ] Create issues query in linear-sdk:
  ```graphql
  query ListIssues($first: Int!) {
    issues(first: $first) {
      nodes {
        id
        identifier
        title
        state { name }
        assignee { name }
      }
    }
  }
  ```
- [ ] Extend LinearClient:
  - [ ] Add `list_issues(&self, limit: i32)` method
  - [ ] Return simplified Issue struct (not raw GraphQL types)
- [ ] Add CLI structure with clap:
  ```rust
  #[derive(Parser)]
  struct Cli {
      #[command(subcommand)]
      command: Commands,
  }
  
  #[derive(Subcommand)]
  enum Commands {
      /// List issues
      Issues {
          /// Maximum number of issues to fetch
          #[arg(short, long, default_value = "20")]
          limit: i32,
      },
  }
  ```
- [ ] Implement basic output:
  - [ ] Format: "ISSUE-ID: Title (Status) - Assignee"
  - [ ] Handle unassigned issues gracefully
  - [ ] One line per issue
- [ ] Add tests:
  - [ ] Unit test with mocked response
  - [ ] Integration test that fetches real issues
  - [ ] Test empty results case
- [ ] Update error handling to be command-specific

## Technical Details

```rust
// Simplified Issue struct
pub struct Issue {
    pub id: String,
    pub identifier: String,
    pub title: String,
    pub status: String,
    pub assignee: Option<String>,
}
```

## Dependencies

- Depends on: #4 (CI Pipeline)

## Definition of Done

- [ ] `linear issues` lists up to 20 issues
- [ ] `linear issues --limit 5` limits results
- [ ] Unassigned issues show gracefully
- [ ] Tests pass in CI
- [ ] Error messages are helpful and contextual