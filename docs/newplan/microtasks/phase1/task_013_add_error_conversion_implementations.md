# Task 1.013: Add Explicit Error Conversion Implementations

**Time Estimate**: 10 minutes
**Dependencies**: Task 001 (InvalidVector error variant)
**File(s) to Modify**: `src/error.rs`, `src/storage/simple_vectordb.rs`

## Objective
Add proper From implementations for StorageError to EmbedError conversions.

## Success Criteria
- [ ] StorageError converts cleanly to EmbedError
- [ ] All error variants are properly handled
- [ ] No information loss in error conversion
- [ ] Error chain is preserved

## Instructions

### Step 1: Add From implementation in error.rs
```rust
// Add this implementation in error.rs after existing From implementations:
impl From<crate::storage::simple_vectordb::StorageError> for EmbedError {
    fn from(err: crate::storage::simple_vectordb::StorageError) -> Self {
        EmbedError::Storage {
            message: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}
```

### Step 2: Ensure StorageError implements required traits
```rust
// In simple_vectordb.rs, ensure StorageError has proper derives:
#[derive(Debug, Error)]
#[cfg(feature = "vectordb")]
pub enum StorageError {
    // ... variants
    #[error("Invalid vector: {reason}")]
    InvalidVector { reason: String },
}
```

### Step 3: Update error propagation
```rust
// In functions that return Result<T, StorageError>, ensure they can be converted:
pub async fn insert_embedding(&self, ...) -> Result<(), StorageError> {
    // Implementation that can use ? operator
}
```

### Step 4: Test error conversion
```rust
// Add a test to verify conversion works:
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_storage_error_conversion() {
        let storage_err = StorageError::InvalidVector {
            reason: "test".to_string(),
        };
        let embed_err: EmbedError = storage_err.into();
        assert!(matches!(embed_err, EmbedError::Storage { .. }));
    }
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features vectordb
cargo test test_storage_error_conversion --features vectordb
```

## Next Task
task_014 - Fix async trait implementations