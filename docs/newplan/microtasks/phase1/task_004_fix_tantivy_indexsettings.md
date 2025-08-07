# Task 1.004: Fix Tantivy IndexSettings API Usage

**Time Estimate**: 7 minutes
**Dependencies**: None
**File(s) to Modify**: `src/search/tantivy_search.rs`

## Objective
Update Tantivy IndexSettings configuration to use correct API for current version.

## Success Criteria
- [ ] IndexSettings uses correct field names
- [ ] No deprecated field usage (sort_by_field)
- [ ] Proper compression settings
- [ ] Code compiles with tantivy feature

## Instructions

### Step 1: Fix IndexSettings configuration
```rust
// Replace the IndexSettings block around line 164:
let index_settings = IndexSettings {
    docstore_compression: tantivy::store::Compressor::Lz4,
    docstore_blocksize: 16384,
    // Remove sort_by_field - it doesn't exist in current API
};
```

### Step 2: Remove sort_by_field comment
```rust
// Remove this line:
// sort_by_field: None,  // No pre-sorting - explicit configuration
```

### Step 3: Verify tantivy compilation
```bash
cargo check --features tantivy
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features tantivy
```

## Troubleshooting
- If IndexSettings fields are different, check tantivy docs
- If compression enum changed, verify correct variant name

## Next Task
task_005 - Fix Sled Batch API usage