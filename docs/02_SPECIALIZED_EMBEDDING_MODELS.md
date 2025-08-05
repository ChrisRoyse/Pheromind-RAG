# Phase 2: Search & Simple Fusion

## **PHASE OVERVIEW - UNIFIED SEARCH**

**GOAL**: Implement search with simple fusion of exact + semantic results  
**APPROACH**: Combine ripgrep exact matches with vector similarity search  
**MEASUREMENT**: Basic accuracy testing with common queries  
**TIMELINE**: Week 2 (Tasks 011-020)

## **KEY INSIGHT: SIMPLE FUSION WORKS**

**PROVEN**: Simple scoring and deduplication achieves great results without complexity

**Core Components**:
1. **Exact Search**: Ripgrep for text matching
2. **Semantic Search**: Vector similarity with MiniLM embeddings
3. **Simple Fusion**: Basic deduplication and scoring
4. **3-Chunk Results**: Always return full context

## **SEARCH & FUSION TASK BREAKDOWN (011-020)**

### **Search Implementation Tasks (011-015): Core Search**

#### **Task 011: Vector Similarity Search**
**Goal**: Implement similarity search in LanceDB  
**Duration**: 3 hours  
**Dependencies**: Phase 1 completion

**Implementation**:
```rust
impl VectorStorage {
    pub fn search_similar(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<SimilarityMatch>> {
        let dataset = self.dataset.as_ref().unwrap();
        
        // Create query vector
        let query_vector = Array::from_vec(query_embedding);
        
        // Perform ANN search
        let results = dataset
            .scan()
            .nearest("embedding", &query_vector)?
            .limit(limit)
            .execute()?;
        
        // Convert to matches
        let mut matches = Vec::new();
        for batch in results {
            let file_paths = batch.column_by_name("file_path").unwrap();
            let chunk_indices = batch.column_by_name("chunk_index").unwrap();
            let contents = batch.column_by_name("content").unwrap();
            let scores = batch.column_by_name("_distance").unwrap();
            
            for i in 0..batch.num_rows() {
                matches.push(SimilarityMatch {
                    file_path: file_paths.as_string().value(i).to_string(),
                    chunk_index: chunk_indices.as_i32().value(i) as usize,
                    content: contents.as_string().value(i).to_string(),
                    similarity: 1.0 - scores.as_f32().value(i), // Convert distance to similarity
                });
            }
        }
        
        Ok(matches)
    }
}
```

#### **Task 012: Simple Fusion Algorithm**
**Goal**: Combine exact and semantic results with basic scoring  
**Duration**: 3 hours  
**Dependencies**: Task 011

**Implementation**:
```rust
pub struct SimpleFusion;

impl SimpleFusion {
    pub fn fuse_results(
        exact_matches: Vec<ExactMatch>,
        semantic_matches: Vec<SimilarityMatch>,
    ) -> Vec<FusedResult> {
        let mut seen = HashSet::new();
        let mut results = Vec::new();
        
        // Process exact matches first (higher priority)
        for exact in exact_matches {
            let key = (exact.file_path.clone(), exact.line_number);
            if seen.insert(key) {
                results.push(FusedResult {
                    file_path: exact.file_path,
                    line_number: Some(exact.line_number),
                    chunk_index: None,
                    score: 1.0, // Exact matches get perfect score
                    match_type: MatchType::Exact,
                    content: exact.content,
                });
            }
        }
        
        // Add semantic matches with lower scores
        for semantic in semantic_matches {
            let key = (semantic.file_path.clone(), semantic.chunk_index);
            // Check if we already have an exact match for this location
            let already_exact = results.iter().any(|r| {
                r.file_path == semantic.file_path && 
                r.match_type == MatchType::Exact
            });
            
            if !already_exact {
                results.push(FusedResult {
                    file_path: semantic.file_path,
                    line_number: None,
                    chunk_index: Some(semantic.chunk_index),
                    score: semantic.similarity * 0.8, // Slightly lower than exact
                    match_type: MatchType::Semantic,
                    content: semantic.content,
                });
            }
        }
        
        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        // Take top results
        results.truncate(20);
        results
    }
}
```

#### **Task 013: Complete Search Pipeline**
**Goal**: Wire together exact + semantic + fusion  
**Duration**: 4 hours  
**Dependencies**: Task 012

**Implementation**:
```rust
pub struct UnifiedSearcher {
    ripgrep: RipgrepSearcher,
    embedder: MiniLMEmbedder,
    storage: VectorStorage,
    chunker: SimpleRegexChunker,
    expander: ThreeChunkExpander,
    fusion: SimpleFusion,
}

impl UnifiedSearcher {
    pub async fn search(&self, query: &str, project_path: &Path) -> Result<Vec<SearchResult>> {
        // 1. Run exact search
        let exact_matches = self.ripgrep.search(query, project_path)?;
        
        // 2. Run semantic search
        let query_embedding = self.embedder.embed(query)?;
        let semantic_matches = self.storage.search_similar(query_embedding, 30)?;
        
        // 3. Fuse results
        let fused = self.fusion.fuse_results(exact_matches, semantic_matches);
        
        // 4. Expand to 3-chunk contexts
        let mut results = Vec::new();
        for fused_match in fused {
            let file_content = std::fs::read_to_string(&fused_match.file_path)?;
            let chunks = self.chunker.chunk_file(&file_content);
            
            // Find the relevant chunk
            let chunk_idx = match fused_match.match_type {
                MatchType::Exact => {
                    self.find_chunk_for_line(&chunks, fused_match.line_number.unwrap())
                },
                MatchType::Semantic => {
                    fused_match.chunk_index.unwrap()
                }
            };
            
            let three_chunk = self.expander.expand(&chunks, chunk_idx);
            
            results.push(SearchResult {
                file: fused_match.file_path,
                three_chunk_context: three_chunk,
                score: fused_match.score,
                match_type: fused_match.match_type,
            });
        }
        
        Ok(results)
    }
}
```

