# Phase 1: Stop the Bleeding - Compilation Fixes

**Duration**: 4-6 hours  
**Goal**: Make everything compile without errors  
**Success Metric**: `cargo build --all-features` succeeds

## Task 1.1: Fix Tantivy API Incompatibility (30 minutes)

### The Problem
```rust
// File: src/search/tantivy_search.rs:165
error[E0560]: struct `IndexSettings` has no field named `sort_by_field`
```

### The Fix
```rust
// DELETE THIS ENTIRE BLOCK:
let index_settings = IndexSettings {
    sort_by_field: None,  // This field doesn't exist in v0.24
    docstore_compression: Compressor::Lz4,
    docstore_blocksize: 16384,
};

// REPLACE WITH:
let index_settings = IndexSettings {
    docstore_compression: Compressor::Lz4,
    docstore_blocksize: 16384,
};
```

### Verification
```bash
cargo check --features tantivy
# Must compile without errors
```

## Task 1.2: Fix Binary Error Handling (15 minutes)

### The Problem
```rust
// File: src/bin/verify_symbols.rs:7
error[E0277]: the `?` operator requires proper return type
```

### The Fix
```rust
// CHANGE:
fn main() {
    // code using ? operator
}

// TO:
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // existing code stays the same
    Ok(())
}
```

Apply this fix to ALL binaries:
- `src/bin/verify_symbols.rs`
- `src/bin/tantivy_migrator.rs`
- `src/bin/test_persistence.rs`

### Verification
```bash
cargo build --bins --features tree-sitter
# All binaries must compile
```

## Task 1.3: Fix Sled Batch API (1 hour)

### The Problem
```rust
// File: src/storage/lancedb.rs:234
error[E0599]: no method named `new` found for struct `sled::Batch`
```

### Investigation Required
```bash
# Check actual Sled API version
cargo tree -p sled
# Look up correct batch API in Sled docs
```

### The Fix (Option A - Remove Migration)
```rust
// DELETE all migration code using sled::Batch
// This is dead code anyway since we're using LanceDB

impl LanceDBStorage {
    pub fn new(path: &Path, dimension: usize) -> Result<Self> {
        // Remove any sled-related initialization
        Ok(Self {
            db: Arc::new(Mutex::new(None)),
            table_name: "embeddings".to_string(),
            dimension,
            // DELETE: migration_db: Some(sled::open(path)?),
        })
    }
}
```

### The Fix (Option B - Update to Current API)
```rust
// Research current Sled batch API and update:
// Instead of: let batch = sled::Batch::new();
// Use whatever the current API provides
// This requires reading Sled documentation
```

### Verification
```bash
cargo check --features vectordb
# Must compile without Sled errors
```

## Task 1.4: Fix Missing Error Variants (30 minutes)

### The Problem
```rust
// Multiple files
error: no variant named 'InvalidVector' for enum 'StorageError'
```

### The Fix
```rust
// File: src/error.rs
// ADD these variants to StorageError enum:

#[derive(Error, Debug)]
pub enum StorageError {
    // ... existing variants ...
    
    #[error("Invalid vector dimensions: expected {expected}, got {actual}")]
    InvalidVector { expected: usize, actual: usize },
    
    #[error("Storage backend not initialized")]
    NotInitialized,
    
    #[error("Invalid storage operation: {0}")]
    InvalidOperation(String),
}
```

### Verification
```bash
cargo check --features "ml,vectordb"
# No missing variant errors
```

## Task 1.5: Fix Cache Type Mismatches (45 minutes)

### The Problem
```rust
// File: src/embedding/nomic.rs:747
error[E0308]: mismatched types - expected Result, found Option
```

### The Fix
```rust
// FIND:
match embedding_cache.get(&cache_key).await {
    Some(cached) => return Ok(cached),
    None => {}
}

// REPLACE WITH:
match embedding_cache.get(&cache_key).await {
    Ok(Some(cached)) => return Ok(cached),
    Ok(None) | Err(_) => {
        // Continue to generate embedding
    }
}
```

### Similar fixes needed in:
- `src/search/unified.rs:676-677` - Stats access
- `src/search/fusion.rs:114` - Integer types

### Verification
```bash
cargo check --features "ml"
# No type mismatch errors
```

## Task 1.6: Standardize Integer Types (30 minutes)

### The Problem
```rust
// Various files
error: expected u32, found u64
```

### The Fix
**Choose ONE type throughout the codebase: u64**

```rust
// Find all chunk_index fields and standardize:
pub struct ChunkMetadata {
    pub chunk_index: u64,  // NOT u32
    // ...
}

// Update all function signatures to match
fn process_chunk(index: u64) { /* ... */ }
```

### Files to check:
- `src/search/fusion.rs`
- `src/chunking/mod.rs`
- `src/storage/lancedb.rs`

### Verification
```bash
cargo check --all-features
# No integer type mismatches
```

## Task 1.7: Final Compilation Test (15 minutes)

### Run Complete Build
```bash
# Clean build to ensure no cached errors
cargo clean
cargo build --all-features
```

### Expected Result
- **ALL features must compile**
- Warnings are acceptable
- Zero compilation errors

### If Still Failing
Document exact error messages and file locations. Do NOT proceed to Phase 2 until compilation succeeds.

## Success Criteria Checklist

- [ ] Tantivy compiles without errors
- [ ] All binaries compile
- [ ] ML features compile
- [ ] VectorDB features compile
- [ ] Tree-sitter features compile
- [ ] Full system compiles with --all-features
- [ ] No "method not found" errors
- [ ] No "variant not found" errors
- [ ] No type mismatch errors

## Time Estimate

- Total: 4-6 hours
- Can be parallelized across multiple developers
- Must be completed sequentially per component

## Next Phase

Only proceed to Phase 2 (Core Search Repair) after ALL compilation errors are fixed.