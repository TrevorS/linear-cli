## Description

Add bulk operations for efficiency, allowing users to update multiple issues at once.

## Context

From the implementation plan (Prompt 16), we need to:
- Implement bulk update command
- Add preview mode for safety
- Show progress during bulk operations

## Acceptance Criteria

- [ ] Implement bulk update command:
  ```rust
  /// Update multiple issues at once
  Bulk {
      #[command(subcommand)]
      action: BulkAction,
  }
  
  #[derive(Subcommand)]
  enum BulkAction {
      /// Update issues matching filters
      Update {
          // Same filters as 'issues' command
          #[arg(long)]
          assignee: Option<String>,
          
          #[arg(long)]
          status: Option<String>,
          
          #[arg(long)]
          team: Option<String>,
          
          // Updates to apply
          #[arg(long)]
          set_status: Option<String>,
          
          #[arg(long)]
          set_assignee: Option<String>,
          
          /// Skip confirmation
          #[arg(long)]
          force: bool,
      },
  }
  ```
- [ ] Implement preview mode:
  ```
  Found 5 issues matching filters:
  
  ENG-123  Fix login race condition
  ENG-124  Implement OAuth flow
  ENG-125  Add user preferences
  ENG-126  Refactor auth module
  ENG-127  Update documentation
  
  Will update:
  - Status: Todo → In Progress
  
  Continue? [y/N]
  ```
- [ ] Add progress bar:
  - [ ] Show current issue being updated
  - [ ] Display success/failure count
  - [ ] Allow cancellation with Ctrl+C
- [ ] Implement transaction safety:
  - [ ] Collect all changes first
  - [ ] Apply in batch
  - [ ] Handle failures gracefully
- [ ] Add bulk close/reopen:
  ```bash
  linear bulk update --status done --set-status cancelled --force
  linear bulk update --assignee me --set-status "in progress"
  ```
- [ ] Safety features:
  - [ ] Max 50 issues without --force
  - [ ] Dry run by default for >10 issues
  - [ ] Clear summary of changes
- [ ] Add tests for bulk operations

## Example Usage

```bash
# Move all my todo items to in progress
linear bulk update --assignee me --status todo --set-status "in progress"

# Close all done issues from last sprint
linear bulk update --status done --team ENG --set-status closed
```

## Technical Details

- Use indicatif for progress bars
- Batch API calls for efficiency
- Handle partial failures gracefully

## Dependencies

- Depends on: #15 (Update Operations)

## Definition of Done

- [ ] Bulk updates work with filters
- [ ] Preview shows affected issues
- [ ] Progress bar shows during execution
- [ ] Cancellation works properly
- [ ] Safety limits prevent accidents
- [ ] Partial failures handled gracefully