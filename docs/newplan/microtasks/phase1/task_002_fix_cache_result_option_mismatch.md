# Task 1.002: Fix Result vs Option Cache Type Mismatches

**Time Estimate**: 8 minutes
**Dependencies**: None
**File(s) to Modify**: `src/cache/bounded_cache.rs`

## Objective
Standardize cache return types to use consistent Option<T> instead of mixing Result and Option.

## Success Criteria
- [ ] Cache get methods consistently return Option<T>
- [ ] No mixing of Result/Option in similar operations
- [ ] All callers handle the standardized return type
- [ ] Code compiles without warnings

## Instructions

### Step 1: Review cache methods
```rust
// Ensure these methods return Option<T>:
pub fn get(&self, key: &K) -> Option<V>
pub fn remove(&self, key: &K) -> Option<V>
```

### Step 2: Check EmbeddingCache consistency
```rust
// In EmbeddingCache, ensure:
pub fn get(&self, text: &str) -> Option<Vec<f32>>
// Not Result<Option<Vec<f32>>>
```

### Step 3: Verify SearchCache methods
```rust
// In SearchCache:
pub fn get(&self, query: &str, top_k: usize) -> Option<Vec<SearchResult>>
```

### Step 4: Verify
```bash
cargo check
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check
```

## Troubleshooting
- If callers expect Result, update them to handle Option
- Ensure error handling is moved to appropriate layers

## Next Task
task_003 - Standardize integer types (u32 vs u64)