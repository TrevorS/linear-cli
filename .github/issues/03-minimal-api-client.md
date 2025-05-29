## Description

Implement the smallest possible working Linear CLI that actually queries the API. This proves our integration works end-to-end.

## Context

From the implementation plan (Prompt 3), we need to:
- Create a minimal LinearClient in the SDK
- Execute a real API query using generated types
- Display results in the CLI
- Handle authentication errors gracefully

## Acceptance Criteria

- [ ] Implement LinearClient in linear-sdk:
  ```rust
  pub struct LinearClient {
      client: reqwest::Client,
      api_key: String,
  }
  ```
- [ ] Add `execute_viewer_query()` method using generated GraphQL types
- [ ] Add integration test (behind feature flag):
  ```rust
  #[test]
  #[ignore] // Run with: cargo test -- --ignored
  fn test_real_api() { ... }
  ```
- [ ] Update linear-cli to:
  - [ ] Read API key from `LINEAR_API_KEY` env var
  - [ ] Execute viewer query
  - [ ] Print viewer name and email
  - [ ] Show helpful error if no API key:
    ```
    Error: No LINEAR_API_KEY environment variable found
    
    Please set your Linear API key:
    export LINEAR_API_KEY=lin_api_xxxxx
    
    Get your API key from: https://linear.app/settings/api
    ```
- [ ] Add logging with env_logger:
  - [ ] Log HTTP requests/responses when `RUST_LOG=debug`

## Technical Details

- Use anyhow for error handling
- Proper timeout configuration (30s default)
- User-Agent header: "linear-cli/0.1.0"

## Dependencies

- Depends on: #2 (Workspace Setup)

## Definition of Done

- [ ] `LINEAR_API_KEY=xxx cargo run -p linear-cli` shows user info
- [ ] Missing API key shows helpful error message
- [ ] Integration test passes with real API key
- [ ] HTTP debugging works with RUST_LOG=debug