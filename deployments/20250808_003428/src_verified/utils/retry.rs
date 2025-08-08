use std::time::Duration;
use std::future::Future;
use std::pin::Pin;
use std::fmt;
use backoff::{ExponentialBackoff, backoff::Backoff};
use anyhow::{Result, Context};
use tracing::{debug, warn, error};

/// Configuration for retry operations with exponential backoff
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: usize,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub jitter: bool,
}

// PRINCIPLE 0 ENFORCEMENT: No Default implementation
// All retry configuration must be explicitly provided

impl RetryConfig {
    /// Create new RetryConfig with explicit parameters - no defaults provided
    pub fn new(max_retries: usize, initial_delay: Duration, max_delay: Duration, multiplier: f64, jitter: bool) -> Self {
        Self {
            max_retries,
            initial_delay,
            max_delay,
            multiplier,
            jitter,
        }
    }

    pub fn max_retries(mut self, retries: usize) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    pub fn multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }
}

/// Trait for operations that can be retried
pub trait RetryableOperation<T, E> {
    fn call(&mut self) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + '_>>;
    fn is_retryable(&self, error: &E) -> bool;
    fn operation_name(&self) -> &str {
        "operation"
    }
}

/// Retry an operation with exponential backoff
pub async fn retry_with_backoff<T, E, Op>(
    mut operation: Op,
    config: RetryConfig,
) -> Result<T, E>
where
    Op: RetryableOperation<T, E>,
    E: fmt::Debug + fmt::Display,
{
    let mut backoff = ExponentialBackoff {
        initial_interval: config.initial_delay,
        max_interval: config.max_delay,
        multiplier: config.multiplier,
        max_elapsed_time: None,
        // Explicit configuration - no defaults
        current_interval: config.initial_delay,
        start_time: std::time::Instant::now(),
        randomization_factor: if config.jitter { 0.5 } else { 0.0 },
        // Note: reset_interval field doesn't exist in backoff crate - configuration is explicit via other fields
        clock: backoff::SystemClock {},
    };

    let mut attempt = 0;
    let operation_name = operation.operation_name().to_string();

    loop {
        attempt += 1;
        debug!("Attempting {} (attempt {}/{})", operation_name, attempt, config.max_retries + 1);

        match operation.call().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!("Operation {} succeeded after {} attempts", operation_name, attempt);
                }
                return Ok(result);
            }
            Err(error) => {
                if attempt > config.max_retries || !operation.is_retryable(&error) {
                    error!("Operation {} failed after {} attempts: {}", operation_name, attempt, error);
                    return Err(error);
                }

                if let Some(delay) = backoff.next_backoff() {
                    warn!(
                        "Operation {} failed (attempt {}/{}), retrying in {:?}: {}",
                        operation_name,
                        attempt,
                        config.max_retries + 1,
                        delay,
                        error
                    );
                    tokio::time::sleep(delay).await;
                } else {
                    error!("Backoff expired for operation {}: {}", operation_name, error);
                    return Err(error);
                }
            }
        }
    }
}

/// Wrapper for database operations
pub struct DatabaseOperation<F, T>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, anyhow::Error>> + Send + 'static>>,
{
    operation: F,
    name: String,
}

impl<F, T> DatabaseOperation<F, T>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, anyhow::Error>> + Send + 'static>>,
{
    pub fn new(name: String, operation: F) -> Self {
        Self { operation, name }
    }
}

impl<F, T> RetryableOperation<T, anyhow::Error> for DatabaseOperation<F, T>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, anyhow::Error>> + Send + 'static>>,
{
    fn call(&mut self) -> Pin<Box<dyn Future<Output = Result<T, anyhow::Error>> + Send + '_>> {
        (self.operation)()
    }

    fn is_retryable(&self, error: &anyhow::Error) -> bool {
        let error_str = error.to_string().to_lowercase();
        
        // Retry on common transient database errors
        error_str.contains("connection") ||
        error_str.contains("timeout") ||
        error_str.contains("temporary") ||
        error_str.contains("busy") ||
        error_str.contains("lock") ||
        error_str.contains("network") ||
        error_str.contains("i/o")
    }

    fn operation_name(&self) -> &str {
        &self.name
    }
}

