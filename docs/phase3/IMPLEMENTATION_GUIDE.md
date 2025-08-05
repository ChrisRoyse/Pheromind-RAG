# Phase 3 Implementation Guide: Git File Watching with TDD

## **QUICK START**

This guide provides a structured approach to implementing Phase 3 using Test-Driven Development (TDD) with the micro tasks defined in `PHASE3_MICRO_TASKS_BREAKDOWN.md`.

## **PROJECT SETUP**

### **1. Directory Structure**
```
embed/
├── src/
│   ├── git_watcher/
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   ├── updater.rs
│   │   ├── batch.rs
│   │   ├── watch.rs
│   │   └── state.rs
│   └── tests/
│       └── git_watcher_tests.rs
├── Cargo.toml
└── docs/
    └── phase3/
```

### **2. Dependencies to Add**
```toml
[dependencies]
anyhow = "1.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
indicatif = "0.17"
glob = "0.3"
thiserror = "1.0"

[dev-dependencies]
tempfile = "3.0"
mockall = "0.11"
```

## **TDD WORKFLOW FOR EACH MICRO TASK**

### **Step 1: RED Phase (5 minutes)**
1. Create/update test file
2. Write failing test that defines expected behavior
3. Run test to confirm it fails
4. Commit test

### **Step 2: GREEN Phase (5 minutes)**
1. Write minimal implementation to pass test
2. Run test to confirm it passes
3. Don't worry about code quality yet
4. Commit implementation

### **Step 3: REFACTOR Phase (5 minutes)**
1. Improve code structure and readability
2. Extract common patterns
3. Add documentation
4. Ensure tests still pass
5. Commit refactored code

## **IMPLEMENTATION TEMPLATES**

### **Test Template (RED Phase)**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_git_watcher_creation() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();
        
        // Act
        let result = GitWatcher::new(repo_path.clone());
        
        // Assert
        assert!(result.is_ok());
        let watcher = result.unwrap();
        assert_eq!(watcher.repo_path, repo_path);
    }
}
```

### **Implementation Template (GREEN Phase)**
```rust
use std::path::PathBuf;
use anyhow::Result;

pub struct GitWatcher {
    pub repo_path: PathBuf,
}

impl GitWatcher {
    pub fn new(repo_path: PathBuf) -> Result<Self> {
        Ok(Self { repo_path })
    }
}
```

### **Refactored Template (REFACTOR Phase)**
```rust
use std::path::PathBuf;
use anyhow::{Result, Context};

/// Monitors a Git repository for file changes
#[derive(Debug)]
pub struct GitWatcher {
    repo_path: PathBuf,
}

impl GitWatcher {
    /// Creates a new GitWatcher for the specified repository path
    ///
    /// # Arguments
    /// * `repo_path` - Path to the Git repository root
    ///
    /// # Returns
    /// * `Result<GitWatcher>` - The watcher instance or an error
    pub fn new(repo_path: PathBuf) -> Result<Self> {
        // Verify the path exists
        if !repo_path.exists() {
            return Err(anyhow::anyhow!("Repository path does not exist"));
        }
        
        Ok(Self { repo_path })
    }
    
    /// Gets the repository path
    pub fn repo_path(&self) -> &PathBuf {
        &self.repo_path
    }
}
```

## **TASK IMPLEMENTATION ORDER**

### **Week 3, Day 1: Core Git Integration**
- **Morning**: Tasks 021.1 - 021.6 (Git Status Parser basics)
- **Afternoon**: Tasks 021.7 - 021.12 (File filtering and integration)

### **Week 3, Day 2: Vector Database Updates**
- **Morning**: Tasks 022.1 - 022.8 (Basic update operations)
- **Afternoon**: Tasks 022.9 - 022.16 (Advanced features and integration)

### **Week 3, Day 3: Batch Processing**
- **Morning**: Tasks 023.1 - 023.6 (Batch infrastructure)
- **Afternoon**: Tasks 023.7 - 023.12 (Performance and integration)

### **Week 3, Day 4: Watch Command & Persistence**
- **Morning**: Tasks 024.1 - 024.8 (Watch command)
- **Afternoon**: Tasks 025.1 - 025.8 (State persistence)

### **Week 3, Day 5: Production Features**
- **Morning**: Tasks 026.1 - 027.8 (Progress & Ignore patterns)
- **Afternoon**: Tasks 028.1 - 030.4 (Error recovery & completion)

## **TESTING STRATEGY**

### **1. Unit Tests**
- Test each component in isolation
- Use mocks for external dependencies
- Focus on edge cases and error conditions

### **2. Integration Tests**
- Test component interactions
- Use temporary Git repositories
- Verify end-to-end workflows

### **3. Performance Tests**
- Measure operation timing
- Test with large repositories
- Verify memory usage constraints

## **COMMON PATTERNS**

### **Error Handling Pattern**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitWatcherError {
    #[error("Git command failed: {0}")]
    GitCommand(String),
    
    #[error("File IO error: {0}")]
    FileIO(#[from] std::io::Error),
    
    #[error("Database error: {0}")]
    Database(String),
}
```

### **Async Pattern**
```rust
use tokio::sync::Mutex;
use std::sync::Arc;

pub struct AsyncUpdater {
    inner: Arc<Mutex<VectorUpdater>>,
}

impl AsyncUpdater {
    pub async fn update_file(&self, path: &Path) -> Result<()> {
        let mut updater = self.inner.lock().await;
        updater.update_file(path).await
    }
}
```

### **Progress Reporting Pattern**
```rust
use indicatif::{ProgressBar, ProgressStyle};

fn create_progress_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40} {pos}/{len} {msg}")
            .unwrap()
    );
    pb
}
```

## **DEBUGGING TIPS**

### **1. Git Command Debugging**
```rust
// Add debug output for git commands
if log::log_enabled!(log::Level::Debug) {
    log::debug!("Running git command: {:?}", command);
    log::debug!("Git output: {}", String::from_utf8_lossy(&output.stdout));
}
```

### **2. Test Isolation**
```rust
// Use separate temp directories for each test
#[test]
fn test_isolated() {
    let temp_dir = TempDir::new().unwrap();
    // Test runs in isolation
}
```

### **3. Async Test Debugging**
```rust
#[tokio::test]
async fn test_async_operation() {
    // Use tokio::test for async tests
    let result = async_function().await;
    assert!(result.is_ok());
}
```

## **COMPLETION CHECKLIST**

For each micro task:
- [ ] RED: Failing test written and committed
- [ ] GREEN: Test passes with minimal code
- [ ] REFACTOR: Code cleaned up and documented
- [ ] All tests still pass
- [ ] No compiler warnings
- [ ] Code follows Rust conventions

For each major task (021-030):
- [ ] All micro tasks completed
- [ ] Integration test passes
- [ ] Performance meets requirements
- [ ] Documentation updated
- [ ] Code reviewed

## **NEXT STEPS**

After completing Phase 3:
1. Run full integration tests
2. Benchmark performance
3. Update project documentation
4. Prepare for Phase 4 (MCP Server integration)