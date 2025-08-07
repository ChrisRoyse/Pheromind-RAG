# Phase 6: Testing Reality Check - No Fake Tests

**Duration**: 1 day  
**Goal**: Verify everything actually works with real data  
**Success Metric**: All tests pass with real components, no mocking

## The Testing Philosophy

**NO MOCKING**: Every test must use real components  
**NO SHORTCUTS**: Tests must reflect actual usage  
**NO LIES**: If a test fails, the feature is broken

## Task 6.1: Create Real Test Data (1 hour)

### Setup Test Corpus

```bash
# Create test data directory
mkdir -p test_data/code
mkdir -p test_data/docs
mkdir -p test_data/mixed

# Copy real code files
cp -r src test_data/code/
cp -r tests test_data/code/

# Create documentation files
echo "# User Guide
This is a comprehensive user guide for the embed search system.
It includes information about search methods and usage." > test_data/docs/user_guide.md

echo "# API Reference
Complete API documentation for all search methods.
Includes BM25, Tantivy, Native, and Semantic search." > test_data/docs/api_reference.md

# Create mixed content
echo "The quick brown fox jumps over the lazy dog.
This is a test document with common English words.
It should be found by multiple search methods." > test_data/mixed/test1.txt
```

## Task 6.2: BM25 Reality Test (1 hour)

### Test with Real Documents

```rust
// File: tests/bm25_reality_test.rs

#[test]
fn test_bm25_with_real_code() {
    let mut engine = BM25Engine::new(1.2, 0.75);
    
    // Index all Rust files in src
    for entry in WalkDir::new("src").into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension() == Some(OsStr::new("rs")) {
            let content = fs::read_to_string(entry.path()).unwrap();
            let doc_id = entry.path().to_string_lossy().to_string();
            engine.add_document(doc_id, content).unwrap();
        }
    }
    
    // CRITICAL: Build the index
    engine.build_index().unwrap();
    println!("Indexed {} documents", engine.document_count());
    
    // Search for actual code terms
    let test_cases = vec![
        ("impl", 10),      // Should find many implementations
        ("Result", 10),    // Common Rust type
        ("async fn", 5),   // Async functions
        ("todo!()", 1),    // Specific macro
    ];
    
    for (query, min_expected) in test_cases {
        let results = engine.search(query, 100);
        assert!(
            results.len() >= min_expected,
            "Query '{}' returned {} results, expected at least {}",
            query, results.len(), min_expected
        );
        
        // Verify results actually contain the query term
        for result in &results[..min_expected.min(results.len())] {
            let doc = &engine.documents[&result.document_id];
            assert!(
                doc.content.contains(query),
                "Result doesn't contain query term '{}'",
                query
            );
        }
    }
}
```

## Task 6.3: Tantivy Reality Test (1 hour)

### Test with Real Index

```rust
// File: tests/tantivy_reality_test.rs

#[test]
fn test_tantivy_with_real_index() {
    let index_path = PathBuf::from(".tantivy_test_real");
    
    // Clean previous test index
    if index_path.exists() {
        fs::remove_dir_all(&index_path).unwrap();
    }
    
    let mut tantivy = TantivySearch::new(&index_path).unwrap();
    
    // Index real files
    let mut doc_count = 0;
    for entry in WalkDir::new("test_data").into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let content = fs::read_to_string(entry.path()).unwrap_or_continue();
            tantivy.add_document(Document {
                path: entry.path().to_string_lossy().to_string(),
                content,
                chunk_index: 0,
                start_line: 1,
                end_line: 1,
            }).unwrap();
            doc_count += 1;
        }
    }
    
    tantivy.commit().unwrap();
    println!("Indexed {} documents in Tantivy", doc_count);
    
    // Test exact search
    let exact_results = tantivy.search("embed search", 10).unwrap();
    assert!(!exact_results.is_empty(), "Should find 'embed search' in docs");
    
    // Test fuzzy search
    let fuzzy_results = tantivy.search_fuzzy("embedd", 1).unwrap();
    assert!(!fuzzy_results.is_empty(), "Fuzzy search should find 'embed' variants");
    
    // Test phrase search
    let phrase_results = tantivy.search("\"quick brown fox\"", 10).unwrap();
    assert!(!phrase_results.is_empty(), "Should find exact phrase");
    
    // Clean up
    fs::remove_dir_all(&index_path).unwrap();
}
```

