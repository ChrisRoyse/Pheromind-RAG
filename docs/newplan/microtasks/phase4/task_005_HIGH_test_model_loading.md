# Task 005 - HIGH: Test Basic Model Loading Pipeline

## Priority: HIGH
## Estimated Time: 10 minutes
## Dependencies: Task 004

## Objective
Create a minimal test to verify the complete model loading pipeline works end-to-end.

## Current Issue
- Multiple compilation errors in loading code
- Need to identify exact failure points
- Validate each step of loading process

## Tasks
1. **Create minimal loader test** (6 min)
   ```rust
   // In src/ml/tests/model_loading.rs
   use crate::ml::EmbeddingService;
   use anyhow::Result;
   
   #[tokio::test]
   async fn test_minimal_model_loading() -> Result<()> {
       // Step 1: Initialize service
       let service = EmbeddingService::new().await?;
       
       // Step 2: Verify model is loaded
       assert!(service.is_loaded());
       
       // Step 3: Test basic properties
       assert_eq!(service.embedding_dimension(), 768);
       
       Ok(())
   }
   ```

2. **Test each loading stage** (3 min)
   - File opening and reading
   - GGUF parsing
   - Tensor extraction
   - Model construction

3. **Document failure points** (1 min)
   - Record exact error messages
   - Identify missing dependencies
   - Note performance issues

## Success Criteria
- [ ] Basic loading test compiles
- [ ] Can identify specific failure points
- [ ] Error messages are clear and actionable
- [ ] Loading stages are isolated
- [ ] Memory usage is reasonable

## Files to Create
- `src/ml/tests/model_loading.rs`
- `docs/model_loading_issues.md`

## Expected Results
- Test will likely fail initially
- Should provide clear error messages
- Will guide implementation priorities

## Validation
```bash
cargo test ml::tests::model_loading --verbose

# Capture and analyze all error output
# Document each compilation/runtime error
```

## Next Task
→ Task 006: Parse GGUF header structure