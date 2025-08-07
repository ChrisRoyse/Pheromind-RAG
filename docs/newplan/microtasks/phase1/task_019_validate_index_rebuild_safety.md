# Task 1.019: Validate Index Rebuild Safety

**Time Estimate**: 9 minutes
**Dependencies**: None
**File(s) to Modify**: `src/search/tantivy_search.rs`

## Objective
Ensure index rebuild operations are safe and don't cause data corruption or race conditions.

## Success Criteria
- [ ] Index rebuilds are atomic operations
- [ ] No partial corruption during rebuild
- [ ] Proper backup and recovery procedures
- [ ] Clear error handling for rebuild failures

## Instructions

### Step 1: Improve remove_index_files safety
```rust
// Enhance remove_index_files function around line 205:
fn remove_index_files(index_path: &Path) -> Result<()> {
    if !index_path.exists() {
        return Ok(()); // Already removed
    }
    
    if !index_path.is_dir() {
        return Err(anyhow::anyhow!("Index path is not a directory: {:?}", index_path));
    }
    
    // Create backup name for safety
    let backup_path = index_path.with_extension("backup");
    
    // Move existing index to backup before removal
    if backup_path.exists() {
        fs::remove_dir_all(&backup_path)
            .with_context(|| format!("Failed to remove old backup: {:?}", backup_path))?;
    }
    
    fs::rename(index_path, &backup_path)
        .with_context(|| format!("Failed to backup index before removal: {:?}", index_path))?;
    
    // Remove the backup (original index)
    fs::remove_dir_all(&backup_path)
        .with_context(|| format!("Failed to remove index backup: {:?}", backup_path))?;
    
    Ok(())
}
```

### Step 2: Make rebuild_index atomic
```rust
// Improve rebuild_index function around line 633:
pub async fn rebuild_index(&mut self) -> Result<()> {
    if let Some(index_path) = &self.index_path {
        let schema = self.index.schema();
        
        // Create new index in temporary location first
        let temp_path = index_path.with_extension("rebuilding");
        
        if temp_path.exists() {
            fs::remove_dir_all(&temp_path)
                .with_context(|| format!("Failed to clean temp rebuild path: {:?}", temp_path))?;
        }
        
        // Create new index in temp location
        let new_index = Self::create_or_open_index(&temp_path, schema)
            .with_context(|| format!("Failed to create new index during rebuild: {:?}", temp_path))?;
        
        // Atomically replace old index with new one
        Self::remove_index_files(index_path)?;
        fs::rename(&temp_path, index_path)
            .with_context(|| format!("Failed to move rebuilt index to final location: {:?}", index_path))?;
        
        // Update our index reference
        self.index = new_index;
        
        println!("✨ Index rebuilt at {:?}", index_path);
    } else {
        // For in-memory, just clear
        self.clear_index().await?;
    }
    
    Ok(())
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features tantivy
```

## Next Task
task_020 - Fix retry mechanism edge cases