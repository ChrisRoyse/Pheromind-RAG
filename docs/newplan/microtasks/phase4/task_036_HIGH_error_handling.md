# Task 036 - HIGH: Implement Error Handling Throughout Pipeline

## Priority: HIGH
## Estimated Time: 10 minutes
## Dependencies: Task 035

## Objective
Implement comprehensive error handling and recovery mechanisms throughout the embedding pipeline.

## Current Issue
- Inconsistent error handling across components
- Need graceful degradation and recovery
- Better error messages for debugging

## Tasks
1. **Define comprehensive error types** (4 min)
   ```rust
   // In src/ml/errors.rs
   use thiserror::Error;
   use std::fmt;
   
   #[derive(Error, Debug)]
   pub enum EmbeddingError {
       #[error("Tokenization failed: {message}")]
       TokenizationError { message: String },
       
       #[error("Model loading failed: {message}")]
       ModelLoadError { message: String },
       
       #[error("Tensor conversion failed: {message}")]
       TensorConversionError { message: String },
       
       #[error("Transformer forward pass failed at layer {layer}: {message}")]
       TransformerError { layer: usize, message: String },
       
       #[error("Pooling operation failed: {message}")]
       PoolingError { message: String },
       
       #[error("Normalization failed: {message}")]
       NormalizationError { message: String },
       
       #[error("Dimension validation failed: expected {expected:?}, got {actual:?}")]
       DimensionError { expected: Vec<usize>, actual: Vec<usize> },
       
       #[error("Batch processing failed: {message}")]
       BatchProcessingError { message: String },
       
       #[error("Memory allocation failed: requested {requested_mb}MB, available {available_mb}MB")]
       MemoryError { requested_mb: f64, available_mb: f64 },
       
       #[error("Device error: {message}")]
       DeviceError { message: String },
       
       #[error("Timeout error: operation took {elapsed_ms}ms, limit was {limit_ms}ms")]
       TimeoutError { elapsed_ms: u64, limit_ms: u64 },
       
       #[error("Configuration error: {message}")]
       ConfigError { message: String },
       
       #[error("I/O error: {message}")]
       IoError { message: String },
       
       #[error("Quantization error: {message}")]
       QuantizationError { message: String },
       
       #[error("Cache error: {message}")]
       CacheError { message: String },
       
       #[error("Validation error: {message}")]
       ValidationError { message: String },
   }
   
   impl EmbeddingError {
       pub fn is_recoverable(&self) -> bool {
           match self {
               EmbeddingError::TimeoutError { .. } => true,
               EmbeddingError::MemoryError { .. } => true,
               EmbeddingError::BatchProcessingError { .. } => true,
               EmbeddingError::CacheError { .. } => true,
               _ => false,
           }
       }
       
       pub fn error_code(&self) -> &'static str {
           match self {
               EmbeddingError::TokenizationError { .. } => "E001",
               EmbeddingError::ModelLoadError { .. } => "E002",
               EmbeddingError::TensorConversionError { .. } => "E003",
               EmbeddingError::TransformerError { .. } => "E004",
               EmbeddingError::PoolingError { .. } => "E005",
               EmbeddingError::NormalizationError { .. } => "E006",
               EmbeddingError::DimensionError { .. } => "E007",
               EmbeddingError::BatchProcessingError { .. } => "E008",
               EmbeddingError::MemoryError { .. } => "E009",
               EmbeddingError::DeviceError { .. } => "E010",
               EmbeddingError::TimeoutError { .. } => "E011",
               EmbeddingError::ConfigError { .. } => "E012",
               EmbeddingError::IoError { .. } => "E013",
               EmbeddingError::QuantizationError { .. } => "E014",
               EmbeddingError::CacheError { .. } => "E015",
               EmbeddingError::ValidationError { .. } => "E016",
           }
       }
   }
   
   pub type EmbeddingResult<T> = Result<T, EmbeddingError>;
   ```

2. **Implement error recovery strategies** (4 min)
   ```rust
   // Error recovery utilities
   pub struct ErrorRecovery {
       max_retries: usize,
       retry_delay_ms: u64,
   }
   
   impl ErrorRecovery {
       pub fn new(max_retries: usize, retry_delay_ms: u64) -> Self {
           Self { max_retries, retry_delay_ms }
       }
       
       pub async fn with_retry<F, T, Fut>(
           &self,
           operation: F,
           operation_name: &str,
       ) -> EmbeddingResult<T>
       where
           F: Fn() -> Fut,
           Fut: std::future::Future<Output = EmbeddingResult<T>>,
       {
           let mut last_error = None;
           
           for attempt in 0..=self.max_retries {
               match operation().await {
                   Ok(result) => {
                       if attempt > 0 {
                           println!("Operation '{}' succeeded on attempt {}", operation_name, attempt + 1);
                       }
                       return Ok(result);
                   },
                   Err(e) => {
                       last_error = Some(e.clone());
                       
                       if !e.is_recoverable() || attempt == self.max_retries {
                           break;
                       }
                       
                       println!(
                           "Operation '{}' failed on attempt {} ({}), retrying in {}ms: {}",
                           operation_name,
                           attempt + 1,
                           e.error_code(),
                           self.retry_delay_ms,
                           e
                       );
                       
                       tokio::time::sleep(tokio::time::Duration::from_millis(
                           self.retry_delay_ms * (2_u64.pow(attempt as u32))
                       )).await;
                   }
               }
           }
           
           Err(last_error.unwrap())
       }
   }
   
   // Graceful degradation strategies
   pub enum DegradationStrategy {
       ReduceBatchSize,
       SimplifyModel,
       UseCache,
       FallbackToDefault,
   }
   
   pub struct GracefulDegradation {
       strategies: Vec<DegradationStrategy>,
   }
   
   impl GracefulDegradation {
       pub fn new(strategies: Vec<DegradationStrategy>) -> Self {
           Self { strategies }
       }
       
       pub fn handle_error(&self, error: &EmbeddingError) -> Option<DegradationStrategy> {
           match error {
               EmbeddingError::MemoryError { .. } => Some(DegradationStrategy::ReduceBatchSize),
               EmbeddingError::TimeoutError { .. } => Some(DegradationStrategy::SimplifyModel),
               EmbeddingError::BatchProcessingError { .. } => Some(DegradationStrategy::ReduceBatchSize),
               EmbeddingError::TransformerError { .. } => Some(DegradationStrategy::UseCache),
               _ => Some(DegradationStrategy::FallbackToDefault),
           }
       }
   }
   ```

