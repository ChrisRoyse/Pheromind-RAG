# Phase 5: Integration Surgery - Making Everything Work Together

**Duration**: 1 day  
**Goal**: Fix UnifiedSearcher to coordinate all 4 search methods  
**Success Metric**: Parallel search execution with proper result fusion

## The Core Problem

UnifiedSearcher has multiple type mismatches and logic errors preventing it from coordinating the search backends.

## Task 5.1: Fix UnifiedSearcher Compilation (2 hours)

### Fix Result Type Handling

```rust
// File: src/search/unified.rs

impl UnifiedSearcher {
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        // FIX: Properly handle Results from each search method
        
        // Launch all searches in parallel
        let (bm25_result, tantivy_result, native_result, semantic_result) = tokio::join!(
            self.search_bm25(query),
            self.search_tantivy(query),
            self.search_native(query),
            self.search_semantic(query),
        );
        
        // Collect successful results, log failures
        let mut all_results = Vec::new();
        
        match bm25_result {
            Ok(results) => all_results.push(("bm25", results)),
            Err(e) => log::warn!("BM25 search failed: {}", e),
        }
        
        match tantivy_result {
            Ok(results) => all_results.push(("tantivy", results)),
            Err(e) => log::warn!("Tantivy search failed: {}", e),
        }
        
        match native_result {
            Ok(results) => all_results.push(("native", results)),
            Err(e) => log::warn!("Native search failed: {}", e),
        }
        
        match semantic_result {
            Ok(results) => all_results.push(("semantic", results)),
            Err(e) => log::warn!("Semantic search failed: {}", e),
        }
        
        // If no search methods succeeded, return error
        if all_results.is_empty() {
            return Err(EmbedError::AllSearchMethodsFailed);
        }
        
        // Fuse results
        let fused = self.fusion.fuse(all_results)?;
        
        Ok(fused)
    }
}
```

### Fix Individual Search Methods

```rust
// File: src/search/unified.rs

impl UnifiedSearcher {
    async fn search_bm25(&self, query: &str) -> Result<Vec<SearchResult>> {
        // NO FALLBACKS - if BM25 fails, it fails
        let engine = self.bm25_engine.read().await;
        let matches = engine.search(query, 100);
        
        if matches.is_empty() {
            log::debug!("BM25 returned no results for query: {}", query);
        }
        
        Ok(matches.into_iter().map(|m| SearchResult {
            file_path: m.document_id,
            score: m.score,
            content: m.content,
            start_line: m.start_line,
            end_line: m.end_line,
            match_type: "bm25".to_string(),
        }).collect())
    }
    
    async fn search_tantivy(&self, query: &str) -> Result<Vec<SearchResult>> {
        let tantivy = self.tantivy_search.read().await;
        let matches = tantivy.search(query, 100)?;
        
        Ok(matches.into_iter().map(|m| SearchResult {
            file_path: m.path,
            score: m.score,
            content: m.content,
            start_line: m.start_line,
            end_line: m.end_line,
            match_type: "tantivy".to_string(),
        }).collect())
    }
    
    async fn search_native(&self, query: &str) -> Result<Vec<SearchResult>> {
        let native = self.native_search.read().await;
        let matches = native.search_regex(query)?;
        
        Ok(matches.into_iter().map(|m| SearchResult {
            file_path: m.file_path,
            score: 1.0,  // Native search doesn't score
            content: m.line_content,
            start_line: m.line_number,
            end_line: m.line_number,
            match_type: "native".to_string(),
        }).collect())
    }
    
    async fn search_semantic(&self, query: &str) -> Result<Vec<SearchResult>> {
        // Generate query embedding
        let embedder = self.embedder.read().await;
        let query_embedding = embedder.generate_embedding(query).await?;
        
        // Search in vector store
        let storage = self.vector_storage.read().await;
        let matches = storage.search(&query_embedding, 100).await?;
        
        Ok(matches.into_iter().map(|m| SearchResult {
            file_path: m.file_path,
            score: m.similarity,
            content: m.content,
            start_line: m.start_line,
            end_line: m.end_line,
            match_type: "semantic".to_string(),
        }).collect())
    }
}
```

## Task 5.2: Fix Result Fusion (2 hours)

### Implement Proper Fusion Logic

```rust
// File: src/search/fusion.rs

pub struct SimpleFusion {
    weights: HashMap<String, f64>,
}

impl SimpleFusion {
    pub fn new() -> Self {
        let mut weights = HashMap::new();
        weights.insert("bm25".to_string(), 0.25);
        weights.insert("tantivy".to_string(), 0.25);
        weights.insert("native".to_string(), 0.15);
        weights.insert("semantic".to_string(), 0.35);
        
        Self { weights }
    }
    
    pub fn fuse(&self, results: Vec<(&str, Vec<SearchResult>)>) -> Result<Vec<SearchResult>> {
        // Collect all unique file paths
        let mut file_scores: HashMap<String, FusedScore> = HashMap::new();
        
        for (method, method_results) in results {
            let weight = self.weights.get(method).unwrap_or(&0.25);
            
            for result in method_results {
                let entry = file_scores.entry(result.file_path.clone())
                    .or_insert_with(|| FusedScore::new(&result.file_path));
                
                // Add weighted score
                entry.add_score(method, result.score * weight);
                
                // Keep best content match
                if result.score > entry.best_score {
                    entry.best_score = result.score;
                    entry.best_content = result.content.clone();
                    entry.start_line = result.start_line;
                    entry.end_line = result.end_line;
                }
            }
        }
        
        // Convert to sorted results
        let mut fused_results: Vec<SearchResult> = file_scores
            .into_iter()
            .map(|(path, score)| SearchResult {
                file_path: path,
                score: score.total_score,
                content: score.best_content,
                start_line: score.start_line,
                end_line: score.end_line,
                match_type: "fused".to_string(),
            })
            .collect();
        
        // Sort by score descending
        fused_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        Ok(fused_results)
    }
}

struct FusedScore {
    file_path: String,
    total_score: f64,
    best_score: f64,
    best_content: String,
    start_line: usize,
    end_line: usize,
    methods: Vec<String>,
}

impl FusedScore {
    fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
            total_score: 0.0,
            best_score: 0.0,
            best_content: String::new(),
            start_line: 0,
            end_line: 0,
            methods: Vec::new(),
        }
    }
    
    fn add_score(&mut self, method: &str, score: f64) {
        self.total_score += score;
        self.methods.push(method.to_string());
    }
}
```

