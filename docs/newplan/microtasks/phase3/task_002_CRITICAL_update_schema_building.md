# Task 3.002: Update Schema Building for v0.24 Compatibility

**Time Estimate**: 8 minutes
**Priority**: CRITICAL
**Dependencies**: task_001
**File(s) to Modify**: `src/search/tantivy_search.rs`

## Objective
Ensure schema building API is compatible with Tantivy v0.24 and verify all field types work correctly.

## Success Criteria
- [ ] Schema builder uses v0.24 API correctly
- [ ] All field types (TEXT, STORED, u64) are valid
- [ ] Schema builds without errors
- [ ] Field access patterns are correct

## Instructions

### Step 1: Verify schema builder API
```rust
// Check this pattern in build_schema() function:
let schema = Schema::builder()
    .add_text_field("body", TEXT | STORED)
    .add_text_field("path", STORED)
    .add_u64_field("chunk_index", STORED)
    .add_u64_field("start_line", STORED)
    .add_u64_field("end_line", STORED)
    .build();
```

### Step 2: Check field flag imports
```rust
// Ensure these imports are present:
use tantivy::schema::{STORED, TEXT};
use tantivy::schema::{Schema, Field};
```

### Step 3: Verify field access
```rust
// Check that field access uses correct pattern:
let body_field = schema.get_field("body").unwrap();
let path_field = schema.get_field("path").unwrap();
```

### Step 4: Test schema creation
```bash
cargo test --features tantivy test_schema_building
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features tantivy
cargo test --features tantivy test_schema_building -v
```

## Troubleshooting
- If field flag imports fail, check Tantivy v0.24 docs
- If schema.get_field fails, verify field names match exactly
- Ensure all necessary traits are imported

## Next Task
task_003 - Update query parser for v0.24 API