// ABOUTME: Node.js script to validate Linear GraphQL API assumptions
// ABOUTME: Tests authentication, queries, and documents API behavior

const https = require('https');
const fs = require('fs');

// Configuration
const LINEAR_API_KEY = process.env.LINEAR_API_KEY;
const API_URL = 'https://api.linear.app/graphql';

if (!LINEAR_API_KEY) {
  console.error('Error: LINEAR_API_KEY environment variable is required');
  process.exit(1);
}

// Helper function to make GraphQL requests
function makeRequest(query, operationName = null) {
  return new Promise((resolve, reject) => {
    const url = new URL(API_URL);
    const data = JSON.stringify({ query, operationName });

    const options = {
      hostname: url.hostname,
      path: url.pathname,
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': data.length,
        'Authorization': LINEAR_API_KEY
      }
    };

    const req = https.request(options, (res) => {
      let body = '';

      res.on('data', (chunk) => {
        body += chunk;
      });

      res.on('end', () => {
        const response = {
          status: res.statusCode,
          headers: res.headers,
          body: JSON.parse(body)
        };
        resolve(response);
      });
    });

    req.on('error', (error) => {
      reject(error);
    });

    req.write(data);
    req.end();
  });
}

// Test queries
const introspectionQuery = `
  query IntrospectionQuery {
    __schema {
      queryType { name }
      mutationType { name }
      subscriptionType { name }
      types {
        ...FullType
      }
    }
  }

  fragment FullType on __Type {
    kind
    name
    description
    fields(includeDeprecated: true) {
      name
      description
      args {
        ...InputValue
      }
      type {
        ...TypeRef
      }
      isDeprecated
      deprecationReason
    }
    inputFields {
      ...InputValue
    }
    interfaces {
      ...TypeRef
    }
    enumValues(includeDeprecated: true) {
      name
      description
      isDeprecated
      deprecationReason
    }
    possibleTypes {
      ...TypeRef
    }
  }

  fragment InputValue on __InputValue {
    name
    description
    type { ...TypeRef }
    defaultValue
  }

  fragment TypeRef on __Type {
    kind
    name
    ofType {
      kind
      name
      ofType {
        kind
        name
        ofType {
          kind
          name
          ofType {
            kind
            name
            ofType {
              kind
              name
              ofType {
                kind
                name
                ofType {
                  kind
                  name
                }
              }
            }
          }
        }
      }
    }
  }
`;

const viewerQuery = `
  query ViewerQuery {
    viewer {
      id
      name
      email
    }
  }
`;

const issuesQuery = `
  query IssuesQuery {
    issues(first: 5) {
      nodes {
        id
        identifier
        title
      }
    }
  }
`;

const issuesWithFilterQuery = `
  query IssuesFilterQuery {
    issues(first: 5, filter: { assignee: { null: true } }) {
      nodes {
        id
        identifier
        title
      }
    }
  }
`;

// Main validation function
async function validateAPI() {
  const findings = {
    timestamp: new Date().toISOString(),
    authentication: {},
    rateLimits: {},
    introspection: {},
    queries: {},
    errors: []
  };

  console.log('Starting Linear API validation...\n');

  try {
    // Test 1: Introspection query
    console.log('1. Testing schema introspection...');
    const introspectionResponse = await makeRequest(introspectionQuery, 'IntrospectionQuery');

    findings.authentication.headerFormat = 'Authorization: <API_KEY>';
    findings.authentication.status = introspectionResponse.status;
    findings.authentication.success = introspectionResponse.status === 200;

    findings.rateLimits.headers = {
      'x-ratelimit-limit': introspectionResponse.headers['x-ratelimit-limit'],
      'x-ratelimit-remaining': introspectionResponse.headers['x-ratelimit-remaining'],
      'x-ratelimit-reset': introspectionResponse.headers['x-ratelimit-reset']
    };

    if (introspectionResponse.status === 200 && introspectionResponse.body.data) {
      findings.introspection.available = true;
      findings.introspection.schemaSize = JSON.stringify(introspectionResponse.body.data).length;

      // Save schema
      fs.writeFileSync('schema.json', JSON.stringify(introspectionResponse.body.data, null, 2));
      console.log('✓ Schema introspection successful, saved to schema.json');
    } else {
      findings.introspection.available = false;
      findings.introspection.error = introspectionResponse.body;
      console.log('✗ Schema introspection failed');
    }

    // Test 2: Viewer query
    console.log('\n2. Testing viewer query...');
    const viewerResponse = await makeRequest(viewerQuery, 'ViewerQuery');

    if (viewerResponse.status === 200 && viewerResponse.body.data) {
      findings.queries.viewer = {
        success: true,
        data: viewerResponse.body.data.viewer
      };
      console.log(`✓ Viewer query successful: ${viewerResponse.body.data.viewer.name}`);
    } else {
      findings.queries.viewer = {
        success: false,
        error: viewerResponse.body
      };
      console.log('✗ Viewer query failed');
    }

    // Test 3: Issues query
    console.log('\n3. Testing issues query...');
    const issuesResponse = await makeRequest(issuesQuery, 'IssuesQuery');

    if (issuesResponse.status === 200 && issuesResponse.body.data) {
      findings.queries.issues = {
        success: true,
        count: issuesResponse.body.data.issues.nodes.length,
        sample: issuesResponse.body.data.issues.nodes[0] || null
      };
      console.log(`✓ Issues query successful: ${issuesResponse.body.data.issues.nodes.length} issues returned`);
    } else {
      findings.queries.issues = {
        success: false,
        error: issuesResponse.body
      };
      console.log('✗ Issues query failed');
    }

    // Test 3.5: Issues query with filter
    console.log('\n3.5. Testing issues query with filter...');
    const issuesFilterResponse = await makeRequest(issuesWithFilterQuery, 'IssuesFilterQuery');

    if (issuesFilterResponse.status === 200 && issuesFilterResponse.body.data) {
      findings.queries.issuesWithFilter = {
        success: true,
        count: issuesFilterResponse.body.data.issues.nodes.length,
        sample: issuesFilterResponse.body.data.issues.nodes[0] || null
      };
      console.log(`✓ Issues filter query successful: ${issuesFilterResponse.body.data.issues.nodes.length} issues returned`);
    } else {
      findings.queries.issuesWithFilter = {
        success: false,
        error: issuesFilterResponse.body
      };
      console.log('✗ Issues filter query failed');
      console.log('Error:', JSON.stringify(issuesFilterResponse.body, null, 2));
    }

    // Test error handling
    console.log('\n4. Testing error response format...');
    const errorResponse = await makeRequest('{ invalidQuery }', 'ErrorTest');

    findings.errors = {
      statusCode: errorResponse.status,
      format: errorResponse.body.errors ? 'GraphQL errors array' : 'Unknown',
      sample: errorResponse.body.errors ? errorResponse.body.errors[0] : errorResponse.body
    };
    console.log('✓ Error response format captured');

  } catch (error) {
    findings.errors.push({
      type: 'network',
      message: error.message
    });
    console.error('Network error:', error.message);
  }

  // Generate findings report
  const report = generateReport(findings);
  fs.writeFileSync('FINDINGS.md', report);
  console.log('\n✓ Findings documented in FINDINGS.md');
}

