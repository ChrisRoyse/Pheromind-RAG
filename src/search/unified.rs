use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::Result;
use tokio::sync::RwLock;

use crate::chunking::{SimpleRegexChunker, Chunk, ThreeChunkExpander};
#[cfg(feature = "ml")]
use crate::embedding::nomic::NomicEmbedder;
#[cfg(feature = "vectordb")]
use crate::storage::lancedb_storage::LanceDBStorage;
use crate::search::fusion::{SimpleFusion, FusedResult, MatchType};
// Import from basic modules with no dependencies first
use crate::search::ExactMatch;
use crate::search::preprocessing::QueryPreprocessor;
use crate::search::bm25::{BM25Engine, BM25Match, BM25Document, Token as BM25Token};
use crate::search::text_processor::CodeTextProcessor;

// Import from modules that depend on basic modules
use crate::search::inverted_index::{InvertedIndex, DocumentMetadata};
use crate::search::cache::SearchCache;
#[cfg(feature = "tantivy")]
use crate::search::search_adapter::{TextSearcher, create_text_searcher_with_root};
#[cfg(feature = "tree-sitter")]
use crate::search::symbol_index::{SymbolIndexer, SymbolDatabase, Symbol};
use crate::config::{Config, SearchBackend};

// Helper struct for when ml feature is not available
#[cfg(not(feature = "ml"))]
struct MockCacheStats {
    entries: usize,
    max_size: usize,
}

// Use SearchResult from cache module to avoid duplication
pub use crate::search::cache::SearchResult;

pub struct UnifiedSearcher {
    #[cfg(feature = "tantivy")]
    text_searcher: Arc<RwLock<Box<dyn TextSearcher>>>,
    #[cfg(feature = "ml")]
    embedder: Arc<NomicEmbedder>,
    #[cfg(feature = "vectordb")]
    storage: Arc<RwLock<LanceDBStorage>>,
    #[cfg(feature = "tree-sitter")]
    symbol_indexer: Arc<RwLock<SymbolIndexer>>,
    #[cfg(feature = "tree-sitter")]
    symbol_db: Arc<RwLock<SymbolDatabase>>,
    // BM25 components (always available)
    bm25_engine: Arc<RwLock<BM25Engine>>,
    inverted_index: Arc<RwLock<InvertedIndex>>,
    text_processor: CodeTextProcessor,
    bm25_enabled: bool,
    chunker: SimpleRegexChunker,
    fusion: SimpleFusion,
    preprocessor: QueryPreprocessor,
    cache: SearchCache,
    project_path: PathBuf,
    include_test_files: bool,
}

impl UnifiedSearcher {
    pub async fn new(project_path: PathBuf, db_path: PathBuf) -> Result<Self> {
        Self::new_with_config(project_path, db_path, false).await
    }
    
    /// Create a new UnifiedSearcher with a specific search backend
    pub async fn new_with_backend(project_path: PathBuf, db_path: PathBuf, backend: SearchBackend) -> Result<Self> {
        Self::new_with_backend_and_config(project_path, db_path, backend, false).await
    }
    
    pub async fn new_with_config(project_path: PathBuf, db_path: PathBuf, include_test_files: bool) -> Result<Self> {
        let backend = Config::search_backend()?;
        Self::new_with_backend_and_config(project_path, db_path, backend, include_test_files).await
    }
    
