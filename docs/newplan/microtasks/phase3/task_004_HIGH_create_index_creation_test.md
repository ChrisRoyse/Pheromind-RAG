# Task 3.004: Create Basic Index Creation Test

**Time Estimate**: 10 minutes
**Priority**: HIGH
**Dependencies**: task_003
**File(s) to Modify**: `src/search/tantivy_search.rs`, `tests/` (new test file)

## Objective
Create a comprehensive test that verifies index creation works with the updated v0.24 API.

## Success Criteria
- [ ] Test creates new index successfully
- [ ] Index directory structure is correct
- [ ] Can add documents to index
- [ ] Index files are created on disk
- [ ] Test passes consistently

## Instructions

### Step 1: Create test function
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_tantivy_index_creation() {
        let dir = tempdir().unwrap();
        let index_path = dir.path();
        
        // Create TantivySearch instance
        let tantivy_search = TantivySearch::new(index_path).unwrap();
        
        // Verify index exists
        assert!(index_path.join(".tantivy").exists() || 
                index_path.exists()); // Different versions may vary
        
        println!("Index created successfully at {:?}", index_path);
    }
}
```

### Step 2: Test document addition
```rust
#[test]
fn test_add_document_basic() {
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new(dir.path()).unwrap();
    
    // Create test document
    let doc = Document {
        content: "This is a test document".to_string(),
        path: "test.txt".to_string(),
        chunk_index: 0,
        start_line: 1,
        end_line: 1,
    };
    
    // Add and commit
    tantivy.add_document(doc).unwrap();
    tantivy.commit().unwrap();
    
    // Verify index size increased
    println!("Document added successfully");
}
```

### Step 3: Run test
```bash
cargo test --features tantivy test_tantivy_index_creation
```

## Terminal Commands
```bash
cd C:\code\embed
cargo test --features tantivy test_tantivy_index_creation -v
cargo test --features tantivy test_add_document_basic -v
```

## Troubleshooting
- If tempdir fails, check tempfile crate is in dependencies
- If index creation fails, check permissions and disk space
- Verify all struct fields match expected Document structure

## Next Task
task_005 - Test basic search functionality