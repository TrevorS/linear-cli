# Output Improvements: Migrate Print Statements to Structured Logging

## Background

The Linear CLI codebase currently has **51 print statements** scattered across multiple files using manual approaches like `println!`, `eprintln!`, and environment variable checks. While some are appropriate, many should be migrated to structured logging and better CLI output patterns for improved maintainability and user experience.

## Current State Analysis

### Print Statement Distribution
- **11 error/diagnostic** (appropriate, keep as-is)
- **13 user-facing CLI output** (appropriate, keep as-is)
- **26 debug/development** (🔴 should migrate to `log` crate)
- **1 structured data output** (appropriate for build tool)

### Problematic Patterns Found
```rust
// ❌ Manual environment variable checks (26 instances)
if std::env::var("LINEAR_CLI_VERBOSE").is_ok() {
    eprintln!("Debug message: {}", value);
}

// ❌ Direct debug prints in production code
eprintln!("Creating image manager for issue processing...");
```

### Good Patterns Already in Use
- ✅ `env_logger::init()` already configured in main.rs
- ✅ Proper use of `eprintln!` for errors
- ✅ Proper use of `println!` for user output
- ✅ `indicatif` for progress bars

## Migration Strategy

### Phase 1: Replace Debug Print Statements with Structured Logging

**Pattern Migration:**
```rust
// Replace this:
if std::env::var("LINEAR_CLI_VERBOSE").is_ok() {
    eprintln!("Creating image manager for issue processing...");
}

// With this:
log::debug!("Creating image manager for issue processing");
```

**Log Level Guidelines:**
- `log::error!` - Actual errors that need attention
- `log::warn!` - Warnings, retry attempts, recoverable issues
- `log::info!` - Important status information
- `log::debug!` - Detailed debugging information (replaces verbose checks)
- `log::trace!` - Very detailed tracing information

### Phase 2: Enhanced CLI Output Management

**Create centralized CLI output utilities** in `output.rs`:
```rust
pub struct CliOutput {
    verbose: bool,
    use_color: bool,
    is_interactive: bool,
}

impl CliOutput {
    pub fn status(&self, message: &str) { /* formatted status */ }
    pub fn success(&self, message: &str) { /* ✓ with colors */ }
    pub fn warning(&self, message: &str) { /* ⚠ with colors */ }
    pub fn info(&self, message: &str) { /* ℹ with colors */ }
}
```

### Phase 3: Remove Manual Environment Variable Checks

**Remove all `LINEAR_CLI_VERBOSE` checks** since `env_logger` handles this via `RUST_LOG`:
```bash
# Instead of LINEAR_CLI_VERBOSE=1, users will use:
RUST_LOG=debug linear issues
RUST_LOG=linear_cli=debug linear issues  # specific to this crate
```

## Implementation Todo List

### Phase 1: Structured Logging Migration

#### High Priority Files (Most Debug Prints)
- [ ] **`linear-cli/src/image_protocols/manager.rs`** - Migrate 5 debug prints
  - [ ] Line 96: `eprintln!("Debug: ImageManager created with {} protocols", ...)`
  - [ ] Line 129: `eprintln!("Processing URL: {}", url)`
  - [ ] Line 140: `eprintln!("Image manager enabled: {}", ...)`
  - [ ] Line 176: `eprintln!("Image processing failed, falling back: {}", ...)`
  - [ ] Line 180: `eprintln!("Image rendered successfully: {} chars", ...)`

- [ ] **`linear-cli/src/output.rs`** - Migrate 8 debug prints
  - [ ] Line 498: `eprintln!("Processing issue with image manager...")`
  - [ ] Line 506: `eprintln!("Found description, processing for images...")`
  - [ ] Line 519: `eprintln!("Processed description contains {} characters", ...)`
  - [ ] Line 535: `eprintln!("Updated issue description with processed images")`
  - [ ] Line 540: `eprintln!("No description found in issue")`
  - [ ] Line 558: `eprintln!("Processing markdown for images...")`
  - [ ] Line 595: `eprintln!("Image rendered successfully: {} chars", ...)`
  - [ ] Line 607: `eprintln!("Replaced markdown pattern with Kitty sequence")`

- [ ] **`linear-cli/src/image_protocols/cache.rs`** - Migrate 7 debug prints
  - [ ] Line 53: Cache hit debug message
  - [ ] Line 63: Cache miss debug message
  - [ ] Line 93: Cache store debug message
  - [ ] Line 138: Cache directory creation debug
  - [ ] Line 152: Cache file write debug
  - [ ] Line 174: Cache cleanup debug
  - [ ] Line 179: Cache operation completion debug

