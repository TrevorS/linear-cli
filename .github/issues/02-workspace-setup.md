## Description

Create the production Linear CLI workspace with proper structure and real schema integration. This establishes the foundation for all subsequent development.

## Context

From the implementation plan (Prompt 2), we need to:
- Set up a Rust workspace structure
- Integrate the real Linear GraphQL schema
- Configure GraphQL code generation
- Create the basic project skeleton

## Acceptance Criteria

- [ ] Create Rust workspace with this structure:
  ```
  linear-cli/
  ├── Cargo.toml (workspace)
  ├── xtask/
  │   ├── Cargo.toml
  │   └── src/main.rs
  ├── linear-sdk/
  │   ├── Cargo.toml
  │   ├── build.rs
  │   ├── graphql/
  │   │   ├── schema.json
  │   │   └── queries/viewer.graphql
  │   └── src/lib.rs
  └── linear-cli/
      ├── Cargo.toml
      └── src/main.rs
  ```
- [ ] Configure workspace dependencies:
  - [ ] linear-sdk: tokio, reqwest, serde, graphql_client, anyhow
  - [ ] linear-cli: tokio, clap, anyhow, linear-sdk (path dependency)
- [ ] Set up xtask for schema management:
  - [ ] Command to download fresh schema from Linear
  - [ ] Save to `linear-sdk/graphql/schema.json`
- [ ] Configure GraphQL code generation:
  - [ ] build.rs that generates types from schema.json
  - [ ] Simple viewer.graphql query
- [ ] Project compiles successfully
- [ ] `cargo run -p linear-cli` prints "Linear CLI"

## Technical Details

```toml
# Workspace Cargo.toml
[workspace]
members = ["xtask", "linear-sdk", "linear-cli"]
resolver = "2"

# Feature flags
default = []
integration-tests = []
```

## Dependencies

- Depends on: #1 (API Validation Spike) - need schema.json

## Definition of Done

- [ ] All crates compile without errors
- [ ] GraphQL types are generated from the real schema
- [ ] Basic CLI executable runs
- [ ] .gitignore includes generated code
- [ ] schema.json is committed to the repository
