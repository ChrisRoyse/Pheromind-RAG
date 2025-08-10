use anyhow::Result;
use tantivy::{Index, IndexWriter, schema::{Schema, Field, TEXT, STORED, Value}};
use tantivy::query::QueryParser;
use tantivy::collector::TopDocs;
use std::collections::HashMap;

use crate::simple_storage::{VectorStorage, SearchResult as VectorResult};
use crate::simple_embedder::NomicEmbedder;
// BM25Engine and BM25Match temporarily removed
// FusionConfig and MatchType temporarily removed
// ChunkContext and Chunk temporarily removed
// BoundedCache temporarily removed

/// Simple hybrid search combining LanceDB + Tantivy
pub struct HybridSearch {
    vector_storage: VectorStorage,
    text_index: Index,
    text_writer: IndexWriter,
    embedder: NomicEmbedder,
    
    // Schema fields
    content_field: Field,
    path_field: Field,
}

pub struct SearchResult {
    pub content: String,
    pub file_path: String,
    pub score: f32,
    pub match_type: String,
}

impl HybridSearch {
    pub async fn new(db_path: &str) -> Result<Self> {
        // Initialize vector storage
        let vector_storage = VectorStorage::new(db_path).await?;
        
        // Initialize Tantivy for full-text search
        let mut schema_builder = Schema::builder();
        let content_field = schema_builder.add_text_field("content", TEXT | STORED);
        let path_field = schema_builder.add_text_field("path", TEXT | STORED);
        let schema = schema_builder.build();
        
        let text_index = Index::create_in_ram(schema);
        let text_writer = text_index.writer(50_000_000)?; // 50MB heap
        
        // Initialize embedder
        let embedder = NomicEmbedder::new()?;

        Ok(Self {
            vector_storage,
            text_index,
            text_writer,
            embedder,
            content_field,
            path_field,
        })
    }

    /// Index documents in both vector and text indices
    pub async fn index(&mut self, contents: Vec<String>, file_paths: Vec<String>) -> Result<()> {
        // Generate embeddings
        let embeddings = self.embedder.embed_batch(contents.clone())?;
        
        // Store in vector database
        self.vector_storage.store(contents.clone(), embeddings, file_paths.clone()).await?;
        
        // Store in text index
        for (content, path) in contents.iter().zip(file_paths.iter()) {
            let mut doc = tantivy::doc!();
            doc.add_text(self.content_field, content);
            doc.add_text(self.path_field, path);
            self.text_writer.add_document(doc)?;
        }
        self.text_writer.commit()?;

        Ok(())
    }

    /// Hybrid search with simple RRF fusion
    pub async fn search(&mut self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // Vector search
        let query_embedding = self.embedder.embed_query(query)?;
        let vector_results = self.vector_storage.search(query_embedding, limit * 2).await?;
        
        // Text search
        let text_results = self.text_search(query, limit * 2)?;
        
        // Simple RRF fusion
        let fused_results = self.simple_rrf_fusion(vector_results, text_results, limit);
        
        Ok(fused_results)
    }

    fn text_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // Create reader without reload policy (not available in tantivy 0.22)
        let reader = self.text_index.reader()?;
        
        let searcher = reader.searcher();
        let query_parser = QueryParser::for_index(&self.text_index, vec![self.content_field]);
        
        // Try both exact and fuzzy search
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
            
            results.push(SearchResult {
                content,
                file_path: path,
                score,
                match_type: "text".to_string(),
            });
        }
        
        Ok(results)
    }

    /// Simple RRF fusion - not over-engineered
    fn simple_rrf_fusion(&self, 
                         vector_results: Vec<VectorResult>, 
                         text_results: Vec<SearchResult>, 
                         limit: usize) -> Vec<SearchResult> {
        let mut score_map: HashMap<String, (SearchResult, f32)> = HashMap::new();
        
        // Add vector results with RRF scoring
        for (rank, result) in vector_results.into_iter().enumerate() {
            let key = format!("{}:{}", result.file_path, &result.content[..50.min(result.content.len())]);
            let rrf_score = 1.0 / (60.0 + rank as f32 + 1.0);
            
            score_map.insert(key, (SearchResult {
                content: result.content,
                file_path: result.file_path,
                score: rrf_score,
                match_type: "vector".to_string(),
            }, rrf_score));
        }
        
        // Add text results with RRF scoring
        for (rank, result) in text_results.into_iter().enumerate() {
            let key = format!("{}:{}", result.file_path, &result.content[..50.min(result.content.len())]);
            let rrf_score = 1.0 / (60.0 + rank as f32 + 1.0);
            
            if let Some((existing_result, existing_score)) = score_map.get_mut(&key) {
                *existing_score += rrf_score;
                existing_result.match_type = "hybrid".to_string();
                existing_result.score = *existing_score;
            } else {
                score_map.insert(key, (result, rrf_score));
            }
        }
        
        // Sort by combined score
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
    async fn test_hybrid_search() -> Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();
        
        let mut search = HybridSearch::new(&db_path).await?;
        
        let contents = vec![
            "fn main() { println!(\"Hello world\"); }".to_string(),
            "struct User { name: String }".to_string(),
        ];
        let paths = vec!["main.rs".to_string(), "user.rs".to_string()];
        
        search.index(contents, paths).await?;
        
        let results = search.search("main function", 5).await?;
        assert!(!results.is_empty());
        println!("Found {} results", results.len());
        
        Ok(())
    }
}