## Task 6.4: ML/Semantic Reality Test (2 hours)

### Test with Real Embeddings

```rust
// File: tests/semantic_reality_test.rs

#[tokio::test]
async fn test_semantic_search_with_real_model() {
    // Check if model exists
    let model_path = Path::new("models/nomic-embed-text-v1.5.Q4_K_M.gguf");
    if !model_path.exists() {
        eprintln!("Skipping test: Model not found at {:?}", model_path);
        eprintln!("Download from: https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF");
        return;
    }
    
    // Initialize real components
    let embedder = NomicEmbedder::new().await.unwrap();
    let storage = LanceDBStorage::new("test_vectors_real", 768).await.unwrap();
    
    // Test documents with semantic meaning
    let documents = vec![
        ("rust_doc", "Rust is a systems programming language focused on safety"),
        ("python_doc", "Python is a high-level interpreted programming language"),
        ("ml_doc", "Machine learning uses neural networks for pattern recognition"),
        ("web_doc", "Web development involves HTML, CSS, and JavaScript"),
        ("db_doc", "Databases store structured data with SQL queries"),
    ];
    
    // Generate and store real embeddings
    for (id, content) in &documents {
        println!("Generating embedding for: {}", id);
        let start = Instant::now();
        let embedding = embedder.generate_embedding(content).await.unwrap();
        println!("Generated in {:?}", start.elapsed());
        
        assert_eq!(embedding.len(), 768, "Wrong embedding dimension");
        
        // Verify normalization (L2 norm should be ~1.0)
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01, "Embedding not normalized: {}", norm);
        
        storage.add_embedding(id, content, &embedding).await.unwrap();
    }
    
    // Test semantic similarity
    let test_queries = vec![
        ("programming languages", vec!["rust_doc", "python_doc"]),
        ("artificial intelligence", vec!["ml_doc"]),
        ("frontend development", vec!["web_doc"]),
        ("data storage", vec!["db_doc"]),
    ];
    
    for (query, expected_top) in test_queries {
        println!("Searching for: {}", query);
        let query_embedding = embedder.generate_embedding(query).await.unwrap();
        let results = storage.search(&query_embedding, 3).await.unwrap();
        
        assert!(!results.is_empty(), "No results for query: {}", query);
        
        // Check if expected documents appear in top results
        let result_ids: Vec<String> = results.iter().map(|r| r.id.clone()).collect();
        for expected_id in expected_top {
            assert!(
                result_ids.contains(&expected_id.to_string()),
                "Expected '{}' in top results for '{}', got: {:?}",
                expected_id, query, result_ids
            );
        }
    }
    
    // Clean up
    fs::remove_dir_all("test_vectors_real").ok();
}
```

## Task 6.5: Integration Reality Test (2 hours)

### Test Full Pipeline

```rust
// File: tests/integration_reality_test.rs

#[tokio::test]
async fn test_full_search_pipeline() {
    // Setup real system
    let config = Config {
        index_path: PathBuf::from("test_index"),
        vector_path: PathBuf::from("test_vectors"),
        model_path: PathBuf::from("models/nomic-embed-text-v1.5.Q4_K_M.gguf"),
        // ...
    };
    
    let mut searcher = UnifiedSearcher::new(config).await.unwrap();
    
    // Index test data
    searcher.index_directory("test_data").await.unwrap();
    
    // Test different query types
    let test_cases = vec![
        // Code search
        ("async fn search", vec!["unified.rs", "tantivy_search.rs"]),
        
        // Documentation search
        ("user guide", vec!["user_guide.md"]),
        
        // Semantic search
        ("error handling in rust", vec!["error.rs", "result"]),
        
        // Mixed search
        ("quick brown fox", vec!["test1.txt"]),
    ];
    
    for (query, expected_files) in test_cases {
        println!("\n=== Testing query: {} ===", query);
        
        let start = Instant::now();
        let results = searcher.search(query).await.unwrap();
        let duration = start.elapsed();
        
        println!("Found {} results in {:?}", results.len(), duration);
        
        // Should return results
        assert!(!results.is_empty(), "No results for query: {}", query);
        
        // Should be fast
        assert!(duration < Duration::from_secs(1), "Search too slow: {:?}", duration);
        
        // Should find expected files
        for expected in expected_files {
            assert!(
                results.iter().any(|r| r.file_path.contains(expected)),
                "Expected to find '{}' for query '{}'",
                expected, query
            );
        }
        
        // Print top 3 results
        for (i, result) in results.iter().take(3).enumerate() {
            println!("  {}. {} (score: {:.3}, method: {})",
                i + 1, result.file_path, result.score, result.match_type);
        }
    }
}
```

