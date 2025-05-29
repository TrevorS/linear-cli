## Description

Create a throwaway spike to validate Linear's GraphQL API assumptions before building any production code. This is a critical de-risking step to ensure our understanding of the API is correct.

## Context

From the implementation plan (Prompt 1), we need to validate:
- Authentication header format
- Rate limiting behavior
- Schema introspection availability
- Error response formats
- Basic query functionality

## Acceptance Criteria

- [ ] Create a `linear-api-spike` directory with a Node.js script
- [ ] Successfully authenticate using a real Linear API key (from `LINEAR_API_KEY` env var)
- [ ] Execute and validate these queries:
  - [ ] Schema introspection query
  - [ ] Simple viewer query: `{ viewer { id name email } }`
  - [ ] Issues query: `{ issues(first: 5) { nodes { id identifier title } } }`
- [ ] Document findings in `FINDINGS.md`:
  - [ ] Exact authentication header format
  - [ ] Rate limit headers returned
  - [ ] Any unexpected response structures
  - [ ] Schema introspection availability
  - [ ] Error response formats
- [ ] Save `schema.json` for use in the production project

## Technical Details

```javascript
// Example structure for the spike
const LINEAR_API_KEY = process.env.LINEAR_API_KEY;
const API_URL = 'https://api.linear.app/graphql';

// Test authentication and basic queries
// Document all findings
```

## Definition of Done

- [ ] All three test queries execute successfully
- [ ] FINDINGS.md contains all required documentation
- [ ] schema.json is saved and valid
- [ ] Code is throwaway (not production quality)

## Notes

This is exploratory code - focus on learning, not code quality. The findings will inform all subsequent development.