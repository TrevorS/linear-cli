// ABOUTME: Linear SDK library providing type-safe GraphQL client for Linear API
// ABOUTME: Includes authentication, queries, mutations, and generated types

use anyhow::Result;
use graphql_client::{GraphQLQuery, Response};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use std::time::Duration;

#[cfg(test)]
pub mod test_helpers;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/queries/viewer.graphql",
    response_derives = "Debug"
)]
pub struct Viewer;

pub use viewer::ResponseData as ViewerResponseData;

pub struct LinearClient {
    client: reqwest::Client,
    base_url: String,
    _api_key: String,
}

impl LinearClient {
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
}
