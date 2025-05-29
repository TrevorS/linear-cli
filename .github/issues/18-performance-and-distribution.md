## Description

Optimize performance and prepare for distribution including binary releases and package managers.

## Context

From the implementation plan (Prompt 18), we need to:
- Optimize binary size and performance
- Create release automation
- Prepare for Homebrew distribution

## Acceptance Criteria

- [ ] Performance optimizations:
  - [ ] Lazy load large dependencies
  - [ ] Use `once_cell` for caching
  - [ ] Profile with cargo-flamegraph
  - [ ] Minimize allocations in hot paths
- [ ] Binary size optimization:
  ```toml
  [profile.release]
  opt-level = "z"
  lto = true
  codegen-units = 1
  strip = true
  ```
- [ ] Build-time feature detection:
  - [ ] Detect if schema.json exists
  - [ ] Provide helpful error if missing
  - [ ] Support offline builds
- [ ] Create release workflow:
  ```yaml
  name: Release
  on:
    push:
      tags: ['v*']
  
  jobs:
    release:
      # Build for macOS (Intel and Apple Silicon)
      # Create GitHub release
      # Upload binaries
  ```
- [ ] Add Homebrew formula template:
  ```ruby
  class LinearCli < Formula
    desc "Fast command-line client for Linear"
    homepage "https://github.com/user/linear-cli"
    # Auto-updated by release process
  end
  ```
- [ ] Create man page generation:
  - [ ] Use clap_mangen
  - [ ] Include in releases
  - [ ] Install with Homebrew
- [ ] Final documentation:
  - [ ] README with GIFs/screenshots
  - [ ] CHANGELOG.md
  - [ ] LICENSE file
  - [ ] Security policy
  - [ ] Contributing guidelines
- [ ] Performance benchmarks:
  - [ ] Startup time < 50ms
  - [ ] Issues list < 200ms
  - [ ] JSON parsing benchmark

## Technical Details

- Use GitHub Actions for automated releases
- Support multiple architectures (x86_64, aarch64)
- Sign macOS binaries if possible

## Dependencies

- Depends on: #17 (Config and Completions)

## Definition of Done

- [ ] Release builds are optimized for size
- [ ] CI automatically creates releases
- [ ] Binaries work on target platforms
- [ ] Homebrew formula ready
- [ ] Documentation complete
- [ ] Performance meets targets
- [ ] Man pages generate correctly