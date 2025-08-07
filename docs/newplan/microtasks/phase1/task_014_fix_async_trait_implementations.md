# Task 1.014: Fix Async Trait Implementations

**Time Estimate**: 7 minutes
**Dependencies**: None
**File(s) to Modify**: Files with async trait implementations

## Objective
Ensure all async trait implementations are properly declared and consistent.

## Success Criteria
- [ ] All async traits use #[async_trait] correctly
- [ ] Return types are consistent
- [ ] No Send/Sync issues in async contexts
- [ ] Clean compilation of async code

## Instructions

### Step 1: Check TextSearcher trait implementation
```rust
// In tantivy_search.rs, ensure proper async_trait usage:
#[async_trait]
impl crate::search::search_adapter::TextSearcher for TantivySearcher {
    async fn search(&self, query: &str) -> Result<Vec<ExactMatch>> {
        self.search(query).await
    }
    
    async fn index_file(&mut self, file_path: &Path) -> Result<()> {
        self.index_file(file_path).await
    }
    
    async fn clear_index(&mut self) -> Result<()> {
        self.clear_index().await
    }
}
```

### Step 2: Verify other async trait implementations
```rust
// Check for any other async trait implementations and ensure they have:
#[async_trait]
impl SomeTrait for SomeStruct {
    // async methods
}
```

### Step 3: Check for Send + Sync bounds
```rust
// If needed, ensure async traits have proper bounds:
#[async_trait]
pub trait SomeAsyncTrait: Send + Sync {
    async fn some_method(&self) -> Result<()>;
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --all-features
```

## Troubleshooting
- If Send/Sync errors occur, add appropriate bounds
- Ensure all async methods in traits are properly marked

## Next Task
task_015 - Validate memory management patterns