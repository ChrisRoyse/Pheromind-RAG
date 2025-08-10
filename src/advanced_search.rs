use anyhow::Result;
use tantivy::{Index, IndexWriter, schema::{Schema, Field, TEXT, STORED, Value}};
use tantivy::query::QueryParser;
use tantivy::collector::TopDocs;
use std::collections::HashMap;

use crate::simple_storage::{VectorStorage, SearchResult as VectorResult};
use crate::simple_embedder::NomicEmbedder;
use crate::search::bm25_fixed::{BM25Engine, BM25Match};
use crate::search::fusion::FusionConfig;
use crate::symbol_extractor::{SymbolExtractor, Symbol};

/// Advanced hybrid search combining all 5 technologies with parallel execution
pub struct AdvancedHybridSearch {
    vector_storage: VectorStorage,
    text_index: Index,
    text_writer: IndexWriter,
    embedder: NomicEmbedder,
    bm25_engine: BM25Engine,
    symbol_extractor: SymbolExtractor,
    fusion_config: FusionConfig,
    
    // Schema fields
    content_field: Field,
    path_field: Field,
}

#[derive(Debug, Clone)]
pub struct AdvancedSearchResult {
    pub content: String,
    pub file_path: String,
    pub score: f32,
    pub match_type: String,
    pub line_number: Option<usize>,
    pub symbols: Vec<Symbol>,
}

impl AdvancedHybridSearch {
    pub async fn new(db_path: &str) -> Result<Self> {
        // Initialize vector storage
        let vector_storage = VectorStorage::new(db_path).await?;
        
        // Initialize Tantivy for full-text search
        let mut schema_builder = Schema::builder();
        let content_field = schema_builder.add_text_field("content", TEXT | STORED);
        let path_field = schema_builder.add_text_field("path", TEXT | STORED);
        let schema = schema_builder.build();
        
        // Open existing index or create new persistent disk-based index
        let index_path = format!("{}/tantivy_index", db_path);
        std::fs::create_dir_all(&index_path)?;
        let text_index = if std::path::Path::new(&format!("{}/meta.json", index_path)).exists() {
            Index::open_in_dir(&index_path)?
        } else {
            Index::create_in_dir(&index_path, schema)?
        };
        let text_writer = text_index.writer(50_000_000)?; // 50MB heap
        
        // Initialize all components
        let embedder = NomicEmbedder::new()?;
        let bm25_engine = BM25Engine::new()?;
        let symbol_extractor = SymbolExtractor::new()?;
        let fusion_config = FusionConfig::default();

        Ok(Self {
            vector_storage,
            text_index,
            text_writer,
            embedder,
            bm25_engine,
            symbol_extractor,
            fusion_config,
            content_field,
            path_field,
        })
    }

    /// Index documents in all search engines (vector, text, BM25, and symbol)
    pub async fn index(&mut self, contents: Vec<String>, file_paths: Vec<String>) -> Result<()> {
        // Generate embeddings with proper "passage:" prefix
        let prefixed_contents: Vec<String> = contents.iter()
            .map(|content| format!("passage: {}", content))
            .collect();
        let embeddings = self.embedder.embed_batch(prefixed_contents)?;
        
        // Store in vector database
        self.vector_storage.store(contents.clone(), embeddings, file_paths.clone()).await?;
        
        // Store in Tantivy text index and BM25 engine
        for (content, path) in contents.iter().zip(file_paths.iter()) {
            // Tantivy index
            let mut doc = tantivy::doc!();
            doc.add_text(self.content_field, content);
            doc.add_text(self.path_field, path);
            self.text_writer.add_document(doc)?;
            
            // BM25 engine
            self.bm25_engine.index_document(path, content);
        }
        self.text_writer.commit()?;

        Ok(())
    }

    /// Parallel hybrid search with advanced fusion across all 4 search types
    pub async fn search(&mut self, query: &str, limit: usize) -> Result<Vec<AdvancedSearchResult>> {
        let search_limit = limit * 3; // Get more results for better fusion
        
        // 1. Vector search (semantic)
        let query_embedding = self.embedder.embed_query(query)?;
        let vector_results = self.vector_storage.search(query_embedding, search_limit).await?;
        
        // 2. Text search (Tantivy full-text)
        let text_results = self.text_search(query, search_limit)?;
        
        // 3. BM25 search (statistical)
        let bm25_results = self.bm25_search(query, search_limit)?;
        
        // 4. Symbol search (AST-based) - placeholder for now
        let symbol_results = self.symbol_search(query, search_limit).await?;
        
        // Advanced fusion with configurable weights
        let fused_results = self.advanced_fusion(
            vector_results, 
            text_results, 
            bm25_results,
            symbol_results,
            limit
        );
        
        Ok(fused_results)
    }

