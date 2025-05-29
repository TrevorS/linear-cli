## Description

Set up continuous integration and testing infrastructure before adding more features. This ensures code quality from the start.

## Context

From the implementation plan (Prompt 4), we need:
- GitHub Actions CI pipeline
- Test helpers for mocking API responses
- Automated code quality checks

## Acceptance Criteria

- [ ] Create `.github/workflows/ci.yml`:
  - [ ] Runs on: push, pull_request
  - [ ] Matrix: macOS-latest (primary), ubuntu-latest
  - [ ] Caches: cargo registry, cargo index, target/
  - [ ] Steps:
    - [ ] `cargo fmt -- --check`
    - [ ] `cargo clippy -- -D warnings`
    - [ ] `cargo test`
    - [ ] `cargo test --features integration-tests` (using secrets.LINEAR_API_KEY)
- [ ] Add test helpers to linear-sdk:
  ```rust
  #[cfg(test)]
  pub mod test_helpers {
      pub fn mock_linear_server() -> mockito::ServerGuard { }
      pub fn mock_viewer_response() -> serde_json::Value { }
  }
  ```
- [ ] Create mockito-based unit tests:
  - [ ] Test successful API calls
  - [ ] Test authentication errors (401)
  - [ ] Test GraphQL errors
  - [ ] Test network timeouts
- [ ] Create `fixtures/` directory with sample responses:
  - [ ] viewer_response.json
  - [ ] issues_response.json
  - [ ] error_response.json
- [ ] Add Makefile for common commands:
  ```makefile
  test:
      cargo test

  test-integration:
      cargo test --features integration-tests -- --ignored

  fmt:
      cargo fmt --all

  lint:
      cargo clippy -- -D warnings
  ```
- [ ] Configure Dependabot for dependency updates

## Technical Details

- Use mockito for HTTP mocking
- Store realistic fixture data from actual API responses
- Ensure CI can run without Linear API access (except integration tests)

## Dependencies

- Depends on: #3 (Minimal API Client)

## Definition of Done

- [ ] CI pipeline passes on GitHub
- [ ] Unit tests run without network access
- [ ] Integration tests run with API key from secrets
- [ ] All quality checks (fmt, clippy) pass
- [ ] Dependabot is configured