## Task 6.6: Performance Reality Test (1 hour)

### Benchmark Real Performance

```rust
// File: tests/performance_reality_test.rs

#[tokio::test]
async fn test_real_world_performance() {
    let searcher = setup_indexed_searcher().await;
    
    // Benchmark different query complexities
    let queries = vec![
        ("simple", "test"),
        ("two words", "async fn"),
        ("complex", "impl UnifiedSearcher search semantic"),
        ("regex", r"\bfn\s+\w+_test\b"),
    ];
    
    for (query_type, query) in queries {
        let mut times = Vec::new();
        
        // Run multiple times for average
        for _ in 0..10 {
            let start = Instant::now();
            let _ = searcher.search(query).await.unwrap();
            times.push(start.elapsed());
        }
        
        let avg = times.iter().sum::<Duration>() / times.len() as u32;
        let min = times.iter().min().unwrap();
        let max = times.iter().max().unwrap();
        
        println!("{} query '{}': avg={:?}, min={:?}, max={:?}",
            query_type, query, avg, min, max);
        
        // Performance assertions
        assert!(avg < Duration::from_millis(500), 
            "{} query too slow: {:?}", query_type, avg);
    }
}

#[tokio::test]
async fn test_scaling_performance() {
    let mut searcher = UnifiedSearcher::new(test_config()).await.unwrap();
    
    // Index increasing amounts of data
    let sizes = vec![10, 100, 1000, 10000];
    
    for size in sizes {
        // Add documents
        for i in 0..size {
            let doc = format!("Document {} with test content", i);
            searcher.add_document(&format!("doc{}", i), &doc).await.unwrap();
        }
        
        // Measure search time
        let start = Instant::now();
        let results = searcher.search("test").await.unwrap();
        let duration = start.elapsed();
        
        println!("{} documents: {:?} for {} results", 
            size, duration, results.len());
        
        // Should scale sub-linearly
        let ms_per_doc = duration.as_millis() as f64 / size as f64;
        assert!(ms_per_doc < 1.0, "Poor scaling: {}ms per doc", ms_per_doc);
    }
}
```

## Success Criteria

- [ ] All tests use real components (no mocks)
- [ ] BM25 finds results in real code
- [ ] Tantivy indexes and searches real files
- [ ] ML generates real 768-dim embeddings
- [ ] Semantic search returns relevant results
- [ ] Integration test passes end-to-end
- [ ] Performance meets targets with real data
- [ ] No silent failures or fallbacks

## Performance Targets (Real World)

- Simple query: <100ms
- Complex query: <500ms
- Semantic search: <200ms (after embedding cached)
- Index 1000 files: <10 seconds
- Search 10000 documents: <500ms

## Common Real-World Issues

1. **Model not downloaded**: Provide clear download instructions
2. **Insufficient memory**: Test with memory constraints
3. **Slow file I/O**: Test with network drives
4. **Large files**: Test with 10MB+ source files
5. **Unicode content**: Test with non-ASCII text

## Final Validation Checklist

- [ ] Can index entire Rust project
- [ ] Can search with all 4 methods
- [ ] Results are relevant and ranked correctly
- [ ] Performance acceptable for production
- [ ] Error messages are clear when things fail
- [ ] No mock objects or fake data in tests

## Next Phase

Proceed to Phase 7 (Production Hardening) only after all reality tests pass.