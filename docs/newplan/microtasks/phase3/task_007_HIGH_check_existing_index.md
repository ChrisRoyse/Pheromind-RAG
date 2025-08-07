# Task 3.007: Check Existing Index Compatibility

**Time Estimate**: 8 minutes
**Priority**: HIGH
**Dependencies**: task_006
**File(s) to Modify**: None (investigation task)

## Objective
Determine if existing Tantivy indexes are compatible with v0.24 and plan migration strategy.

## Success Criteria
- [ ] Existing index status is determined
- [ ] Compatibility with v0.24 is tested
- [ ] Migration path is identified
- [ ] Index structure is documented
- [ ] Decision on rebuild vs migrate is made

## Instructions

### Step 1: Check for existing index
```bash
# Look for existing Tantivy index directories
ls -la | grep -i tantivy
find . -name "*.tantivy*" -type d
ls -la .tantivy_index/  # Common location
```

### Step 2: Try to open existing index
```rust
// Create diagnostic function
fn diagnose_existing_index() {
    use tantivy::Index;
    use std::path::Path;
    
    let possible_paths = vec![
        ".tantivy_index",
        ".tantivy",
        "tantivy_index",
        "index",
    ];
    
    for path in possible_paths {
        if Path::new(path).exists() {
            println!("Found index at: {}", path);
            
            match Index::open_in_dir(path) {
                Ok(index) => {
                    println!("✓ Index at {} is compatible with v0.24", path);
                    
                    // Try to get schema
                    let schema = index.schema();
                    println!("Schema fields: {:?}", schema.fields());
                },
                Err(e) => {
                    println!("✗ Index at {} is incompatible: {}", path, e);
                    println!("  -> Will need migration or rebuild");
                }
            }
        }
    }
}
```

### Step 3: Test index opening
```bash
# Create a simple test program
cargo run --bin diagnose_index --features tantivy
```

### Step 4: Document findings
```rust
// Add to diagnostics
fn check_index_size_and_content(path: &str) {
    use std::fs;
    
    if let Ok(entries) = fs::read_dir(path) {
        let mut total_size = 0;
        let mut file_count = 0;
        
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                    file_count += 1;
                }
            }
        }
        
        println!("Index {} contains {} files, total size: {} bytes", 
                path, file_count, total_size);
    }
}
```

## Terminal Commands
```bash
cd C:\code\embed
ls -la | grep -i tantivy
find . -name "*.tantivy*" -type d 2>/dev/null
cargo run --bin diagnose_index --features tantivy || echo "No diagnose binary yet"
```

## Expected Outcomes
1. **Compatible Index**: Can proceed with existing index
2. **Incompatible Index**: Need migration (next task)
3. **No Index**: Clean slate - proceed with new index

## Troubleshooting
- If multiple indexes found, identify the most recent/largest
- If permission errors, check file ownership
- Document any error messages for migration planning

## Next Task
task_008 - Create index migration strategy