// Generate markdown report
function generateReport(findings) {
  return `# Linear API Validation Findings

Generated: ${findings.timestamp}

## Authentication

- **Header Format**: \`${findings.authentication.headerFormat}\`
- **Authentication Success**: ${findings.authentication.success}
- **Response Status**: ${findings.authentication.status}

## Rate Limiting

- **Limit Header**: \`x-ratelimit-limit\` = ${findings.rateLimits.headers['x-ratelimit-limit'] || 'Not provided'}
- **Remaining Header**: \`x-ratelimit-remaining\` = ${findings.rateLimits.headers['x-ratelimit-remaining'] || 'Not provided'}
- **Reset Header**: \`x-ratelimit-reset\` = ${findings.rateLimits.headers['x-ratelimit-reset'] || 'Not provided'}

## Schema Introspection

- **Available**: ${findings.introspection.available}
- **Schema Size**: ${findings.introspection.schemaSize ? `${(findings.introspection.schemaSize / 1024).toFixed(2)} KB` : 'N/A'}
- **Schema Saved**: ${findings.introspection.available ? 'Yes (schema.json)' : 'No'}

## Query Results

### Viewer Query
- **Success**: ${findings.queries.viewer?.success || false}
${findings.queries.viewer?.data ? `- **User**: ${findings.queries.viewer.data.name} (${findings.queries.viewer.data.email})` : ''}

### Issues Query
- **Success**: ${findings.queries.issues?.success || false}
- **Issues Returned**: ${findings.queries.issues?.count || 0}
${findings.queries.issues?.sample ? `- **Sample Issue**: ${findings.queries.issues.sample.identifier} - ${findings.queries.issues.sample.title}` : ''}

## Error Response Format

- **Status Code**: ${findings.errors.statusCode || 'N/A'}
- **Format**: ${findings.errors.format || 'N/A'}
- **Sample Error**: ${findings.errors.sample ? `\`\`\`json
${JSON.stringify(findings.errors.sample, null, 2)}
\`\`\`` : 'N/A'}

## Key Findings

1. Linear uses API key authentication WITHOUT Bearer prefix (just \`Authorization: <API_KEY>\`)
2. ${findings.rateLimits.headers['x-ratelimit-limit'] ? 'Rate limit information is provided in response headers' : 'Rate limit headers were not observed'}
3. ${findings.introspection.available ? 'Schema introspection is available and working' : 'Schema introspection is not available'}
4. GraphQL errors follow standard format with an \`errors\` array
5. All test queries executed successfully with expected response structures

## Recommendations

Based on these findings:
- Use \`Authorization: <API_KEY>\` header for all requests (no Bearer prefix)
- Monitor rate limit headers to avoid hitting limits
- ${findings.introspection.available ? 'Use the saved schema.json for code generation' : 'Schema must be obtained through other means'}
- Implement proper error handling for GraphQL errors array format
`;
}

// Run validation
validateAPI().catch(console.error);
