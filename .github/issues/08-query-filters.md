## Description

Add filtering capabilities to the issues command, allowing users to query specific subsets of issues.

## Context

From the implementation plan (Prompt 8), we need to:
- Extend GraphQL query to accept filters
- Add filter CLI arguments
- Implement special filter values like "me" and "unassigned"

## Acceptance Criteria

- [ ] Extend GraphQL query:
  ```graphql
  query ListIssues($first: Int!, $filter: IssueFilter) {
    issues(first: $first, filter: $filter) {
      nodes {
        id
        identifier
        title
        state { name }
        assignee { id name }
        team { key }
      }
    }
  }
  ```
- [ ] Add filter arguments:
  ```rust
  Issues {
      /// Filter by assignee (use "me" for yourself)
      #[arg(long)]
      assignee: Option<String>,

      /// Filter by status (case insensitive)
      #[arg(long)]
      status: Option<String>,

      /// Filter by team
      #[arg(long)]
      team: Option<String>,
  }
  ```
- [ ] Implement special assignee values:
  - [ ] "me": Fetch viewer.id and filter by it
  - [ ] "unassigned": Filter for null assignee
  - [ ] Regular names: Filter by assignee name
- [ ] Add status normalization:
  - [ ] "todo" → "Todo"
  - [ ] "in progress" → "In Progress"
  - [ ] "done" → "Done"
  - [ ] Show error for unknown statuses
- [ ] Build GraphQL filter object dynamically
- [ ] Cache "me" lookup for performance
- [ ] Add tests:
  - [ ] Each filter individually
  - [ ] Combined filters (AND logic)
  - [ ] Special values
  - [ ] Case variations
  - [ ] Error cases

## Example Usage

```bash
linear issues --assignee me --status "in progress"
linear issues --team ENG --status todo
linear issues --assignee unassigned
```

## Technical Details

- Use Linear's IssueFilter GraphQL input type
- Cache viewer query result to avoid repeated lookups
- Validate filter values before sending query

## Dependencies

- Depends on: #7 (JSON Output)

## Definition of Done

- [ ] All filter types work correctly
- [ ] "me" resolves to current user
- [ ] Status names are case-insensitive
- [ ] Combined filters use AND logic
- [ ] Invalid filters show helpful errors
- [ ] Performance is acceptable with caching
