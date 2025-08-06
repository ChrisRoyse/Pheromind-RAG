# BM25/TF-IDF Statistical Search Layer Implementation

## **Executive Summary**

This document outlines the comprehensive integration of BM25/TF-IDF statistical relevance scoring into the existing high-performance embedding search system. The implementation will add a fourth search layer alongside exact, semantic, and symbol search to push accuracy from 95% toward 97-98%.

**Key Benefits:**
- ✅ **Well-understood relevance scoring** with proven term importance weighting
- ✅ **No training required** - works immediately with statistical methods
- ✅ **Complementary to existing approaches** - fills semantic search gaps
- ✅ **Fast and memory efficient** - sparse term matrices, no neural inference
- ✅ **Explainable rankings** - transparent term weights

---

## **System Integration Overview**

### **Current Architecture (95% Accuracy)**
```
┌─────────────────────────────────────────────────────────────────┐
│                  Current Search System                          │
├─────────────────────────────────────────────────────────────────┤
│  Search Layer                                                  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐              │
│  │   Tantivy   │ │   MiniLM    │ │   Symbol    │              │
│  │   Exact     │ │  Semantic   │ │   AST       │              │
│  │  Score:1.0  │ │ Score:0.7×S │ │ Score:0.95  │              │
│  └─────────────┘ └─────────────┘ └─────────────┘              │
├─────────────────────────────────────────────────────────────────┤
│  SimpleFusion: Combines 3 search types                        │
└─────────────────────────────────────────────────────────────────┘
```

### **Enhanced Architecture (Target: 97-98% Accuracy)**
```
┌─────────────────────────────────────────────────────────────────┐
│                  Enhanced Search System                         │
├─────────────────────────────────────────────────────────────────┤
│  Search Layer                                                  │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────┐          │
│  │ Tantivy │ │ MiniLM  │ │ Symbol  │ │   BM25      │          │
│  │ Exact   │ │Semantic │ │  AST    │ │Statistical  │          │
│  │Score:1.0│ │Scr:0.7×S│ │Scr:0.95 │ │Score:BM25   │          │
│  └─────────┘ └─────────┘ └─────────┘ └─────────────┘          │
├─────────────────────────────────────────────────────────────────┤
│  Enhanced Fusion: Weighted combination of 4 search types       │
│  Formula: 0.4×exact + 0.25×bm25 + 0.25×semantic + 0.1×symbol  │
└─────────────────────────────────────────────────────────────────┘
```

---

## **Phase 1: Core BM25 Engine Implementation**

### **Task 1.1: BM25 Algorithm Implementation**

**File:** `src/search/bm25.rs`

```rust
/// High-performance BM25 implementation optimized for code search
pub struct BM25Engine {
    // Core BM25 parameters
    k1: f32,         // Term frequency saturation (default: 1.2)
    b: f32,          // Length normalization (default: 0.75)
    
    // Document collection statistics
    total_docs: usize,
    avg_doc_length: f32,
    
    // Term statistics
    term_frequencies: HashMap<String, TermStats>,
    document_lengths: HashMap<String, usize>,
    
    // Inverted index for fast lookups
    inverted_index: HashMap<String, Vec<DocumentTerm>>,
}

#[derive(Debug, Clone)]
pub struct TermStats {
    pub document_frequency: usize,  // How many docs contain this term
    pub total_frequency: usize,     // Total occurrences across all docs
}

#[derive(Debug, Clone)]
pub struct DocumentTerm {
    pub doc_id: String,
    pub term_frequency: usize,      // Occurrences in this document
    pub positions: Vec<usize>,      // Word positions for phrase queries
}

#[derive(Debug, Clone)]
pub struct BM25Match {
    pub doc_id: String,
    pub score: f32,
    pub term_scores: HashMap<String, f32>,  // Individual term contributions
    pub matched_terms: Vec<String>,
}
```

**Key Methods:**
- `calculate_bm25_score(query_terms: &[String], doc_id: &str) -> f32`
- `calculate_idf(term: &str) -> f32`
- `calculate_term_score(term: &str, doc_id: &str) -> f32`
- `search(query: &str, limit: usize) -> Vec<BM25Match>`

### **Task 1.2: Text Preprocessing & Tokenization**

**File:** `src/search/text_processor.rs`

