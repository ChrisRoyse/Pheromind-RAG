# Testing Patterns and Strategies

## Test Organization

### Unit Tests
- **Location**: In same file as code, in `#[cfg(test)]` module
- **Pattern**: At bottom of each .rs file
- **Find**: `search_for_pattern "#\[cfg\(test\)\]" relative_path="src"`

### Integration Tests
- **Location**: `/tests` directory
- **Files**:
  - `chunker_integration_tests.rs` - Chunking logic
  - `line_tracking_tests.rs` - Line number tracking
  - `nomic_embedding_tests.rs` - ML embeddings
  - `real_embedding_system_tests.rs` - Full system
  - `search_accuracy_test.rs` - Search quality
  - `compile_time_feature_tests.rs` - Feature combinations

### Benchmarks
- **Location**: `/benches` directory
- **Run**: `cargo bench`
- **File**: `line_tracking_bench.rs`

## Running Tests

### Basic Commands
```bash
# All tests
cargo test

# With all features
cargo test --all-features

# Specific test file
cargo test --test search_accuracy_test

# Specific test function
cargo test test_function_name

# With output
cargo test -- --nocapture

# Single threaded (for debugging)
cargo test -- --test-threads=1
```

### Feature-Specific Tests
```bash
# ML tests
cargo test --features "ml,vectordb"

# Symbol tests  
cargo test --features "tree-sitter"

# Full system
cargo test --features "full-system"
```

## Common Test Patterns

### Setup Pattern
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn setup() -> (TempDir, Config) {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::default();
        (temp_dir, config)
    }
    
    #[test]
    fn test_something() {
        let (temp_dir, config) = setup();
        // test code
    }
}
```

### Async Test Pattern
```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### Feature-Gated Tests
```rust
#[test]
#[cfg(feature = "ml")]
fn test_ml_feature() {
    // Only runs when ml feature is enabled
}
```

### Benchmark Pattern
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("function_name", |b| {
        b.iter(|| {
            function_to_benchmark(black_box(input))
        });
    });
}
```

## Test Data

### Creating Test Files
```rust
use std::fs;
use tempfile::TempDir;

let temp_dir = TempDir::new()?;
let test_file = temp_dir.path().join("test.rs");
fs::write(&test_file, "fn main() {}")?;
```

### Test Fixtures
- Look for test data in test functions
- Common patterns: 
  - Inline string literals for small tests
  - Temporary files for file operations
  - Mock structs for complex types

## Debugging Failed Tests

### Step 1: Identify Failure
```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo test failing_test

# Run with full backtrace
RUST_BACKTRACE=full cargo test failing_test
```

### Step 2: Add Debug Output
```rust
#[test]
fn debug_test() {
    dbg!(&variable);  // Debug print
    println!("Value: {:?}", value);  // With --nocapture
    assert_eq!(expected, actual);
}
```

### Step 3: Check Features
```bash
# Ensure correct features
cargo test --all-features failing_test
```

## Writing New Tests

### Guidelines
1. **Name clearly**: `test_specific_behavior_with_context`
2. **One assertion focus**: Test one thing per test
3. **Use helpers**: Extract common setup
4. **Clean up**: Use TempDir for file operations
5. **Document why**: Add comments for complex tests

### Test Template
```rust
#[test]
fn test_new_functionality() {
    // Arrange
    let input = prepare_test_data();
    
    // Act
    let result = function_under_test(input);
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_value);
}
```

## Coverage and Quality

### Check What's Tested
```
# Find untested public functions
search_for_pattern "^pub fn" relative_path="src"
# Then check for corresponding tests
```

### Find Test Gaps
```
# Find modules without tests
search_for_pattern "#\[cfg\(test\)\]" 
# Compare with module list
```

## Performance Testing

### Profiling Tests
```bash
# Build with profiling
cargo build --release --features "full-system"

# Run with profiling
cargo test --release -- --bench
```

### Memory Testing
Check for memory leaks in tests:
```rust
#[test]
fn test_no_memory_leak() {
    let initial = get_memory_usage();
    // Run operation many times
    for _ in 0..1000 {
        operation();
    }
    let final = get_memory_usage();
    assert!(final - initial < threshold);
}
```

## CI/CD Test Commands

For automation:
```bash
# Full test suite
cargo test --all-features --release

# Format check
cargo fmt -- --check

# Lint check
cargo clippy --all-features -- -D warnings

# Security audit
cargo audit
```