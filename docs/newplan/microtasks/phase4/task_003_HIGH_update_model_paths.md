# Task 003 - HIGH: Update Model Paths in Configuration

## Priority: HIGH
## Estimated Time: 10 minutes
## Dependencies: Task 002

## Objective
Update all hardcoded model paths and configuration to point to the correct GGUF model file.

## Current Issue
- Hardcoded paths in multiple files
- Environment variables not properly set
- Configuration inconsistencies

## Tasks
1. **Update config constants** (4 min)
   ```rust
   // In src/config.rs
   pub const DEFAULT_MODEL_PATH: &str = "models/nomic-embed-text-v1.5-Q4_K_M.gguf";
   pub const MODEL_EMBEDDING_DIM: usize = 768;
   pub const MODEL_MAX_SEQUENCE_LENGTH: usize = 8192;
   pub const MODEL_VOCAB_SIZE: usize = 30522;
   ```

2. **Update environment handling** (3 min)
   ```rust
   // In src/ml/mod.rs
   pub fn get_model_path() -> String {
       std::env::var("EMBED_MODEL_PATH")
           .unwrap_or_else(|_| DEFAULT_MODEL_PATH.to_string())
   }
   ```

3. **Fix embedding service** (3 min)
   - Update EmbeddingService constructor
   - Fix model loading path
   - Update error messages

## Success Criteria
- [ ] All hardcoded paths removed
- [ ] Environment variable handling added
- [ ] Default path points to downloaded model
- [ ] Configuration is consistent across files
- [ ] Model parameters match GGUF metadata

## Files to Modify
- `src/config.rs`
- `src/ml/mod.rs`
- `src/ml/embedding_service.rs`
- `src/storage/vector_store.rs`

## Validation
```bash
# Test configuration loading
export EMBED_MODEL_PATH="models/nomic-embed-text-v1.5-Q4_K_M.gguf"
cargo test config::tests::test_model_path

# Test default fallback
unset EMBED_MODEL_PATH
cargo test config::tests::test_default_model_path
```

## Next Task
→ Task 004: Check model format compatibility with Candle