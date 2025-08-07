# Task 1.012: Fix Similarity Calculation Edge Cases

**Time Estimate**: 8 minutes
**Dependencies**: Task 001 (InvalidVector error variant)
**File(s) to Modify**: `src/storage/simple_vectordb.rs`

## Objective
Ensure similarity calculations handle all edge cases (NaN, infinite, zero vectors) without fallbacks.

## Success Criteria
- [ ] NaN and infinite similarities are detected and rejected
- [ ] Zero vectors are handled explicitly
- [ ] No silent fallbacks or defaults
- [ ] Clear error messages for invalid similarity calculations

## Instructions

### Step 1: Enhance cosine_similarity function
```rust
// Improve cosine_similarity function around line 292:
fn cosine_similarity(a: &[f32], b: &[f32]) -> Result<f32, StorageError> {
    if a.len() != b.len() {
        return Err(StorageError::InvalidVector {
            reason: format!("Vector dimension mismatch: {} vs {}", a.len(), b.len()),
        });
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return Err(StorageError::InvalidVector {
            reason: "Cannot compute similarity with zero vector".to_string(),
        });
    }
    
    let similarity = dot_product / (norm_a * norm_b);
    
    if !similarity.is_finite() {
        return Err(StorageError::InvalidVector {
            reason: format!("Similarity calculation produced invalid result: {}", similarity),
        });
    }
    
    Ok(similarity)
}
```

### Step 2: Update search_similar usage
```rust
// Update the similarity calculation call:
let similarity = cosine_similarity(&query_embedding, &record.embedding)
    .map_err(|e| StorageError::SearchError(e.to_string()))?;
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features vectordb
```

## Next Task
task_013 - Add explicit error conversion implementations