#### **Task 014: Result Ranking Optimization**
**Goal**: Improve result ranking with simple heuristics  
**Duration**: 2 hours  
**Dependencies**: Task 013

**Implementation**:
```rust
impl UnifiedSearcher {
    fn optimize_ranking(&self, results: &mut Vec<SearchResult>, query: &str) {
        // Simple heuristics to improve ranking
        for result in results.iter_mut() {
            // Boost if query appears in target chunk
            if result.three_chunk_context.target.content.contains(query) {
                result.score *= 1.2;
            }
            
            // Boost if match is at beginning of chunk (likely function/class definition)
            let first_lines = result.three_chunk_context.target.content
                .lines()
                .take(3)
                .collect::<Vec<_>>()
                .join("\n");
                
            if first_lines.contains(query) {
                result.score *= 1.1;
            }
            
            // Slight penalty for very large chunks (less focused)
            let chunk_size = result.three_chunk_context.target.content.lines().count();
            if chunk_size > 150 {
                result.score *= 0.95;
            }
        }
        
        // Re-sort after optimization
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    }
}
```

#### **Task 015: Parallel Search Execution**
**Goal**: Run exact and semantic search in parallel  
**Duration**: 2 hours  
**Dependencies**: Task 014

### **Quality & Performance Tasks (016-020): Polish**

#### **Task 016: Search Result Caching**
**Goal**: Cache recent search results  
**Duration**: 2 hours  
**Dependencies**: Task 015

**Implementation**:
```rust
use lru::LruCache;

pub struct SearchCache {
    cache: Mutex<LruCache<String, Vec<SearchResult>>>,
}

impl SearchCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(capacity)),
        }
    }
    
    pub fn get(&self, query: &str) -> Option<Vec<SearchResult>> {
        self.cache.lock().unwrap().get(query).cloned()
    }
    
    pub fn insert(&self, query: String, results: Vec<SearchResult>) {
        self.cache.lock().unwrap().put(query, results);
    }
}
```

#### **Task 017: Query Preprocessing**
**Goal**: Normalize and improve queries before search  
**Duration**: 2 hours  
**Dependencies**: Task 016

**Implementation**:
```rust
pub struct QueryPreprocessor;

impl QueryPreprocessor {
    pub fn preprocess(&self, query: &str) -> String {
        let mut processed = query.to_lowercase();
        
        // Remove common noise words
        let noise_words = ["the", "a", "an", "in", "of", "for"];
        for word in noise_words {
            processed = processed.replace(&format!(" {} ", word), " ");
        }
        
        // Normalize whitespace
        processed = processed.split_whitespace().collect::<Vec<_>>().join(" ");
        
        // Expand common abbreviations
        processed = processed
            .replace("fn ", "function ")
            .replace("impl ", "implementation ")
            .replace("struct ", "structure ");
        
        processed.trim().to_string()
    }
}
```

#### **Task 018: Search Metrics**
**Goal**: Track search performance and accuracy  
**Duration**: 2 hours  
**Dependencies**: Task 017

#### **Task 019: Error Handling**
**Goal**: Graceful handling of search failures  
**Duration**: 1 hour  
**Dependencies**: Task 018

#### **Task 020: Phase 2 Completion**
**Goal**: Integration testing and documentation  
**Duration**: 1 hour  
**Dependencies**: Task 019

## **SUCCESS CRITERIA**

### **Phase 2 Targets**
- **Unified Search**: Exact + semantic working together
- **Simple Fusion**: Basic scoring and deduplication
- **Performance**: <500ms total search time
- **Accuracy**: 70%+ on test queries
- **3-Chunk Results**: Every result has context

### **Deliverables**
- Working similarity search
- Simple fusion algorithm
- Unified search API
- Basic result optimization
- Performance metrics

## **ARCHITECTURE**

```rust
// Phase 2 additions
pub struct Phase2Search {
    pub searcher: UnifiedSearcher,
    pub cache: SearchCache,
    pub preprocessor: QueryPreprocessor,
}

// Result types
#[derive(Debug, Clone)]
pub enum MatchType {
    Exact,
    Semantic,
}

pub struct FusedResult {
    pub file_path: String,
    pub line_number: Option<usize>,
    pub chunk_index: Option<usize>,
    pub score: f32,
    pub match_type: MatchType,
    pub content: String,
}

pub struct SimilarityMatch {
    pub file_path: String,
    pub chunk_index: usize,
    pub content: String,
    pub similarity: f32,
}
```

## **WEEK 2 DELIVERABLES**

1. **Vector Search**: Working similarity search
2. **Simple Fusion**: Deduplication and scoring
3. **Unified API**: Single search interface
4. **Optimizations**: Basic ranking improvements
5. **Ready for Git**: Foundation for file updates

**Next Phase**: Git-based file watching and updates