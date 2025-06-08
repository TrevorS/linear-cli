# Linear CLI Integration Plan - Search Functionality

**PR**: #89 - Implement comprehensive search functionality for Linear CLI
**Features**: Search command with query parsing, multi-type results, and JSON output
**Target Branch**: `feature/issue-13-search-functionality-clean`
**Issue**: #13 - Search Functionality
**Integration Date**: _TBD_
**Integrator**: @TrevorS

## 🎯 **Integration Overview**

This integration plan covers the deployment of comprehensive search functionality that allows users to search across Linear issues, documents, and projects with advanced query parsing capabilities.

### Key Features Being Integrated
- `linear search <query>` command with comprehensive CLI options
- Advanced query parsing (exact phrases, exclusions, field searches)
- Multi-type search results (Issues, Documents, Projects)
- Flexible output formats (table and JSON with pretty printing)
- Result grouping and limit controls
- Archive inclusion options

---

## 🏗️ **Phase 1: Pre-Integration Verification**

### Prerequisites
```bash
# Ensure we're on the correct branch and environment is ready
git checkout feature/issue-13-search-functionality-clean
git status  # Should be clean
source .env
echo "Testing with API key: ${LINEAR_API_KEY:0:20}..."
```

### Build and Test Verification
```bash
# Full development workflow
make dev-setup
make all

# Verify all tests pass (should be 187 total)
make test
```

**Expected**:
- Build succeeds without warnings
- All 187 tests pass (110 CLI + 76 SDK + 1 xtask)
- No clippy warnings

### Baseline CLI Verification
```bash
# Verify existing functionality still works
linear status --verbose
linear issues --limit 5
linear --help
```

**Expected**:
- Connection successful
- Issues list displayed correctly
- Help shows new search command

---

## 🔍 **Phase 2: Search Command Integration Testing**

### Basic Search Functionality
```bash
# Test basic search command availability
linear search --help

# Basic search tests
linear search "bug"
linear search "authentication"
linear search "login"
```

**Expected**:
- Help displays all search options
- Basic searches return grouped results (Issues/Documents sections)
- Results are formatted cleanly with identifiers and titles

### Query Parser Integration
```bash
# Test exact phrase parsing
linear search "\"login system\""
linear search "\"user authentication\" dashboard"

# Test exclusions
linear search "login -mobile"
linear search "authentication -mobile -tablet"

# Test field searches (parsed but not implemented yet)
linear search "assignee:john team:ENG"
linear search "status:todo priority:high"

# Test mixed queries
linear search "\"login bug\" assignee:john -mobile dashboard"
```

**Expected**:
- Exact phrases processed correctly (combined into search text)
- Exclusions parsed but basic text search performed
- Field searches parsed but basic text search performed
- Complex queries handled gracefully

---

## 📊 **Phase 3: Output Format Integration**

### JSON Output Testing
```bash
# Test JSON output formats
linear search "bug" --json
linear search "authentication" --json --pretty

# Test with different search types
linear search "api" --json --issues-only
linear search "documentation" --json --docs-only
```

**Expected**:
- Compact JSON output is valid and parseable
- Pretty JSON is formatted correctly
- Type-specific searches return appropriate result structure

### Table Output Testing
```bash
# Test grouped table output
linear search "dashboard"
linear search "authentication" --limit 5

# Test empty results
linear search "xyznonexistentquery123"
```

**Expected**:
- Results grouped by type (Issues:, Documents:)
- Clean formatting with identifiers and titles
- Appropriate "No results found" message for empty results

---

## ⚙️ **Phase 4: Advanced Options Integration**

### Limit and Archive Controls
```bash
# Test limit controls
linear search "bug" --limit 5
linear search "authentication" --limit 1
linear search "dashboard" --limit 100  # Max allowed

# Test archive inclusion
linear search "legacy" --include-archived
linear search "deprecated" --include-archived --json
```

**Expected**:
- Limit properly constrains results per type
- Archive inclusion works with both output formats
- Max limit validation (1-100) enforced