## Task 5.3: Fix Cache Integration (1 hour)

### Fix Cache Statistics Access

```rust
// File: src/search/unified.rs

impl UnifiedSearcher {
    pub fn get_stats(&self) -> SearchStats {
        let cache_stats = self.cache.get_stats();  // This returns CacheStats, not Result
        
        SearchStats {
            total_searches: self.search_count.load(Ordering::Relaxed),
            cache_hits: cache_stats.hits,
            cache_misses: cache_stats.misses,
            average_latency_ms: self.calculate_average_latency(),
        }
    }
}
```

## Task 5.4: Implement Proper Error Handling (1 hour)

### Add Missing Error Types

```rust
// File: src/error.rs

#[derive(Error, Debug)]
pub enum EmbedError {
    // ... existing variants ...
    
    #[error("All search methods failed")]
    AllSearchMethodsFailed,
    
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    #[error("Invalid dimension: expected {expected}, got {actual}")]
    InvalidDimension { expected: usize, actual: usize },
    
    #[error("Invalid model format: {0}")]
    InvalidModel(String),
}
```

## Task 5.5: Integration Testing (2 hours)

### Test Parallel Execution

```rust
#[tokio::test]
async fn test_parallel_search_execution() {
    let searcher = UnifiedSearcher::new(test_config()).await.unwrap();
    
    // Measure execution time
    let start = Instant::now();
    let results = searcher.search("test query").await.unwrap();
    let duration = start.elapsed();
    
    // Should execute in parallel, not sequentially
    // If sequential, would take 4x longer
    assert!(duration < Duration::from_millis(500));
    
    // Should have results from multiple methods
    let methods: HashSet<String> = results.iter()
        .map(|r| r.match_type.clone())
        .collect();
    
    assert!(methods.contains("fused"));
}
```

### Test Fusion Logic

```rust
#[test]
fn test_result_fusion() {
    let fusion = SimpleFusion::new();
    
    let bm25_results = vec![
        SearchResult {
            file_path: "file1.rs".to_string(),
            score: 0.8,
            // ...
        },
        SearchResult {
            file_path: "file2.rs".to_string(),
            score: 0.6,
            // ...
        },
    ];
    
    let semantic_results = vec![
        SearchResult {
            file_path: "file1.rs".to_string(),
            score: 0.9,
            // ...
        },
        SearchResult {
            file_path: "file3.rs".to_string(),
            score: 0.7,
            // ...
        },
    ];
    
    let fused = fusion.fuse(vec![
        ("bm25", bm25_results),
        ("semantic", semantic_results),
    ]).unwrap();
    
    // file1.rs should rank highest (appears in both)
    assert_eq!(fused[0].file_path, "file1.rs");
    assert!(fused[0].score > fused[1].score);
}
```

### Test Failure Handling

```rust
#[tokio::test]
async fn test_partial_failure_handling() {
    let mut searcher = UnifiedSearcher::new(test_config()).await.unwrap();
    
    // Deliberately break one search method
    searcher.disable_semantic_search();
    
    // Should still return results from other methods
    let results = searcher.search("test").await.unwrap();
    assert!(!results.is_empty());
    
    // Check that warning was logged
    // (would need to capture logs to verify)
}
```

## Task 5.6: Performance Optimization (1 hour)

### Optimize Parallel Execution

```rust
impl UnifiedSearcher {
    pub async fn search_optimized(&self, query: &str) -> Result<Vec<SearchResult>> {
        // Use timeout for each search method
        let timeout_duration = Duration::from_millis(200);
        
        let searches = vec![
            timeout(timeout_duration, self.search_bm25(query)),
            timeout(timeout_duration, self.search_tantivy(query)),
            timeout(timeout_duration, self.search_native(query)),
            timeout(timeout_duration, self.search_semantic(query)),
        ];
        
        let results = futures::future::join_all(searches).await;
        
        // Process results even if some timed out
        // ...
    }
}
```

## Success Criteria

- [ ] UnifiedSearcher compiles without errors
- [ ] All 4 search methods can be called
- [ ] Results are properly fused
- [ ] Parallel execution verified (not sequential)
- [ ] Failure of one method doesn't break everything
- [ ] Cache statistics work correctly
- [ ] Performance meets targets (<500ms total)
- [ ] Integration tests pass

## Performance Targets

- Total search time: <500ms
- Parallel overhead: <10ms
- Fusion processing: <5ms
- Result deduplication: <2ms

## Common Issues and Solutions

1. **Deadlock in parallel execution**: Use timeouts
2. **Memory spike with large results**: Limit results per method
3. **Fusion bias**: Adjust weights based on testing
4. **Cache invalidation**: Clear cache on index updates

## Next Phase

Proceed to Phase 6 (Testing Reality Check) only after integration works correctly.