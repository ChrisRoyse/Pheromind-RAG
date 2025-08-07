# REAL FIX PLAN - Make It Actually Work

## Critical Issue #1: Config Initialization (30 min)
**Problem**: "Configuration not initialized. Call Config::init() first."

**Fix**:
```rust
// In main.rs or before any component usage:
Config::init()?;
```

## Critical Issue #2: Nomic Runtime (1 hour)
**Problem**: "Cannot start a runtime from within a runtime"

**Fix**:
```rust
// Remove nested tokio runtime creation
// Use existing runtime context
```

## Critical Issue #3: Integration Test (1 hour)
Create simple working example:
```rust
#[tokio::test]
async fn test_actual_integration() {
    Config::init().unwrap();
    let searcher = UnifiedSearcher::new(...).await.unwrap();
    
    // Test each component
    let ast_results = searcher.search_symbols("function").await.unwrap();
    let bm25_results = searcher.search_bm25("test").await.unwrap();
    let tantivy_results = searcher.search_tantivy("fuzzy~").await.unwrap();
    
    assert!(!ast_results.is_empty());
    assert!(!bm25_results.is_empty());
    assert!(!tantivy_results.is_empty());
}
```

## Critical Issue #4: Nomic Model Path (30 min)
**Problem**: Model file not found or wrong path

**Fix**:
```rust
const MODEL_PATH: &str = "./models/nomic-embed-text-v1.5.Q4_K_M.gguf";
// Or use environment variable
let model_path = env::var("NOMIC_MODEL_PATH")
    .unwrap_or_else(|_| MODEL_PATH.to_string());
```

## Success Criteria:
1. Config initializes properly
2. All 4 components return results
3. No runtime panics
4. Integration test passes

## Total Time: 2-3 hours

**This is the ACTUAL work needed, not 40+ hours of phantom fixes.**