# Linear CLI Create Command - Consolidation Implementation Guide

## Overview

This document provides a comprehensive guide for implementing the consolidated `linear create` command by combining the functionality from PRs #77, #79, #80, #82, and #83 into a single, cohesive implementation.

## Executive Summary

**Goal**: Replace the current placeholder create command with a full-featured issue creation system that includes:
- Core GraphQL-based issue creation with validation
- Intelligent interactive prompts with environment detection
- Team and user resolution with human-friendly names
- Smart defaults and templates system
- Performance optimizations and comprehensive error handling

**Current State**: Placeholder command that only displays provided arguments
**Target State**: 142 tests, 95%+ coverage, production-ready issue creation

## Implementation Phases

### Phase 1: Core Issue Creation Foundation
**Estimated Time**: 2-3 hours
**Complexity**: Medium
**Base**: Current main branch

#### 1.1 GraphQL Infrastructure
- [ ] Create `linear-sdk/graphql/mutations/create_issue.graphql`
- [ ] Add `CreateIssue` struct to SDK with build.rs integration
- [ ] Implement `create_issue()` method in LinearClient
- [ ] Add new types: `CreateIssueInput`, `CreatedIssue`

#### 1.2 CLI Implementation
- [ ] Replace placeholder create command handler in main.rs (lines 1021-1064)
- [ ] Add argument validation and processing
- [ ] Implement "me" assignee resolution for create context
- [ ] Add priority validation (1-4 scale)
- [ ] Add dry-run mode support
- [ ] Add browser opening with `--open` flag

#### 1.3 Basic Testing
- [ ] Core creation workflow tests
- [ ] Argument validation tests
- [ ] GraphQL mutation error handling tests
- [ ] "me" assignee resolution tests

### Phase 2: Interactive Prompts System
**Estimated Time**: 3-4 hours
**Complexity**: High
**Dependencies**: Phase 1 complete

#### 2.1 Interactive Framework
- [ ] Create `linear-cli/src/interactive.rs` module
- [ ] Add dependencies: `dialoguer` or `inquire` for prompts
- [ ] Implement `InteractivePrompter` struct with TTY detection
- [ ] Add CI environment detection logic
- [ ] Implement field collection for missing arguments

#### 2.2 Environment Detection
- [ ] Smart TTY detection (terminal vs pipe/redirect)
- [ ] CI environment detection (GitHub Actions, Jenkins, etc.)
- [ ] Override system for testing (`with_tty_override`, `with_ci_override`)
- [ ] Graceful fallback when prompts unavailable

#### 2.3 Interactive UX
- [ ] Team selection with arrow-key navigation
- [ ] Priority selection interface
- [ ] Assignee options: "me", "unassigned", custom input
- [ ] Input validation and retry logic

#### 2.4 Testing & CI Fix
- [ ] Implement `ci_override` functionality to fix failing tests
- [ ] Interactive prompt tests with environment simulation
- [ ] TTY detection accuracy tests
- [ ] CI environment isolation tests

### Phase 3: Team & User Resolution
**Estimated Time**: 2-3 hours
**Complexity**: Medium
**Dependencies**: Phase 2 complete

#### 3.1 Team Resolution System
- [ ] Create `linear-sdk/graphql/queries/teams.graphql`
- [ ] Add `ListTeams` struct and integration
- [ ] Implement team key → UUID mapping (e.g., "ENG" → team-uuid)
- [ ] Add case-insensitive team matching
- [ ] Team suggestion system for invalid keys

#### 3.2 User Search & Resolution
- [ ] Create `linear-sdk/graphql/queries/users.graphql`
- [ ] Add `SearchUsers` struct and integration
- [ ] Implement user search by name, email, username
- [ ] Team-based user filtering for assignee suggestions
- [ ] Enhanced assignee validation

#### 3.3 SDK Extensions
- [ ] Add `list_teams()` method to LinearClient
- [ ] Add `search_users()` method to LinearClient
- [ ] Team caching logic with error handling
- [ ] User search result formatting