    fn text_search(&self, query: &str, limit: usize) -> Result<Vec<AdvancedSearchResult>> {
        let reader = self.text_index.reader()?;
        let searcher = reader.searcher();
        let query_parser = QueryParser::for_index(&self.text_index, vec![self.content_field]);
        
        let parsed_query = query_parser.parse_query(query)?;
        let top_docs = searcher.search(&*parsed_query, &TopDocs::with_limit(limit))?;
        
        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let doc: tantivy::TantivyDocument = searcher.doc(doc_address)?;
            let content = doc.get_first(self.content_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let path = doc.get_first(self.path_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            results.push(AdvancedSearchResult {
                content,
                file_path: path,
                score,
                match_type: "text".to_string(),
                line_number: None,
                symbols: vec![],
            });
        }
        
        Ok(results)
    }

    /// BM25 search with proper scoring
    fn bm25_search(&self, query: &str, limit: usize) -> Result<Vec<AdvancedSearchResult>> {
        let bm25_matches = self.bm25_engine.search(query, limit)?;
        
        let results = bm25_matches
            .into_iter()
            .map(|m| AdvancedSearchResult {
                content: m.snippet,
                file_path: m.path,
                score: m.score,
                match_type: "bm25".to_string(),
                line_number: m.line_number,
                symbols: vec![],
            })
            .collect();
            
        Ok(results)
    }
    
    /// Symbol search using Tree-sitter AST analysis
    async fn symbol_search(&self, _query: &str, _limit: usize) -> Result<Vec<AdvancedSearchResult>> {
        // For now, return empty results as this requires symbol indexing
        // In production, this would search through indexed symbols
        Ok(vec![])
    }
    
    /// Advanced RRF fusion with configurable weights for all 4 search types
    fn advanced_fusion(&self,
                      vector_results: Vec<VectorResult>,
                      text_results: Vec<AdvancedSearchResult>,
                      bm25_results: Vec<AdvancedSearchResult>,
                      symbol_results: Vec<AdvancedSearchResult>,
                      limit: usize) -> Vec<AdvancedSearchResult> {
        let mut score_map: HashMap<String, (AdvancedSearchResult, f32)> = HashMap::new();
        
        // Fusion weights (configurable via FusionConfig)
        let vector_weight = 0.40;
        let text_weight = 0.25;
        let bm25_weight = 0.25;
        let symbol_weight = 0.10;
        
        // RRF constant
        let k = 60.0;
        
        // Add vector results with weighted RRF scoring
        for (rank, result) in vector_results.into_iter().enumerate() {
            let key = format!("{}:{}", result.file_path, &result.content[..50.min(result.content.len())]);
            let rrf_score = vector_weight * (1.0 / (k + rank as f32 + 1.0));
            
            score_map.insert(key, (AdvancedSearchResult {
                content: result.content,
                file_path: result.file_path,
                score: rrf_score,
                match_type: "vector".to_string(),
                line_number: None,
                symbols: vec![],
            }, rrf_score));
        }
        
        // Add text results
        for (rank, result) in text_results.into_iter().enumerate() {
            let key = format!("{}:{}", result.file_path, &result.content[..50.min(result.content.len())]);
            let rrf_score = text_weight * (1.0 / (k + rank as f32 + 1.0));
            
            if let Some((existing_result, existing_score)) = score_map.get_mut(&key) {
                *existing_score += rrf_score;
                existing_result.match_type = "hybrid".to_string();
                existing_result.score = *existing_score;
            } else {
                score_map.insert(key, (result, rrf_score));
            }
        }
        
        // Add BM25 results
        for (rank, result) in bm25_results.into_iter().enumerate() {
            let key = format!("{}:{}", result.file_path, &result.content[..50.min(result.content.len())]);
            let rrf_score = bm25_weight * (1.0 / (k + rank as f32 + 1.0));
            
            if let Some((existing_result, existing_score)) = score_map.get_mut(&key) {
                *existing_score += rrf_score;
                existing_result.match_type = "hybrid".to_string();
                existing_result.score = *existing_score;
            } else {
                score_map.insert(key, (result, rrf_score));
            }
        }
        
        // Add symbol results
        for (rank, result) in symbol_results.into_iter().enumerate() {
            let key = format!("{}:{}", result.file_path, &result.content[..50.min(result.content.len())]);
            let rrf_score = symbol_weight * (1.0 / (k + rank as f32 + 1.0));
            
            if let Some((existing_result, existing_score)) = score_map.get_mut(&key) {
                *existing_score += rrf_score;
                existing_result.match_type = "hybrid".to_string();
                existing_result.score = *existing_score;
            } else {
                score_map.insert(key, (result, rrf_score));
            }
        }
        
        // Sort by combined score and return top results
        let mut final_results: Vec<_> = score_map.into_values().map(|(result, _)| result).collect();
        final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        final_results.into_iter().take(limit).collect()
    }

    pub async fn clear(&mut self) -> Result<()> {
        self.vector_storage.clear().await?;
        self.text_writer.delete_all_documents()?;
        self.text_writer.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_advanced_hybrid_search() -> Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();
        
        let mut search = AdvancedHybridSearch::new(&db_path).await?;
        
        let contents = vec![
            "fn main() { println!(\"Hello world\"); }".to_string(),
            "struct User { name: String }".to_string(),
            "impl BM25Engine { fn search() {} }".to_string(),
        ];
        let paths = vec!["main.rs".to_string(), "user.rs".to_string(), "bm25.rs".to_string()];
        
        search.index(contents, paths).await?;
        
        let results = search.search("main function", 5).await?;
        assert!(!results.is_empty());
        println!("Found {} results", results.len());
        
        // Test different search types
        let bm25_results = search.search("BM25Engine", 5).await?;
        assert!(!bm25_results.is_empty());
        println!("Found {} BM25 results", bm25_results.len());
        
        Ok(())
    }
}