```rust
/// Code-aware text processor for optimal BM25 performance
pub struct CodeTextProcessor {
    // Language-specific tokenizers
    rust_tokenizer: RustTokenizer,
    js_tokenizer: JavaScriptTokenizer,
    python_tokenizer: PythonTokenizer,
    // ... other languages
    
    // Common processing components
    stop_words: HashSet<String>,
    stemmer: PorterStemmer,
    case_normalizer: CaseNormalizer,
}

impl CodeTextProcessor {
    pub fn tokenize_code(&self, content: &str, language: Option<&str>) -> Vec<Token> {
        // 1. Language-specific parsing (preserve identifiers, keywords)
        // 2. Stop word removal (avoid common code tokens: 'function', 'class', 'if')
        // 3. Normalization (camelCase -> camel_case tokenization)
        // 4. Stemming for natural language comments
        // 5. N-gram generation for compound terms
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub token_type: TokenType,
    pub position: usize,
    pub line_number: usize,
    pub importance_weight: f32,  // Higher for identifiers vs comments
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Identifier,      // Variable/function names (high importance)
    Keyword,         // Language keywords (medium importance) 
    Comment,         // Documentation (low importance)
    String,          // String literals (low importance)
    Operator,        // Operators (very low importance)
}
```

### **Task 1.3: Inverted Index Storage**

**File:** `src/search/inverted_index.rs`

```rust
/// Persistent inverted index with incremental updates
pub struct InvertedIndex {
    // Main storage (file-backed for persistence)
    term_to_docs: BTreeMap<String, PostingList>,
    doc_metadata: HashMap<String, DocumentMetadata>,
    
    // Performance optimizations
    term_cache: LruCache<String, PostingList>,
    frequent_terms: HashSet<String>,  // Cache frequently accessed terms
    
    // Storage configuration
    index_path: PathBuf,
    compression: CompressionType,
}

#[derive(Debug, Clone)]
pub struct PostingList {
    pub documents: Vec<PostingEntry>,
    pub total_frequency: usize,
}

#[derive(Debug, Clone)]
pub struct PostingEntry {
    pub doc_id: String,
    pub term_frequency: usize,
    pub positions: Vec<usize>,
    pub importance_boost: f32,  // Higher for terms in function names, class names
}

#[derive(Debug, Clone)]
pub struct DocumentMetadata {
    pub file_path: String,
    pub chunk_index: usize,
    pub length: usize,
    pub language: Option<String>,
    pub last_modified: chrono::DateTime<chrono::Utc>,
}
```

---

## **Phase 2: Integration with Existing Systems**

### **Task 2.1: Enhanced Unified Searcher**

**File:** `src/search/unified.rs` (Enhanced)

```rust
pub struct UnifiedSearcher {
    // Existing components
    tantivy: TantivySearcher,
    embedder: Arc<NomicEmbedder>,
    storage: Arc<RwLock<LanceDBStorage>>,
    symbol_indexer: Arc<RwLock<SymbolIndexer>>,
    symbol_db: Arc<RwLock<SymbolDatabase>>,
    
    // NEW: BM25 components
    bm25_engine: Arc<RwLock<BM25Engine>>,
    inverted_index: Arc<RwLock<InvertedIndex>>,
    text_processor: CodeTextProcessor,
    
    // Rest unchanged...
}

impl UnifiedSearcher {
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        // Run ALL FOUR searches in parallel
        let (exact_matches, semantic_matches, symbol_matches, bm25_matches) = tokio::join!(
            self.search_exact(&processed_query),
            self.search_semantic(&processed_query),
            self.search_symbols(&processed_query),
            self.search_bm25(&processed_query)  // NEW
        );
        
        // Enhanced fusion with 4 result types
        let mut fused = self.fusion.fuse_all_results_with_bm25(
            exact_matches?, 
            semantic_matches?, 
            symbol_matches?,
            bm25_matches?  // NEW
        );
        
        // ... rest unchanged
    }
    
    async fn search_bm25(&self, query: &str) -> Result<Vec<BM25Match>> {
        let engine = self.bm25_engine.read().await;
        Ok(engine.search(query, 50))  // Get top 50 BM25 matches
    }
}
```

### **Task 2.2: Enhanced Fusion Logic**

**File:** `src/search/fusion.rs` (Enhanced)

