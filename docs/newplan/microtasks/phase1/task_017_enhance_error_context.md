# Task 1.017: Enhance Error Context and Debugging Information

**Time Estimate**: 10 minutes
**Dependencies**: None
**File(s) to Modify**: `src/error.rs`, various modules with error handling

## Objective
Improve error messages with more context for debugging without exposing sensitive information.

## Success Criteria
- [ ] Error messages contain sufficient debugging context
- [ ] No sensitive information in error messages
- [ ] Error chain is preserved
- [ ] Consistent error formatting

## Instructions

### Step 1: Enhance error context helpers
```rust
// In error.rs, improve context methods:
impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    fn context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|e| EmbedError::Internal {
            message: format!("{context}"),
            backtrace: Some(format!("Caused by: {e}")),
        })
    }
}
```

### Step 2: Add structured error information
```rust
// Add helper for creating detailed errors:
impl EmbedError {
    pub fn with_context<T: fmt::Display>(
        error_type: Self,
        context: T,
        file: &str,
        line: u32,
    ) -> Self {
        match error_type {
            EmbedError::Internal { message, .. } => EmbedError::Internal {
                message: format!("{message} (at {file}:{line}: {context})"),
                backtrace: Some(format!("Context: {context}")),
            },
            other => other,
        }
    }
}
```

### Step 3: Use enhanced context in operations
```rust
// In file operations, add better context:
fs::read_to_string(file_path)
    .with_context(|| format!("Failed to read file for indexing: {:?}", file_path))?;
```

### Step 4: Test error messages
```rust
// Add test for error context:
#[cfg(test)]
mod tests {
    #[test]
    fn test_error_context_preservation() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let result: std::result::Result<(), _> = Err(io_error);
        let embed_error = result.context("Reading configuration file");
        
        assert!(embed_error.is_err());
        let error_msg = embed_error.unwrap_err().to_string();
        assert!(error_msg.contains("Reading configuration file"));
    }
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo test test_error_context_preservation
```

## Next Task
task_018 - Fix schema compatibility checking