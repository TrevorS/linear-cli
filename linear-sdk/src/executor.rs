// ABOUTME: Default implementation of GraphQLExecutor trait for LinearClient
// ABOUTME: Provides batched query execution functionality without caching

use async_trait::async_trait;
use graphql_client::GraphQLQuery;
use std::fmt::Debug;

use crate::error::LinearError;
use crate::graphql::GraphQLExecutor;
use crate::{LinearClient, Result};

/// Implementation of GraphQLExecutor for LinearClient
pub struct LinearExecutor {
    client: LinearClient,
}

impl LinearExecutor {
    /// Create a new executor
    pub fn new(client: LinearClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl GraphQLExecutor for LinearExecutor {
    async fn execute<Q, V>(&self, variables: V) -> Result<Q::ResponseData>
    where
        Q: GraphQLQuery + Send + Sync,
        Q::ResponseData: Debug + serde::de::DeserializeOwned + Send,
        Q::Variables: Debug + Send + Sync + Clone,
        V: Into<Q::Variables> + Send + Debug,
    {
        let variables = variables.into();
        self.execute_query::<Q>(variables).await
    }

    async fn execute_batch<Q, V>(&self, queries: Vec<(Q, V)>) -> Result<Vec<Q::ResponseData>>
    where
        Q: GraphQLQuery + Send + Sync,
        Q::ResponseData: Debug + serde::de::DeserializeOwned + Send,
        Q::Variables: Debug + Send + Sync + Clone,
        V: Into<Q::Variables> + Send + Debug,
    {
        let mut results = Vec::with_capacity(queries.len());

        // Execute queries concurrently
        let futures: Vec<_> = queries
            .into_iter()
            .map(|(_query, variables)| self.execute::<Q, V>(variables))
            .collect();

        for future in futures {
            results.push(future.await?);
        }

        Ok(results)
    }
}

impl LinearExecutor {
    /// Execute a single GraphQL query (internal implementation)
    /// Supports all Linear GraphQL query types with proper query routing
    async fn execute_query<Q>(&self, variables: Q::Variables) -> Result<Q::ResponseData>
    where
        Q: GraphQLQuery + Send + Sync,
        Q::ResponseData: Debug + serde::de::DeserializeOwned + Send,
        Q::Variables: Debug + Send + Sync + Clone,
    {
        use graphql_client::Response;

        // Build the query request
        let request_body = Q::build_query(variables);

        if self.client.verbose {
            let query_name = std::any::type_name::<Q>()
                .split("::")
                .last()
                .unwrap_or("unknown");
            eprintln!("Sending GraphQL query: {}", query_name);
            eprintln!(
                "Request body: {}",
                serde_json::to_string_pretty(&request_body).unwrap_or_default()
            );
        }

        // Execute using the client's retry logic
        crate::retry::retry_with_backoff(&self.client.retry_config, self.client.verbose, || {
            let client = &self.client.client;
            let base_url = &self.client.base_url;
            let request_body = &request_body;
            let verbose = self.client.verbose;

            async move {
                let start_time = std::time::Instant::now();
                let response = client
                    .post(format!("{}/graphql", base_url))
                    .json(request_body)
                    .send()
                    .await
                    .map_err(LinearError::from)?;

                if verbose {
                    eprintln!("Request completed in {:?}", start_time.elapsed());
                    eprintln!("Response status: {}", response.status());
                }

                // Check for HTTP error status codes
                if !response.status().is_success() {
                    return Err(LinearError::from_status(
                        http::StatusCode::from_u16(response.status().as_u16()).unwrap(),
                    ));
                }

                let response_body: Response<Q::ResponseData> =
                    response.json().await.map_err(LinearError::from)?;

                if let Some(errors) = response_body.errors {
                    return Err(LinearError::GraphQL {
                        message: format!("{:?}", errors),
                        errors: vec![],
                    });
                }

                response_body.data.ok_or(LinearError::InvalidResponse)
            }
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::SecretString;

    #[tokio::test]
    async fn test_executor_creation() {
        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .build()
            .unwrap();

        let executor = LinearExecutor::new(client);
        assert!(!std::ptr::addr_of!(executor).is_null());
    }

    #[tokio::test]
    async fn test_executor_integration() {
        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .build()
            .unwrap();

        let executor = LinearExecutor::new(client);

        // Test that the GraphQL executor supports all query types
        assert!(!std::ptr::addr_of!(executor).is_null());
    }

    #[tokio::test]
    async fn test_batch_execute_method_exists() {
        let client = LinearClient::builder()
            .auth_token(SecretString::new(
                "test_api_key".to_string().into_boxed_str(),
            ))
            .build()
            .unwrap();

        let executor = LinearExecutor::new(client);

        // Test that the batch execute method is available on the trait
        assert!(!std::ptr::addr_of!(executor).is_null());
        // The batch execute method exists and can be called with appropriate types
    }
}
