# Task 1.021: Validate Thread Safety in Concurrent Operations

**Time Estimate**: 9 minutes
**Dependencies**: None
**File(s) to Modify**: `src/cache/bounded_cache.rs`, concurrent modules

## Objective
Ensure all concurrent operations are properly synchronized and thread-safe.

## Success Criteria
- [ ] No data races in shared state
- [ ] Proper lock ordering to prevent deadlocks
- [ ] Send + Sync bounds are correctly applied
- [ ] Thread-safe operations are properly documented

## Instructions

### Step 1: Review BoundedCache thread safety
```rust
// Ensure proper trait bounds:
impl<K, V> BoundedCache<K, V> 
where 
    K: Hash + Eq + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    // All methods should be safe for concurrent access
}

// Add explicit Send + Sync implementation if needed:
unsafe impl<K, V> Send for BoundedCache<K, V> 
where 
    K: Hash + Eq + Clone + Send + Sync,
    V: Clone + Send + Sync,
{}

unsafe impl<K, V> Sync for BoundedCache<K, V> 
where 
    K: Hash + Eq + Clone + Send + Sync,
    V: Clone + Send + Sync,
{}
```

### Step 2: Check lock ordering consistency
```rust
// Ensure consistent lock ordering to prevent deadlocks:
pub fn get(&self, key: &K) -> Option<V> {
    // Always acquire locks in the same order
    let mut cache = self.inner.write();
    let mut stats = self.stats.write();
    
    // ... implementation
}

pub fn put(&self, key: K, value: V) -> Option<V> {
    // Same lock order as get()
    let mut cache = self.inner.write();
    let mut stats = self.stats.write();
    
    // ... implementation
}
```

### Step 3: Minimize lock scope
```rust
// Ensure locks are held for minimal time:
pub fn stats(&self) -> CacheStats {
    // Short-lived lock, clone data and return
    self.stats.read().clone()
}

pub fn cleanup_expired(&self) -> usize {
    let ttl = match self.ttl {
        Some(ttl) => ttl,
        None => return 0,
    };
    
    // Minimize lock scope
    {
        let mut cache = self.inner.write();
        let mut stats = self.stats.write();
        // ... do work quickly
    } // Locks released here
    
    removed
}
```

### Step 4: Add concurrency test
```rust
// Add test to verify thread safety:
#[cfg(test)]
mod concurrency_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    
    #[test]
    fn test_concurrent_access() {
        let cache = Arc::new(BoundedCache::<String, i32>::new(100).unwrap());
        let mut handles = vec![];
        
        // Spawn multiple threads
        for i in 0..10 {
            let cache_clone = cache.clone();
            handles.push(thread::spawn(move || {
                for j in 0..100 {
                    let key = format!("key_{}_{}", i, j);
                    cache_clone.put(key.clone(), i * 100 + j);
                    cache_clone.get(&key);
                }
            }));
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify cache is in valid state
        assert!(cache.len() <= 100); // Respects capacity
    }
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo test concurrency_tests
```

## Next Task
task_022 - Fix resource cleanup and disposal