    /// Create a new UnifiedSearcher with a specific backend and config
    pub async fn new_with_backend_and_config(project_path: PathBuf, db_path: PathBuf, backend: SearchBackend, include_test_files: bool) -> Result<Self> {
        println!("🔄 Initializing Unified Searcher (backend: {:?}, include_test_files: {})...", backend, include_test_files);
        
        // Initialize Nomic embedder with permanent model caching
        #[cfg(feature = "ml")]
        let embedder = NomicEmbedder::get_global()
            .map_err(|e| anyhow::anyhow!("Failed to initialize Nomic embedder: {}", e))?;
        
        #[cfg(feature = "vectordb")]
        let storage = Arc::new(RwLock::new(LanceDBStorage::new(db_path.clone()).await?));
        
        // Initialize storage table
        #[cfg(feature = "vectordb")]
        storage.write().await.init_table().await?;
        
        // Initialize symbol indexer and database
        #[cfg(feature = "tree-sitter")]
        let symbol_indexer = Arc::new(RwLock::new(SymbolIndexer::new()?));
        #[cfg(feature = "tree-sitter")]
        let symbol_db = Arc::new(RwLock::new(SymbolDatabase::new()));
        
        // Load configuration
        let config = Config::default();
        
        // Initialize BM25 components
        let bm25_engine = Arc::new(RwLock::new(BM25Engine::with_params(config.bm25_k1, config.bm25_b)));
        let bm25_index_path = db_path.join(&config.bm25_index_path);
        let mut inverted_index = InvertedIndex::new(bm25_index_path, config.bm25_cache_size)?;
        
        // Try to load existing index
        if let Err(e) = inverted_index.load().await {
            println!("⚠️ Could not load existing BM25 index: {}. Starting fresh.", e);
        }
        
        let inverted_index = Arc::new(RwLock::new(inverted_index));
        let text_processor = CodeTextProcessor::with_config(
            config.enable_stemming,
            config.enable_ngrams,
            config.max_ngram_size,
            config.bm25_min_term_length,
            config.bm25_max_term_length,
            config.bm25_stop_words.clone(),
        );
        let bm25_enabled = config.bm25_enabled;
        
        #[cfg(feature = "ml")]
        println!("✅ Nomic embedder initialized with 768-dimensional embeddings");
        #[cfg(feature = "tree-sitter")]
        println!("✅ Symbol indexer initialized with tree-sitter parsers");
        println!("✅ BM25 engine initialized with TF-IDF scoring");
        
        // Create text searcher with specified backend
        #[cfg(feature = "tantivy")]
        let text_searcher = create_text_searcher_with_root(&backend, project_path.clone()).await
            .map_err(|e| anyhow::anyhow!("Failed to create text searcher with backend {:?}: {}", backend, e))?;
        
        #[cfg(feature = "tantivy")]
        println!("✅ Text searcher initialized with backend: {:?}", backend);
            
        Ok(Self {
            #[cfg(feature = "tantivy")]
            text_searcher: Arc::new(RwLock::new(text_searcher)),
            #[cfg(feature = "ml")]
            embedder,
            #[cfg(feature = "vectordb")]
            storage,
            #[cfg(feature = "tree-sitter")]
            symbol_indexer,
            #[cfg(feature = "tree-sitter")]
            symbol_db,
            bm25_engine,
            inverted_index,
            text_processor,
            bm25_enabled,
            chunker: SimpleRegexChunker::new(),
            fusion: SimpleFusion::new(),
            preprocessor: QueryPreprocessor::new(),
            cache: SearchCache::new(100),
            project_path,
            include_test_files,
        })
    }
    
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        // Check cache first
        if let Some(cached) = self.cache.get(query) {
            println!("📦 Returning cached results for query: {}", query);
            return Ok(cached);
        }
        
        // Preprocess query
        let processed_query = self.preprocessor.preprocess(query);
        println!("🔍 Searching for: '{}' (preprocessed: '{}')", query, processed_query);
        
