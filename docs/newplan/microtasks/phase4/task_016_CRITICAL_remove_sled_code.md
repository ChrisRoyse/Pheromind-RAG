# Task 016 - CRITICAL: Remove Sled Database Code Completely

## Priority: CRITICAL
## Estimated Time: 10 minutes
## Dependencies: Task 015

## Objective
Remove all Sled database references and code to eliminate compilation errors and prepare for LanceDB.

## Current Issue
- Sled causing compilation errors
- Incompatible with LanceDB approach
- Dead code causing confusion

## Tasks
1. **Remove Sled from dependencies** (2 min)
   ```toml
   # Remove from Cargo.toml
   # [dependencies]
   # sled = "0.34"  # REMOVE THIS LINE
   ```

2. **Remove Sled imports and usage** (6 min)
   ```rust
   // In src/storage/vector_store.rs - REMOVE these lines:
   // use sled::{Db, Tree};
   // use sled::Config;
   
   // Remove SledVectorStore struct completely:
   /*
   pub struct SledVectorStore {
       db: Db,
       embeddings_tree: Tree,
       metadata_tree: Tree,
   }
   
   impl SledVectorStore {
       // REMOVE ALL IMPLEMENTATION
   }
   */
   ```

3. **Update vector store trait** (2 min)
   ```rust
   // In src/storage/vector_store.rs
   use anyhow::Result;
   use crate::types::{EmbeddingVector, SearchResult};
   
   #[async_trait::async_trait]
   pub trait VectorStore: Send + Sync {
       async fn add_embedding(
           &self,
           id: String,
           embedding: EmbeddingVector,
           metadata: serde_json::Value,
       ) -> Result<()>;
       
       async fn search(
           &self,
           query_embedding: &EmbeddingVector,
           limit: usize,
           threshold: Option<f32>,
       ) -> Result<Vec<SearchResult>>;
       
       async fn get_embedding(&self, id: &str) -> Result<Option<EmbeddingVector>>;
       
       async fn delete_embedding(&self, id: &str) -> Result<bool>;
       
       async fn count(&self) -> Result<usize>;
       
       async fn close(&self) -> Result<()>;
   }
   
   // Remove all Sled-specific implementations
   ```

## Files to Clean

### Remove Sled references from:
- `Cargo.toml`
- `src/storage/vector_store.rs`
- `src/storage/mod.rs`
- `src/lib.rs`
- Any test files using Sled

### Update imports in:
- `src/main.rs`
- `src/api/handlers.rs`
- `src/search/engine.rs`

## Success Criteria
- [ ] No Sled references in Cargo.toml
- [ ] No Sled imports in any source files
- [ ] All Sled structs and implementations removed
- [ ] Code compiles without Sled errors
- [ ] VectorStore trait is clean and generic

## Validation
```bash
# Check for remaining Sled references
grep -r "sled" src/
grep -r "Sled" src/
grep "sled" Cargo.toml

# Should return no results

# Test compilation
cargo check
# Should compile without Sled-related errors
```

## Files to Modify
- `Cargo.toml`
- `src/storage/vector_store.rs`
- `src/storage/mod.rs`
- Any files with Sled imports

## Expected Outcome
- Cleaner codebase
- Faster compilation
- No Sled-related errors
- Ready for LanceDB integration

## Next Task
→ Task 017: Setup LanceDB connection and configuration