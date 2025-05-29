// ABOUTME: Test helper utilities for mocking Linear API responses and server
// ABOUTME: Provides mockito-based helpers for unit testing API interactions

#[cfg(test)]
use mockito::{Server, ServerGuard};
#[cfg(test)]
use serde_json::json;

#[cfg(test)]
pub async fn mock_linear_server() -> ServerGuard {
    Server::new_async().await
}

#[cfg(test)]
pub fn mock_viewer_response() -> serde_json::Value {
    json!({
        "data": {
            "viewer": {
                "id": "test-user-id",
                "name": "Test User",
                "email": "test@example.com"
            }
        }
    })
}

#[cfg(test)]
pub fn mock_issues_response() -> serde_json::Value {
    json!({
        "data": {
            "issues": {
                "nodes": [
                    {
                        "id": "issue-1",
                        "title": "Test Issue 1",
                        "identifier": "TEST-1",
                        "description": "Test description 1"
                    },
                    {
                        "id": "issue-2",
                        "title": "Test Issue 2",
                        "identifier": "TEST-2",
                        "description": "Test description 2"
                    }
                ]
            }
        }
    })
}

#[cfg(test)]
pub fn mock_error_response() -> serde_json::Value {
    json!({
        "errors": [
            {
                "message": "Authentication required",
                "extensions": {
                    "code": "UNAUTHENTICATED"
                }
            }
        ]
    })
}

#[cfg(test)]
pub fn mock_graphql_error_response() -> serde_json::Value {
    json!({
        "errors": [
            {
                "message": "Cannot query field 'unknown' on type 'Query'",
                "locations": [
                    {
                        "line": 2,
                        "column": 3
                    }
                ],
                "extensions": {
                    "code": "GRAPHQL_VALIDATION_FAILED"
                }
            }
        ]
    })
}
