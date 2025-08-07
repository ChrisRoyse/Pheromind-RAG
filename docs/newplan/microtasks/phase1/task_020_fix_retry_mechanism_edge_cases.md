# Task 1.020: Fix Retry Mechanism Edge Cases

**Time Estimate**: 10 minutes
**Dependencies**: None
**File(s) to Modify**: `src/error.rs`

## Objective
Fix edge cases in retry logic to prevent infinite loops and ensure proper error handling.

## Success Criteria
- [ ] Retry mechanism handles all edge cases
- [ ] No infinite loops or deadlocks
- [ ] Proper timeout handling
- [ ] Clear error messages for retry failures

## Instructions

### Step 1: Fix retry loop termination
```rust
// Improve retry_with_backoff function around line 475:
pub async fn retry_with_backoff<F, Fut, T>(
    config: RetryConfig,
    mut operation: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    if config.max_attempts == 0 {
        return Err(EmbedError::Configuration {
            message: "Retry config max_attempts must be greater than 0".to_string(),
            source: None,
        });
    }
    
    let mut delay_ms = config.initial_delay_ms;
    let mut last_error = None;
    
    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e.clone());
                
                if attempt < config.max_attempts {
                    // Check if error is retryable
                    if !is_retryable_error(&e) {
                        log::error!("Non-retryable error encountered: {}", e);
                        return Err(e);
                    }
                    
                    log::warn!(
                        "Operation failed (attempt {}/{}), retrying in {}ms: {}",
                        attempt, config.max_attempts, delay_ms, e
                    );
                    
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    
                    // Calculate next delay with overflow protection
                    delay_ms = delay_ms
                        .saturating_mul((config.exponential_base as u64).max(1))
                        .min(config.max_delay_ms);
                }
            }
        }
    }
    
    // Return the last error we encountered
    Err(last_error.unwrap_or_else(|| EmbedError::Internal {
        message: "Retry loop completed without recording any error".to_string(),
        backtrace: None,
    }))
}
```

### Step 2: Improve error classification
```rust
// Enhance is_retryable_error function:
fn is_retryable_error(error: &EmbedError) -> bool {
    match error {
        EmbedError::Io { .. } => true,
        EmbedError::Database { .. } => true,
        EmbedError::Timeout { .. } => true,
        EmbedError::ResourceExhausted { .. } => true,
        EmbedError::Concurrency { .. } => true,
        // Explicitly non-retryable
        EmbedError::Configuration { .. } => false,
        EmbedError::Validation { .. } => false,
        EmbedError::PermissionDenied { .. } => false,
        EmbedError::NotFound { .. } => false,
        _ => false, // Conservative: don't retry unknown errors
    }
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check
```

## Next Task
task_021 - Validate thread safety in concurrent operations