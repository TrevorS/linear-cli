// ABOUTME: GraphQL abstraction layer providing executor trait and query builders
// ABOUTME: Implements comprehensive abstraction for Linear GraphQL API interactions

use async_trait::async_trait;
use graphql_client::GraphQLQuery;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::error::LinearError;

/// Trait for executing GraphQL queries with instrumentation support
#[async_trait]
pub trait GraphQLExecutor: Send + Sync {
    /// Execute a GraphQL query with instrumentation
    async fn execute<Q, V>(&self, variables: V) -> Result<Q::ResponseData, LinearError>
    where
        Q: GraphQLQuery + Send + Sync,
        Q::ResponseData: Debug + serde::de::DeserializeOwned + Send,
        Q::Variables: Debug + Send + Sync + Clone,
        V: Into<Q::Variables> + Send + Debug;

    /// Execute a batch of GraphQL queries
    async fn execute_batch<Q, V>(
        &self,
        queries: Vec<(Q, V)>,
    ) -> Result<Vec<Q::ResponseData>, LinearError>
    where
        Q: GraphQLQuery + Send + Sync,
        Q::ResponseData: Debug + serde::de::DeserializeOwned + Send,
        Q::Variables: Debug + Send + Sync + Clone,
        V: Into<Q::Variables> + Send + Debug;
}

/// Builder for constructing complex GraphQL queries
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    query: String,
    variables: HashMap<String, serde_json::Value>,
    extensions: HashMap<String, serde_json::Value>,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            variables: HashMap::new(),
            extensions: HashMap::new(),
        }
    }

    /// Add a variable to the query
    pub fn variable<T: serde::Serialize>(mut self, name: impl Into<String>, value: T) -> Self {
        self.variables.insert(
            name.into(),
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
        self
    }

    /// Add an extension to the query
    pub fn extension<T: serde::Serialize>(mut self, name: impl Into<String>, value: T) -> Self {
        self.extensions.insert(
            name.into(),
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
        self
    }

    /// Get the built query string
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Get the variables
    pub fn variables(&self) -> &HashMap<String, serde_json::Value> {
        &self.variables
    }

    /// Get the extensions
    pub fn extensions(&self) -> &HashMap<String, serde_json::Value> {
        &self.extensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder() {
        let builder = QueryBuilder::new("query { viewer { id } }")
            .variable("first", 10)
            .variable("after", "cursor")
            .extension("tracing", true);

        assert_eq!(builder.query(), "query { viewer { id } }");
        assert_eq!(builder.variables().len(), 2);
        assert_eq!(builder.extensions().len(), 1);
    }
}
