# Task 1.016: Fix File Path Validation and Sanitization

**Time Estimate**: 8 minutes
**Dependencies**: None
**File(s) to Modify**: `src/search/tantivy_search.rs`, file handling modules

## Objective
Ensure all file path operations are validated and handle non-UTF8 paths properly.

## Success Criteria
- [ ] File paths are validated before use
- [ ] Non-UTF8 paths are handled explicitly
- [ ] No path traversal vulnerabilities
- [ ] Clear errors for invalid paths

## Instructions

### Step 1: Fix file path string conversion
```rust
// In tantivy_search.rs, replace lossy conversion:
// Before:
file_path.to_string_lossy().to_string()

// After:
file_path.to_str()
    .ok_or_else(|| anyhow::anyhow!("File path contains invalid UTF-8: {:?}", file_path))?
    .to_string()
```

### Step 2: Add path validation helper
```rust
// Add helper function for path validation:
fn validate_file_path(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("File does not exist: {:?}", path));
    }
    
    if !path.is_file() {
        return Err(anyhow::anyhow!("Path is not a file: {:?}", path));
    }
    
    // Ensure path contains valid UTF-8
    path.to_str()
        .ok_or_else(|| anyhow::anyhow!("File path contains invalid UTF-8: {:?}", path))?;
    
    Ok(())
}
```

### Step 3: Use validation in file operations
```rust
// Before indexing files:
pub async fn index_file(&mut self, file_path: &Path) -> Result<()> {
    validate_file_path(file_path)?;
    
    // Skip if not a code file
    if !self.is_code_file(file_path) {
        return Ok(());
    }
    
    // ... rest of implementation
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features tantivy
```

## Next Task
task_017 - Enhance error context and debugging information