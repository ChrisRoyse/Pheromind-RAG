// BRUTAL TRUTH VERIFICATION TEST
// This test verifies what actually works vs what was claimed

use anyhow::Result;
use std::time::Instant;

#[test]
fn test_model_file_exists() -> Result<()> {
    println!("🔍 TRUTH CHECK: Model file existence");
    
    let model_path = "./src/model/nomic-embed-code.Q4_K_M.gguf";
    let exists = std::path::Path::new(model_path).exists();
    
    if exists {
        let metadata = std::fs::metadata(model_path)?;
        let size_mb = metadata.len() / (1024 * 1024);
        println!("✅ TRUTH: Model exists - {} MB", size_mb);
        assert!(size_mb > 50, "Model should be substantial size");
    } else {
        println!("❌ TRUTH: Model file does NOT exist");
        panic!("Agents claimed model exists but it doesn't");
    }
    
    Ok(())
}

#[test]
fn test_basic_imports() -> Result<()> {
    println!("🔍 TRUTH CHECK: Basic imports");
    
    // Can we import the types?
    use embed_search::gguf_embedder::{GGUFEmbedder, GGUFEmbedderConfig};
    println!("✅ TRUTH: Basic types import successfully");
    
    // Can we create a config?
    let config = GGUFEmbedderConfig {
        model_path: "./src/model/nomic-embed-code.Q4_K_M.gguf".to_string(),
        context_size: 2048,
        max_batch_size: 32,
        embedding_dim: 768,
        vocab_only: false,
        use_mlock: false,
        use_mmap: true,
        n_threads: 4,
    };
    println!("✅ TRUTH: Config creation works");
    
    Ok(())
}

#[test]
fn test_actual_embedding_capability() -> Result<()> {
    println!("🔍 TRUTH CHECK: Can we actually create embeddings?");
    
    use embed_search::gguf_embedder::{GGUFEmbedder, GGUFEmbedderConfig};
    use embed_search::embedding_prefixes::EmbeddingTask;
    
    let config = GGUFEmbedderConfig {
        model_path: "./src/model/nomic-embed-code.Q4_K_M.gguf".to_string(),
        context_size: 2048,
        max_batch_size: 32,
        embedding_dim: 768,
        vocab_only: false,
        use_mlock: false,
        use_mmap: true,
        n_threads: 4,
    };
    
    let start = Instant::now();
    match GGUFEmbedder::new(config) {
        Ok(embedder) => {
            let init_time = start.elapsed();
            println!("✅ TRUTH: Embedder created in {:?}", init_time);
            
            // Try to embed something simple
            let test_text = "hello world";
            match embedder.embed(test_text, EmbeddingTask::SearchDocument) {
                Ok(embedding) => {
                    println!("✅ TRUTH: Embedding created - {} dimensions", embedding.len());
                    assert_eq!(embedding.len(), 768, "Should be 768 dimensions");
                    
                    // Check that embedding is not all zeros
                    let sum: f32 = embedding.iter().sum();
                    assert!(sum.abs() > 0.001, "Embedding should not be all zeros");
                    
                    println!("✅ TRUTH: Embedding appears valid (sum: {:.6})", sum);
                },
                Err(e) => {
                    println!("❌ TRUTH: Embedding failed - {}", e);
                    return Err(anyhow::anyhow!("Embedding creation failed: {}", e));
                }
            }
        },
        Err(e) => {
            println!("❌ TRUTH: Embedder creation failed - {}", e);
            println!("   This means the GGUF integration is NOT actually working");
            return Err(anyhow::anyhow!("Embedder initialization failed: {}", e));
        }
    }
    
    Ok(())
}

#[test]
fn test_basic_chunking() -> Result<()> {
    println!("🔍 TRUTH CHECK: Basic text chunking");
    
    use embed_search::chunking::SimpleRegexChunker;
    
    let chunker = SimpleRegexChunker::new(100, 20);
    let test_text = "This is a test document. It has multiple sentences. We want to see if chunking works properly.";
    
    let chunks = chunker.chunk_text(test_text);
    
    if chunks.is_empty() {
        println!("❌ TRUTH: Chunking produced no chunks");
        return Err(anyhow::anyhow!("Chunking failed"));
    }
    
    println!("✅ TRUTH: Chunking works - {} chunks created", chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        println!("  Chunk {}: \"{}\"", i, chunk.content.trim());
    }
    
    Ok(())
}

#[test] 
fn test_storage_basics() -> Result<()> {
    println!("🔍 TRUTH CHECK: Basic vector storage");
    
    use embed_search::simple_storage::VectorStorage;
    use tempfile::tempdir;
    
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    
    match VectorStorage::new(db_path.to_str().unwrap()) {
        Ok(mut storage) => {
            println!("✅ TRUTH: Vector storage can be created");
            
            // Try to store a simple embedding
            let test_embedding = vec![0.1f32; 768];
            let doc_id = "test_doc";
            
            match storage.store_embedding(doc_id, &test_embedding, "test content") {
                Ok(_) => println!("✅ TRUTH: Can store embeddings"),
                Err(e) => {
                    println!("❌ TRUTH: Storage failed - {}", e);
                    return Err(anyhow::anyhow!("Storage failed: {}", e));
                }
            }
        },
        Err(e) => {
            println!("❌ TRUTH: Vector storage creation failed - {}", e);
            return Err(anyhow::anyhow!("Storage creation failed: {}", e));
        }
    }
    
    Ok(())
}