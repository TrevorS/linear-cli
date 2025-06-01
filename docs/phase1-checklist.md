# Phase 1: Enhanced Protocol Support - Action Checklist

## Summary
This checklist covers the immediate high-priority tasks for Phase 1 of the inline image enhancements. Focus on iTerm2 protocol implementation and improved terminal detection.

## Current Branch
- Branch: `feature/inline-image-enhancements-41-followup`
- Base: `feature/inline-image-display-41` (PR #71)
- Target: Create follow-up PR pointing to PR #71

## Priority Tasks

### 1.1 iTerm2 Protocol Implementation

#### Core Implementation
- [ ] **Create iTerm2 protocol file**
  - File: `linear-cli/src/image_protocols/iterm2.rs`
  - Implement `ImageProtocol` trait
  - iTerm2 format: `\x1b]1337;File=name=filename;size=filesize;inline=1:base64data\x07`

- [ ] **Base64 encoding implementation**
  - Use standard base64 encoding (no chunking like Kitty)
  - Handle filename parameter properly
  - Add file size parameter

- [ ] **Image format support**
  - Support same formats as Kitty: PNG, JPEG, GIF, WebP
  - Use same image format detection logic
  - Handle format-specific optimizations

#### Integration
- [ ] **Update manager.rs**
  - Add iTerm2 to protocol routing in `get_protocol()`
  - Ensure proper fallback chain: Kitty → iTerm2 → Link
  - Test protocol selection logic

- [ ] **Update detection.rs**
  - Improve iTerm2 detection beyond just `TERM_PROGRAM=iTerm.app`
  - Add version detection if needed
  - Test detection across different iTerm2 versions

#### Testing
- [ ] **Unit tests for iTerm2 protocol**
  - Test base64 encoding
  - Test escape sequence generation
  - Test image format handling
  - Test error conditions

- [ ] **Integration tests**
  - Test iTerm2 protocol selection
  - Test fallback behavior
  - Test with mock HTTP responses

### 1.2 Enhanced Terminal Detection

#### Better Detection Logic
- [ ] **Expand terminal support**
  - Add more iTerm2-compatible terminals
  - Research Hyper, Warp, and other modern terminals
  - Add version-specific support where needed

- [ ] **Runtime protocol testing**
  - Implement test sequence sending (optional feature)
  - Add capability probing for uncertain terminals
  - Create fallback detection chain

- [ ] **User override support**
  - Add `LINEAR_CLI_FORCE_PROTOCOL` environment variable
  - Support values: `kitty`, `iterm2`, `sixel`, `none`
  - Document override options

#### Testing Improvements
- [ ] **Cross-terminal testing**
  - Test detection in iTerm2, kitty, WezTerm, Ghostty
  - Test edge cases and unknown terminals
  - Verify fallback behavior

### 1.3 Manager and Integration Updates

#### Protocol Management
- [ ] **Enhanced protocol routing**
  - Update `manager.rs` to handle iTerm2
  - Implement protocol preference logic
  - Add protocol override support

- [ ] **Error handling improvements**
  - Better error messages for protocol failures
  - Clearer fallback logging
  - Protocol-specific error codes

#### CLI Integration
- [ ] **Add protocol information to verbose output**
  - Show detected terminal and protocol
  - Display protocol selection reasoning
  - Add protocol override confirmation

## Testing Strategy

### Manual Testing Environments
- [ ] **iTerm2 (macOS)**
  - Latest stable version
  - Test with various image formats
  - Verify escape sequence rendering

- [ ] **Kitty (existing)**
  - Ensure no regression
  - Test protocol selection priority
  - Verify graceful fallback

- [ ] **Other terminals**
  - Terminal.app (macOS) - should fallback to links
  - WezTerm - should use Kitty protocol
  - Ghostty - should use Kitty protocol

### Automated Testing
- [ ] **Unit test coverage**
  - 100% coverage for new iTerm2 protocol
  - Test all protocol detection paths
  - Mock environment variable scenarios

- [ ] **Integration tests**
  - Test full image processing pipeline
  - Test protocol fallback chains
  - Test error scenarios

## Implementation Order

### Week 1: iTerm2 Protocol Core
1. Create `iterm2.rs` with basic structure
2. Implement `ImageProtocol` trait
3. Add base64 encoding and escape sequences
4. Write unit tests

### Week 2: Integration and Detection
1. Update `manager.rs` for protocol routing
2. Enhance `detection.rs` for better iTerm2 support
3. Add environment variable overrides
4. Integration testing

### Week 3: Testing and Polish
1. Cross-terminal testing
2. Error handling improvements
3. Documentation updates
4. Prepare for PR

## Success Criteria

- [ ] iTerm2 protocol displays images correctly in iTerm2
- [ ] Kitty protocol still works (no regression)
- [ ] Graceful fallback to links in unsupported terminals
- [ ] Protocol detection works reliably across tested terminals
- [ ] User can override protocol selection
- [ ] All tests pass with >90% coverage
- [ ] No breaking changes to existing CLI interface

## Files to Create/Modify

### New Files
- `linear-cli/src/image_protocols/iterm2.rs`

### Modified Files
- `linear-cli/src/image_protocols/manager.rs` (protocol routing)
- `linear-cli/src/image_protocols/detection.rs` (improved detection)
- `linear-cli/src/image_protocols/mod.rs` (exports)

### Test Files
- Update existing tests for new protocol
- Add iTerm2-specific test cases

## Notes

- Keep changes minimal and focused on iTerm2 support
- Maintain backward compatibility at all times
- Test thoroughly in actual terminals, not just unit tests
- Document any terminal-specific quirks discovered
- Consider adding iTerm2 protocol to feature documentation

## Next Steps After Phase 1

Once Phase 1 is complete:
1. Create PR against the branch containing PR #71
2. Test across multiple environments
3. Document new protocol support
4. Begin Phase 2 planning for advanced features