#### 3.4 Enhanced Error Handling
- [ ] Team not found errors with suggestions
- [ ] User not found errors with search hints
- [ ] Network error handling for team/user operations
- [ ] Validation error messages with context

### Phase 4: Smart Defaults & Templates
**Estimated Time**: 2-3 hours
**Complexity**: Medium
**Dependencies**: Phase 3 complete

#### 4.1 Preferences System
- [ ] Create `linear-cli/src/preferences.rs` module
- [ ] User settings persistence (last-used values)
- [ ] Preferences file location and format
- [ ] Settings validation and migration

#### 4.2 Template System
- [ ] Create `linear-cli/src/templates.rs` module
- [ ] Issue template definitions (bug, feature, epic)
- [ ] Template selection interface
- [ ] Template variable substitution

#### 4.3 Smart Defaults
- [ ] Git branch context detection for auto-titling
- [ ] Usage pattern analysis for intelligent defaults
- [ ] Last-used value suggestions in interactive mode
- [ ] Context-aware field pre-population

#### 4.4 Integration
- [ ] Integrate preferences with interactive prompts
- [ ] Template selection in create workflow
- [ ] Smart default application logic
- [ ] Settings management commands (future consideration)

### Phase 5: Performance & Advanced Validation
**Estimated Time**: 2-3 hours
**Complexity**: Medium
**Dependencies**: Phase 4 complete

#### 5.1 Performance Optimizations
- [ ] Team/user data caching with 5-minute TTL
- [ ] Async I/O optimizations for network calls
- [ ] GraphQL query optimization for minimal data transfer
- [ ] Cache invalidation and error handling

#### 5.2 Advanced Validation
- [ ] Fuzzy matching for team names with intelligent suggestions
- [ ] Email format detection and validation for assignees
- [ ] Enhanced user existence validation
- [ ] Input sanitization and security considerations

#### 5.3 Enhanced Error Handling
- [ ] Contextual error messages for each failure type
- [ ] Retry logic with exponential backoff
- [ ] Network timeout handling
- [ ] Rate limiting response handling

#### 5.4 Final Polish
- [ ] Comprehensive error message review
- [ ] Performance testing with large datasets
- [ ] Memory usage optimization
- [ ] Final validation of all features

## Implementation Strategy

### 1. Branch Setup
```bash
# Create consolidated feature branch
git checkout master
git pull origin master
git checkout -b feature/consolidated-create-command

# Set up tracking
git push -u origin feature/consolidated-create-command
```

### 2. Development Approach
- **Iterative implementation**: Complete each phase fully before moving to next
- **Test-driven development**: Write tests for each component as it's built
- **Incremental validation**: Test functionality after each major component
- **CI feedback loop**: Fix any CI issues immediately

### 3. Testing Strategy
- **Unit tests**: Each module and function tested independently
- **Integration tests**: End-to-end workflows with mocked responses
- **Interactive tests**: TTY and environment detection validation
- **Snapshot tests**: CLI output format verification
- **CI environment tests**: Proper isolation and override testing

## File Structure

### New Files to Create
```
linear-cli/src/
├── interactive.rs          # Interactive prompts with TTY detection
├── preferences.rs          # User settings persistence
└── templates.rs           # Issue template system

linear-sdk/src/
├── teams.rs               # Team resolution logic (optional module)
└── users.rs              # User search functionality (optional module)

linear-sdk/graphql/
├── mutations/
│   └── create_issue.graphql   # Issue creation mutation
└── queries/
    ├── teams.graphql          # Team listing/search
    └── users.graphql          # User search queries
```

### Files to Modify
```
linear-cli/src/main.rs         # Replace create command placeholder
linear-cli/Cargo.toml          # Add interactive prompt dependencies
linear-sdk/src/lib.rs          # Add new client methods and exports
linear-sdk/src/types.rs        # Add creation-related types
linear-sdk/Cargo.toml          # Any new dependencies needed
linear-sdk/build.rs            # Include new GraphQL files
```

## Dependencies to Add

### CLI Dependencies (linear-cli/Cargo.toml)
```toml
# For interactive prompts
dialoguer = "0.11"
console = "0.15"

# For preferences storage
dirs = "5.0"
```

