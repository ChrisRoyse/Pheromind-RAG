# Task 3.001: Fix Tantivy API Incompatibility (sort_by_field Removed)

**Time Estimate**: 10 minutes
**Priority**: CRITICAL
**Dependencies**: None
**File(s) to Modify**: `src/search/tantivy_search.rs`

## Objective
Fix compilation error caused by `sort_by_field` being removed in Tantivy v0.24. This is blocking all Tantivy functionality.

## Success Criteria
- [ ] `sort_by_field` reference removed from IndexSettings
- [ ] Code compiles without Tantivy API errors
- [ ] Index creation works with v0.24 API
- [ ] No breaking API changes remain

## Instructions

### Step 1: Remove sort_by_field from IndexSettings
```rust
// Find around line 165 in src/search/tantivy_search.rs
// REMOVE this old code:
let index_settings = IndexSettings {
    sort_by_field: None,  // DELETE THIS LINE
    docstore_compression: Compressor::Lz4,
    docstore_blocksize: 16384,
};

// REPLACE with v0.24 compatible code:
let index_settings = IndexSettings {
    docstore_compression: Compressor::Lz4,
    docstore_blocksize: 16384,
};
```

### Step 2: Verify compilation
```bash
cargo check --features tantivy
```

### Step 3: Test index creation
```bash
cargo test --features tantivy test_tantivy_index_creation
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features tantivy
cargo test --features tantivy -v
```

## Troubleshooting
- If other Tantivy API errors appear, they need separate tasks
- Check Tantivy documentation for v0.24 changes
- Ensure all imports are compatible with v0.24

## Next Task
task_002 - Update schema building for v0.24 compatibility