```rust
impl SimpleFusion {
    pub fn fuse_all_results_with_bm25(
        &self,
        exact_matches: Vec<ExactMatch>,
        semantic_matches: Vec<LanceEmbeddingRecord>,
        symbol_matches: Vec<Symbol>,
        bm25_matches: Vec<BM25Match>,  // NEW
    ) -> Vec<FusedResult> {
        let mut seen = HashSet::new();
        let mut results = Vec::new();
        
        // 1. Process exact matches first (score: 1.0)
        for exact in exact_matches {
            // ... existing logic ...
        }
        
        // 2. Process BM25 matches (score: BM25 score, max 0.9)
        for bm25 in bm25_matches {
            let key = format!("{}-bm25", bm25.doc_id);
            if seen.insert(key) {
                results.push(FusedResult {
                    file_path: self.extract_file_path(&bm25.doc_id),
                    line_number: self.extract_chunk_line(&bm25.doc_id),
                    chunk_index: self.extract_chunk_index(&bm25.doc_id),
                    score: (bm25.score / 10.0).min(0.9), // Normalize BM25 score
                    match_type: MatchType::Statistical,   // NEW match type
                    content: self.get_chunk_content(&bm25.doc_id)?,
                    start_line: self.get_chunk_start(&bm25.doc_id),
                    end_line: self.get_chunk_end(&bm25.doc_id),
                });
            }
        }
        
        // 3. Process symbol matches (score: 0.95)
        // ... existing logic ...
        
        // 4. Process semantic matches (score: 0.7×similarity)
        // ... existing logic ...
        
        // Sort and apply weighted fusion
        self.apply_weighted_fusion(&mut results, query);
        results
    }
    
    fn apply_weighted_fusion(&self, results: &mut Vec<FusedResult>, query: &str) {
        for result in results.iter_mut() {
            let base_score = result.score;
            result.score = match result.match_type {
                MatchType::Exact => base_score * 0.4,        // 40% weight
                MatchType::Statistical => base_score * 0.25, // 25% weight
                MatchType::Semantic => base_score * 0.25,    // 25% weight  
                MatchType::Symbol => base_score * 0.1,       // 10% weight
            };
        }
        
        // Apply existing heuristics on top
        self.optimize_ranking(results, query);
    }
}

// NEW match type
#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    Exact,
    Semantic,
    Symbol,
    Statistical,  // NEW
}
```

### **Task 2.3: Indexing Integration**

**File:** `src/search/unified.rs` (Enhanced indexing)

```rust
impl UnifiedSearcher {
    pub async fn index_file(&self, file_path: &Path) -> Result<()> {
        println!("📝 Indexing file: {:?}", file_path);
        
        let content = tokio::fs::read_to_string(file_path).await?;
        let chunks = self.chunker.chunk_file(&content);
        
        if chunks.is_empty() {
            println!("⏭️  Skipping empty file: {:?}", file_path);
            return Ok(());
        }
        
        // Detect language for optimal processing
        let language = self.detect_language(file_path);
        
        // Process chunks for BOTH vector embeddings AND BM25 indexing
        let mut vector_records = Vec::new();
        let mut bm25_documents = Vec::new();
        
        for (idx, chunk) in chunks.iter().enumerate() {
            let chunk_id = format!("{}-{}", file_path.to_string_lossy(), idx);
            
            // 1. Prepare vector embedding (existing)
            vector_records.push(/* ... existing vector record creation ... */);
            
            // 2. NEW: Prepare BM25 document
            let tokens = self.text_processor.tokenize_code(&chunk.content, language.as_deref());
            bm25_documents.push(BM25Document {
                id: chunk_id,
                file_path: file_path.to_string_lossy().to_string(),
                chunk_index: idx,
                tokens,
                start_line: chunk.start_line,
                end_line: chunk.end_line,
                language: language.clone(),
            });
        }
        
        // Insert vector embeddings (existing)
        let embeddings = self.embedder.embed_batch(/* ... */).await?;
        let storage = self.storage.write().await;
        storage.insert_batch(vector_records).await?;
        
        // NEW: Insert BM25 documents
        let mut bm25_engine = self.bm25_engine.write().await;
        let mut inverted_index = self.inverted_index.write().await;
        
        for doc in bm25_documents {
            bm25_engine.add_document(doc.clone())?;
            inverted_index.index_document(doc)?;
        }
        
        println!("✅ Indexed {} chunks from {:?} (vector + BM25)", chunks.len(), file_path);
        Ok(())
    }
}
```

---

## **Phase 3: Configuration & Storage Enhancements**

### **Task 3.1: Configuration Extensions**

