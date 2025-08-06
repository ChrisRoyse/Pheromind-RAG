use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::{Result, anyhow};
use tokio::sync::RwLock;

use crate::chunking::{SimpleRegexChunker, Chunk, ThreeChunkExpander, ChunkContext};
use crate::embedding::NomicEmbedder;
use crate::storage::lancedb_storage::LanceDBStorage;
use crate::search::{
    RipgrepSearcher, SimpleFusion, QueryPreprocessor, SearchCache,
    FusedResult, MatchType,
};

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file: String,
    pub three_chunk_context: ChunkContext,
    pub score: f32,
    pub match_type: MatchType,
}

pub struct UnifiedSearcher {
    ripgrep: RipgrepSearcher,
    embedder: Arc<NomicEmbedder>,
    storage: Arc<RwLock<LanceDBStorage>>,
    chunker: SimpleRegexChunker,
    expander: ThreeChunkExpander,
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
    
    pub async fn new_with_config(project_path: PathBuf, db_path: PathBuf, include_test_files: bool) -> Result<Self> {
        println!("🔄 Initializing Unified Searcher (include_test_files: {})...", include_test_files);
        
        // Initialize Nomic embedder with permanent model caching
        let embedder = NomicEmbedder::get_global().await
            .map_err(|e| anyhow!("Failed to initialize Nomic embedder: {}", e))?;
        
        let storage = Arc::new(RwLock::new(LanceDBStorage::new(db_path).await?));
        
        // Initialize storage table
        storage.write().await.init_table().await?;
        
        println!("✅ Nomic embedder initialized with 768-dimensional embeddings");
        
        Ok(Self {
            ripgrep: RipgrepSearcher::new(),
            embedder,
            storage,
            chunker: SimpleRegexChunker::new(),
            expander: ThreeChunkExpander,
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
        
        // Run searches in parallel
        let (exact_matches, semantic_matches) = tokio::join!(
            self.search_exact(&processed_query),
            self.search_semantic(&processed_query)
        );
        
        let exact_matches = exact_matches?;
        let semantic_matches = semantic_matches?;
        
        println!("📊 Found {} exact matches and {} semantic matches", 
                 exact_matches.len(), semantic_matches.len());
        
        // Fuse results
        let mut fused = self.fusion.fuse_results(exact_matches, semantic_matches);
        
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
    
    async fn search_exact(&self, query: &str) -> Result<Vec<crate::search::ripgrep::ExactMatch>> {
        tokio::task::spawn_blocking({
            let ripgrep = RipgrepSearcher::new();
            let project_path = self.project_path.clone();
            let query = query.to_string();
            move || ripgrep.search(&query, &project_path)
        }).await?
    }
    
    async fn search_semantic(&self, query: &str) -> Result<Vec<crate::storage::lancedb_storage::LanceEmbeddingRecord>> {
        // Generate query embedding using cached embedder
        let query_embedding = self.embedder.embed(query)
            .map_err(|e| anyhow!("Failed to generate query embedding: {}", e))?;
        
        // Search in vector database
        let storage = self.storage.read().await;
        storage.search_similar(query_embedding, 30).await
            .map_err(|e| anyhow::anyhow!("Vector search failed: {}", e))
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
        
        // Prepare chunk contents for batch embedding
        let chunk_contents: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
        
        // Generate embeddings using Nomic embedder
        let embeddings = self.embedder.embed_batch(&chunk_contents)
            .map_err(|e| anyhow!("Failed to generate embeddings for file {:?}: {}", file_path, e))?;
        
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
        let storage = self.storage.write().await;
        storage.clear_all().await?;
        self.cache.clear();
        println!("🧹 Cleared all indexed data");
        Ok(())
    }
    
    pub async fn get_stats(&self) -> Result<SearcherStats> {
        let storage = self.storage.read().await;
        let total_embeddings = storage.count().await?;
        let search_cache_stats = self.cache.stats();
        let embedding_cache_stats = crate::embedding::CacheStats {
            entries: 0,
            max_size: 100_000,
            hit_ratio: 0.0,
        };
        
        Ok(SearcherStats {
            total_embeddings,
            cache_entries: search_cache_stats.valid_entries,
            cache_max_size: search_cache_stats.max_size,
            embedding_cache_entries: embedding_cache_stats.entries,
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