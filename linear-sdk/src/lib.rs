// ABOUTME: Linear SDK library providing type-safe GraphQL client for Linear API
// ABOUTME: Includes authentication, queries, mutations, and generated types

use anyhow::Result;
use graphql_client::{GraphQLQuery, Response};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use std::time::Duration;

// Custom scalar types used by Linear's GraphQL schema
type DateTimeOrDuration = String;
type TimelessDateOrDuration = String;

#[cfg(test)]
pub mod test_helpers;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/viewer.graphql",
    response_derives = "Debug"
)]
pub struct Viewer;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/issues.graphql",
    response_derives = "Debug, Clone"
)]
pub struct ListIssues;

pub use viewer::ResponseData as ViewerResponseData;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub id: String,
    pub identifier: String,
    pub title: String,
    pub status: String,
    pub assignee: Option<String>,
    pub assignee_id: Option<String>,
    pub team: Option<String>,
}

pub struct LinearClient {
    client: reqwest::Client,
    base_url: String,
    _api_key: String,
}

pub struct IssueFilters {
    pub assignee: Option<String>,
    pub status: Option<String>,
    pub team: Option<String>,
}

impl LinearClient {
    async fn build_issue_filter(
        &self,
        _filters: &IssueFilters,
    ) -> Result<Option<list_issues::IssueFilter>> {
        // For now, let's disable filtering until we can implement it properly
        // This allows us to test the basic CLI structure
        Ok(None)
    }

    #[allow(dead_code)]
    fn normalize_status(status: &str) -> String {
        match status.to_lowercase().as_str() {
            "todo" => "Todo".to_string(),
            "in progress" | "inprogress" | "in_progress" => "In Progress".to_string(),
            "done" => "Done".to_string(),
            _ => {
                // Return the title case version for unknown statuses
                let mut result = String::new();
                let mut capitalize_next = true;
                for c in status.chars() {
                    if c.is_whitespace() {
                        result.push(c);
                        capitalize_next = true;
                    } else if capitalize_next {
                        result.push(c.to_uppercase().next().unwrap_or(c));
                        capitalize_next = false;
                    } else {
                        result.push(c.to_lowercase().next().unwrap_or(c));
                    }
                }
                result
            }
        }
    }
    pub fn new(api_key: String) -> Result<Self> {
        Self::with_base_url(api_key, "https://api.linear.app".to_string())
    }

