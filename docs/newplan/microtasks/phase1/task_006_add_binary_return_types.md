# Task 1.006: Add Missing Binary Return Types

**Time Estimate**: 8 minutes
**Dependencies**: None
**File(s) to Modify**: Binary files in `src/bin/`

## Objective
Ensure all binary entry points have proper Result return types for error handling.

## Success Criteria
- [ ] All main functions return Result<()>
- [ ] Proper error propagation from binary entry points
- [ ] No unwrap() calls in main functions
- [ ] Clean error messages to users

## Instructions

### Step 1: Check tantivy_migrator.rs main function
```rust
// Ensure main function signature:
async fn main() -> Result<()> {
    // ... function body
    Ok(())
}
```

### Step 2: Check other binary files
```rust
// For each file in src/bin/, ensure:
async fn main() -> anyhow::Result<()> {
    // or
    Result<()> // if using crate::error::Result
}
```

### Step 3: Replace any unwrap() calls
```rust
// Replace:
something.unwrap()
// With:
something?
```

### Step 4: Verify
```bash
cargo check --all-targets
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --all-targets --all-features
```

## Troubleshooting
- If Result types conflict, use fully qualified paths
- Ensure error types are compatible

## Next Task
task_007 - Fix unused imports and dead code warnings