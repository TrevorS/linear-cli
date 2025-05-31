// ABOUTME: Retry logic with exponential backoff for handling transient failures
// ABOUTME: Implements retry mechanism for network errors and rate limits

use crate::constants::retry;
use crate::error::LinearError;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: retry::MAX_RETRIES,
            initial_delay: retry::INITIAL_DELAY,
            max_delay: retry::MAX_DELAY,
            backoff_multiplier: retry::BACKOFF_MULTIPLIER,
        }
    }
}

pub async fn retry_with_backoff<F, Fut, T>(
    config: &RetryConfig,
    verbose: bool,
    mut operation: F,
) -> Result<T, LinearError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, LinearError>>,
{
    let mut delay = config.initial_delay;
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        if attempt > 0 {
            if verbose {
                eprintln!(
                    "Retrying operation (attempt {}/{})",
                    attempt, config.max_retries
                );
            }
            sleep(delay).await;
            delay = std::cmp::min(
                Duration::from_millis(
                    (delay.as_millis() as f64 * config.backoff_multiplier) as u64,
                ),
                config.max_delay,
            );
        }

        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if !error.is_retryable() || attempt == config.max_retries {
                    return Err(error);
                }

                if verbose {
                    eprintln!("Request failed (retryable): {}", error);
                }
                last_error = Some(error);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| LinearError::Network {
        message: "Retry failed".to_string(),
        retryable: false,
        source: Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Retry failed",
        )),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_retry_success_on_first_attempt() {
        let config = RetryConfig::default();
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        let result = retry_with_backoff(&config, false, || {
            let count = call_count_clone.clone();
            async move {
                let mut c = count.lock().unwrap();
                *c += 1;
                Ok::<i32, LinearError>(42)
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(*call_count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let config = RetryConfig::default();
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        let result = retry_with_backoff(&config, false, || {
            let count = call_count_clone.clone();
            async move {
                let mut c = count.lock().unwrap();
                *c += 1;
                if *c < 3 {
                    Err(LinearError::Network {
                        message: "Temporary failure".to_string(),
                        retryable: true,
                        source: Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Temporary failure",
                        )),
                    })
                } else {
                    Ok::<i32, LinearError>(42)
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(*call_count.lock().unwrap(), 3);
    }

    #[tokio::test]
    async fn test_retry_non_retryable_error() {
        let config = RetryConfig::default();
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        let result = retry_with_backoff(&config, false, || {
            let count = call_count_clone.clone();
            async move {
                let mut c = count.lock().unwrap();
                *c += 1;
                Err::<i32, LinearError>(LinearError::Auth {
                    reason: Cow::Borrowed("Test auth error"),
                    source: None,
                })
            }
        })
        .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LinearError::Auth { .. }));
        assert_eq!(*call_count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_retry_max_attempts_exceeded() {
        let config = RetryConfig {
            max_retries: 2,
            ..Default::default()
        };
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        let result = retry_with_backoff(&config, false, || {
            let count = call_count_clone.clone();
            async move {
                let mut c = count.lock().unwrap();
                *c += 1;
                Err::<i32, LinearError>(LinearError::Network {
                    message: "Always fail".to_string(),
                    retryable: true,
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Always fail",
                    )),
                })
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(*call_count.lock().unwrap(), 3); // Initial attempt + 2 retries
    }
}