### SDK Dependencies (linear-sdk/Cargo.toml)
```toml
# For caching
cached = "0.46"

# For fuzzy matching (Phase 5)
fuzzy-matcher = "0.3"
```

## Testing Requirements

### Test Categories
1. **Unit Tests** (per module)
   - Pure function testing
   - Input validation
   - Error condition handling

2. **Integration Tests** (end-to-end workflows)
   - Complete create workflows
   - Mocked API responses
   - Error scenario testing

3. **Interactive Tests** (environment simulation)
   - TTY detection accuracy
   - CI environment handling
   - Prompt behavior validation

4. **Snapshot Tests** (output verification)
   - CLI output formatting
   - Error message consistency
   - JSON output validation

### CI Test Fix Requirements
Every interactive test must use both overrides:
```rust
let prompter = InteractivePrompter::new(&client)
    .with_tty_override(true)      // Simulate terminal
    .with_ci_override(false);     // Simulate non-CI environment
```

## Quality Assurance

### Pre-Implementation Checklist
- [ ] Current functionality fully understood
- [ ] All PR branches analyzed for reusable code
- [ ] Test failures root cause identified
- [ ] Implementation plan validated

### During Implementation
- [ ] Each phase tested independently
- [ ] CI passing after each major component
- [ ] Code review of complex components
- [ ] Performance validation on large datasets

### Pre-Merge Checklist
- [ ] All 142+ tests passing
- [ ] Manual testing of complete workflows
- [ ] Interactive prompts working in various environments
- [ ] Backward compatibility verified
- [ ] Documentation updated
- [ ] Performance acceptable

## Risk Mitigation

### Technical Risks
- **Complex TTY detection**: Use proven libraries, comprehensive testing
- **GraphQL mutation errors**: Robust error handling, retry logic
- **Performance issues**: Caching strategy, async I/O
- **CI test failures**: Proper environment isolation

### Implementation Risks
- **Scope creep**: Stick to defined phases, resist feature additions
- **Over-engineering**: Focus on core functionality first
- **Test complexity**: Start with simple tests, build up coverage
- **Integration issues**: Test each phase thoroughly before proceeding

## Success Metrics

### Functionality Metrics
- [ ] All core create features working
- [ ] Interactive mode working in appropriate environments
- [ ] Team/user resolution working with real Linear data
- [ ] Performance acceptable (< 2s for typical operations)

### Quality Metrics
- [ ] 142+ tests with 95%+ coverage
- [ ] All CI environments passing
- [ ] No breaking changes to existing functionality
- [ ] Clean code review feedback

### User Experience Metrics
- [ ] Intuitive CLI interface
- [ ] Helpful error messages
- [ ] Fast response times
- [ ] Graceful fallback behaviors

## Post-Implementation

### PR Creation
- Use the consolidated PR description from the design phase
- Include comprehensive test plan results
- Highlight backward compatibility preservation
- Document any breaking changes (should be none)

### Follow-up Tasks
- Monitor usage patterns for template system improvements
- Gather feedback on interactive UX
- Performance monitoring in production
- Iterate on smart defaults based on usage data

---

## Quick Reference

### Key Commands During Implementation
```bash
# Development workflow
make dev                    # Quick check (fmt, lint, test)
make test-snapshots        # Test with snapshot review
make run-debug             # Test with debug logging

# Testing specific features
cargo test create          # Test create-related functionality
cargo test interactive     # Test interactive components
cargo test --test integration  # Integration test suite

# Performance validation
cargo test --release       # Test with optimizations
make run -- create --help  # Manual CLI testing
```

### Critical Implementation Notes
1. **Always implement CI override in interactive components**
2. **Test TTY detection thoroughly in various environments**
3. **Use mocked responses for reliable integration tests**
4. **Cache team/user data but handle cache failures gracefully**
5. **Provide meaningful error messages with actionable guidance**

This guide provides the roadmap for implementing the consolidated create command. Each phase builds on the previous one, ensuring a systematic approach to delivering the complete functionality.