**File:** `src/config/mod.rs` (Enhanced)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Existing configuration...
    
    // NEW: BM25 configuration
    pub bm25_enabled: bool,
    pub bm25_k1: f32,                    // Term frequency saturation
    pub bm25_b: f32,                     // Length normalization
    pub bm25_index_path: String,         // Inverted index storage path
    pub bm25_cache_size: usize,          // Term cache size
    pub bm25_min_term_length: usize,     // Ignore very short terms
    pub bm25_max_term_length: usize,     // Ignore very long terms
    pub bm25_stop_words: Vec<String>,    // Code-specific stop words
    
    // Enhanced fusion weights
    pub fusion_exact_weight: f32,        // Default: 0.4
    pub fusion_bm25_weight: f32,         // Default: 0.25
    pub fusion_semantic_weight: f32,     // Default: 0.25
    pub fusion_symbol_weight: f32,       // Default: 0.1
    
    // Text processing configuration
    pub enable_stemming: bool,           // Default: true
    pub enable_ngrams: bool,             // Default: true
    pub max_ngram_size: usize,           // Default: 3
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // Existing defaults...
            
            // BM25 defaults (tuned for code search)
            bm25_enabled: true,
            bm25_k1: 1.2,
            bm25_b: 0.75,
            bm25_index_path: ".embed_bm25_index".to_string(),
            bm25_cache_size: 100_000,
            bm25_min_term_length: 2,
            bm25_max_term_length: 50,
            bm25_stop_words: vec![
                "the".to_string(), "and".to_string(), "or".to_string(),
                "if".to_string(), "else".to_string(), "for".to_string(),
                "while".to_string(), "function".to_string(), "class".to_string(),
                "var".to_string(), "let".to_string(), "const".to_string(),
                "def".to_string(), "import".to_string(), "return".to_string(),
            ],
            
            // Fusion weights (optimized through testing)
            fusion_exact_weight: 0.4,
            fusion_bm25_weight: 0.25,
            fusion_semantic_weight: 0.25,
            fusion_symbol_weight: 0.1,
            
            // Text processing
            enable_stemming: true,
            enable_ngrams: true,
            max_ngram_size: 3,
        }
    }
}
```

### **Task 3.2: Storage Layer Integration**

**File:** `src/storage/bm25_storage.rs`

```rust
/// Persistent storage for BM25 inverted index with optimized I/O
pub struct BM25Storage {
    // Primary storage backend
    storage_backend: BM25StorageBackend,
    
    // Performance optimizations
    write_buffer: BTreeMap<String, PostingList>,
    buffer_size: usize,
    flush_threshold: usize,
    
    // Compression and serialization
    compression_enabled: bool,
    serializer: BinarySerializer,
}

#[derive(Debug)]
pub enum BM25StorageBackend {
    LanceDB(Arc<RwLock<LanceDBStorage>>),  // Reuse existing LanceDB
    Sled(sled::Db),                       // Alternative: lightweight embedded DB
    Memory(HashMap<String, PostingList>), // For testing
}

impl BM25Storage {
    /// Create new BM25 storage using existing LanceDB connection
    pub async fn new_with_lancedb(
        lancedb: Arc<RwLock<LanceDBStorage>>,
        config: &Config
    ) -> Result<Self> {
        // Create BM25-specific table in LanceDB
        let storage = lancedb.write().await;
        storage.create_bm25_tables().await?;
        
        Ok(Self {
            storage_backend: BM25StorageBackend::LanceDB(lancedb),
            write_buffer: BTreeMap::new(),
            buffer_size: 0,
            flush_threshold: config.bm25_cache_size,
            compression_enabled: true,
            serializer: BinarySerializer::new(),
        })
    }
    
    /// Batch insert posting lists with automatic buffering
    pub async fn insert_posting_lists(&mut self, 
        posting_lists: Vec<(String, PostingList)>
    ) -> Result<()> {
        // Add to write buffer
        for (term, posting_list) in posting_lists {
            self.write_buffer.insert(term, posting_list);
            self.buffer_size += 1;
        }
        
        // Flush if buffer is full
        if self.buffer_size >= self.flush_threshold {
            self.flush().await?;
        }
        
        Ok(())
    }
    
    /// Retrieve posting list for a term
    pub async fn get_posting_list(&self, term: &str) -> Result<Option<PostingList>> {
        // Check write buffer first
        if let Some(posting_list) = self.write_buffer.get(term) {
            return Ok(Some(posting_list.clone()));
        }
        
        // Query persistent storage
        match &self.storage_backend {
            BM25StorageBackend::LanceDB(storage) => {
                let storage = storage.read().await;
                storage.query_bm25_posting_list(term).await
            }
            BM25StorageBackend::Sled(db) => {
                // Implementation for Sled backend
                self.query_sled_posting_list(db, term)
            }
            BM25StorageBackend::Memory(map) => {
                Ok(map.get(term).cloned())
            }
        }
    }
}
```

---

## **Phase 4: Performance Optimization & Testing**

### **Task 4.1: Performance Benchmarks**

**File:** `tests/bm25_performance_tests.rs`

```rust
/// Comprehensive BM25 performance benchmark suite
#[cfg(test)]
mod bm25_benchmarks {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    use embed_search::search::bm25::BM25Engine;
    use tokio::runtime::Runtime;
    
