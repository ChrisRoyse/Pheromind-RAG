# Task 1.003: Standardize Integer Types (u32 vs u64)

**Time Estimate**: 10 minutes
**Dependencies**: None
**File(s) to Modify**: Multiple files with type inconsistencies

## Objective
Standardize integer types throughout codebase to prevent casting errors and inconsistencies.

## Success Criteria
- [ ] Consistent use of u32 for chunk indices
- [ ] Consistent use of usize for line numbers
- [ ] Consistent use of u64 for timestamps and large counters
- [ ] No unnecessary type casting

## Instructions

### Step 1: Review chunk index types
```rust
// In error.rs, ensure chunk_index is u32:
#[error("Missing required similarity score for {file_path} chunk {chunk_index}")]
MissingSimilarityScore {
    file_path: String,
    chunk_index: u32,  // Ensure this is u32
},
```

### Step 2: Check EmbeddingRecord types
```rust
// In simple_vectordb.rs, ensure consistent types:
pub struct EmbeddingRecord {
    pub chunk_index: u32,  // Should be u32
    pub start_line: usize, // Should be usize
    pub end_line: usize,   // Should be usize
}
```

### Step 3: Review RetryConfig types
```rust
// In error.rs, ensure consistent timing types:
pub struct RetryConfig {
    pub max_attempts: u32,    // Keep as u32
    pub initial_delay_ms: u64,// Keep as u64
    pub max_delay_ms: u64,    // Keep as u64
}
```

### Step 4: Verify
```bash
cargo check --all-features
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --all-features
```

## Troubleshooting
- If type mismatches occur, add explicit casts with as keyword
- Ensure mathematical operations use compatible types

## Next Task
task_004 - Fix Tantivy IndexSettings API usage