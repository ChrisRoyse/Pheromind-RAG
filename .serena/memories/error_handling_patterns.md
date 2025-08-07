# Error Handling Patterns

## Error Types in This Codebase

### Main Error Type
- **Location**: `src/error.rs`
- **Type**: Uses `thiserror` for custom errors
- **Pattern**: `anyhow::Result` for propagation

### Finding Error Definitions
```
get_symbols_overview "src/error.rs"
find_symbol "Error" relative_path="src/error.rs"
search_for_pattern "pub enum.*Error"
```

## Error Handling Patterns

### 1. Result Type Usage
```rust
// Standard pattern
use anyhow::Result;

pub async fn operation() -> Result<Data> {
    // Use ? for propagation
    let data = risky_operation()?;
    Ok(data)
}
```

### 2. Adding Context
```rust
// Pattern throughout codebase
use anyhow::Context;

operation()
    .context("Failed to perform operation")?;

operation()
    .with_context(|| format!("Failed processing file: {}", path))?;
```

### 3. Custom Error Types
```rust
#[derive(thiserror::Error, Debug)]
pub enum SearchError {
    #[error("Index not found: {0}")]
    IndexNotFound(String),
    
    #[error("Embedding failed")]
    EmbeddingError(#[from] EmbeddingError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Common Error Patterns to Search

### Find Error Handling
```
# All Result returns
search_for_pattern "-> Result<"

# Error propagation
search_for_pattern "\?\s*;"

# Context additions
search_for_pattern "\.context\("
search_for_pattern "\.with_context\("

# Error matching
search_for_pattern "match.*Err\("

# Error creation
search_for_pattern "Err\(.*Error::"
```

### Find Error Recovery
```
# Unwrap alternatives
search_for_pattern "\.unwrap_or"
search_for_pattern "\.unwrap_or_else"
search_for_pattern "\.unwrap_or_default"

# Error logging
search_for_pattern "error!\("
search_for_pattern "warn!\("

# Panic points (avoid these)
search_for_pattern "\.unwrap\(\)"
search_for_pattern "\.expect\("
search_for_pattern "panic!\("
```

## Error Handling by Module

### Storage Errors
```
find_symbol "StorageError"
# Handles: Database failures, migration issues
# Recovery: Retry with backoff, reinitialize
```

### Embedding Errors  
```
find_symbol "EmbeddingError"
# Handles: Model loading, dimension mismatches
# Recovery: Fallback to cached, skip item
```

### Search Errors
```
find_symbol "SearchError"  
# Handles: Query parsing, no results
# Recovery: Suggest alternatives, return empty
```

### Config Errors
```
find_symbol "ConfigError"
# Handles: Missing config, invalid values
# Recovery: Use defaults, prompt user
```

## Best Practices in This Codebase

### 1. Early Returns
```rust
// Good - early return on error
if condition_failed {
    return Err(anyhow!("Condition not met"));
}

// Continue with success path
```

### 2. Error Chains
```rust
// Preserve error chain
operation()
    .map_err(|e| SearchError::OperationFailed(e))?;
```

### 3. Structured Errors
```rust
// Include relevant data
Err(SearchError::NotFound {
    query: query.clone(),
    searched_paths: paths,
})
```

## Debugging Error Flows

### 1. Trace Error Source
```
# Find where error originates
find_referencing_symbols "ErrorType"

# Find error creation
search_for_pattern "return Err\("
```

### 2. Follow Error Propagation
```
# Find ? operators in call chain
search_for_pattern "function_name.*\?"

# Find error transformations
search_for_pattern "map_err"
```

### 3. Check Error Handling
```
# Find recovery attempts
search_for_pattern "if let Err\("
search_for_pattern "match.*\{.*Err\("
```

## Common Issues and Fixes

### Issue: "Unhelpful error messages"
**Fix**: Add context at each level
```rust
operation()
    .context("High-level description")?
```

### Issue: "Lost error details"
**Fix**: Use error chains
```rust
#[error("Operation failed")]
OperationError(#[source] anyhow::Error)
```

### Issue: "Panics in production"
**Fix**: Replace unwrap with proper handling
```rust
// Bad
let value = map.get(key).unwrap();

// Good  
let value = map.get(key)
    .ok_or_else(|| anyhow!("Key not found: {}", key))?;
```

## Testing Error Cases

### Unit Test Pattern
```rust
#[test]
fn test_error_condition() {
    let result = operation_that_should_fail();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("expected"));
}
```

### Integration Test Pattern
```rust
#[tokio::test]
async fn test_error_recovery() {
    // Trigger error condition
    let result = service.process_invalid().await;
    
    // Verify graceful handling
    match result {
        Err(e) if e.is::<SpecificError>() => {
            // Expected error type
        }
        _ => panic!("Unexpected result"),
    }
}
```

## Monitoring Errors

### Logging Pattern
```rust
use tracing::{error, warn, info};

if let Err(e) = operation() {
    error!("Operation failed: {:?}", e);
    // Handle error
}
```

### Metrics Pattern
```rust
// Track error rates
metrics.increment_counter("errors", &[("type", "search")]);
```