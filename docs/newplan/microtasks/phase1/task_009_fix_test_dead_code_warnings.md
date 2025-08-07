# Task 1.009: Fix Test File Dead Code Warnings

**Time Estimate**: 8 minutes
**Dependencies**: None
**File(s) to Modify**: Test files with unused functions

## Objective
Fix dead code warnings in test files for clean compilation.

## Success Criteria
- [ ] No dead code warnings in test files
- [ ] Test functions are used or marked appropriately
- [ ] Clean test compilation
- [ ] Tests still pass

## Instructions

### Step 1: Fix search_accuracy_test.rs
```rust
// Mark unused field:
struct TestEnvironment {
    searcher: UnifiedSearcher,
    #[allow(dead_code)]
    vectortest_path: PathBuf,
}
```

### Step 2: Fix cosine_similarity functions
```rust
// In multiple test files, mark unused functions:
#[allow(dead_code)]
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    // ... implementation
}

#[allow(dead_code)]
fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    // ... implementation
}
```

### Step 3: Find all test files with warnings
```bash
# Check which test files have warnings
cargo check --tests 2>&1 | grep "warning.*never used"
```

### Step 4: Verify cleanup
```bash
cargo check --tests
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --tests --all-features
```

## Troubleshooting
- Only mark functions as dead_code if they're truly unused
- Consider if test helper functions should be in a common module

## Next Task
task_010 - Validate configuration error handling