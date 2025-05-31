// ABOUTME: GraphQL abstraction layer providing executor trait, query builders, and caching
// ABOUTME: Implements comprehensive abstraction for Linear GraphQL API interactions

use ahash::AHasher;
use async_trait::async_trait;
use graphql_client::GraphQLQuery;
use lru::LruCache;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

use crate::error::LinearError;

/// Trait for executing GraphQL queries with tracing and caching support
#[async_trait]
pub trait GraphQLExecutor: Send + Sync {
    /// Execute a GraphQL query with instrumentation
    async fn execute<Q, V>(&self, variables: V) -> Result<Q::ResponseData, LinearError>
    where
        Q: GraphQLQuery + Send + Sync,
        Q::ResponseData: Debug + serde::de::DeserializeOwned + serde::Serialize + Send,
        Q::Variables: Debug + serde::Serialize + Send + Clone,
        V: Into<Q::Variables> + Send + Debug;

    /// Execute a batch of GraphQL queries
    async fn execute_batch<Q, V>(
        &self,
        queries: Vec<(Q, V)>,
    ) -> Result<Vec<Q::ResponseData>, LinearError>
    where
        Q: GraphQLQuery + Send + Sync,
        Q::ResponseData: Debug + serde::de::DeserializeOwned + serde::Serialize + Send,
        Q::Variables: Debug + serde::Serialize + Send + Clone,
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

/// Cached entry for query results
#[derive(Debug, Clone)]
struct CachedEntry {
    data: serde_json::Value,
    created_at: Instant,
}

impl CachedEntry {
    fn new(data: serde_json::Value) -> Self {
        Self {
            data,
            created_at: Instant::now(),
        }
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
}

/// LRU cache for GraphQL query results
pub struct QueryCache {
    cache: RwLock<LruCache<u64, CachedEntry>>,
    ttl: Duration,
}

impl QueryCache {
    /// Create a new query cache with specified capacity and TTL
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        Self {
            cache: RwLock::new(LruCache::new(
                std::num::NonZeroUsize::new(capacity)
                    .unwrap_or(std::num::NonZeroUsize::new(100).unwrap()),
            )),
            ttl,
        }
    }

    /// Generate cache key from query and variables
    fn cache_key<Q, V>(variables: &V) -> u64
    where
        Q: GraphQLQuery + Send + Sync,
        V: Debug + serde::Serialize + Send + Clone,
    {
        let mut hasher = AHasher::default();
        // Use a combination of module path and operation name for stability
        // This is more stable than type_name across Rust versions
        let query_type = std::any::type_name::<Q>();
        let operation_name = if query_type.contains("Viewer") {
            "viewer"
        } else if query_type.contains("ListIssues") {
            "listIssues"
        } else if query_type.contains("GetIssue") {
            "getIssue"
        } else {
            query_type // fallback to full type name
        };
        operation_name.hash(&mut hasher);
        if let Ok(var_bytes) = serde_json::to_vec(variables) {
            var_bytes.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Get cached result if available and not expired
    pub fn get<Q, V>(&self, variables: &V) -> Option<serde_json::Value>
    where
        Q: GraphQLQuery + Send + Sync,
        V: Debug + serde::Serialize + Send + Clone,
    {
        let key = Self::cache_key::<Q, V>(variables);
        let mut cache = self.cache.write();

        if let Some(entry) = cache.get(&key) {
            if !entry.is_expired(self.ttl) {
                return Some(entry.data.clone());
            } else {
                cache.pop(&key);
            }
        }
        None
    }

    /// Store result in cache
    pub fn set<Q, V>(&self, variables: &V, data: serde_json::Value)
    where
        Q: GraphQLQuery + Send + Sync,
        V: Debug + serde::Serialize + Send + Clone,
    {
        let key = Self::cache_key::<Q, V>(variables);
        let entry = CachedEntry::new(data);
        self.cache.write().put(key, entry);
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache.write().clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.read();
        CacheStats {
            capacity: cache.cap().get(),
            size: cache.len(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub capacity: usize,
    pub size: usize,
}

/// Extensions for GraphQL queries
#[derive(Debug, Clone, Default)]
pub struct QueryExtensions {
    pub tracing: Option<TracingExtension>,
    pub caching: Option<CachingExtension>,
}

/// Tracing extension configuration
#[derive(Debug, Clone)]
pub struct TracingExtension {
    pub enabled: bool,
    pub include_variables: bool,
}

/// Caching extension configuration
#[derive(Debug, Clone)]
pub struct CachingExtension {
    pub enabled: bool,
    pub ttl: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

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

    #[test]
    fn test_query_cache_basic() {
        let cache = QueryCache::new(10, Duration::from_secs(60));
        let stats = cache.stats();

        assert_eq!(stats.capacity, 10);
        assert_eq!(stats.size, 0);
    }

    #[test]
    fn test_cached_entry_expiry() {
        let entry = CachedEntry::new(serde_json::json!({"test": "data"}));

        assert!(!entry.is_expired(Duration::from_secs(1)));

        std::thread::sleep(Duration::from_millis(10));
        assert!(entry.is_expired(Duration::from_millis(5)));
    }

    #[test]
    fn test_cache_clear() {
        let cache = QueryCache::new(10, Duration::from_secs(60));
        cache.clear();

        let stats = cache.stats();
        assert_eq!(stats.size, 0);
    }
}
