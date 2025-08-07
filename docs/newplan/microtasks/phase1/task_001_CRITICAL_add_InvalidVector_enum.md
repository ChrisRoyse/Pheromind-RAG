# Task 1.001: Add Missing InvalidVector Error Variant

**Time Estimate**: 5 minutes
**Dependencies**: None
**File(s) to Modify**: `src/storage/simple_vectordb.rs`

## Objective
Fix compilation error by adding missing `InvalidVector` variant to `StorageError` enum.

## Success Criteria
- [ ] `StorageError::InvalidVector` variant exists
- [ ] Variant has proper structure matching usage
- [ ] Display implementation handles new variant
- [ ] Code compiles without errors

## Instructions

### Step 1: Add InvalidVector variant to enum
```rust
// In the StorageError enum, add after InvalidInput:
InvalidVector {
    reason: String,
},
```

### Step 2: Update Display implementation
```rust
// In the Display implementation, add case:
StorageError::InvalidVector(msg) => write!(f, "Invalid vector: {}", msg.reason),
```

### Step 3: Verify
```bash
cargo check --features vectordb
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features vectordb
```

## Troubleshooting
- If other enum variants break, ensure consistent structure
- Check that all match arms are updated

## Next Task
task_002 - Fix Result vs Option cache mismatches