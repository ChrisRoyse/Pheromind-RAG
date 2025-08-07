# Error Handling Patterns (Verified Implementation)

## Core Error Types (src/error.rs)

### Primary Error Enums
```rust
#[derive(Debug, thiserror::Error)]
pub enum EmbedError {
    // Main application error type - aggregates all others
}

pub enum StorageError {
    DatabaseError(String),
    SchemaError(String),
    InsertError(String),
    SearchError(String),
    InvalidInput(String),
    InvalidVector { expected: usize, got: usize },
}

pub enum EmbeddingError {
    // ML embedding errors
}

pub enum SearchError {
    // Search-specific errors
}

pub enum LoggingError {
    // Logging/metrics errors
}
```

### Storage-Specific Error Types
```rust
// In lancedb_storage.rs
pub enum LanceStorageError {
    DatabaseError(String),
    SchemaError(String),
    InsertError(String),
    SearchError(String),
    InvalidInput(String),
    ConfigError(String),
    InsufficientRecords(String),
    IndexingNotImplemented,
}

// Conversion implementations
impl From<StorageError> for EmbedError
impl From<EmbeddingError> for EmbedError
impl From<SearchError> for EmbedError
impl From<LoggingError> for EmbedError
impl From<anyhow::Error> for EmbedError
impl From<io::Error> for EmbedError
```

## Error Handling Patterns in Use

### 1. Result Type Alias
```rust
// Common pattern throughout codebase
pub type Result<T> = std::result::Result<T, EmbedError>;
```

### 2. Error Propagation with Context
```rust
// Pattern used extensively in storage modules
operation()
    .map_err(|e| StorageError::DatabaseError(format!("Failed to open: {}", e)))?;

// In lancedb_storage.rs (frequent pattern)
.map_err(|e| LanceStorageError::DatabaseError(format!("Connection failed: {}", e)))?;
```

### 3. Error Context Trait (Custom)
```rust
pub trait ErrorContext<T> {
    fn context(self, msg: &str) -> Result<T>;
    fn with_context<F>(self, f: F) -> Result<T>
    where F: FnOnce() -> String;
}
```

### 4. Safe Unwrap Trait (Custom)
```rust
pub trait SafeUnwrap<T> {
    fn safe_unwrap(self, default: T) -> T;
    fn safe_unwrap_or_else<F>(self, f: F) -> T
    where F: FnOnce() -> T;
}
```

### 5. Retry with Backoff
```rust
pub struct RetryConfig {
    max_attempts: u32,
    initial_delay_ms: u64,
    max_delay_ms: u64,
    exponential_base: f64,
}

pub async fn retry_with_backoff<F, Fut, T, E>(
    config: &RetryConfig,
    operation: F,
) -> Result<T, E>

fn is_retryable_error<E: std::fmt::Display>(error: &E) -> bool
```

## Common Error Handling Patterns

### Storage Module Pattern
```rust
// Consistent pattern in lancedb_storage.rs
std::fs::create_dir_all(&dir)
    .map_err(|e| LanceStorageError::DatabaseError(
        format!("Failed to create directory: {}", e)
    ))?;

// Search operations
table.vector_search(query_vector)
    .limit(limit)
    .execute()
    .await
    .map_err(|e| LanceStorageError::SearchError(
        format!("Search execution failed: {}", e)
    ))?;
```

### Early Return Pattern
```rust
// Check conditions early
if self.table_name.is_empty() {
    return Err(LanceStorageError::InvalidInput(
        "Table name cannot be empty".to_string()
    ));
}
```

### Option to Result Conversion
```rust
// Common in configuration
let value = map.get(key)
    .ok_or_else(|| anyhow!("Key not found: {}", key))?;
```

## Error Recovery Strategies

### 1. Fallback Values
```rust
// Using safe_unwrap trait
let value = operation().safe_unwrap(default_value);
```

### 2. Retry Logic
```rust
// Exponential backoff for transient failures
retry_with_backoff(&retry_config, || async {
    database.connect().await
}).await?;
```

### 3. Graceful Degradation
```rust
// Continue with partial results
match search_backend.search(query).await {
    Ok(results) => results,
    Err(e) => {
        log::warn!("Search backend failed: {}, using fallback", e);
        Vec::new() // Return empty results
    }
}
```

## Testing Error Cases

### Unit Test Examples (from tests)
```rust
#[test]
fn test_error_context() {
    let result: Result<(), _> = Err(io::Error::new(io::ErrorKind::NotFound, "test"));
    let with_context = result.context("Additional context");
    assert!(with_context.is_err());
}

#[test]
fn test_safe_unwrap() {
    let none_value: Option<i32> = None;
    assert_eq!(none_value.safe_unwrap(42), 42);
}
```

## Logging Integration
```rust
use tracing::{error, warn, info, debug};

// Error logging pattern
if let Err(e) = operation() {
    error!("Operation failed: {:?}", e);
    // Handle or propagate
}

// Warning for recoverable issues
warn!("Retrying operation after error: {}", e);
```

## Common Anti-Patterns to Avoid

### ❌ Avoid unwrap() in production
```rust
// BAD
let value = map.get(key).unwrap();

// GOOD
let value = map.get(key)
    .ok_or_else(|| EmbedError::InvalidInput("Key not found".into()))?;
```

### ❌ Avoid silent failures
```rust
// BAD
let _ = operation(); // Ignoring result

// GOOD
if let Err(e) = operation() {
    log::error!("Operation failed: {}", e);
}
```

### ❌ Avoid generic error messages
```rust
// BAD
.map_err(|_| "Operation failed")?;

// GOOD
.map_err(|e| format!("Failed to process file {}: {}", path, e))?;
```

## Module-Specific Patterns

### Storage Modules
- Consistent use of `map_err` with formatted messages
- Error type conversion to module-specific errors
- Database operation errors include context

### Search Modules
- Query parsing errors with helpful messages
- Score validation and bounds checking
- Fallback to empty results on non-critical failures

### Embedding Modules
- Model loading error recovery
- Dimension mismatch detection
- Cache fallback on embedding failure

## Best Practices Summary

1. **Use thiserror** for error definitions
2. **Add context** at each error boundary
3. **Format error messages** with relevant details
4. **Implement From traits** for error conversion
5. **Use Result<T>** consistently
6. **Log errors** appropriately (error!, warn!)
7. **Test error paths** explicitly
8. **Provide fallbacks** where sensible
9. **Retry transient failures** with backoff
10. **Never panic** in library code