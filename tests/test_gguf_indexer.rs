use anyhow::Result;
use tempfile::tempdir;
use std::path::Path;

use embed_search::{
    indexer::IncrementalIndexer,
    config::IndexingConfig,
    gguf_embedder::{GGUFEmbedder, GGUFEmbedderConfig},
    simple_storage::VectorStorage,
    search::bm25_fixed::BM25Engine,
};

#[tokio::test]
async fn test_gguf_embedder_integration_with_indexer() -> Result<()> {
    // Create temporary directory
    let temp_dir = tempdir()?;
    let test_dir = temp_dir.path().join("test_docs");
    std::fs::create_dir_all(&test_dir)?;
    
    // Create a test file
    let test_file = test_dir.join("test.rs");
    std::fs::write(&test_file, "fn main() {\n    println!(\"Hello, world!\");\n}")?;
    
    // Initialize components
    let config = IndexingConfig {
        chunk_size: 100,
        chunk_overlap: 10,
        max_file_size: 10000,
        supported_extensions: vec!["rs".to_string(), "py".to_string(), "md".to_string()],
        enable_incremental: true,
    };
    
    let mut indexer = IncrementalIndexer::new(config)?;
    
    // Initialize GGUFEmbedder with test model
    let embedder_config = GGUFEmbedderConfig {
        model_path: "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf".to_string(),
        context_size: 2048,
        gpu_layers: 0,
        batch_size: 4,
        cache_size: 100,
        normalize: true,
        threads: 2,
    };
    
    let embedder = GGUFEmbedder::new(embedder_config)?;
    
    // Initialize storage and BM25
    let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();
    let mut storage = VectorStorage::new(&db_path)?;
    let mut bm25 = BM25Engine::new();
    
    // Test indexing
    let indexed_count = indexer.index_incremental(
        &test_dir,
        &embedder,
        &mut storage,
        &mut bm25,
    ).await?;
    
    // Verify indexing worked
    assert_eq!(indexed_count, 1, "Should have indexed 1 file");
    
    println!("✅ Successfully integrated GGUFEmbedder with IncrementalIndexer");
    println!("   - Indexed {} file(s)", indexed_count);
    println!("   - Used model: {}", embedder_config.model_path);
    
    Ok(())
}

#[test]
fn test_gguf_embedder_creation() -> Result<()> {
    let config = GGUFEmbedderConfig {
        model_path: "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf".to_string(),
        ..Default::default()
    };
    
    let _embedder = GGUFEmbedder::new(config)?;
    println!("✅ GGUFEmbedder created successfully");
    
    Ok(())
}