#### Medium Priority Files
- [ ] **`linear-sdk/src/lib.rs`** - Migrate 4 debug prints
  - [ ] Line 215: GraphQL request debug
  - [ ] Line 217: GraphQL variables debug
  - [ ] Line 219: GraphQL response debug
  - [ ] Line 239: GraphQL error debug

- [ ] **`linear-cli/src/main.rs`** - Migrate 2 debug prints
  - [ ] Line 442: `eprintln!("Creating image manager for issue processing...")`
  - [ ] Line 469: `eprintln!("Image processing failed, falling back: {}", ...)`

- [ ] **`linear-sdk/src/retry.rs`** - Migrate 2 retry prints to log::warn!
  - [ ] Line 44: Retry attempt warning
  - [ ] Line 65: Max retries reached warning

- [ ] **`linear-cli/src/image_protocols/downloader.rs`** - Migrate 1 debug print
  - [ ] Line 87: Download warning debug message

### Phase 2: CLI Output Utilities

- [ ] **Create `CliOutput` struct in `output.rs`**
  - [ ] Add `CliOutput` struct with color/interactive state
  - [ ] Implement `status()` method for general status messages
  - [ ] Implement `success()` method with ✓ icon and green color
  - [ ] Implement `warning()` method with ⚠ icon and yellow color
  - [ ] Implement `info()` method with ℹ icon and blue color
  - [ ] Implement `error()` method with ✗ icon and red color

- [ ] **Integrate `CliOutput` in main.rs**
  - [ ] Replace OAuth status messages (lines 192, 196, 220, 222)
  - [ ] Replace connection status messages (lines 552, 554, 573)
  - [ ] Replace general status messages (lines 367, 388, 532)

- [ ] **Integrate `CliOutput` in oauth.rs**
  - [ ] Replace OAuth flow messages (lines 115, 178)

### Phase 3: Environment Variable Cleanup

- [ ] **Remove all `LINEAR_CLI_VERBOSE` checks**
  - [ ] Search and remove all `std::env::var("LINEAR_CLI_VERBOSE").is_ok()` patterns
  - [ ] Update any documentation referencing `LINEAR_CLI_VERBOSE`
  - [ ] Add documentation for `RUST_LOG` usage

- [ ] **Update documentation**
  - [ ] Update README.md with `RUST_LOG` examples
  - [ ] Update CLAUDE.md debugging section
  - [ ] Add examples: `RUST_LOG=debug linear issues`

### Phase 4: Enhanced Integration

- [ ] **Consider adding `indicatif-log-bridge`**
  - [ ] Add dependency to Cargo.toml
  - [ ] Integrate LogWrapper to prevent log/progress bar conflicts
  - [ ] Test progress bar + logging interaction

- [ ] **Testing and Validation**
  - [ ] Add tests for `CliOutput` utilities
  - [ ] Verify log output doesn't break existing functionality
  - [ ] Test different `RUST_LOG` levels
  - [ ] Validate color output in different terminals

## Expected Benefits

### 1. Better Debugging Experience
- Structured log levels instead of binary verbose mode
- Standard `RUST_LOG` environment variable usage
- Integration with logging tools and log analysis

### 2. Improved Code Quality
- Remove scattered environment variable checks
- Centralized output formatting logic
- Better separation of concerns

### 3. Enhanced User Experience
- Consistent CLI output formatting
- Better progress bar integration
- Professional appearance

### 4. Developer Experience
- Standard Rust logging patterns
- Easier testing and debugging
- Better integration with Rust tooling

## Implementation Notes

### Dependencies
Current logging setup is already sufficient:
- ✅ `env_logger` - already in use
- ✅ `log` crate - likely already available
- ✅ `indicatif` - already integrated for progress bars

### Usage Examples After Migration
```bash
# Enable debug logging for all crates
RUST_LOG=debug linear issues

# Enable debug logging only for linear-cli
RUST_LOG=linear_cli=debug linear issues

# Enable different levels for different modules
RUST_LOG=linear_cli=info,linear_sdk=debug linear issues

# Trace level for very detailed output
RUST_LOG=trace linear issue ENG-123
```

## Success Criteria

- [ ] Zero manual `LINEAR_CLI_VERBOSE` environment variable checks
- [ ] All debug output uses `log::debug!` or appropriate log levels
- [ ] Centralized `CliOutput` utility for user-facing messages
- [ ] Documentation updated to reflect `RUST_LOG` usage instead of `LINEAR_CLI_VERBOSE`
- [ ] Tests pass and logging doesn't interfere with output formatting
- [ ] Integration with `indicatif` progress bars remains smooth

---

**Estimated Effort:** 2-3 hours for Phase 1, 1-2 hours for Phase 2, 30 minutes for Phase 3
**Risk Level:** Low - mostly mechanical changes with well-established patterns
**Impact:** High - significantly improves debugging experience and code quality