/// Wrapper for file operations
pub struct FileOperation<F, T>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, std::io::Error>> + Send + 'static>>,
{
    operation: F,
    name: String,
}

impl<F, T> FileOperation<F, T>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, std::io::Error>> + Send + 'static>>,
{
    pub fn new(name: String, operation: F) -> Self {
        Self { operation, name }
    }
}

impl<F, T> RetryableOperation<T, std::io::Error> for FileOperation<F, T>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, std::io::Error>> + Send + 'static>>,
{
    fn call(&mut self) -> Pin<Box<dyn Future<Output = Result<T, std::io::Error>> + Send + '_>> {
        (self.operation)()
    }

    fn is_retryable(&self, error: &std::io::Error) -> bool {
        use std::io::ErrorKind;
        
        // Retry on transient I/O errors
        matches!(error.kind(),
            ErrorKind::Interrupted |
            ErrorKind::WouldBlock |
            ErrorKind::TimedOut |
            ErrorKind::BrokenPipe |
            ErrorKind::ConnectionReset |
            ErrorKind::ConnectionAborted
        )
    }

    fn operation_name(&self) -> &str {
        &self.name
    }
}

/// Convenience function for retrying database operations
pub async fn retry_database_operation<T, F>(
    name: &str,
    operation: F,
    config: RetryConfig,
) -> Result<T>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, anyhow::Error>> + Send + 'static>>,
{
    let db_op = DatabaseOperation::new(name.to_string(), operation);
    
    retry_with_backoff(db_op, config)
        .await
        .context(format!("Database operation '{}' failed after retries", name))
}

/// Convenience function for retrying file operations
/// Configuration must be explicitly provided - no fallback values
pub async fn retry_file_operation<T, F>(
    name: &str,
    operation: F,
    config: RetryConfig,
) -> std::io::Result<T>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = std::io::Result<T>> + Send + 'static>>,
{
    let file_op = FileOperation::new(name.to_string(), operation);
    
    retry_with_backoff(file_op, config).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct TestOperation {
        attempts: Arc<AtomicUsize>,
        fail_count: usize,
        name: String,
    }

    impl TestOperation {
        fn new(name: &str, fail_count: usize) -> Self {
            Self {
                attempts: Arc::new(AtomicUsize::new(0)),
                fail_count,
                name: name.to_string(),
            }
        }
    }

    impl RetryableOperation<String, anyhow::Error> for TestOperation {
        fn call(&mut self) -> Pin<Box<dyn Future<Output = Result<String, anyhow::Error>> + Send + '_>> {
            let attempts = self.attempts.clone();
            let fail_count = self.fail_count;
            
            Box::pin(async move {
                let attempt_count = attempts.fetch_add(1, Ordering::SeqCst) + 1;
                
                if attempt_count <= fail_count {
                    anyhow::bail!("temporary failure")
                } else {
                    Ok("success".to_string())
                }
            })
        }

        fn is_retryable(&self, _error: &anyhow::Error) -> bool {
            true
        }

        fn operation_name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_successful_retry() {
        let operation = TestOperation::new("test_op", 2);
        let config = RetryConfig::new(
            3,
            Duration::from_millis(100),
            Duration::from_secs(30),
            2.0,
            true
        );
        
        let result = retry_with_backoff(operation, config).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_exhausted_retries() {
        let operation = TestOperation::new("test_op", 5);
        let config = RetryConfig::new(
            3,
            Duration::from_millis(100),
            Duration::from_secs(30),
            2.0,
            true
        );
        
        let result = retry_with_backoff(operation, config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_immediate_success() {
        let operation = TestOperation::new("test_op", 0);
        let config = RetryConfig::new(
            3,
            Duration::from_millis(100),
            Duration::from_secs(30),
            2.0,
            true
        );
        
        let result = retry_with_backoff(operation, config).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }
}