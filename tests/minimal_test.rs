use tempfile::TempDir;

use embed_search::search::tantivy_search::TantivySearcher;

#[tokio::test]
async fn test_minimal_creation() {
    let temp_dir = TempDir::new().unwrap();
    let index_path = temp_dir.path().join("minimal_index");
    
    println!("Creating TantivySearcher with path: {:?}", index_path);
    
    // Just try to create the searcher
    let result = TantivySearcher::new_with_path(&index_path).await;
    
    match result {
        Ok(searcher) => {
            println!("✅ Successfully created persistent TantivySearcher");
            println!("Is persistent: {}", searcher.is_persistent());
            println!("Index path: {:?}", searcher.index_path());
            
            // Test basic stats
            match searcher.get_index_stats() {
                Ok(stats) => {
                    println!("Index stats: {}", stats);
                }
                Err(e) => {
                    println!("⚠️  Failed to get index stats: {}", e);
                }
            }
        }
        Err(e) => {
            panic!("❌ Failed to create persistent TantivySearcher: {}", e);
        }
    }
}