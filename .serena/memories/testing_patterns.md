# Testing Patterns (Verified)

## Test Organization

### Integration Tests (`/tests/`)
19 test files covering different aspects:
```
tests/
├── search_validation/              # Search validation subdirectory
├── bm25_integration_tests.rs       # BM25 search testing
├── chunker_integration_tests.rs    # Code chunking tests
├── compile_time_feature_tests.rs   # Feature flag compilation
├── comprehensive_search_test.py    # Python-based search tests
├── config_search_backend_tests.rs  # Configuration testing
├── core_tests.rs                   # Core functionality
├── embedding_performance_benchmark.rs # Performance benchmarks
├── fallback_prevention_test.rs     # Fallback behavior
├── line_tracking_tests.rs          # Line number tracking
├── nomic_embedding_tests.rs        # Nomic model tests
├── production_embedding_verification.rs # Production embedding
├── production_q4km_verification.rs # Q4KM model verification
├── real_embedding_system_tests.rs  # Real embedding tests
├── safety_audit.rs                 # Safety checks
├── search_accuracy_test.rs         # Search accuracy metrics
├── symbol_indexing_tests.rs        # Symbol extraction
├── test-better-sqlite.js           # JavaScript SQLite tests
├── test-claude-flow-memory.js      # Claude Flow memory tests
└── tree_sitter_feature_tests.rs    # Tree-sitter features
```

### Unit Tests
Located within source files using `#[cfg(test)]` modules

### Binary Tests (`/src/bin/`)
Standalone test executables:
- test_persistence.rs
- test_project_scoping.rs
- test_unified_project_scope.rs

## Common Test Patterns

### 1. Async Test Pattern
```rust
#[tokio::test]
async fn test_async_operation() {
    // Setup
    let config = Config::default();
    let searcher = UnifiedSearcher::new(config).await.unwrap();
    
    // Execute
    let results = searcher.search("query").await.unwrap();
    
    // Assert
    assert!(!results.is_empty());
    assert_eq!(results[0].score, expected_score);
}
```

### 2. Feature-Gated Tests
```rust
#[cfg(feature = "ml")]
#[test]
fn test_embedding_generation() {
    // Test only runs when ml feature is enabled
    let embedder = NomicEmbedder::new().unwrap();
    let embedding = embedder.embed("test").unwrap();
    assert!(embedding.len() > 0); // Verify embedding was generated
}
```

### 3. Error Testing Pattern
```rust
#[test]
fn test_error_handling() {
    let result = risky_operation();
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(err.to_string().contains("expected error"));
}
```

### 4. Temporary Directory Pattern
```rust
use tempfile::TempDir;

#[test]
fn test_with_temp_storage() {
    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        vector_db_path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    
    // Test operations
    // temp_dir automatically cleaned up
}
```

### 5. Mock/Stub Pattern
```rust
// Common in storage tests
struct MockStorage {
    data: HashMap<String, Vec<u8>>,
}

impl Storage for MockStorage {
    async fn store(&mut self, key: String, value: Vec<u8>) -> Result<()> {
        self.data.insert(key, value);
        Ok(())
    }
}
```

## Test Utilities

### Common Setup Functions
```rust
fn setup_test_config() -> Config {
    Config {
        chunk_size: 100,
        batch_size: 10,
        max_search_results: 5,
        ..Default::default()
    }
}

async fn setup_test_searcher() -> UnifiedSearcher {
    let config = setup_test_config();
    UnifiedSearcher::new(config).await.unwrap()
}
```

### Assertion Helpers
```rust
fn assert_search_result_valid(result: &SearchResult) {
    assert!(!result.content.is_empty());
    assert!(result.score >= 0.0 && result.score <= 1.0);
    assert!(!result.file_path.is_empty());
}
```

## Performance Testing

### Benchmark Pattern (benches/)
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_search(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let searcher = runtime.block_on(setup_test_searcher());
    
    c.bench_function("search_performance", |b| {
        b.to_async(&runtime).iter(|| async {
            searcher.search(black_box("test query")).await
        });
    });
}

criterion_group!(benches, benchmark_search);
criterion_main!(benches);
```

## Test Data Management

### Test Databases
- `.test_bm25_db/` - BM25 test data
- `test_accuracy_db/` - Accuracy testing data
- Temporary databases created per test

### Test Fixtures
```rust
const TEST_CODE: &str = r#"
fn main() {
    println!("Hello, world!");
}
"#;

const TEST_QUERIES: &[&str] = &[
    "hello world",
    "main function",
    "println macro",
];
```

## Coverage Areas

### Core Functionality Tests
- Configuration loading and validation
- Error propagation and handling
- Async operation correctness

### Search Tests
- BM25 ranking accuracy
- Tantivy fuzzy matching
- Unified search result fusion
- Symbol extraction and indexing

### Storage Tests
- LanceDB operations
- Vector storage and retrieval
- Database migrations
- Concurrent access safety

### Embedding Tests
- Model loading
- Dimension consistency
- Caching behavior
- Performance benchmarks

## Running Tests

### Commands
```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test bm25_integration_tests

# Run with specific features
cargo test --features "ml,vectordb"

# Run benchmarks
cargo bench

# Run with output
cargo test -- --nocapture
```

### Test Environment Variables
```bash
RUST_LOG=debug cargo test
RUST_BACKTRACE=1 cargo test
```

## Common Test Issues and Solutions

### Issue: "Test databases not cleaned up"
**Solution**: Use TempDir or cleanup in drop impl

### Issue: "Tests fail intermittently"
**Solution**: Check for race conditions, use serial test execution

### Issue: "Feature-gated tests not running"
**Solution**: Enable features explicitly in test command

### Issue: "Async test hangs"
**Solution**: Add timeout, check for deadlocks
```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[timeout(Duration::from_secs(10))]
async fn test_with_timeout() {
    // Test code
}
```