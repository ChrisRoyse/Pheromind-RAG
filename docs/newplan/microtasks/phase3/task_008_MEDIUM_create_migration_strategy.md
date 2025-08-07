# Task 3.008: Create Index Migration Strategy

**Time Estimate**: 10 minutes
**Priority**: MEDIUM
**Dependencies**: task_007
**File(s) to Modify**: `src/migration/tantivy_migrator.rs` (new file)

## Objective
Create a robust migration system for incompatible Tantivy indexes, with fallback to rebuild.

## Success Criteria
- [ ] Migration utility handles version compatibility
- [ ] Backup of old index before migration
- [ ] Graceful fallback to rebuild if migration fails
- [ ] Progress reporting during migration
- [ ] Validation of migrated index

## Instructions

### Step 1: Create migration module
```rust
// src/migration/tantivy_migrator.rs
use std::path::{Path, PathBuf};
use std::fs;
use tantivy::Index;
use crate::search::tantivy_search::TantivySearch;

pub struct TantivyMigrator {
    old_path: PathBuf,
    new_path: PathBuf,
}

impl TantivyMigrator {
    pub fn new(old_path: impl AsRef<Path>, new_path: impl AsRef<Path>) -> Self {
        Self {
            old_path: old_path.as_ref().to_path_buf(),
            new_path: new_path.as_ref().to_path_buf(),
        }
    }
    
    pub fn migrate(&self) -> Result<MigrationResult, MigrationError> {
        println!("Starting Tantivy index migration...");
        println!("  From: {:?}", self.old_path);
        println!("  To: {:?}", self.new_path);
        
        // Step 1: Backup old index
        self.backup_old_index()?;
        
        // Step 2: Try direct compatibility
        if let Ok(result) = self.try_direct_migration() {
            return Ok(result);
        }
        
        // Step 3: Fallback to rebuild
        println!("Direct migration failed, rebuilding index...");
        self.rebuild_index()
    }
}
```

### Step 2: Implement migration methods
```rust
impl TantivyMigrator {
    fn backup_old_index(&self) -> Result<(), MigrationError> {
        let backup_path = self.old_path.with_extension("backup");
        
        if self.old_path.exists() {
            println!("Backing up old index to {:?}", backup_path);
            fs_extra::dir::copy(&self.old_path, &backup_path, &CopyOptions::new())
                .map_err(|e| MigrationError::BackupFailed(e.to_string()))?;
        }
        
        Ok(())
    }
    
    fn try_direct_migration(&self) -> Result<MigrationResult, MigrationError> {
        // Try to open old index with new API
        match Index::open_in_dir(&self.old_path) {
            Ok(_index) => {
                // Index is compatible, just move it
                println!("Index is compatible with v0.24, moving...");
                fs::rename(&self.old_path, &self.new_path)
                    .map_err(|e| MigrationError::MoveFailed(e.to_string()))?;
                    
                Ok(MigrationResult::DirectMove)
            },
            Err(e) => {
                println!("Index incompatible: {}", e);
                Err(MigrationError::IncompatibleIndex(e.to_string()))
            }
        }
    }
    
    fn rebuild_index(&self) -> Result<MigrationResult, MigrationError> {
        println!("Rebuilding index from scratch...");
        
        // Remove old incompatible index
        if self.new_path.exists() {
            fs::remove_dir_all(&self.new_path)
                .map_err(|e| MigrationError::CleanupFailed(e.to_string()))?;
        }
        
        // Create new index
        let _tantivy = TantivySearch::new(&self.new_path)
            .map_err(|e| MigrationError::RebuildFailed(e.to_string()))?;
            
        println!("New index created successfully");
        Ok(MigrationResult::Rebuilt)
    }
}
```

### Step 3: Add error types
```rust
#[derive(Debug)]
pub enum MigrationError {
    BackupFailed(String),
    IncompatibleIndex(String),
    MoveFailed(String),
    CleanupFailed(String),
    RebuildFailed(String),
}

#[derive(Debug)]
pub enum MigrationResult {
    DirectMove,
    Rebuilt,
    NoMigrationNeeded,
}
```

### Step 4: Create migration binary
```rust
// src/bin/migrate_tantivy.rs
use embed::migration::tantivy_migrator::TantivyMigrator;

fn main() {
    let migrator = TantivyMigrator::new(".tantivy_index", "tantivy_index_new");
    
    match migrator.migrate() {
        Ok(result) => println!("Migration completed: {:?}", result),
        Err(e) => eprintln!("Migration failed: {:?}", e),
    }
}
```

## Terminal Commands
```bash
cd C:\code\embed
mkdir -p src/migration
cargo check --features tantivy
cargo run --bin migrate_tantivy --features tantivy
```

## Troubleshooting
- If fs_extra is missing, add to Cargo.toml
- If backup fails, check disk space and permissions
- Ensure old index path exists before migration

## Next Task
task_009 - Optimize index performance settings