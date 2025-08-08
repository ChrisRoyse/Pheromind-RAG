use std::path::Path;
use anyhow::Result;
use async_trait::async_trait;

use crate::search::{ExactMatch, TantivySearcher};

/// Trait for text search backends that can be used interchangeably
/// 
/// This trait provides a unified interface for TantivySearcher and other search implementations,
/// enabling seamless backend switching based on configuration without changing calling code.
#[async_trait]
pub trait TextSearcher: Send + Sync {
    /// Search for the given query and return matching results
    /// 
    /// # Arguments
    /// * `query` - The search query string
    /// 
    /// # Returns
    /// * `Result<Vec<ExactMatch>>` - A vector of exact matches found
    async fn search(&self, query: &str) -> Result<Vec<ExactMatch>>;
    
    /// Index a single file for searching
    /// 
    /// # Arguments
    /// * `file_path` - Path to the file to index
    /// 
    /// # Returns
    /// * `Result<()>` - Success or error result
    async fn index_file(&mut self, file_path: &Path) -> Result<()>;
    
    /// Clear all indexed data
    /// 
    /// # Returns
    /// * `Result<()>` - Success or error result
    async fn clear_index(&mut self) -> Result<()>;
    
    /// Update a document in the index
    /// This removes the old version and re-indexes the new version
    /// 
    /// # Arguments
    /// * `file_path` - Path to the file to update
    /// 
    /// # Returns
    /// * `Result<()>` - Success or error result
    async fn update_document(&mut self, file_path: &Path) -> Result<()>;
    
    /// Remove a document from the index
    /// 
    /// # Arguments
    /// * `file_path` - Path to the file to remove
    /// 
    /// # Returns
    /// * `Result<()>` - Success or error result
    async fn remove_document(&mut self, file_path: &Path) -> Result<()>;
    
    /// Reload the index reader to reflect recent changes
    /// 
    /// # Returns
    /// * `Result<()>` - Success or error result
    async fn reload_reader(&self) -> Result<()>;
}

/// Factory function to create a text searcher based on configuration
/// 
/// # Arguments
/// * `backend` - The search backend to use
/// 
/// # Returns
/// * `Result<Box<dyn TextSearcher>>` - A boxed trait object for the searcher
pub async fn create_text_searcher(backend: &crate::config::SearchBackend) -> Result<Box<dyn TextSearcher>> {
    use crate::config::SearchBackend;
    
    match backend {
        SearchBackend::Tantivy => {
            let searcher = TantivySearcher::new().await?;
            Ok(Box::new(searcher))
        }
    }
}

/// Create a text searcher with a specific project root directory
pub async fn create_text_searcher_with_root(backend: &crate::config::SearchBackend, project_root: std::path::PathBuf) -> Result<Box<dyn TextSearcher>> {
    use crate::config::SearchBackend;
    
    match backend {
        SearchBackend::Tantivy => {
            let searcher = TantivySearcher::new_with_root(project_root).await?;
            Ok(Box::new(searcher))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SearchBackend;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_create_tantivy_searcher() {
        let searcher = create_text_searcher(&SearchBackend::Tantivy).await.unwrap();
        
        // Basic trait object functionality test
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn test() { println!(\"hello\"); }").unwrap();
        
        // This should work without panicking (actual functionality tested elsewhere)
        assert!(searcher.search("test").await.is_ok());
    }
}