    pub fn with_base_url(api_key: String, base_url: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&api_key)?);
        headers.insert(USER_AGENT, HeaderValue::from_static("linear-cli/0.1.0"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            base_url,
            _api_key: api_key,
        })
    }

    pub async fn execute_viewer_query(&self) -> Result<viewer::ResponseData> {
        let request_body = Viewer::build_query(viewer::Variables {});

        let response = self
            .client
            .post(format!("{}/graphql", self.base_url))
            .json(&request_body)
            .send()
            .await?;

        let response_body: Response<viewer::ResponseData> = response.json().await?;

        if let Some(errors) = response_body.errors {
            return Err(anyhow::anyhow!("GraphQL errors: {:?}", errors));
        }

        response_body
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in response"))
    }

    pub async fn list_issues(&self, limit: i32) -> Result<Vec<Issue>> {
        self.list_issues_with_filter(limit, None).await
    }

    pub async fn list_issues_filtered(
        &self,
        limit: i32,
        filters: Option<IssueFilters>,
    ) -> Result<Vec<Issue>> {
        let graphql_filter = if let Some(filters) = filters {
            self.build_issue_filter(&filters).await?
        } else {
            None
        };

        self.list_issues_with_filter(limit, graphql_filter).await
    }

    pub async fn list_issues_with_filter(
        &self,
        limit: i32,
        filter: Option<list_issues::IssueFilter>,
    ) -> Result<Vec<Issue>> {
        let request_body = ListIssues::build_query(list_issues::Variables {
            first: limit as i64,
            filter,
        });

        let response = self
            .client
            .post(format!("{}/graphql", self.base_url))
            .json(&request_body)
            .send()
            .await?;

        let response_body: Response<list_issues::ResponseData> = response.json().await?;

        if let Some(errors) = response_body.errors {
            return Err(anyhow::anyhow!("GraphQL errors: {:?}", errors));
        }

        let data = response_body
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in response"))?;

        let issues = data
            .issues
            .nodes
            .into_iter()
            .map(|issue| Issue {
                id: issue.id,
                identifier: issue.identifier,
                title: issue.title,
                status: issue.state.name,
                assignee: issue.assignee.as_ref().map(|a| a.name.clone()),
                assignee_id: issue.assignee.map(|a| a.id),
                team: Some(issue.team.key),
            })
            .collect();

        Ok(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn test_linear_client_creation() {
        let client = LinearClient::new("test_api_key".to_string());
        assert!(client.is_ok());
    }

    #[test]
    fn test_viewer_query_builds() {
        let _query = Viewer::build_query(viewer::Variables {});
    }

    #[tokio::test]
    async fn test_successful_api_call() {
        let mut server = mock_linear_server().await;
        let mock = server
            .mock("POST", "/graphql")
            .match_header("authorization", "test_api_key")
            .match_header("user-agent", "linear-cli/0.1.0")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_viewer_response().to_string())
            .create();

        let client = LinearClient::with_base_url("test_api_key".to_string(), server.url()).unwrap();
        let result = client.execute_viewer_query().await;

        mock.assert();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.viewer.id, "test-user-id");
        assert_eq!(data.viewer.name, "Test User");
        assert_eq!(data.viewer.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_authentication_error() {
        let mut server = mock_linear_server().await;
        let mock = server
            .mock("POST", "/graphql")
            .match_header("authorization", "invalid_key")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(mock_error_response().to_string())
            .create();

        let client = LinearClient::with_base_url("invalid_key".to_string(), server.url()).unwrap();
        let result = client.execute_viewer_query().await;

        mock.assert();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("GraphQL errors"));
    }

    #[tokio::test]
    async fn test_graphql_errors() {
        let mut server = mock_linear_server().await;
        let mock = server
            .mock("POST", "/graphql")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_graphql_error_response().to_string())
            .create();

        let client = LinearClient::with_base_url("test_api_key".to_string(), server.url()).unwrap();
        let result = client.execute_viewer_query().await;

        mock.assert();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("GraphQL errors"));
    }

    #[tokio::test]
    async fn test_network_timeout() {
        let mut server = mock_linear_server().await;
        let mock = server
            .mock("POST", "/graphql")
            .with_status(408)
            .with_body("Request Timeout")
            .create();

        let client = LinearClient::with_base_url("test_api_key".to_string(), server.url()).unwrap();
        let result = client.execute_viewer_query().await;

        mock.assert();
        assert!(result.is_err());
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")]
    async fn test_real_api() {
        let api_key = std::env::var("LINEAR_API_KEY")
            .expect("LINEAR_API_KEY must be set for integration tests");

        let client = LinearClient::new(api_key).expect("Failed to create client");
        let result = client.execute_viewer_query().await;

        assert!(result.is_ok(), "Query should succeed with valid API key");
        let data = result.unwrap();
        assert!(!data.viewer.id.is_empty(), "Viewer should have an ID");
    }

    #[test]
    fn test_list_issues_query_builds() {
        let _query = ListIssues::build_query(list_issues::Variables {
            first: 20,
            filter: None,
        });
    }

    #[tokio::test]
    async fn test_list_issues_success() {
        let mut server = mock_linear_server().await;
        let mock = server
            .mock("POST", "/graphql")
            .match_header("authorization", "test_api_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_issues_response().to_string())
            .create();

        let client = LinearClient::with_base_url("test_api_key".to_string(), server.url()).unwrap();
        let result = client.list_issues(20).await;

        mock.assert();
        assert!(result.is_ok());
        let issues = result.unwrap();
        assert_eq!(issues.len(), 3);

        assert_eq!(issues[0].identifier, "TEST-1");
        assert_eq!(issues[0].title, "Test Issue 1");
        assert_eq!(issues[0].status, "Todo");
        assert_eq!(issues[0].assignee, Some("Alice".to_string()));
        assert_eq!(issues[0].assignee_id, Some("user-1".to_string()));
        assert_eq!(issues[0].team, Some("ENG".to_string()));

        assert_eq!(issues[1].identifier, "TEST-2");
        assert_eq!(issues[1].title, "Test Issue 2");
        assert_eq!(issues[1].status, "In Progress");
        assert_eq!(issues[1].assignee, Some("Bob".to_string()));
        assert_eq!(issues[1].assignee_id, Some("user-2".to_string()));
        assert_eq!(issues[1].team, Some("DESIGN".to_string()));

        assert_eq!(issues[2].identifier, "TEST-3");
        assert_eq!(issues[2].title, "Test Issue 3");
        assert_eq!(issues[2].status, "Done");
        assert_eq!(issues[2].assignee, None);
        assert_eq!(issues[2].assignee_id, None);
        assert_eq!(issues[2].team, Some("QA".to_string()));
    }

    #[tokio::test]
    async fn test_list_issues_empty() {
        let mut server = mock_linear_server().await;
        let mock = server
            .mock("POST", "/graphql")
            .match_header("authorization", "test_api_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_empty_issues_response().to_string())
            .create();

        let client = LinearClient::with_base_url("test_api_key".to_string(), server.url()).unwrap();
        let result = client.list_issues(20).await;

        mock.assert();
        assert!(result.is_ok());
        let issues = result.unwrap();
        assert_eq!(issues.len(), 0);
    }

    #[tokio::test]
    async fn test_list_issues_error() {
        let mut server = mock_linear_server().await;
        let mock = server
            .mock("POST", "/graphql")
            .match_header("authorization", "test_api_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_graphql_error_response().to_string())
            .create();

        let client = LinearClient::with_base_url("test_api_key".to_string(), server.url()).unwrap();
        let result = client.list_issues(20).await;

        mock.assert();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("GraphQL errors"));
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")]
    async fn test_list_issues_real_api() {
        let api_key = std::env::var("LINEAR_API_KEY")
            .expect("LINEAR_API_KEY must be set for integration tests");

        let client = LinearClient::new(api_key).expect("Failed to create client");
        let result = client.list_issues(5).await;

        assert!(result.is_ok(), "Query should succeed with valid API key");
        let issues = result.unwrap();
        assert!(issues.len() <= 5, "Should return at most 5 issues");
    }
}
