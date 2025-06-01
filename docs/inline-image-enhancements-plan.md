# Inline Image Enhancements Plan - Issue #41 Follow-up

## Overview

This plan outlines the remaining work to complete issue #41 based on PR #71. The core functionality (Kitty protocol, downloading, caching, security) is already implemented. This follow-up focuses on enhanced protocol support and advanced features.

## Current State (PR #71 ✅)

**Core Features Complete:**
- ✅ Kitty Graphics Protocol implementation (`kitty.rs`)
- ✅ Image detection and URL validation (`url_validator.rs`)
- ✅ Secure HTTP downloading with timeouts (`downloader.rs`)
- ✅ File-based caching system (`cache.rs`)
- ✅ Terminal capability detection (`detection.rs`)
- ✅ Graceful fallback to clickable links
- ✅ CLI flags (`--force-images`, `--no-images`)
- ✅ Security (domain allowlist, size limits)
- ✅ Comprehensive test coverage (84 tests)

**Architecture Summary:**
- Modular `image_protocols/` subsystem with trait-based design
- Async image processing pipeline with caching
- TTY detection for automatic enabling
- Feature flag (`inline-images`) for optional compilation

## Phase 1: Enhanced Protocol Support 🎯

### 1.1 iTerm2 Protocol Implementation
**Goal:** Support iTerm2 terminals for broader compatibility

**Tasks:**
- [ ] Create `image_protocols/iterm2.rs` implementing `ImageProtocol` trait
- [ ] Add iTerm2 base64 encoding with escape sequences
- [ ] Update terminal detection in `detection.rs` for better iTerm2 support
- [ ] Add iTerm2-specific tests
- [ ] Update manager to route to iTerm2 protocol when appropriate

**iTerm2 Protocol Format:**
```
\x1b]1337;File=name=filename;size=filesize;inline=1:base64data\x07
```

**Files to modify:**
- New: `linear-cli/src/image_protocols/iterm2.rs`
- Update: `linear-cli/src/image_protocols/manager.rs` (protocol routing)
- Update: `linear-cli/src/image_protocols/detection.rs` (better detection)

### 1.2 Sixel Graphics Support
**Goal:** Support xterm and other legacy terminals

**Tasks:**
- [ ] Research Sixel format implementation requirements
- [ ] Create `image_protocols/sixel.rs` implementing `ImageProtocol` trait
- [ ] Add Sixel encoding (may require image processing library)
- [ ] Update detection for Sixel-capable terminals
- [ ] Add Sixel-specific tests

**Considerations:**
- Sixel is more complex than base64 protocols
- May require image-to-sixel conversion library
- Limited color depth compared to modern protocols

### 1.3 Improved Terminal Detection
**Goal:** More robust capability detection

**Tasks:**
- [ ] Add runtime protocol testing (send test sequences)
- [ ] Improve version detection for partial support (e.g., Konsole)
- [ ] Add fallback protocol ordering (try Kitty → iTerm2 → Sixel → link)
- [ ] Add user override environment variables
- [ ] Test detection across more terminal emulators

## Phase 2: Advanced Image Features 🚀

### 2.1 Image Scaling and Resizing
**Goal:** Automatically fit images to terminal dimensions

**Tasks:**
- [ ] Add terminal size detection using termion/crossterm
- [ ] Implement image metadata parsing (dimensions without full decode)
- [ ] Add image resizing using image processing library
- [ ] Create scaling strategies (fit-width, fit-height, maintain-aspect)
- [ ] Add configuration options for scaling behavior

**Files to create/modify:**
- New: `image_protocols/scaling.rs`
- Update: Protocol implementations to use scaled images

### 2.2 Format Conversion
**Goal:** Support more image formats by converting to supported ones

**Tasks:**
- [ ] Add image format detection and validation
- [ ] Implement format conversion (TIFF→PNG, BMP→JPEG, etc.)
- [ ] Add WebP support for terminals that support it
- [ ] Handle format-specific optimizations
- [ ] Add conversion error handling

### 2.3 Animation Support
**Goal:** Display GIF and WebP animations where supported

**Tasks:**
- [ ] Research animation support in Kitty protocol
- [ ] Implement frame extraction for animated images
- [ ] Add animation controls (play once, loop, stop)
- [ ] Handle animation fallback (show first frame)
- [ ] Test animation performance and memory usage

## Phase 3: Enhanced Caching and Performance 🏎️

### 3.1 Smart Cache Management
**Goal:** More intelligent caching with size and age limits

**Tasks:**
- [ ] Implement LRU (Least Recently Used) cache eviction
- [ ] Add cache size monitoring and cleanup
- [ ] Implement cache statistics and reporting
- [ ] Add cache warming for frequently accessed images
- [ ] Create cache maintenance commands

