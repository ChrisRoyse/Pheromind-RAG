// Comprehensive Error Handling System - Phase 1: Foundation & Safety
// This module provides robust error types to replace panic-prone patterns

use std::fmt;
use std::error::Error as StdError;
use std::io;
use thiserror::Error;

/// Main error type for the embed-search system
#[derive(Debug, Error)]
pub enum EmbedError {
    #[error("Configuration error: {message}")]
    Configuration { 
        message: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    
    #[error("Storage error: {message}")]
    Storage { 
        message: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    
    #[error("Embedding error: {message}")]
    Embedding { 
        message: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    
    #[error("Search error: {message}")]
    Search { 
        message: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    
    #[error("Model error: {message}")]
    Model {
        message: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    
    #[error("Tensor operation error: {message}")]
    Tensor {
        message: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    
    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { 
        resource: String,
        limit: Option<usize>,
        current: Option<usize>,
    },
    
    #[error("Invalid operation: {operation} in state {state}")]
    InvalidOperation { 
        operation: String, 
        state: String,
        details: Option<String>,
    },
    
    #[error("IO error: {message}")]
    Io {
        message: String,
        #[source]
        source: io::Error,
    },
    
    #[error("Serialization error: {message}")]
    Serialization {
        message: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    
    #[error("Database error: {message}")]
    Database {
        message: String,
        query: Option<String>,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    
    #[error("Validation error: {field} - {reason}")]
    Validation {
        field: String,
        reason: String,
        value: Option<String>,
    },
    
    #[error("Concurrency error: {message}")]
    Concurrency {
        message: String,
        operation: Option<String>,
    },
    
    #[error("Not found: {resource}")]
    NotFound {
        resource: String,
        id: Option<String>,
    },
    
    #[error("Already exists: {resource}")]
    AlreadyExists {
        resource: String,
        id: Option<String>,
    },
    
    #[error("Permission denied: {action} on {resource}")]
    PermissionDenied {
        action: String,
        resource: String,
    },
    
    #[error("Timeout: operation {operation} exceeded {duration_ms}ms")]
    Timeout {
        operation: String,
        duration_ms: u64,
    },
    
    #[error("Internal error: {message}")]
    Internal {
        message: String,
        backtrace: Option<String>,
    },
}

/// Result type alias for embed operations
pub type Result<T> = std::result::Result<T, EmbedError>;

/// Storage-specific error type
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Connection failed: {message}")]
    ConnectionFailed {
        message: String,
        url: Option<String>,
    },
    
    #[error("Query failed: {message}")]
    QueryFailed {
        message: String,
        query: String,
    },
    
    #[error("Transaction failed: {message}")]
    TransactionFailed {
        message: String,
    },
    
    #[error("Index error: {message}")]
    IndexError {
        message: String,
        index_name: Option<String>,
    },
    
    #[error("Schema mismatch: expected {expected}, got {actual}")]
    SchemaMismatch {
        expected: String,
        actual: String,
    },
    
    #[error("Lock timeout: {message}")]
    LockTimeout {
        message: String,
        duration_ms: u64,
    },
}

/// Embedding-specific error type
#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("Model not loaded: {model_name}")]
    ModelNotLoaded {
        model_name: String,
    },
    
    #[error("Invalid input: {message}")]
    InvalidInput {
        message: String,
        input_length: Option<usize>,
    },
    
    #[error("Tokenization failed: {message}")]
    TokenizationFailed {
        message: String,
    },
    
    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch {
        expected: usize,
        actual: usize,
    },
    
    #[error("Computation failed: {message}")]
    ComputationFailed {
        message: String,
    },
}

/// Search-specific error type
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Index not ready: {index_name}")]
    IndexNotReady {
        index_name: String,
    },
    
    #[error("Query invalid: {message}")]
    QueryInvalid {
        message: String,
        query: String,
    },
    
    #[error("No results found")]
    NoResults,
    
    #[error("Too many results: {count} exceeds limit {limit}")]
    TooManyResults {
        count: usize,
        limit: usize,
    },
}

// ==================== CONVERSION IMPLEMENTATIONS ====================

impl From<io::Error> for EmbedError {
    fn from(err: io::Error) -> Self {
        EmbedError::Io {
            message: err.to_string(),
            source: err,
        }
    }
}

impl From<StorageError> for EmbedError {
    fn from(err: StorageError) -> Self {
        EmbedError::Storage {
            message: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<EmbeddingError> for EmbedError {
    fn from(err: EmbeddingError) -> Self {
        EmbedError::Embedding {
            message: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<SearchError> for EmbedError {
    fn from(err: SearchError) -> Self {
        EmbedError::Search {
            message: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<anyhow::Error> for EmbedError {
    fn from(err: anyhow::Error) -> Self {
        EmbedError::Internal {
            message: err.to_string(),
            backtrace: None,
        }
    }
}

// ==================== ERROR CONTEXT HELPERS ====================

/// Extension trait for adding context to Results
pub trait ErrorContext<T> {
    /// Add context to an error
    fn context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static;
    
    /// Add context with a closure (lazy evaluation)
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    fn context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|e| EmbedError::Internal {
            message: format!("{}: {}", context, e),
            backtrace: None,
        })
    }
    
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| EmbedError::Internal {
            message: format!("{}: {}", f(), e),
            backtrace: None,
        })
    }
}

// ==================== SAFE UNWRAP REPLACEMENTS ====================

/// Safe replacement for unwrap() with better error messages
pub trait SafeUnwrap<T> {
    /// Unwrap with a descriptive error message
    fn safe_unwrap(self, context: &str) -> Result<T>;
    
    /// Unwrap or return a default value
    fn unwrap_or_error<E>(self, error: E) -> Result<T>
    where
        E: Into<EmbedError>;
}

impl<T> SafeUnwrap<T> for Option<T> {
    fn safe_unwrap(self, context: &str) -> Result<T> {
        self.ok_or_else(|| EmbedError::Internal {
            message: format!("Unwrap failed: {}", context),
            backtrace: None,
        })
    }
    
    fn unwrap_or_error<E>(self, error: E) -> Result<T>
    where
        E: Into<EmbedError>,
    {
        self.ok_or_else(|| error.into())
    }
}

impl<T, E> SafeUnwrap<T> for std::result::Result<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    fn safe_unwrap(self, context: &str) -> Result<T> {
        self.map_err(|e| EmbedError::Internal {
            message: format!("{}: {}", context, e),
            backtrace: None,
        })
    }
    
    fn unwrap_or_error<Er>(self, error: Er) -> Result<T>
    where
        Er: Into<EmbedError>,
    {
        self.map_err(|_| error.into())
    }
}

// ==================== RETRY MECHANISM ====================

/// Retry configuration for transient errors
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_base: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            exponential_base: 2.0,
        }
    }
}

impl RetryConfig {
    /// Create a new RetryConfig with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the maximum number of retry attempts
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_attempts = max_retries;
        self
    }
    
    /// Set the initial delay in milliseconds
    pub fn initial_delay_ms(mut self, delay_ms: u64) -> Self {
        self.initial_delay_ms = delay_ms;
        self
    }
    
    /// Set the maximum delay in milliseconds
    pub fn max_delay_ms(mut self, delay_ms: u64) -> Self {
        self.max_delay_ms = delay_ms;
        self
    }
    
    /// Set the exponential base for backoff
    pub fn exponential_base(mut self, base: f64) -> Self {
        self.exponential_base = base;
        self
    }
}

/// Retry an operation with exponential backoff
pub async fn retry_with_backoff<F, Fut, T>(
    config: RetryConfig,
    mut operation: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut delay_ms = config.initial_delay_ms;
    
    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < config.max_attempts => {
                // Check if error is retryable
                if !is_retryable_error(&e) {
                    return Err(e);
                }
                
                log::warn!(
                    "Operation failed (attempt {}/{}), retrying in {}ms: {}",
                    attempt, config.max_attempts, delay_ms, e
                );
                
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                
                // Exponential backoff with max delay
                delay_ms = ((delay_ms as f64) * config.exponential_base) as u64;
                delay_ms = delay_ms.min(config.max_delay_ms);
            }
            Err(e) => return Err(e),
        }
    }
    
    unreachable!("Retry loop should have returned by now")
}

/// Check if an error is retryable
fn is_retryable_error(error: &EmbedError) -> bool {
    matches!(
        error,
        EmbedError::Io { .. }
        | EmbedError::Database { .. }
        | EmbedError::Timeout { .. }
        | EmbedError::ResourceExhausted { .. }
    )
}

// ==================== ERROR RECOVERY ====================

/// Attempt to recover from an error
pub trait ErrorRecovery<T> {
    /// Try to recover with a fallback value
    fn recover_with<F>(self, fallback: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>;
    
    /// Log error and continue with default
    fn log_and_default(self, default: T) -> T
    where
        T: Clone;
}

impl<T> ErrorRecovery<T> for Result<T> {
    fn recover_with<F>(self, fallback: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        match self {
            Ok(value) => Ok(value),
            Err(e) => {
                log::warn!("Recovering from error: {}", e);
                fallback()
            }
        }
    }
    
    fn log_and_default(self, default: T) -> T
    where
        T: Clone,
    {
        match self {
            Ok(value) => value,
            Err(e) => {
                log::error!("Error occurred, using default: {}", e);
                default
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let embed_err: EmbedError = io_err.into();
        assert!(matches!(embed_err, EmbedError::Io { .. }));
    }
    
    #[test]
    fn test_safe_unwrap() {
        let some_value: Option<i32> = Some(42);
        let result = some_value.safe_unwrap("getting value");
        assert_eq!(result.unwrap(), 42);
        
        let none_value: Option<i32> = None;
        let result = none_value.safe_unwrap("getting missing value");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_error_context() {
        let result: std::result::Result<(), io::Error> = 
            Err(io::Error::new(io::ErrorKind::NotFound, "test"));
        
        let with_context = result.context("performing operation");
        assert!(with_context.is_err());
        assert!(with_context.unwrap_err().to_string().contains("performing operation"));
    }
}