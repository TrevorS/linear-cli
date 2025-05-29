## Description

Expand the CLI with more read operations including projects, teams, comments, and a unified "my work" view.

## Context

From the implementation plan (Prompt 12), we need to:
- Add queries for projects, teams, and comments
- Create appropriate formatters for each type
- Implement a "my work" command combining multiple queries

## Acceptance Criteria

- [ ] Add new GraphQL queries:
  ```graphql
  query ListProjects {
    projects {
      nodes {
        id
        name
        description
        state
        issueCount
      }
    }
  }
  
  query ListTeams {
    teams {
      nodes {
        id
        key
        name
        memberCount
      }
    }
  }
  
  query GetComments($issueId: String!) {
    issue(id: $issueId) {
      comments {
        nodes {
          id
          body
          user { name }
          createdAt
        }
      }
    }
  }
  ```
- [ ] Add commands:
  ```rust
  /// List all projects
  Projects {
      #[arg(long)]
      json: bool,
  },
  
  /// List all teams
  Teams {
      #[arg(long)]
      json: bool,
  },
  
  /// Show comments on an issue
  Comments {
      /// Issue identifier
      issue_id: String,
      
      #[arg(long)]
      json: bool,
  },
  ```
- [ ] Create formatters:
  - [ ] Projects: Table with name, state, issue count
  - [ ] Teams: Table with key, name, member count
  - [ ] Comments: Threaded view with timestamp and author
- [ ] Add caching for teams/projects:
  - [ ] Cache for session duration
  - [ ] Use for validation in future commands
- [ ] Implement "my-work" command:
  - [ ] Show assigned issues
  - [ ] Show created issues
  - [ ] Show recently commented issues
  - [ ] Unified view of current work
- [ ] Add tests for each new command

## Example Output

```
Teams:
 Key   Name          Members
 ─────────────────────────────
 ENG   Engineering   12
 PROD  Product       5
 
Comments on ENG-123:
────────────────────
John Doe - 2024-01-15 10:30 AM
This seems to be a race condition in the auth flow.

Jane Smith - 2024-01-15 11:45 AM
I can reproduce this consistently. Working on a fix.
```

## Dependencies

- Depends on: #11 (OAuth Authentication)

## Definition of Done

- [ ] All new commands work with table and JSON output
- [ ] Comments display in readable format
- [ ] Teams/projects cached appropriately
- [ ] My-work provides useful overview
- [ ] Tests cover all new functionality