### Search Type Filtering
```bash
# Test type-specific searches
linear search "bug" --issues-only
linear search "documentation" --docs-only
linear search "roadmap" --projects-only  # Should work but return empty (not implemented)

# Test conflicting flags (should error)
linear search "test" --issues-only --docs-only  # Should be mutually exclusive
```

**Expected**:
- Type filters work correctly
- Projects search returns empty results gracefully
- Conflicting flags handled appropriately

---

## 🚀 **Phase 5: Performance and Error Integration**

### Performance Testing
```bash
# Test with various query complexities
time linear search "a"  # Broad search
time linear search "very specific unique query"  # Narrow search
time linear search "\"exact phrase match\"" --limit 50
```

**Expected**:
- Reasonable response times (< 5 seconds for most queries)
- No memory issues or crashes
- Spinner displays during search

### Error Handling Integration
```bash
# Test error scenarios
LINEAR_API_KEY="" linear search "test"  # Authentication error
linear search ""  # Empty query
linear search "test" --limit 0  # Invalid limit
linear search "test" --limit 101  # Over max limit
```

**Expected**:
- Clear error messages for authentication failures
- Proper validation of input parameters
- Helpful error text for invalid usage

---

## 🔄 **Phase 6: Integration with Existing Workflow**

### CLI Integration Verification
```bash
# Test with other global flags
linear search "bug" --no-color
linear search "authentication" --force-color
linear search "dashboard" --verbose

# Test help integration
linear --help | grep -i search
linear search --help
```

**Expected**:
- Global flags work correctly with search
- Search appears in main help
- Consistent behavior with other commands

### Regression Testing
```bash
# Verify existing commands still work
linear issues --limit 10
linear issue ENG-123  # Use actual issue ID
linear teams
linear projects

# Test overall CLI stability
linear status
make test  # Re-run all tests
```

**Expected**:
- No regressions in existing functionality
- All tests still pass
- CLI remains stable and responsive

---

## ✅ **Phase 7: Final Integration Checklist**

### Code Quality Verification
```bash
# Final code quality checks
make check  # Format and lint
cargo clippy --workspace --all-targets -- -D warnings
```

### Documentation Integration
- [ ] CLAUDE.md updated with search examples
- [ ] README.md updated with search usage (if applicable)
- [ ] Help text reviewed and accurate
- [ ] Example commands tested and verified

### Deployment Readiness
- [ ] All tests passing (187 total)
- [ ] No clippy warnings
- [ ] Performance acceptable
- [ ] Error handling robust
- [ ] Integration with existing CLI seamless

---

## 🎉 **Integration Success Criteria**

✅ **Must Have**:
- Basic search functionality works across issues and documents
- Query parsing handles complex queries gracefully
- JSON and table output formats work correctly
- CLI integration is seamless with existing commands
- All tests pass and no regressions introduced

✅ **Should Have**:
- Performance is reasonable (< 5 seconds for most queries)
- Error messages are clear and helpful
- Advanced options (limits, archives, type filters) work
- Output formatting is consistent with existing commands

✅ **Nice to Have**:
- Search feels fast and responsive
- Query parsing foundations ready for future enhancements
- Code quality maintains project standards

---

## 📝 **Integration Notes**

### Known Limitations (By Design)
- Field searches parsed but not implemented in GraphQL queries (future enhancement)
- Exclusions parsed but not implemented server-side (Linear API limitation)
- Projects search placeholder (waiting for Linear API support)
- Sequential searches rather than parallel (optimization opportunity)

### Future Enhancement Readiness
- Query parser supports advanced features for future implementation
- Result structure ready for additional search types
- CLI options prepared for feature expansion
- Test coverage established for iterative development

### Integration Dependencies
- Requires valid Linear API key for testing
- Depends on Linear API availability
- GraphQL schema must be current
- Test environment should have sample data for meaningful results

---

**Integration Plan Created**: 2025-06-08
**Ready for Integration**: ✅ All phases planned and prerequisites met