        // Run searches based on available features
        let mut fused = if self.bm25_enabled {
            #[cfg(all(feature = "tantivy", feature = "vectordb", feature = "tree-sitter"))]
            {
                // Run ALL FOUR searches in parallel
                let (exact_matches, semantic_matches, symbol_matches, bm25_matches) = tokio::join!(
                    self.search_exact(&processed_query),
                    self.search_semantic(&processed_query),
                    self.search_symbols(&processed_query),
                    self.search_bm25(&processed_query)
                );
                
                let exact_matches = exact_matches?;
                let semantic_matches = semantic_matches?;
                let symbol_matches = symbol_matches?;
                let bm25_matches = bm25_matches?;
                
                println!("📊 Found {} exact, {} semantic, {} symbol, and {} BM25 matches", 
                         exact_matches.len(), semantic_matches.len(), symbol_matches.len(), bm25_matches.len());
                
                // Fuse all four types of results
                self.fusion.fuse_all_results_with_bm25(exact_matches, semantic_matches, symbol_matches, bm25_matches)
            }
            #[cfg(not(all(feature = "tantivy", feature = "vectordb", feature = "tree-sitter")))]
            {
                // Fallback to BM25 only when other features are missing
                let bm25_matches = self.search_bm25(&processed_query).await?;
                println!("📊 Found {} BM25 matches (limited features)", bm25_matches.len());
                
                // Convert BM25 matches to fused results using inverted index
                let mut results = Vec::new();
                let index = self.inverted_index.read().await;
                
                for bm25_match in bm25_matches {
                    if let Some(doc_metadata) = index.get_document_metadata(&bm25_match.doc_id) {
                        results.push(FusedResult {
                            file_path: doc_metadata.file_path.clone(),
                            score: bm25_match.score,
                            match_type: MatchType::Statistical,
                            line_number: Some(doc_metadata.chunk_index * 50), // Approximate line from chunk
                            start_line: doc_metadata.chunk_index * 50,
                            end_line: (doc_metadata.chunk_index + 1) * 50,
                            chunk_index: Some(doc_metadata.chunk_index),
                            content: format!("BM25 match in {} (score: {:.3})", doc_metadata.file_path, bm25_match.score),
                        });
                    }
                }
                results
            }
        } else {
            #[cfg(all(feature = "tantivy", feature = "vectordb", feature = "tree-sitter"))]
            {
                // Run original three searches
                let (exact_matches, semantic_matches, symbol_matches) = tokio::join!(
                    self.search_exact(&processed_query),
                    self.search_semantic(&processed_query),
                    self.search_symbols(&processed_query)
                );
                
                let exact_matches = exact_matches?;
                let semantic_matches = semantic_matches?;
                let symbol_matches = symbol_matches?;
                
                println!("📊 Found {} exact, {} semantic, and {} symbol matches", 
                         exact_matches.len(), semantic_matches.len(), symbol_matches.len());
                
                // Fuse three types of results
                self.fusion.fuse_all_results(exact_matches, semantic_matches, symbol_matches)
            }
            #[cfg(not(all(feature = "tantivy", feature = "vectordb", feature = "tree-sitter")))]
            {
                // Minimal functionality fallback
                println!("📊 Running with minimal features enabled");
                Vec::new()
            }
        };
        
        // Optimize ranking
        self.fusion.optimize_ranking(&mut fused, &processed_query);
        
        // Expand to 3-chunk contexts
        let mut results = Vec::new();
        for fused_match in fused {
            match self.expand_to_three_chunk(fused_match).await {
                Ok(result) => results.push(result),
                Err(e) => eprintln!("⚠️  Failed to expand chunk: {}", e),
            }
        }
        
        // Cache results
        self.cache.insert(query.to_string(), results.clone());
        
