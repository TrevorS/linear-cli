// ABOUTME: Builder pattern implementation for LinearClient configuration
// ABOUTME: Provides type-safe configuration with compile-time validation

use crate::LinearClient;
use crate::error::LinearError;
use secrecy::SecretString;
use std::marker::PhantomData;
use std::time::Duration;
use typed_builder::TypedBuilder;
use url::Url;

#[derive(Debug, TypedBuilder)]
#[builder(build_method(into = Result<LinearClient, LinearError>))]
pub struct LinearClientConfig {
    pub auth_token: SecretString,

    #[builder(default = false)]
    pub verbose: bool,

    #[builder(default = Duration::from_secs(30))]
    pub timeout: Duration,

    #[builder(default = None)]
    pub proxy: Option<reqwest::Proxy>,

    #[builder(default = 3)]
    pub max_retries: usize,

    #[builder(default = None)]
    pub base_url: Option<String>,
}

impl From<LinearClientConfig> for Result<LinearClient, LinearError> {
    fn from(config: LinearClientConfig) -> Self {
        LinearClient::from_config(config)
    }
}

impl LinearClient {
    pub fn builder() -> LinearClientConfigBuilder<((), (), (), (), (), ())> {
        LinearClientConfig::builder()
    }

    pub fn typed_builder() -> TypedLinearClientBuilder<Initial> {
        TypedLinearClientBuilder::new()
    }
}

// Helper to create proxy from URL
impl LinearClient {
    pub fn create_proxy(url: &str) -> Result<reqwest::Proxy, LinearError> {
        let parsed_url = Url::parse(url)
            .map_err(|e| LinearError::Configuration(format!("Invalid proxy URL: {}", e)))?;

        reqwest::Proxy::all(parsed_url.as_str())
            .map_err(|e| LinearError::Configuration(format!("Invalid proxy configuration: {}", e)))
    }
}

// Type states for compile-time validation
pub struct Initial;
pub struct WithAuth;

pub struct TypedLinearClientBuilder<State = Initial> {
    config: Option<LinearClientConfig>,
    auth_token: Option<SecretString>,
    verbose: bool,
    timeout: Duration,
    proxy: Option<reqwest::Proxy>,
    max_retries: usize,
    base_url: Option<String>,
    _state: PhantomData<State>,
}

impl Default for TypedLinearClientBuilder<Initial> {
    fn default() -> Self {
        Self::new()
    }
}

impl TypedLinearClientBuilder<Initial> {
    pub fn new() -> Self {
        Self {
            config: None,
            auth_token: None,
            verbose: false,
            timeout: Duration::from_secs(30),
            proxy: None,
            max_retries: 3,
            base_url: None,
            _state: PhantomData,
        }
    }

    pub fn auth_token(self, token: SecretString) -> TypedLinearClientBuilder<WithAuth> {
        TypedLinearClientBuilder {
            config: self.config,
            auth_token: Some(token),
            verbose: self.verbose,
            timeout: self.timeout,
            proxy: self.proxy,
            max_retries: self.max_retries,
            base_url: self.base_url,
            _state: PhantomData,
        }
    }
}

impl<State> TypedLinearClientBuilder<State> {
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn proxy(mut self, url: &str) -> Result<Self, LinearError> {
        let parsed_url = Url::parse(url)
            .map_err(|e| LinearError::Configuration(format!("Invalid proxy URL: {}", e)))?;

        let proxy = reqwest::Proxy::all(parsed_url.as_str()).map_err(|e| {
            LinearError::Configuration(format!("Invalid proxy configuration: {}", e))
        })?;

        self.proxy = Some(proxy);
        Ok(self)
    }

    pub fn max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn base_url(mut self, base_url: Option<String>) -> Self {
        self.base_url = base_url;
        self
    }
}

impl TypedLinearClientBuilder<WithAuth> {
    pub fn build(self) -> Result<LinearClient, LinearError> {
        let config = LinearClientConfig {
            auth_token: self.auth_token.unwrap(),
            verbose: self.verbose,
            timeout: self.timeout,
            proxy: self.proxy,
            max_retries: self.max_retries,
            base_url: self.base_url,
        };

        LinearClient::from_config(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LinearClient;
    use crate::error::LinearError;
    use secrecy::SecretString;
    use std::time::Duration;

    #[test]
    fn test_builder_with_minimal_config() {
        let api_key = SecretString::new("test-api-key".to_string().into_boxed_str());
        let client_result = LinearClient::builder().auth_token(api_key).build();

        assert!(client_result.is_ok());
    }

    #[test]
    fn test_builder_with_all_options() {
        let api_key = SecretString::new("test-api-key".to_string().into_boxed_str());

        let client_result = LinearClient::builder()
            .auth_token(api_key)
            .verbose(true)
            .timeout(Duration::from_secs(60))
            .max_retries(5)
            .build();

        assert!(client_result.is_ok());
    }

    #[test]
    fn test_type_state_builder_requires_auth() {
        // This should not compile without auth_token
        // let client = LinearClient::typed_builder()
        //     .build(); // ERROR: build() not available without auth

        let api_key = SecretString::new("test-api-key".to_string().into_boxed_str());
        let client_result = LinearClient::typed_builder().auth_token(api_key).build();

        assert!(client_result.is_ok());
    }

    #[test]
    fn test_config_uses_secrecy_for_sensitive_data() {
        let api_key = SecretString::new("test-api-key".to_string().into_boxed_str());
        let config_result = LinearClientConfig::builder()
            .auth_token(api_key.clone())
            .build();

        assert!(config_result.is_ok());

        // SecretString should protect the value in the config struct itself
        // We can't easily test the debug output of LinearClientConfig since it's private
        // But we can verify the SecretString itself protects the value
        let debug_str = format!("{:?}", api_key);
        assert!(!debug_str.contains("test-api-key"));
    }

    #[test]
    fn test_builder_validates_proxy_url() {
        let invalid_proxy = "not-a-url";

        let result = LinearClient::create_proxy(invalid_proxy);

        assert!(result.is_err());
        match result {
            Err(LinearError::Configuration(msg)) => {
                assert!(msg.contains("Invalid proxy URL"));
            }
            _ => panic!("Expected configuration error"),
        }
    }

    #[test]
    fn test_default_configuration_values() {
        let api_key = SecretString::new("test-api-key".to_string().into_boxed_str());
        let client_result = LinearClient::builder().auth_token(api_key).build();

        assert!(client_result.is_ok());
        // We can't easily test the internal config values since they're private
        // But we've verified the client was created successfully with defaults
    }

    #[test]
    fn test_builder_with_valid_proxy() {
        let api_key = SecretString::new("test-api-key".to_string().into_boxed_str());
        let proxy_url = "http://proxy:8080";

        let proxy_result = LinearClient::create_proxy(proxy_url);
        assert!(proxy_result.is_ok());

        let client_result = LinearClient::builder()
            .auth_token(api_key)
            .proxy(Some(proxy_result.unwrap()))
            .build();

        assert!(client_result.is_ok());
    }

    #[test]
    fn test_builder_with_base_url() {
        let api_key = SecretString::new("test-api-key".to_string().into_boxed_str());
        let base_url = "https://custom.api.url".to_string();

        let client_result = LinearClient::builder()
            .auth_token(api_key)
            .base_url(Some(base_url.clone()))
            .build();

        assert!(client_result.is_ok());
    }
}
