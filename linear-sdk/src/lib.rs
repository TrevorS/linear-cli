// ABOUTME: Linear SDK library providing type-safe GraphQL client for Linear API
// ABOUTME: Includes authentication, queries, mutations, and generated types

use anyhow::Result;
use graphql_client::{GraphQLQuery, Response};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use std::time::Duration;

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
    _api_key: String,
}

impl LinearClient {
    pub fn new(api_key: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&api_key)?);
        headers.insert(USER_AGENT, HeaderValue::from_static("linear-cli/0.1.0"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            _api_key: api_key,
        })
    }

    pub async fn execute_viewer_query(&self) -> Result<viewer::ResponseData> {
        let request_body = Viewer::build_query(viewer::Variables {});

        let response = self
            .client
            .post("https://api.linear.app/graphql")
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

    #[test]
    fn test_linear_client_creation() {
        let client = LinearClient::new("test_api_key".to_string());
        assert!(client.is_ok());
    }

    #[test]
    fn test_viewer_query_builds() {
        // This test verifies that the GraphQL code generation worked
        let _query = Viewer::build_query(viewer::Variables {});
    }

    #[tokio::test]
    #[ignore] // Run with: cargo test -- --ignored
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
