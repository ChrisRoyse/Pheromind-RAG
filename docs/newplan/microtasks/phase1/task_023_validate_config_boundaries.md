# Task 1.023: Validate Configuration Parameter Boundaries

**Time Estimate**: 10 minutes
**Dependencies**: None
**File(s) to Modify**: `src/config/`, `src/error.rs`

## Objective
Add proper validation for all configuration parameters to prevent invalid states.

## Success Criteria
- [ ] All numeric parameters have reasonable bounds
- [ ] String parameters are validated for format
- [ ] Path parameters are checked for accessibility
- [ ] Clear error messages for invalid config

## Instructions

### Step 1: Add configuration validation
```rust
// In config module, add validation:
impl Config {
    pub fn validate(&self) -> Result<()> {
        // Validate embedding dimensions
        if let Some(dim) = self.embedding_dimensions {
            if dim == 0 || dim > 10000 {
                return Err(EmbedError::Configuration {
                    message: format!("Embedding dimensions must be between 1 and 10000, got {}", dim),
                    source: None,
                });
            }
        }
        
        // Validate cache capacity
        if let Some(capacity) = self.cache_capacity {
            if capacity == 0 {
                return Err(EmbedError::Configuration {
                    message: "Cache capacity must be greater than 0".to_string(),
                    source: None,
                });
            }
        }
        
        // Validate timeout values
        if let Some(timeout) = self.request_timeout_ms {
            if timeout < 100 || timeout > 300_000 { // 100ms to 5 minutes
                return Err(EmbedError::Configuration {
                    message: format!("Request timeout must be between 100ms and 300000ms, got {}ms", timeout),
                    source: None,
                });
            }
        }
        
        Ok(())
    }
}
```

### Step 2: Add RetryConfig validation
```rust
// In error.rs, enhance RetryConfig:
impl RetryConfig {
    pub fn new(max_attempts: u32, initial_delay_ms: u64, max_delay_ms: u64, exponential_base: f64) -> Result<Self> {
        if max_attempts == 0 {
            return Err(EmbedError::Configuration {
                message: "max_attempts must be greater than 0".to_string(),
                source: None,
            });
        }
        
        if initial_delay_ms == 0 {
            return Err(EmbedError::Configuration {
                message: "initial_delay_ms must be greater than 0".to_string(),
                source: None,
            });
        }
        
        if max_delay_ms < initial_delay_ms {
            return Err(EmbedError::Configuration {
                message: "max_delay_ms must be >= initial_delay_ms".to_string(),
                source: None,
            });
        }
        
        if exponential_base < 1.0 || exponential_base > 10.0 {
            return Err(EmbedError::Configuration {
                message: format!("exponential_base must be between 1.0 and 10.0, got {}", exponential_base),
                source: None,
            });
        }
        
        Ok(Self {
            max_attempts,
            initial_delay_ms,
            max_delay_ms,
            exponential_base,
        })
    }
}
```

### Step 3: Add cache capacity validation
```rust
// In bounded_cache.rs, enhance validation:
impl<K, V> BoundedCache<K, V> 
where 
    K: Hash + Eq + Clone,
    V: Clone,
{
    pub fn new(capacity: usize) -> Result<Self> {
        if capacity == 0 {
            return Err(EmbedError::Configuration {
                message: "Cache capacity must be greater than 0".to_string(),
                source: None,
            });
        }
        
        if capacity > 1_000_000 {
            return Err(EmbedError::Configuration {
                message: format!("Cache capacity {} exceeds maximum allowed (1,000,000)", capacity),
                source: None,
            });
        }
        
        let capacity = NonZeroUsize::new(capacity).unwrap(); // Safe after validation
        
        // ... rest of implementation
    }
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check
cargo test config_validation
```

## Next Task
task_024 - Fix floating point precision and NaN handling