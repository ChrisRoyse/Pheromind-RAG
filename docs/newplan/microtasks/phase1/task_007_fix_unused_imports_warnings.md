# Task 1.007: Fix Unused Imports and Dead Code Warnings

**Time Estimate**: 10 minutes
**Dependencies**: None
**File(s) to Modify**: `src/observability/logging.rs`, `src/observability/metrics.rs`

## Objective
Remove unused imports and mark intentionally unused code to clean up warnings.

## Success Criteria
- [ ] No unused import warnings
- [ ] Intentionally unused code marked with #[allow(dead_code)]
- [ ] Clean compilation output
- [ ] No functional changes

## Instructions

### Step 1: Fix logging.rs unused imports
```rust
// Remove unused imports from line 279:
// Before:
use tracing::{info, debug, error};
// After:
// use tracing::{info, debug, error}; // Remove if truly unused
// OR use specific imports only
```

### Step 2: Fix unused variables
```rust
// In logging.rs line 317, prefix with underscore:
let _result = log_async_performance!("async_test_operation", {
    // ... code
});
```

### Step 3: Fix metrics.rs unused Result
```rust
// Line 633, handle the Result:
let _ = collector.record_cache_hit("test_cache");
```

### Step 4: Mark intentionally dead code
```rust
// For embedding_metrics field and safe_percentage function:
#[allow(dead_code)]
embedding_metrics: Arc<Mutex<EmbeddingMetrics>>,

#[allow(dead_code)]
pub fn safe_percentage(numerator: f64, denominator: f64) -> Option<f64> {
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --tests
```

## Troubleshooting
- Only remove imports that are truly unused
- Use #[allow(dead_code)] for intentionally unused items

## Next Task
task_008 - Fix unified search unused field warnings