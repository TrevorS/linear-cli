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
                        "state": {
                            "name": "Todo"
                        },
                        "assignee": {
                            "id": "user-1",
                            "name": "Alice"
                        },
                        "team": {
                            "key": "ENG"
                        }
                    },
                    {
                        "id": "issue-2",
                        "title": "Test Issue 2",
                        "identifier": "TEST-2",
                        "state": {
                            "name": "In Progress"
                        },
                        "assignee": {
                            "id": "user-2",
                            "name": "Bob"
                        },
                        "team": {
                            "key": "DESIGN"
                        }
                    },
                    {
                        "id": "issue-3",
                        "title": "Test Issue 3",
                        "identifier": "TEST-3",
                        "state": {
                            "name": "Done"
                        },
                        "assignee": null,
                        "team": {
                            "key": "QA"
                        }
                    }
                ]
            }
        }
    })
}

#[cfg(test)]
pub fn mock_empty_issues_response() -> serde_json::Value {
    json!({
        "data": {
            "issues": {
                "nodes": []
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

#[cfg(test)]
pub fn mock_detailed_issue_response() -> serde_json::Value {
    json!({
        "data": {
            "issue": {
                "id": "issue-detailed-123",
                "identifier": "ENG-123",
                "title": "Fix login race condition",
                "description": "Users are experiencing race conditions when logging in simultaneously from multiple devices.",
                "state": {
                    "name": "In Progress",
                    "type": "started"
                },
                "assignee": {
                    "name": "John Doe",
                    "email": "john@example.com"
                },
                "team": {
                    "key": "ENG",
                    "name": "Engineering"
                },
                "project": {
                    "name": "Web App"
                },
                "labels": {
                    "nodes": [
                        {
                            "name": "bug",
                            "color": "#ff0000"
                        },
                        {
                            "name": "authentication",
                            "color": "#00ff00"
                        }
                    ]
                },
                "priority": 2.0,
                "priorityLabel": "High",
                "createdAt": "2024-01-15T10:30:00Z",
                "updatedAt": "2024-01-16T14:45:00Z",
                "url": "https://linear.app/test/issue/ENG-123"
            }
        }
    })
}

#[cfg(test)]
pub fn mock_minimal_issue_response() -> serde_json::Value {
    json!({
        "data": {
            "issue": {
                "id": "issue-minimal-456",
                "identifier": "ENG-456",
                "title": "Simple issue",
                "description": null,
                "state": {
                    "name": "Todo",
                    "type": "unstarted"
                },
                "assignee": null,
                "team": {
                    "key": "ENG",
                    "name": "Engineering"
                },
                "project": null,
                "labels": {
                    "nodes": []
                },
                "priority": 0.0,
                "priorityLabel": "None",
                "createdAt": "2024-01-01T00:00:00Z",
                "updatedAt": "2024-01-01T00:00:00Z",
                "url": "https://linear.app/test/issue/ENG-456"
            }
        }
    })
}

#[cfg(test)]
pub fn mock_issue_not_found_response() -> serde_json::Value {
    json!({
        "data": {
            "issue": null
        }
    })
}

#[cfg(test)]
pub fn mock_create_issue_success_response() -> serde_json::Value {
    json!({
        "data": {
            "issueCreate": {
                "success": true,
                "issue": {
                    "id": "created-issue-123",
                    "identifier": "ENG-456",
                    "title": "Test Created Issue",
                    "description": "Test description for created issue",
                    "state": {
                        "id": "state-123",
                        "name": "Todo",
                        "type": "unstarted"
                    },
                    "assignee": {
                        "id": "user-456",
                        "name": "Test User",
                        "email": "test@example.com"
                    },
                    "team": {
                        "id": "team-123",
                        "key": "ENG",
                        "name": "Engineering"
                    },
                    "labels": {
                        "nodes": [
                            {
                                "id": "label-789",
                                "name": "bug",
                                "color": "#ff0000"
                            }
                        ]
                    },
                    "priority": 2.0,
                    "priorityLabel": "High",
                    "createdAt": "2024-01-15T10:30:00Z",
                    "updatedAt": "2024-01-15T10:30:00Z",
                    "url": "https://linear.app/test/issue/ENG-456"
                },
                "lastSyncId": 123456
            }
        }
    })
}

#[cfg(test)]
pub fn mock_create_issue_minimal_success_response() -> serde_json::Value {
    json!({
        "data": {
            "issueCreate": {
                "success": true,
                "issue": {
                    "id": "created-issue-minimal-789",
                    "identifier": "ENG-789",
                    "title": "Minimal Issue",
                    "description": null,
                    "state": {
                        "id": "state-123",
                        "name": "Todo",
                        "type": "unstarted"
                    },
                    "assignee": null,
                    "team": {
                        "id": "team-123",
                        "key": "ENG",
                        "name": "Engineering"
                    },
                    "labels": {
                        "nodes": []
                    },
                    "priority": 0.0,
                    "priorityLabel": "None",
                    "createdAt": "2024-01-15T10:30:00Z",
                    "updatedAt": "2024-01-15T10:30:00Z",
                    "url": "https://linear.app/test/issue/ENG-789"
                },
                "lastSyncId": 123456
            }
        }
    })
}

#[cfg(test)]
pub fn mock_create_issue_failure_response() -> serde_json::Value {
    json!({
        "data": {
            "issueCreate": {
                "success": false,
                "issue": null,
                "lastSyncId": 123456
            }
        }
    })
}

#[cfg(test)]
pub fn mock_create_issue_validation_error_response() -> serde_json::Value {
    json!({
        "errors": [
            {
                "message": "Title is required",
                "extensions": {
                    "code": "VALIDATION_ERROR",
                    "field": "title"
                }
            }
        ]
    })
}

#[cfg(test)]
pub fn mock_update_issue_success_response() -> serde_json::Value {
    json!({
        "data": {
            "issueUpdate": {
                "success": true,
                "issue": {
                    "id": "updated-issue-123",
                    "identifier": "ENG-123",
                    "title": "Updated Issue Title",
                    "description": "Updated issue description",
                    "state": {
                        "id": "state-456",
                        "name": "In Progress",
                        "type": "started"
                    },
                    "assignee": {
                        "id": "user-789",
                        "name": "Jane Doe",
                        "email": "jane@example.com"
                    },
                    "team": {
                        "id": "team-123",
                        "key": "ENG",
                        "name": "Engineering"
                    },
                    "labels": {
                        "nodes": [
                            {
                                "id": "label-456",
                                "name": "enhancement",
                                "color": "#00ff00"
                            }
                        ]
                    },
                    "priority": 3.0,
                    "priorityLabel": "Normal",
                    "createdAt": "2024-01-15T10:30:00Z",
                    "updatedAt": "2024-01-16T16:00:00Z",
                    "url": "https://linear.app/test/issue/ENG-123"
                },
                "lastSyncId": 654321
            }
        }
    })
}

#[cfg(test)]
pub fn mock_update_issue_failure_response() -> serde_json::Value {
    json!({
        "data": {
            "issueUpdate": {
                "success": false,
                "issue": null,
                "lastSyncId": 654321
            }
        }
    })
}

#[cfg(test)]
pub fn mock_update_issue_not_found_response() -> serde_json::Value {
    json!({
        "errors": [
            {
                "message": "Issue not found",
                "extensions": {
                    "code": "NOT_FOUND",
                    "field": "id"
                }
            }
        ]
    })
}

#[cfg(test)]
pub fn mock_create_comment_success_response() -> serde_json::Value {
    json!({
        "data": {
            "commentCreate": {
                "success": true,
                "comment": {
                    "id": "comment-123",
                    "body": "This is a test comment",
                    "user": {
                        "id": "user-456",
                        "name": "Test User",
                        "email": "test@example.com"
                    },
                    "issue": {
                        "id": "issue-789",
                        "identifier": "ENG-123",
                        "title": "Test Issue"
                    },
                    "createdAt": "2024-01-16T16:30:00Z",
                    "updatedAt": "2024-01-16T16:30:00Z"
                },
                "lastSyncId": 789456
            }
        }
    })
}

#[cfg(test)]
pub fn mock_create_comment_failure_response() -> serde_json::Value {
    json!({
        "data": {
            "commentCreate": {
                "success": false,
                "comment": null,
                "lastSyncId": 789456
            }
        }
    })
}