3. **Add error context and logging** (2 min)
   ```rust
   use std::collections::HashMap;
   use serde_json::Value;
   
   #[derive(Debug, Clone)]
   pub struct ErrorContext {
       pub operation: String,
       pub timestamp: chrono::DateTime<chrono::Utc>,
       pub input_info: HashMap<String, Value>,
       pub system_info: HashMap<String, Value>,
       pub performance_metrics: HashMap<String, f64>,
   }
   
   impl ErrorContext {
       pub fn new(operation: &str) -> Self {
           Self {
               operation: operation.to_string(),
               timestamp: chrono::Utc::now(),
               input_info: HashMap::new(),
               system_info: HashMap::new(),
               performance_metrics: HashMap::new(),
           }
       }
       
       pub fn with_input_info(mut self, key: &str, value: Value) -> Self {
           self.input_info.insert(key.to_string(), value);
           self
       }
       
       pub fn with_system_info(mut self, key: &str, value: Value) -> Self {
           self.system_info.insert(key.to_string(), value);
           self
       }
       
       pub fn with_metric(mut self, key: &str, value: f64) -> Self {
           self.performance_metrics.insert(key.to_string(), value);
           self
       }
   }
   
   pub struct ErrorLogger;
   
   impl ErrorLogger {
       pub fn log_error(error: &EmbeddingError, context: Option<&ErrorContext>) {
           let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
           
           eprintln!("[{}] ERROR [{}]: {}", timestamp, error.error_code(), error);
           
           if let Some(ctx) = context {
               eprintln!("  Operation: {}", ctx.operation);
               eprintln!("  Timestamp: {}", ctx.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
               
               if !ctx.input_info.is_empty() {
                   eprintln!("  Input Info: {:?}", ctx.input_info);
               }
               
               if !ctx.system_info.is_empty() {
                   eprintln!("  System Info: {:?}", ctx.system_info);
               }
               
               if !ctx.performance_metrics.is_empty() {
                   eprintln!("  Metrics: {:?}", ctx.performance_metrics);
               }
           }
           
           eprintln!("");
       }
       
       pub fn log_recovery_attempt(
           error: &EmbeddingError,
           strategy: &DegradationStrategy,
           attempt: usize,
       ) {
           println!(
               "[RECOVERY] Attempting strategy {:?} for error {} (attempt {})",
               strategy, error.error_code(), attempt
           );
       }
   }
   
   // Error conversion utilities
   impl From<candle_core::Error> for EmbeddingError {
       fn from(error: candle_core::Error) -> Self {
           EmbeddingError::TensorConversionError {
               message: format!("Candle error: {}", error),
           }
       }
   }
   
   impl From<std::io::Error> for EmbeddingError {
       fn from(error: std::io::Error) -> Self {
           EmbeddingError::IoError {
               message: format!("I/O error: {}", error),
           }
       }
   }
   
   impl From<serde_json::Error> for EmbeddingError {
       fn from(error: serde_json::Error) -> Self {
           EmbeddingError::ConfigError {
               message: format!("JSON error: {}", error),
           }
       }
   }
   
   impl From<tokio::time::error::Elapsed> for EmbeddingError {
       fn from(_error: tokio::time::error::Elapsed) -> Self {
           EmbeddingError::TimeoutError {
               elapsed_ms: 0, // Would need to track actual elapsed time
               limit_ms: 0,   // Would need to track timeout limit
           }
       }
   }
   ```

## Success Criteria
- [ ] Comprehensive error types defined
- [ ] Error recovery mechanisms work
- [ ] Graceful degradation implemented
- [ ] Error logging provides useful info
- [ ] Error context tracking works
- [ ] Conversion from other error types

## Files to Create
- `src/ml/errors.rs`

## Files to Modify
- `src/ml/mod.rs`
- All ML pipeline components (add error handling)

## Validation
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_recoverability() {
        let timeout_error = EmbeddingError::TimeoutError {
            elapsed_ms: 5000,
            limit_ms: 3000,
        };
        assert!(timeout_error.is_recoverable());
        
        let model_error = EmbeddingError::ModelLoadError {
            message: "Model not found".to_string(),
        };
        assert!(!model_error.is_recoverable());
    }
    
    #[tokio::test]
    async fn test_error_recovery() {
        let recovery = ErrorRecovery::new(3, 100);
        let mut attempt_count = 0;
        
        let result = recovery.with_retry(
            || async {
                attempt_count += 1;
                if attempt_count < 3 {
                    Err(EmbeddingError::TimeoutError {
                        elapsed_ms: 5000,
                        limit_ms: 3000,
                    })
                } else {
                    Ok("Success")
                }
            },
            "test_operation",
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(attempt_count, 3);
    }
}
```

## Next Task
→ Task 037: Optimize embedding generation performance