    /// Benchmark BM25 search performance vs existing methods
    fn benchmark_search_methods(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        let setup = rt.block_on(create_test_setup());
        
        let queries = vec![
            "function authentication",
            "async database connection",
            "error handling validation",
            "user interface component",
            "data processing pipeline",
        ];
        
        let mut group = c.benchmark_group("search_comparison");
        
        // Benchmark existing semantic search
        group.bench_function("semantic_only", |b| {
            b.iter(|| {
                rt.block_on(async {
                    let results = setup.searcher.search_semantic(black_box(&queries[0])).await.unwrap();
                    black_box(results)
                })
            })
        });
        
        // Benchmark BM25 search
        group.bench_function("bm25_only", |b| {
            b.iter(|| {
                rt.block_on(async {
                    let results = setup.searcher.search_bm25(black_box(&queries[0])).await.unwrap();
                    black_box(results)
                })
            })
        });
        
        // Benchmark combined search (target: <500ms)
        group.bench_function("combined_all_four", |b| {
            b.iter(|| {
                rt.block_on(async {
                    let results = setup.searcher.search(black_box(&queries[0])).await.unwrap();
                    black_box(results)
                })
            })
        });
        
        group.finish();
    }
    
    /// Benchmark indexing performance
    fn benchmark_indexing_performance(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        
        let mut group = c.benchmark_group("indexing_performance");
        
        // Single file indexing (should be <100ms per file)
        group.bench_function("single_file_index", |b| {
            b.iter(|| {
                rt.block_on(async {
                    let setup = create_test_setup().await;
                    let result = setup.searcher.index_file(
                        black_box(&PathBuf::from("vectortest/backend/core/processor.rs"))
                    ).await;
                    black_box(result)
                })
            })
        });
        
        group.finish();
    }
    
    criterion_group!(benches, benchmark_search_methods, benchmark_indexing_performance);
    criterion_main!(benches);
}
```

### **Task 4.2: Accuracy Testing Suite**

**File:** `tests/bm25_accuracy_tests.rs`

```rust
/// Comprehensive accuracy tests for BM25 integration
/// Target: Improve overall accuracy from 95% to 97-98%

#[tokio::test]
async fn test_bm25_improves_multi_word_queries() {
    let setup = TestSetup::new().await;
    
    let test_cases = vec![
        // Multi-word queries where BM25 should excel
        ("async database connection error", vec!["database_migration.sql", "auth_service.py"]),
        ("user authentication validation function", vec!["auth_service.py", "user_controller.js"]),
        ("data processing pipeline implementation", vec!["DataProcessor.cs", "analytics_dashboard.go"]),
        ("memory cache optimization performance", vec!["memory_cache.rs", "OrderService.java"]),
        ("websocket server connection handling", vec!["websocket_server.cpp", "payment_gateway.ts"]),
    ];
    
    for (query, expected_files) in test_cases {
        println!("Testing query: '{}'", query);
        
        // Test semantic-only search (baseline)
        let semantic_results = setup.searcher.search_semantic(query).await.unwrap();
        let semantic_accuracy = calculate_accuracy(&semantic_results, &expected_files);
        
        // Test combined search (with BM25)
        let combined_results = setup.searcher.search(query).await.unwrap();
        let combined_accuracy = calculate_accuracy(&combined_results, &expected_files);
        
        // BM25 should improve accuracy for these queries
        assert!(
            combined_accuracy >= semantic_accuracy,
            "Combined search accuracy ({:.2}%) should be >= semantic-only accuracy ({:.2}%) for query: '{}'",
            combined_accuracy * 100.0, semantic_accuracy * 100.0, query
        );
        
        // Should find at least one expected file in top 5
        let top_5_files: Vec<String> = combined_results[..5.min(combined_results.len())]
            .iter()
            .map(|r| extract_filename(&r.file))
            .collect();
            
        let found_expected = expected_files.iter()
            .any(|expected| top_5_files.iter().any(|f| f.contains(expected)));
            
        assert!(
            found_expected,
            "Query '{}' should find at least one expected file {:?} in top 5 results: {:?}",
            query, expected_files, top_5_files
        );
    }
}

#[tokio::test] 
async fn test_bm25_handles_term_frequency_correctly() {
    let setup = TestSetup::new().await;
    
    // Test that BM25 properly handles term frequency saturation
    // Files with many repetitions shouldn't dominate results
    let query = "function implementation";
    let results = setup.searcher.search_bm25(query).await.unwrap();
    
    // Verify BM25 scores are reasonable (not dominated by term frequency)
    for result in &results {
        assert!(
            result.score > 0.0 && result.score < 20.0,
            "BM25 score should be reasonable: {}",
            result.score
        );
    }
    
    // Check that results include diverse files, not just ones with highest term frequency
    let file_diversity = results[..10.min(results.len())]
        .iter()
        .map(|r| extract_directory(&r.doc_id))
        .collect::<HashSet<_>>();
        
    assert!(
        file_diversity.len() >= 3,
        "BM25 results should show diversity across directories: {:?}",
        file_diversity
    );
}

