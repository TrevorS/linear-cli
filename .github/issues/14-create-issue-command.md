## Description

Begin Phase 3 with the first write operation: creating new issues through the CLI.

## Context

From the implementation plan (Prompt 14), we need to:
- Add create issue mutation
- Support both command-line args and interactive mode
- Validate inputs before submission

## Acceptance Criteria

- [ ] Add create mutation:
  ```graphql
  mutation CreateIssue($input: IssueCreateInput!) {
    issueCreate(input: $input) {
      success
      issue {
        id
        identifier
        url
      }
    }
  }
  ```
- [ ] Add create command:
  ```rust
  /// Create a new issue
  Create {
      /// Issue title
      #[arg(short, long)]
      title: Option<String>,
      
      /// Team key (e.g., ENG)
      #[arg(short = 'T', long)]
      team: Option<String>,
      
      /// Description
      #[arg(short, long)]
      description: Option<String>,
      
      /// Assignee email or "me"
      #[arg(short, long)]
      assignee: Option<String>,
      
      /// Priority (urgent, high, medium, low)
      #[arg(short, long)]
      priority: Option<String>,
      
      /// Open in browser after creation
      #[arg(long)]
      open: bool,
  }
  ```
- [ ] Add interactive mode with dialoguer:
  - [ ] If no args provided, enter interactive mode
  - [ ] Fetch and show team list for selection
  - [ ] Fetch team members for assignee selection
  - [ ] Multi-line description editor
- [ ] Implement validation:
  - [ ] Team exists (use cached data)
  - [ ] Assignee is in selected team
  - [ ] Required fields are present
- [ ] Show success result:
  ```
  ✓ Created issue ENG-126
  
  Title: Implement user settings
  URL: https://linear.app/company/issue/ENG-126
  
  Opening in browser...
  ```
- [ ] Add dry-run mode:
  - [ ] `--dry-run` flag
  - [ ] Shows what would be created
  - [ ] Validates without submitting
- [ ] Add tests:
  - [ ] Successful creation
  - [ ] Validation failures
  - [ ] Interactive mode flow

## Example Usage

```bash
# With arguments
linear create -t "Fix bug" -T ENG -a me -p high

# Interactive mode
linear create
```

## Technical Details

- Use dialoguer for interactive prompts
- Cache team/member data for validation
- Use `open` crate to launch browser

## Dependencies

- Depends on: #13 (Search Functionality)

## Definition of Done

- [ ] Issues can be created with CLI args
- [ ] Interactive mode guides through creation
- [ ] Validation prevents invalid submissions
- [ ] Success shows issue URL
- [ ] Browser opens when requested
- [ ] Dry-run mode works correctly