        println!("✅ Returning {} search results", results.len());
        Ok(results)
    }
    
    #[allow(dead_code)]
    async fn search_exact(&self, _query: &str) -> Result<Vec<ExactMatch>> {
        #[cfg(feature = "tantivy")]
        {
            let searcher = self.text_searcher.read().await;
            searcher.search(_query).await
        }
        #[cfg(not(feature = "tantivy"))]
        {
            Ok(Vec::new())
        }
    }
    
    #[cfg(all(feature = "ml", feature = "vectordb"))]
    async fn search_semantic(&self, query: &str) -> Result<Vec<crate::storage::lancedb_storage::LanceEmbeddingRecord>> {
        // Generate query embedding using cached embedder
        let query_embedding = self.embedder.embed(query)
            .map_err(|e| anyhow::anyhow!("Failed to generate query embedding: {}", e))?;
        
        // Search in vector database
        let storage = self.storage.read().await;
        storage.search_similar(query_embedding, 30).await
            .map_err(|e| anyhow::anyhow!("Vector search failed: {}", e))
    }
    
    #[cfg(not(all(feature = "ml", feature = "vectordb")))]
    #[allow(dead_code)]
    async fn search_semantic(&self, _query: &str) -> Result<Vec<()>> {
        Ok(Vec::new())
    }
    
    #[cfg(feature = "tree-sitter")]
    async fn search_symbols(&self, query: &str) -> Result<Vec<Symbol>> {
        // Search in symbol database
        let db = self.symbol_db.read().await;
        
        // Try to find exact definition first
        if let Some(symbol) = db.find_definition(query) {
            return Ok(vec![symbol]);
        }
        
        // Otherwise find all references
        let symbols = db.find_all_references(query);
        
        // Return owned symbols
        Ok(symbols)
    }
    
    #[cfg(not(feature = "tree-sitter"))]
    #[allow(dead_code)]
    async fn search_symbols(&self, _query: &str) -> Result<Vec<()>> {
        Ok(Vec::new())
    }
    
    async fn search_bm25(&self, query: &str) -> Result<Vec<BM25Match>> {
        if !self.bm25_enabled {
            return Ok(Vec::new());
        }
        
        let engine = self.bm25_engine.read().await;
        Ok(engine.search(query, 50))  // Get top 50 BM25 matches
    }
    
    async fn expand_to_three_chunk(&self, fused_match: FusedResult) -> Result<SearchResult> {
        // Read file content
        let file_path = PathBuf::from(&fused_match.file_path);
        let full_path = if file_path.is_absolute() {
            file_path
        } else {
            self.project_path.join(&file_path)
        };
        
        let content = tokio::fs::read_to_string(&full_path).await?;
        let chunks = self.chunker.chunk_file(&content);
        
        // Find the relevant chunk
        let chunk_idx = match fused_match.match_type {
            MatchType::Exact => {
                // Find chunk containing the line
                self.find_chunk_for_line(&chunks, fused_match.line_number.unwrap_or(0))
            },
            MatchType::Semantic => {
                // Use the chunk index directly
                fused_match.chunk_index.unwrap_or(0).min(chunks.len().saturating_sub(1))
            },
            MatchType::Symbol => {
                // Find chunk containing the symbol's line
                self.find_chunk_for_line(&chunks, fused_match.line_number.unwrap_or(fused_match.start_line))
            },
            MatchType::Statistical => {
                // Use the chunk index from BM25 match
                fused_match.chunk_index.unwrap_or(0).min(chunks.len().saturating_sub(1))
            }
        };
        
        // Expand to 3-chunk context
        let three_chunk = ThreeChunkExpander::expand(&chunks, chunk_idx)?;
        
        Ok(SearchResult {
            file: fused_match.file_path,
            three_chunk_context: three_chunk,
            score: fused_match.score,
            match_type: fused_match.match_type,
        })
    }
    
    fn find_chunk_for_line(&self, chunks: &[Chunk], line_number: usize) -> usize {
        for (idx, chunk) in chunks.iter().enumerate() {
            if line_number >= chunk.start_line && line_number <= chunk.end_line {
                return idx;
            }
        }
        // Default to first chunk if not found
        0
    }
    
    pub async fn index_file(&self, file_path: &Path) -> Result<()> {
        println!("📝 Indexing file: {:?}", file_path);
        
        let content = tokio::fs::read_to_string(file_path).await?;
        let chunks = self.chunker.chunk_file(&content);
        
        if chunks.is_empty() {
            println!("⏭️  Skipping empty file: {:?}", file_path);
            return Ok(());
        }
        
        // Extract and index symbols if it's a supported language
        #[cfg(feature = "tree-sitter")]
        if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
            // Detect language from file extension
            let language = match ext {
                "rs" => Some("rust"),
                "py" => Some("python"),
                "js" => Some("javascript"),
                "ts" => Some("typescript"),
                "tsx" => Some("tsx"),
                "go" => Some("go"),
                "java" => Some("java"),
                "cpp" | "cc" | "cxx" => Some("cpp"),
                "c" | "h" => Some("c"),
                "html" | "htm" => Some("html"),
                "css" => Some("css"),
                "json" => Some("json"),
                "sh" | "bash" => Some("bash"),
                _ => None,
            };
            
            if let Some(lang) = language {
                // Extract symbols using the indexer
                let mut indexer = self.symbol_indexer.write().await;
                match indexer.extract_symbols(&content, lang, file_path.to_str().unwrap_or("")) {
                    Ok(symbols) => {
                        if !symbols.is_empty() {
                            // Add symbols to the database
                            let mut db = self.symbol_db.write().await;
                            db.add_symbols(symbols.clone());
                            println!("🔍 Indexed {} symbols from {:?}", symbols.len(), file_path);
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to extract symbols from {:?}: {}", file_path, e);
                    }
                }
            }
        }
        
        // Prepare chunk contents for batch embedding
        #[cfg(all(feature = "ml", feature = "vectordb"))]
        {
            let chunk_contents: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
            
            // Generate embeddings using Nomic embedder
            let embeddings = self.embedder.embed_batch(&chunk_contents)
                .map_err(|e| anyhow::anyhow!("Failed to generate embeddings for file {:?}: {}", file_path, e))?;
            
            // Create records with embeddings
            let mut records = Vec::with_capacity(chunks.len());
            for ((idx, chunk), embedding) in chunks.iter().enumerate().zip(embeddings.into_iter()) {
                records.push(crate::storage::lancedb_storage::LanceEmbeddingRecord {
                    id: format!("{}-{}", file_path.to_string_lossy(), idx),
                    file_path: file_path.to_string_lossy().to_string(),
                    chunk_index: idx as u64,
                    content: chunk.content.clone(),
                    embedding,
                    start_line: chunk.start_line as u64,
                    end_line: chunk.end_line as u64,
                    similarity_score: None,
                });
            }
            
            // Insert all records in batch
            let storage = self.storage.write().await;
            storage.insert_batch(records).await?;
        }
        
        // Index for BM25 if enabled
        if self.bm25_enabled {
            // Process chunks for BM25 indexing
            for (idx, chunk) in chunks.iter().enumerate() {
                let doc_id = format!("{}-{}", file_path.to_string_lossy(), idx);
                
                // Detect language for optimal processing
                let language = file_path.extension()
                    .and_then(|s| s.to_str())
                    .and_then(|ext| match ext {
                        "rs" => Some("rust"),
                        "py" => Some("python"),
                        "js" => Some("javascript"),
                        "ts" => Some("typescript"),
                        "go" => Some("go"),
                        "java" => Some("java"),
                        "cpp" | "cc" | "cxx" => Some("cpp"),
                        "c" | "h" => Some("c"),
                        _ => None,
                    });
                
                // Tokenize chunk content for BM25
                let processed_tokens = self.text_processor.tokenize_code(&chunk.content, language);
                let bm25_tokens: Vec<BM25Token> = processed_tokens.iter()
                    .map(|pt| BM25Token {
                        text: pt.text.clone(),
                        position: pt.position,
                        importance_weight: pt.importance_weight,
                    })
                    .collect();
                
                // Create BM25 document
                let bm25_doc = BM25Document {
                    id: doc_id.clone(),
                    file_path: file_path.to_string_lossy().to_string(),
                    chunk_index: idx,
                    tokens: bm25_tokens.clone(),
                    start_line: chunk.start_line,
                    end_line: chunk.end_line,
                    language: language.map(|s| s.to_string()),
                };
                
                // Add to BM25 engine
                let mut engine = self.bm25_engine.write().await;
                engine.add_document(bm25_doc)?;
                
                // Add to inverted index
                let metadata = DocumentMetadata {
                    file_path: file_path.to_string_lossy().to_string(),
                    chunk_index: idx,
                    length: bm25_tokens.len(),
                    language: language.map(|s| s.to_string()),
                    last_modified: chrono::Utc::now(),
                };
                
                let mut index = self.inverted_index.write().await;
                index.index_document(doc_id, bm25_tokens, metadata)?;
            }
            
            // Save inverted index periodically
            let mut index = self.inverted_index.write().await;
            index.save().await?;
            
            println!("✅ Indexed {} chunks for BM25 from {:?}", chunks.len(), file_path);
        }
        
        // Index file for text search (important for backends like Tantivy)
        #[cfg(feature = "tantivy")]
        {
            let mut text_searcher = self.text_searcher.write().await;
            text_searcher.index_file(file_path).await?;
        }
        
        println!("✅ Indexed {} chunks from {:?}", chunks.len(), file_path);
        Ok(())
    }
    
    pub async fn index_directory(&self, dir_path: &Path) -> Result<IndexStats> {
        println!("📂 Indexing directory: {:?} (include_test_files: {})", dir_path, self.include_test_files);
        
        let mut stats = IndexStats::default();
        let mut entries = tokio::fs::read_dir(dir_path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_dir() {
                // Skip common non-code directories
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if matches!(name_str.as_ref(), "target" | "node_modules" | ".git" | "dist" | "build") {
                        continue;
                    }
                    
                    // Skip test directories unless explicitly included
                    if !self.include_test_files && self.is_test_directory(&name_str) {
                        println!("⏭️  Skipping test directory: {:?}", path);
                        continue;
                    }
                }
                
                // Skip test directories by path components
                if !self.include_test_files && self.is_test_path(&path) {
                    println!("⏭️  Skipping test path: {:?}", path);
                    continue;
                }
                
                // Recursively index subdirectory
                let sub_stats = Box::pin(self.index_directory(&path)).await?;
                stats.files_indexed += sub_stats.files_indexed;
                stats.chunks_created += sub_stats.chunks_created;
                stats.errors += sub_stats.errors;
            } else if self.is_code_file(&path) && (self.include_test_files || !self.is_test_file(&path)) {
                match self.index_file(&path).await {
                    Ok(_) => {
                        stats.files_indexed += 1;
                        let content = tokio::fs::read_to_string(&path).await?;
                        let chunks = self.chunker.chunk_file(&content);
                        stats.chunks_created += chunks.len();
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to index {:?}: {}", path, e);
                        stats.errors += 1;
                    }
                }
            } else if !self.include_test_files && self.is_test_file(&path) {
                println!("⏭️  Skipping test file: {:?}", path);
            }
        }
        
        Ok(stats)
    }
    
    fn is_code_file(&self, path: &Path) -> bool {
        match path.extension().and_then(|s| s.to_str()) {
            Some(ext) => matches!(
                ext,
                "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | 
                "go" | "java" | "cpp" | "c" | "h" | "hpp" |
                "rb" | "php" | "swift" | "kt" | "scala" | "cs" |
                "sql" | "md"
            ),
            None => false,
        }
    }
    
    fn is_test_directory(&self, dir_name: &str) -> bool {
        matches!(dir_name, "tests" | "test" | "__tests__" | "spec" | "specs")
    }
    
    fn is_test_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        path_str.contains("/tests/") || path_str.contains("/test/") ||
        path_str.contains("\\tests\\") || path_str.contains("\\test\\") ||
        path_str.contains("/__tests__/") || path_str.contains("\\__tests__\\") ||
        path_str.contains("/spec/") || path_str.contains("\\spec\\")
    }
    
    fn is_test_file(&self, path: &Path) -> bool {
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            // Check for test file patterns
            file_name.ends_with("_test.rs") ||
            file_name.ends_with("_tests.rs") ||
            file_name.ends_with(".test.js") ||
            file_name.ends_with(".test.ts") ||
            file_name.ends_with(".test.jsx") ||
            file_name.ends_with(".test.tsx") ||
            file_name.ends_with(".spec.js") ||
            file_name.ends_with(".spec.ts") ||
            file_name.ends_with(".spec.jsx") ||
            file_name.ends_with(".spec.tsx") ||
            file_name.ends_with("_spec.rb") ||
            file_name.ends_with("_test.py") ||
            file_name.ends_with("test_*.py") ||
            file_name.starts_with("test_") ||
            file_name.contains("_test_") ||
            file_name.contains(".test.") ||
            file_name.contains(".spec.")
        } else {
            false
        }
    }
    
    pub async fn clear_index(&self) -> Result<()> {
        #[cfg(feature = "vectordb")]
        {
            let storage = self.storage.write().await;
            storage.clear_all().await?;
        }
        self.cache.clear();
        
        // Clear text searcher index
        #[cfg(feature = "tantivy")]
        {
            let mut text_searcher = self.text_searcher.write().await;
            text_searcher.clear_index().await?;
        }
        
        // Clear symbol database
        #[cfg(feature = "tree-sitter")]
        {
            let mut symbol_db = self.symbol_db.write().await;
            symbol_db.clear();
        }
        
        println!("🧹 Cleared all indexed data");
        Ok(())
    }
    
    pub async fn get_stats(&self) -> Result<SearcherStats> {
        #[cfg(feature = "vectordb")]
        let total_embeddings = {
            let storage = self.storage.read().await;
            storage.count().await?
        };
        #[cfg(not(feature = "vectordb"))]
        let total_embeddings = 0;
        
        let search_cache_stats = self.cache.stats();
        
        #[cfg(feature = "ml")]
        let embedding_cache_stats = crate::embedding::CacheStats {
            entries: 0,
            max_size: 100_000,
            hit_ratio: 0.0,
        };
        #[cfg(not(feature = "ml"))]
        let embedding_cache_stats = MockCacheStats {
            entries: 0,
            max_size: 0,
        };
        
        Ok(SearcherStats {
            total_embeddings,
            cache_entries: search_cache_stats.valid_entries,
            cache_max_size: search_cache_stats.max_size,
            embedding_cache_entries: embedding_cache_stats.entries,
            #[cfg(feature = "ml")]
            embedding_cache_max_size: embedding_cache_stats.max_size,
            #[cfg(not(feature = "ml"))]
            embedding_cache_max_size: embedding_cache_stats.max_size,
        })
    }
}

#[derive(Debug, Default)]
pub struct IndexStats {
    pub files_indexed: usize,
    pub chunks_created: usize,
    pub errors: usize,
}

impl std::fmt::Display for IndexStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Indexed {} files, created {} chunks ({} errors)",
            self.files_indexed, self.chunks_created, self.errors
        )
    }
}

#[derive(Debug)]
pub struct SearcherStats {
    pub total_embeddings: usize,
    pub cache_entries: usize,
    pub cache_max_size: usize,
    pub embedding_cache_entries: usize,
    pub embedding_cache_max_size: usize,
}

impl std::fmt::Display for SearcherStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Embeddings: {}, Search Cache: {}/{}, Embedding Cache: {}/{}",
            self.total_embeddings, 
            self.cache_entries, self.cache_max_size,
            self.embedding_cache_entries, self.embedding_cache_max_size
        )
    }
}