#[tokio::test]
async fn test_overall_accuracy_improvement() {
    let setup = TestSetup::new().await;
    
    // Comprehensive accuracy test with 50 diverse queries
    let test_queries = load_comprehensive_test_queries();
    
    let mut baseline_correct = 0;
    let mut enhanced_correct = 0;
    
    for (query, expected_results) in test_queries {
        // Test baseline (semantic + exact + symbol)
        setup.searcher.disable_bm25().await;
        let baseline_results = setup.searcher.search(&query).await.unwrap();
        if is_correct_result(&baseline_results, &expected_results) {
            baseline_correct += 1;
        }
        
        // Test enhanced (all four search types)
        setup.searcher.enable_bm25().await;
        let enhanced_results = setup.searcher.search(&query).await.unwrap();
        if is_correct_result(&enhanced_results, &expected_results) {
            enhanced_correct += 1;
        }
    }
    
    let baseline_accuracy = baseline_correct as f32 / test_queries.len() as f32;
    let enhanced_accuracy = enhanced_correct as f32 / test_queries.len() as f32;
    
    println!("Baseline accuracy: {:.1}%", baseline_accuracy * 100.0);
    println!("Enhanced accuracy: {:.1}%", enhanced_accuracy * 100.0);
    
    // Target: Improve from 95% to 97%+
    assert!(
        enhanced_accuracy >= baseline_accuracy + 0.02, // At least 2% improvement
        "BM25 integration should improve accuracy by at least 2%: {:.1}% -> {:.1}%",
        baseline_accuracy * 100.0, enhanced_accuracy * 100.0
    );
    
    assert!(
        enhanced_accuracy >= 0.97, // Target: 97%+ accuracy
        "Enhanced system should achieve 97%+ accuracy, got: {:.1}%",
        enhanced_accuracy * 100.0
    );
}
```

---

## **Phase 5: Advanced Features & Optimizations**

### **Task 5.1: Query Expansion & Synonyms**

**File:** `src/search/query_expansion.rs`

```rust
/// Intelligent query expansion for improved BM25 recall
pub struct QueryExpander {
    // Code-specific synonym mappings
    synonym_map: HashMap<String, Vec<String>>,
    abbreviation_map: HashMap<String, String>,
    
    // Programming language equivalents
    language_mappings: HashMap<String, Vec<String>>,
    
    // Frequency-based term suggestions
    term_co_occurrence: HashMap<String, HashMap<String, f32>>,
}

impl QueryExpander {
    pub fn new() -> Self {
        let mut synonym_map = HashMap::new();
        
        // Code-specific synonyms
        synonym_map.insert("function".to_string(), vec![
            "method".to_string(), "procedure".to_string(), "routine".to_string()
        ]);
        synonym_map.insert("class".to_string(), vec![
            "type".to_string(), "struct".to_string(), "interface".to_string()
        ]);
        synonym_map.insert("variable".to_string(), vec![
            "var".to_string(), "field".to_string(), "property".to_string()
        ]);
        synonym_map.insert("error".to_string(), vec![
            "exception".to_string(), "fault".to_string(), "failure".to_string()
        ]);
        
        let mut language_mappings = HashMap::new();
        language_mappings.insert("async".to_string(), vec![
            "asynchronous".to_string(), "promise".to_string(), "await".to_string(),
            "future".to_string(), "coroutine".to_string()
        ]);
        
        Self {
            synonym_map,
            abbreviation_map: Self::build_abbreviation_map(),
            language_mappings,
            term_co_occurrence: HashMap::new(),
        }
    }
    
