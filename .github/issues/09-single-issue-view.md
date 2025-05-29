## Description

Implement detailed view for a single issue, showing all available information in a well-formatted display.

## Context

From the implementation plan (Prompt 9), we need to:
- Create detailed issue query
- Add issue command to show single issue
- Format detailed information nicely

## Acceptance Criteria

- [ ] Create detailed issue query:
  ```graphql
  query GetIssue($id: String!) {
    issue(id: $id) {
      id
      identifier
      title
      description
      state { name type }
      assignee { name email }
      team { key name }
      project { name }
      labels { nodes { name color } }
      priority
      priorityLabel
      createdAt
      updatedAt
      url
    }
  }
  ```
- [ ] Add issue command:
  ```rust
  /// Show details for a single issue
  Issue {
      /// Issue identifier (e.g., ENG-123)
      id: String,
      
      /// Output as JSON
      #[arg(long)]
      json: bool,
  }
  ```
- [ ] Create detailed view formatter:
  ```
  ─────────────────────────────────────────
  ENG-123: Fix login race condition
  ─────────────────────────────────────────
  Status:    In Progress
  Assignee:  John Doe (john@example.com)
  Team:      Engineering (ENG)
  Project:   Web App
  Priority:  High
  
  Description:
  Users are experiencing race conditions when logging in
  simultaneously from multiple devices.
  
  Labels: bug, authentication
  
  Created: 2024-01-15 10:30 AM
  Updated: 2024-01-16 2:45 PM
  
  View in Linear: https://linear.app/...
  ```
- [ ] Handle issue not found:
  - [ ] Clear error message
  - [ ] Suggest checking ID format
- [ ] Support both ID formats:
  - [ ] Issue key (ENG-123)
  - [ ] Issue ID (UUID)
- [ ] Add tests:
  - [ ] Found issue display
  - [ ] Not found error
  - [ ] JSON output format
  - [ ] Various field combinations

## Technical Details

- Format dates in human-readable format
- Handle missing optional fields gracefully
- Support both issue identifier and UUID lookup

## Dependencies

- Depends on: #8 (Query Filters)

## Definition of Done

- [ ] `linear issue ENG-123` shows detailed view
- [ ] Missing fields handled gracefully
- [ ] Dates formatted nicely
- [ ] JSON output available with --json
- [ ] Not found errors are helpful
- [ ] Both ID formats work