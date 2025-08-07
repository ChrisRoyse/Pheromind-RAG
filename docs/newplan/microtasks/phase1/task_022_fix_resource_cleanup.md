# Task 1.022: Fix Resource Cleanup and Disposal

**Time Estimate**: 8 minutes
**Dependencies**: None
**File(s) to Modify**: Resource management modules

## Objective
Ensure all resources are properly cleaned up and disposed of to prevent leaks.

## Success Criteria
- [ ] No resource leaks in normal operation
- [ ] Proper cleanup on error conditions
- [ ] Drop implementations where needed
- [ ] Clear resource lifecycle management

## Instructions

### Step 1: Add Drop implementation for caches
```rust
// In bounded_cache.rs, add proper cleanup:
impl<K, V> Drop for BoundedCache<K, V> 
where 
    K: Hash + Eq + Clone,
    V: Clone,
{
    fn drop(&mut self) {
        // Explicit cleanup - clear all entries
        {
            let mut cache = self.inner.write();
            cache.clear();
        }
        // RwLock and Arc will handle their own cleanup
    }
}
```

### Step 2: Ensure database cleanup
```rust
// In simple_vectordb.rs, add proper cleanup:
impl Drop for VectorStorage {
    fn drop(&mut self) {
        // Ensure database is properly flushed
        if let Err(e) = self.db.flush() {
            log::warn!("Failed to flush database on drop: {}", e);
        }
        // sled::Db handles its own cleanup
    }
}
```

### Step 3: Add cleanup methods for manual resource management
```rust
// Add explicit cleanup methods:
impl BoundedCache<K, V> {
    /// Explicitly clear all resources
    pub fn shutdown(&self) {
        self.clear();
        // Additional cleanup if needed
    }
}

impl VectorStorage {
    /// Explicit shutdown with error handling
    pub async fn shutdown(&self) -> Result<(), StorageError> {
        self.db.flush().map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
```

### Step 4: Review RAII patterns
```rust
// Ensure all resources follow RAII:
pub struct ResourceManager {
    resources: Vec<Box<dyn ResourceCleanup>>,
}

trait ResourceCleanup {
    fn cleanup(&mut self) -> Result<()>;
}

impl Drop for ResourceManager {
    fn drop(&mut self) {
        for resource in &mut self.resources {
            if let Err(e) = resource.cleanup() {
                log::error!("Resource cleanup failed: {}", e);
            }
        }
    }
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --all-features
```

## Next Task
task_023 - Validate configuration parameter boundaries