## Description

Complete write operations with update functionality including status changes, assignments, and comments.

## Context

From the implementation plan (Prompt 15), we need to:
- Add update mutation
- Create update, close, reopen, and comment commands
- Show before/after for updates

## Acceptance Criteria

- [ ] Add update mutation:
  ```graphql
  mutation UpdateIssue($id: String!, $input: IssueUpdateInput!) {
    issueUpdate(id: $id, input: $input) {
      success
      issue {
        id
        identifier
        state { name }
      }
    }
  }
  ```
- [ ] Add update command:
  ```rust
  /// Update an issue
  Update {
      /// Issue identifier
      id: String,

      /// New status
      #[arg(long)]
      status: Option<String>,

      /// New assignee
      #[arg(long)]
      assignee: Option<String>,

      /// New priority
      #[arg(long)]
      priority: Option<String>,

      /// New title
      #[arg(long)]
      title: Option<String>,
  }
  ```
- [ ] Add convenience commands:
  ```rust
  /// Close an issue
  Close {
      /// Issue identifier
      id: String,
  },

  /// Reopen an issue
  Reopen {
      /// Issue identifier
      id: String,
  },
  ```
- [ ] Add comment command:
  ```rust
  /// Add a comment to an issue
  Comment {
      /// Issue identifier
      id: String,

      /// Comment text (or read from stdin)
      message: Option<String>,
  }
  ```
- [ ] Support stdin for comments:
  ```bash
  echo "Fixed in PR #123" | linear comment ENG-126
  ```
- [ ] Show before/after for updates:
  ```
  Updating ENG-126:
  Status: In Progress → Done
  Assignee: John Doe → Jane Smith

  Confirm? [y/N]
  ```
- [ ] Add tests for all update scenarios

## Example Usage

```bash
linear update ENG-123 --status done --assignee "jane@example.com"
linear close ENG-123
linear comment ENG-123 "This has been resolved"
```

## Technical Details

- Fetch current state before updates
- Show diff of changes
- Support confirmation prompts
- Handle stdin for comment input

## Dependencies

- Depends on: #14 (Create Issue Command)

## Definition of Done

- [ ] All update operations work correctly
- [ ] Before/after diff shows clearly
- [ ] Confirmation required for changes
- [ ] Close/reopen shortcuts work
- [ ] Comments can be added
- [ ] Stdin input works for comments
