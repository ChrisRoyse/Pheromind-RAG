# Task 1.005: Fix Sled Batch API Usage

**Time Estimate**: 6 minutes
**Dependencies**: None
**File(s) to Modify**: `src/storage/simple_vectordb.rs`

## Objective
Ensure Sled Batch API is used correctly with proper initialization.

## Success Criteria
- [ ] `sled::Batch::new()` calls are correct
- [ ] Batch operations use proper API
- [ ] No compilation errors with vectordb feature
- [ ] Batch operations work as expected

## Instructions

### Step 1: Verify Batch::new() usage
```rust
// Check these lines are correct (around lines 171, 194, 211):
let mut batch = sled::Batch::new();
```

### Step 2: Check batch operations
```rust
// Ensure these methods exist and are called correctly:
batch.insert(key.as_bytes(), record_json);
batch.remove(key);
self.db.apply_batch(batch)?;
```

### Step 3: Test compilation
```bash
cargo check --features vectordb
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features vectordb
```

## Troubleshooting
- If Batch::new() doesn't exist, check sled version
- If apply_batch method changed, update to correct API

## Next Task
task_006 - Add missing binary return types