**Files to modify:**
- Update: `image_protocols/cache.rs` (major enhancements)
- New: Cache management CLI commands

### 3.2 Performance Optimizations
**Goal:** Faster image processing and display

**Tasks:**
- [ ] Implement concurrent image downloads for multiple images
- [ ] Add image preloading based on markdown parsing
- [ ] Optimize base64 encoding performance
- [ ] Add memory usage monitoring
- [ ] Profile and optimize protocol-specific rendering

### 3.3 Progressive Loading
**Goal:** Show images as they download

**Tasks:**
- [ ] Implement chunked download with progress indication
- [ ] Add progressive JPEG support
- [ ] Show loading indicators for slow downloads
- [ ] Handle partial image display gracefully

## Phase 4: Configuration and User Experience 🎨

### 4.1 Enhanced Configuration
**Goal:** More granular control over image behavior

**Tasks:**
- [ ] Add configuration file support (`.linear-cli.toml`)
- [ ] Implement per-domain image policies
- [ ] Add image quality/compression settings
- [ ] Create size limit per-protocol configuration
- [ ] Add bandwidth-aware settings

### 4.2 Improved CLI Interface
**Goal:** Better user control and feedback

**Tasks:**
- [ ] Add `linear images` subcommand for image management
- [ ] Implement cache inspection and clearing commands
- [ ] Add verbose image processing information
- [ ] Create image capability testing command
- [ ] Add image download progress bars

### 4.3 Error Handling and Diagnostics
**Goal:** Better debugging and error reporting

**Tasks:**
- [ ] Enhanced error messages with resolution hints
- [ ] Add image processing diagnostics mode
- [ ] Implement fallback chain logging
- [ ] Create troubleshooting guide integration
- [ ] Add network connectivity testing

## Phase 5: Testing and Documentation 📚

### 5.1 Comprehensive Testing
**Goal:** Ensure reliability across all features

**Tasks:**
- [ ] Add protocol-specific integration tests
- [ ] Create image format compatibility test suite
- [ ] Add performance benchmarks
- [ ] Test error scenarios and edge cases
- [ ] Add terminal-specific test automation

### 5.2 Documentation Updates
**Goal:** Complete user and developer documentation

**Tasks:**
- [ ] Update README with image features
- [ ] Create image protocol troubleshooting guide
- [ ] Add configuration examples
- [ ] Document terminal compatibility matrix
- [ ] Create developer guide for protocol extensions

## Implementation Priority

### High Priority (Phase 1)
1. **iTerm2 Protocol** - Significant user base, relatively straightforward
2. **Enhanced Terminal Detection** - Improves reliability of existing features
3. **Basic Image Scaling** - High impact on user experience

### Medium Priority (Phase 2)
1. **Format Conversion** - Expands supported image types
2. **Smart Cache Management** - Performance and storage efficiency
3. **Enhanced Configuration** - User control and customization

### Lower Priority (Phase 3)
1. **Sixel Support** - Smaller user base, more complex implementation
2. **Animation Support** - Nice-to-have feature, significant complexity
3. **Progressive Loading** - Performance enhancement for edge cases

## Technical Considerations

### Dependencies
- **Image Processing**: Consider `image` crate for resizing/conversion
- **Terminal Detection**: May need `termion` or `crossterm` for dimensions
- **Configuration**: Use `serde` and `toml` for config files
- **Performance**: Consider `rayon` for parallel processing

### Backward Compatibility
- All enhancements must maintain feature flag compatibility
- Graceful degradation for unsupported features
- No breaking changes to existing CLI interface

### Security
- Maintain existing domain validation and size limits
- Add validation for new image formats
- Ensure safe handling of malformed images

## Success Metrics

- [ ] Support for 3+ terminal protocols (Kitty, iTerm2, Sixel)
- [ ] Image display works in 90%+ of tested terminals
- [ ] Sub-second image loading for typical Linear images
- [ ] Zero breaking changes to existing functionality
- [ ] Comprehensive test coverage (>90%)
- [ ] Complete user documentation

## Estimated Timeline

- **Phase 1 (Enhanced Protocols)**: 2-3 weeks
- **Phase 2 (Advanced Features)**: 3-4 weeks
- **Phase 3 (Performance)**: 2-3 weeks
- **Phase 4 (UX/Config)**: 1-2 weeks
- **Phase 5 (Testing/Docs)**: 1-2 weeks

**Total Estimated Time**: 9-14 weeks for complete implementation

## Notes

- Each phase builds on the previous one
- Phases can be partially parallelized
- Each phase should result in a working, releasable feature set
- Regular testing across different terminals is essential
- User feedback should guide priority adjustments
