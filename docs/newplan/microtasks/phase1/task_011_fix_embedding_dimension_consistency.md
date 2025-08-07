# Task 1.011: Fix Embedding Dimension Consistency

**Time Estimate**: 9 minutes
**Dependencies**: None
**File(s) to Modify**: `src/config/`, `src/storage/simple_vectordb.rs`, `src/cache/bounded_cache.rs`

## Objective
Ensure embedding dimensions are consistently handled without fallback defaults.

## Success Criteria
- [ ] Embedding dimensions are explicitly configured
- [ ] No hardcoded dimension fallbacks
- [ ] Dimension mismatches cause clear errors
- [ ] All embedding operations validate dimensions

## Instructions

### Step 1: Review Config::embedding_dimensions()
```rust
// In config files, ensure method returns Result:
impl Config {
    pub fn embedding_dimensions() -> Result<usize> {
        // Must fail if not configured, no defaults
        self.embedding_config.dimensions
            .ok_or_else(|| EmbedError::Configuration {
                message: "Embedding dimensions not configured".to_string(),
                source: None,
            })
    }
}
```

### Step 2: Check dimension validation in storage
```rust
// In simple_vectordb.rs, ensure strict validation:
if embedding.len() != expected_dim {
    return Err(StorageError::InvalidInput(
        format!("Embedding must be {}-dimensional, got {}", expected_dim, embedding.len())
    ));
}
```

### Step 3: Verify cache dimension handling
```rust
// In bounded_cache.rs, ensure EmbeddingCache validates:
if embedding.len() != self.dimension {
    return Err(EmbedError::Validation {
        field: "embedding".to_string(),
        reason: format!("Expected {} dimensions, got {}", self.dimension, embedding.len()),
        value: Some(embedding.len().to_string()),
    });
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features vectordb
```

## Troubleshooting
- If Config methods don't exist, implement them properly
- Ensure all embedding operations check dimensions

## Next Task
task_012 - Fix similarity calculation edge cases