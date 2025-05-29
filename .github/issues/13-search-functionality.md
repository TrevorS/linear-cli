## Description

Implement full-text search across Linear, allowing users to find issues, projects, and comments quickly.

## Context

From the implementation plan (Prompt 13), we need to:
- Add search GraphQL query
- Implement multi-type search
- Support search operators
- Display grouped results

## Acceptance Criteria

- [ ] Add search query:
  ```graphql
  query Search($query: String!) {
    searchIssues(query: $query) {
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
- [ ] Add search command:
  ```rust
  /// Search issues, projects, and comments
  Search {
      /// Search query
      query: String,
      
      /// Limit results per type
      #[arg(long, default_value = "10")]
      limit: i32,
      
      #[arg(long)]
      json: bool,
  }
  ```
- [ ] Implement multi-type search:
  - [ ] Search issues (primary)
  - [ ] Search projects
  - [ ] Search within comments
  - [ ] Execute queries in parallel
- [ ] Create search results formatter:
  ```
  Issues (5 results):
  ─────────────────────
  ENG-123  Fix login race condition     In Progress
  ENG-125  Login timeout issues          Todo
  
  Projects (1 result):
  ──────────────────
  Login System Refactor
  
  Comments (2 results):
  ──────────────────
  In ENG-120: "The login fix should address..."
  In ENG-118: "Login is working better now..."
  ```
- [ ] Support search operators:
  - [ ] Exact phrase: "exact match"
  - [ ] Exclude: -term
  - [ ] Field search: assignee:john
- [ ] Performance optimization:
  - [ ] Parallel queries for different types
  - [ ] Limit initial results
  - [ ] Provide "show more" option
- [ ] Add tests for search functionality

## Example Usage

```bash
linear search "login bug"
linear search "assignee:me -done"
linear search "\"race condition\""
```

## Technical Details

- Use tokio for parallel query execution
- Parse search operators before sending to API
- Group and sort results by relevance

## Dependencies

- Depends on: #12 (Additional Queries)

## Definition of Done

- [ ] Search finds results across all types
- [ ] Results grouped by type
- [ ] Search operators work correctly
- [ ] Parallel execution improves performance
- [ ] JSON output available
- [ ] Empty results handled gracefully