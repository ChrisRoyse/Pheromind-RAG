# Task 1.024: Fix Floating Point Precision and NaN Handling

**Time Estimate**: 9 minutes
**Dependencies**: Task 001 (InvalidVector error), Task 012 (similarity calculations)
**File(s) to Modify**: `src/storage/simple_vectordb.rs`, `src/cache/bounded_cache.rs`

## Objective
Ensure all floating point operations handle NaN, infinity, and precision issues correctly.

## Success Criteria
- [ ] All similarity calculations validate for NaN/infinity
- [ ] Floating point comparisons use appropriate epsilon
- [ ] No silent NaN propagation
- [ ] Clear errors for invalid floating point results

## Instructions

### Step 1: Add floating point validation helpers
```rust
// In simple_vectordb.rs, add validation helpers:
fn validate_f32_array(values: &[f32], context: &str) -> Result<(), StorageError> {
    for (i, &value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(StorageError::InvalidVector {
                reason: format!("{} contains invalid value at index {}: {} (NaN or infinite)", 
                               context, i, value),
            });
        }
    }
    Ok(())
}

fn safe_f32_compare(a: f32, b: f32) -> std::cmp::Ordering {
    // Handle NaN and infinity safely
    if a.is_nan() && b.is_nan() {
        std::cmp::Ordering::Equal
    } else if a.is_nan() {
        std::cmp::Ordering::Less
    } else if b.is_nan() {
        std::cmp::Ordering::Greater
    } else {
        a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
    }
}
```

### Step 2: Enhance cosine_similarity with validation
```rust
// Update cosine_similarity function:
fn cosine_similarity(a: &[f32], b: &[f32]) -> Result<f32, StorageError> {
    if a.len() != b.len() {
        return Err(StorageError::InvalidVector {
            reason: format!("Vector dimension mismatch: {} vs {}", a.len(), b.len()),
        });
    }
    
    // Validate input vectors
    validate_f32_array(a, "Vector A")?;
    validate_f32_array(b, "Vector B")?;
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    // Check for zero vectors
    if norm_a == 0.0 || norm_b == 0.0 {
        return Err(StorageError::InvalidVector {
            reason: "Cannot compute similarity with zero-magnitude vector".to_string(),
        });
    }
    
    let similarity = dot_product / (norm_a * norm_b);
    
    // Validate result
    if !similarity.is_finite() {
        return Err(StorageError::InvalidVector {
            reason: format!("Similarity calculation produced invalid result: {}", similarity),
        });
    }
    
    // Clamp to valid cosine similarity range [-1.0, 1.0]
    Ok(similarity.clamp(-1.0, 1.0))
}
```

### Step 3: Fix search_similar sorting
```rust
// Update search_similar to use safe comparison:
pub async fn search_similar(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<EmbeddingRecord>, StorageError> {
    // ... existing validation code
    
    let mut similarities = Vec::new();
    
    for result in self.db.scan_prefix(b"embedding:") {
        let (_, value) = result?;
        let record: EmbeddingRecord = serde_json::from_slice(&value)?;
        
        let similarity = cosine_similarity(&query_embedding, &record.embedding)?;
        similarities.push((similarity, record));
    }
    
    // Sort by similarity (descending) using safe comparison
    similarities.sort_by(|a, b| safe_f32_compare(b.0, a.0));
    
    // Take top results
    let results = similarities.into_iter()
        .take(limit)
        .map(|(_, record)| record)
        .collect();
    
    Ok(results)
}
```

### Step 4: Fix cache hit rate calculation
```rust
// In bounded_cache.rs, improve hit_rate calculation:
impl CacheStats {
    pub fn hit_rate(&self) -> Option<f64> {
        let total = self.hits + self.misses;
        if total == 0 {
            None // Cannot calculate hit rate with no operations
        } else {
            let rate = (self.hits as f64 / total as f64) * 100.0;
            if rate.is_finite() {
                Some(rate)
            } else {
                None // Invalid calculation
            }
        }
    }
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features vectordb
cargo test cosine_similarity
```

## Next Task
task_025 - Final compilation verification and cleanup