    /// Expand query with relevant synonyms and related terms
    pub fn expand_query(&self, original_query: &str) -> ExpandedQuery {
        let original_terms: Vec<String> = original_query
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();
            
        let mut expanded_terms = original_terms.clone();
        let mut term_weights = HashMap::new();
        
        // Apply synonyms and related terms
        for term in &original_terms {
            term_weights.insert(term.clone(), 1.0); // Original terms get full weight
            
            // Add synonyms with reduced weight
            if let Some(synonyms) = self.synonym_map.get(term) {
                for synonym in synonyms {
                    expanded_terms.push(synonym.clone());
                    term_weights.insert(synonym.clone(), 0.7);
                }
            }
            
            // Add language equivalents
            if let Some(equivalents) = self.language_mappings.get(term) {
                for equivalent in equivalents {
                    expanded_terms.push(equivalent.clone());
                    term_weights.insert(equivalent.clone(), 0.8);
                }
            }
            
            // Add co-occurring terms
            if let Some(co_terms) = self.term_co_occurrence.get(term) {
                let top_co_terms: Vec<_> = co_terms.iter()
                    .filter(|(_, score)| **score > 0.5)
                    .take(3)
                    .collect();
                    
                for (co_term, score) in top_co_terms {
                    expanded_terms.push(co_term.clone());
                    term_weights.insert(co_term.clone(), score * 0.6);
                }
            }
        }
        
        ExpandedQuery {
            original_query: original_query.to_string(),
            original_terms,
            expanded_terms,
            term_weights,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExpandedQuery {
    pub original_query: String,
    pub original_terms: Vec<String>,
    pub expanded_terms: Vec<String>,
    pub term_weights: HashMap<String, f32>,
}
```

### **Task 5.2: Phrase Query Support**

**File:** `src/search/phrase_search.rs`

```rust
/// Advanced phrase search for BM25 with proximity scoring
pub struct PhraseSearchEngine {
    // Configuration
    max_phrase_distance: usize,  // Maximum words between phrase terms
    phrase_boost_factor: f32,    // Score multiplier for phrase matches
    
    // Phrase detection patterns
    quote_patterns: Regex,       // "exact phrase" detection
    identifier_patterns: Regex,  // snake_case, camelCase detection
}

impl PhraseSearchEngine {
    /// Search for phrases with proximity-based scoring
    pub fn search_phrases(
        &self,
        query: &str,
        inverted_index: &InvertedIndex
    ) -> Vec<PhraseMatch> {
        let phrases = self.extract_phrases(query);
        let mut phrase_matches = Vec::new();
        
        for phrase in phrases {
            let matches = self.find_phrase_matches(&phrase, inverted_index);
            phrase_matches.extend(matches);
        }
        
        // Sort by phrase relevance score
        phrase_matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        phrase_matches
    }
    
    fn find_phrase_matches(
        &self,
        phrase: &Phrase,
        inverted_index: &InvertedIndex
    ) -> Vec<PhraseMatch> {
        let term_postings: Vec<_> = phrase.terms.iter()
            .filter_map(|term| inverted_index.get_posting_list(term))
            .collect();
            
        if term_postings.len() != phrase.terms.len() {
            return Vec::new(); // Some terms not found
        }
        
        // Find documents containing all phrase terms
        let candidate_docs = self.find_candidate_documents(&term_postings);
        
        let mut matches = Vec::new();
        for doc_id in candidate_docs {
            if let Some(phrase_match) = self.calculate_phrase_score(
                &phrase, &doc_id, &term_postings
            ) {
                matches.push(phrase_match);
            }
        }
        
        matches
    }
    
    fn calculate_phrase_score(
        &self,
        phrase: &Phrase,
        doc_id: &str,
        term_postings: &[&PostingList]
    ) -> Option<PhraseMatch> {
        // Get position information for each term in this document
        let mut term_positions: Vec<Vec<usize>> = Vec::new();
        
        for posting_list in term_postings {
            if let Some(entry) = posting_list.documents.iter()
                .find(|entry| entry.doc_id == doc_id) {
                term_positions.push(entry.positions.clone());
            } else {
                return None; // Term not in this document
            }
        }
        
        // Find the best phrase matches (closest proximity)
        let phrase_instances = self.find_phrase_instances(&term_positions);
        
        if phrase_instances.is_empty() {
            return None;
        }
        
        // Calculate proximity-based score
        let best_distance = phrase_instances.iter()
            .map(|instance| instance.span)
            .min()
            .unwrap();
            
        let proximity_score = if best_distance == phrase.terms.len() - 1 {
            // Perfect phrase match
            self.phrase_boost_factor
        } else {
            // Proximity-based scoring
            self.phrase_boost_factor * (1.0 / (1.0 + best_distance as f32))
        };
        
        Some(PhraseMatch {
            doc_id: doc_id.to_string(),
            phrase: phrase.clone(),
            score: proximity_score,
            instances: phrase_instances,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Phrase {
    pub text: String,
    pub terms: Vec<String>,
    pub is_exact: bool,  // From quoted phrase
}

#[derive(Debug, Clone)]
pub struct PhraseMatch {
    pub doc_id: String,
    pub phrase: Phrase,
    pub score: f32,
    pub instances: Vec<PhraseInstance>,
}

#[derive(Debug, Clone)]
pub struct PhraseInstance {
    pub start_position: usize,
    pub end_position: usize,
    pub span: usize,  // Distance between first and last term
}
```

---

## **Implementation Timeline & Milestones**

### **Week 1: Core Implementation**
- ✅ **Day 1-2**: BM25 algorithm implementation (`bm25.rs`)
- ✅ **Day 3-4**: Text processing and tokenization (`text_processor.rs`)
- ✅ **Day 5**: Inverted index structure (`inverted_index.rs`)

### **Week 2: System Integration**
- ✅ **Day 1-2**: Enhanced UnifiedSearcher with BM25 integration
- ✅ **Day 3-4**: Enhanced fusion logic with 4-way scoring
- ✅ **Day 5**: Configuration system updates

### **Week 3: Storage & Performance**
- ✅ **Day 1-2**: BM25 storage layer with LanceDB integration
- ✅ **Day 3-4**: Performance optimization and caching
- ✅ **Day 5**: Memory usage optimization

### **Week 4: Testing & Validation**
- ✅ **Day 1-2**: Comprehensive accuracy testing suite
- ✅ **Day 3-4**: Performance benchmarks and optimization
- ✅ **Day 5**: Integration testing and bug fixes

### **Week 5: Advanced Features (Optional)**
- ✅ **Day 1-2**: Query expansion and synonyms
- ✅ **Day 3-4**: Phrase search with proximity scoring
- ✅ **Day 5**: Documentation and final validation

---

## **Performance Targets**

### **Accuracy Improvements**
- **Current**: 95% search accuracy (19/20 test queries)
- **Target**: 97-98% search accuracy (target: 49/50 test queries)
- **Improvement**: +2-3% accuracy gain through statistical relevance

### **Latency Targets**
- **Single BM25 search**: <50ms (inverted index lookup)
- **Combined search (all 4 types)**: <500ms (current target maintained)
- **Indexing overhead**: <20% increase (BM25 indexing parallel with embedding)

### **Memory Targets**
- **Inverted index size**: <500MB for 100,000 code chunks
- **Term cache**: 100,000 frequently-accessed terms
- **Total memory increase**: <1GB additional (current system: <2GB)

### **Storage Targets**
- **Index compression**: 60-70% compression ratio for posting lists  
- **Incremental updates**: <1s per file update (maintain current target)
- **Index persistence**: Crash-safe storage with automatic recovery

---

## **Risk Mitigation & Fallback Plans**

### **Risk 1: Performance Degradation**
- **Mitigation**: Parallel BM25 search with existing searches
- **Fallback**: Configuration flag to disable BM25 if performance issues
- **Monitoring**: Continuous performance benchmarks during development

### **Risk 2: Memory Consumption**
- **Mitigation**: LRU caching for term indexes, compression for storage
- **Fallback**: Memory-mapped files for large inverted indexes
- **Monitoring**: Memory usage tracking in integration tests

### **Risk 3: Accuracy Regression**
- **Mitigation**: Comprehensive A/B testing against current 95% baseline
- **Fallback**: Adjustable fusion weights to prioritize existing search types
- **Monitoring**: Automated accuracy tests on every build

### **Risk 4: Integration Complexity**
- **Mitigation**: Incremental integration with feature flags
- **Fallback**: BM25 as optional module that can be disabled
- **Testing**: Extensive unit and integration test coverage

---

## **Success Metrics & Validation**

### **Primary Success Criteria**
1. **Accuracy**: ≥97% on comprehensive test suite (vs 95% baseline)
2. **Performance**: Combined search <500ms (maintain current target)
3. **Reliability**: All existing tests continue to pass
4. **Memory**: Total system memory usage <3GB (vs current 2GB)

### **Secondary Success Criteria**
1. **Query Coverage**: Improved results for multi-term queries
2. **Term Relevance**: Better handling of rare vs common terms
3. **Explainability**: Clear BM25 score breakdowns for debugging
4. **Maintainability**: Clean integration with existing codebase

### **Validation Strategy**
1. **Unit Tests**: 95% code coverage for all BM25 components
2. **Integration Tests**: All 100+ existing search tests pass
3. **Performance Tests**: Benchmark suite with regression detection
4. **Accuracy Tests**: A/B comparison against current system
5. **Stress Tests**: Large codebases (100,000+ files) performance validation

---

## **Future Enhancements**

### **Phase 6: Machine Learning Enhancements**
- **Learning to Rank**: Use BM25 scores as features in ML ranking model
- **Query Understanding**: Neural query expansion based on codebase patterns
- **Personalization**: User-specific search result ranking

### **Phase 7: Advanced Search Features**
- **Fuzzy Matching**: Handle typos and approximate string matching
- **Code Structure Awareness**: AST-aware term weighting
- **Cross-Language Search**: Unified search across multiple programming languages

### **Phase 8: Analytics & Insights**
- **Search Analytics**: Query pattern analysis and optimization suggestions
- **Code Quality Metrics**: BM25 scores for code documentation quality
- **Developer Productivity**: Search success rate tracking and improvement

---

This implementation will establish the embedding search system as a **best-in-class code search solution**, combining the strengths of exact matching, semantic understanding, structural analysis, and statistical relevance into a unified, high-performance search experience.