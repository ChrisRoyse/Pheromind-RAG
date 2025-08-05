# TDD Quick Reference for Phase 3

## **15-MINUTE MICRO TASK CYCLE**

### ⏱️ **Minute 0-5: RED**
```bash
# 1. Write test first
cargo test test_name -- --nocapture

# 2. See it fail
# ✗ test_name ... FAILED
```

### ⏱️ **Minute 5-10: GREEN**
```bash
# 1. Write minimal code
# 2. Make test pass
cargo test test_name

# ✓ test_name ... ok
```

### ⏱️ **Minute 10-15: REFACTOR**
```bash
# 1. Clean up code
# 2. Run all tests
cargo test

# test result: ok. X passed
```

## **ESSENTIAL COMMANDS**

```bash
# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run tests in module
cargo test git_watcher::

# Check compilation without running
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

## **TEST PATTERNS**

### **Basic Test Structure**
```rust
#[test]
fn test_feature() {
    // Arrange
    let input = setup();
    
    // Act
    let result = function_under_test(input);
    
    // Assert
    assert_eq!(result, expected);
}
```

### **Async Test**
```rust
#[tokio::test]
async fn test_async_feature() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### **Error Test**
```rust
#[test]
fn test_error_case() {
    let result = function_that_errors();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Expected error message"
    );
}
```

## **COMMIT MESSAGES**

```bash
# After RED phase
git commit -m "test(git-watcher): add failing test for feature X"

# After GREEN phase
git commit -m "feat(git-watcher): implement feature X to pass test"

# After REFACTOR phase
git commit -m "refactor(git-watcher): clean up feature X implementation"
```

## **COMMON ASSERTIONS**

```rust
// Equality
assert_eq!(actual, expected);
assert_ne!(actual, not_expected);

// Boolean
assert!(condition);
assert!(!condition);

// Option/Result
assert!(result.is_ok());
assert!(result.is_err());
assert!(option.is_some());
assert!(option.is_none());

// Panic
#[should_panic(expected = "error message")]
fn test_panic() {
    panic!("error message");
}
```

## **MOCK PATTERNS**

```rust
use mockall::*;

#[automock]
trait Database {
    fn get(&self, key: &str) -> Option<String>;
}

#[test]
fn test_with_mock() {
    let mut mock = MockDatabase::new();
    mock.expect_get()
        .with(eq("key"))
        .times(1)
        .returning(|_| Some("value".to_string()));
    
    // Use mock in test
}
```

## **TEMP FILE PATTERNS**

```rust
use tempfile::TempDir;

#[test]
fn test_with_temp_files() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    
    std::fs::write(&file_path, "content").unwrap();
    
    // Test with file
    
    // Temp dir cleaned up automatically
}
```

## **PROGRESS TRACKING**

After each micro task:
- [ ] Test written and failing (**RED**)
- [ ] Test passing (**GREEN**)
- [ ] Code refactored (**REFACTOR**)
- [ ] Committed to Git

## **TIME BOXING TIPS**

- **Stuck on RED?** Write simpler test
- **Stuck on GREEN?** Use hardcoded values first
- **Stuck on REFACTOR?** Move on, note for later

## **TROUBLESHOOTING**

```bash
# Clean build
cargo clean

# Update dependencies
cargo update

# Verbose test output
RUST_LOG=debug cargo test

# Single threaded tests
cargo test -- --test-threads=1
```

---

**Remember**: The goal is PROGRESS, not PERFECTION. Keep moving forward!