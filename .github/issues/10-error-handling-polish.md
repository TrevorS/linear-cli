## Description

Establish consistent error handling and improve user experience with helpful error messages and progress indicators.

## Context

From the implementation plan (Prompt 10), we need to:
- Create custom error types
- Add user-friendly error display
- Implement progress indicators
- Add retry logic for transient failures

## Acceptance Criteria

- [ ] Create custom error types:
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum LinearError {
      #[error("Authentication failed. Check your LINEAR_API_KEY")]
      Auth,

      #[error("Issue {0} not found")]
      IssueNotFound(String),

      #[error("Network error: {0}")]
      Network(String),

      #[error("GraphQL error: {0}")]
      GraphQL(String),
  }
  ```
- [ ] Add user-friendly error display:
  - [ ] Colored error prefix
  - [ ] Helpful context for each error type
  - [ ] Suggestions for fixing common issues
- [ ] Add progress indicators:
  - [ ] Use indicatif for spinners
  - [ ] "Fetching issues..." while loading
  - [ ] "Connecting to Linear..." for initial request
- [ ] Add `--verbose` flag globally:
  - [ ] Show HTTP requests/responses
  - [ ] Display timing information
  - [ ] Include GraphQL query details
- [ ] Implement retry logic:
  - [ ] Retry network errors (not auth errors)
  - [ ] Exponential backoff
  - [ ] Max 3 retries
  - [ ] Show retry attempts in verbose mode
- [ ] Add status command:
  ```rust
  /// Check connection to Linear
  Status {
      /// Show detailed connection info
      #[arg(long)]
      verbose: bool,
  }
  ```
- [ ] Create tests for error scenarios

## Example Error Display

```
Error: Authentication failed. Check your LINEAR_API_KEY

Get your API key from: https://linear.app/settings/api
```

## Technical Details

- Use thiserror for error definitions
- Use indicatif for progress spinners
- Implement exponential backoff with tokio

## Dependencies

- Depends on: #9 (Single Issue View)

## Definition of Done

- [ ] All errors have helpful messages
- [ ] Progress indicators show during operations
- [ ] Verbose mode provides debugging info
- [ ] Retry logic handles transient failures
- [ ] Status command validates connection
- [ ] Error colors respect NO_COLOR
