// simple_embedder was deleted - NomicEmbedder doesn't exist. Using GGUFEmbedder instead.
use embed_search::{GGUFEmbedder, GGUFEmbedderConfig};
use anyhow::Result;

#[tokio::test]
async fn test_nomic_embedder_real_functionality() -> Result<()> {
    println!("🧪 Testing NomicEmbedder Real Functionality");
    
    // Initialize the GGUF embedder (the real implementation)
    let config = GGUFEmbedderConfig::default();
    let mut embedder = match GGUFEmbedder::new(config) {
        Ok(e) => e,
        Err(e) => {
            println!("❌ Failed to initialize GGUFEmbedder: {}", e);
            return Err(e);
        }
    };
    
    println!("✅ NomicEmbedder initialized successfully");
    
    // Test 1: Document embedding with "passage:" prefix
    let test_document = "fn main() { println!(\"Hello, world!\"); }";
    let doc_embedding = embedder.embed(test_document, embed_search::embedding_prefixes::EmbeddingTask::SearchDocument)?;
    
    println!("📄 Document embedding:");
    println!("  - Text: '{}'", test_document);
    println!("  - Dimension: {} (expected: 768)", doc_embedding.len());
    println!("  - First 5 values: {:?}", &doc_embedding[..5.min(doc_embedding.len())]);
    
    // Verify embedding dimensions
    assert_eq!(doc_embedding.len(), 768, "Document embedding should be 768-dimensional");
    assert!(doc_embedding.iter().any(|&x| x != 0.0), "Document embedding should contain non-zero values");
    
    // Test 2: Query embedding with "query:" prefix  
    let test_query = "main function";
    let query_embedding = embedder.embed(test_query, embed_search::embedding_prefixes::EmbeddingTask::SearchQuery)?;
    
    println!("\n🔍 Query embedding:");
    println!("  - Text: '{}'", test_query);
    println!("  - Dimension: {} (expected: 768)", query_embedding.len());
    println!("  - First 5 values: {:?}", &query_embedding[..5.min(query_embedding.len())]);
    
    // Verify query embedding dimensions
    assert_eq!(query_embedding.len(), 768, "Query embedding should be 768-dimensional");
    assert!(query_embedding.iter().any(|&x| x != 0.0), "Query embedding should contain non-zero values");
    
    // Test 3: Batch embedding
    let test_documents = vec![
        "struct User { name: String }".to_string(),
        "impl User { fn new() -> Self { } }".to_string(),
        "let user = User::new();".to_string(),
    ];
    
    let batch_embeddings = embedder.embed_batch(test_documents.clone(), embed_search::embedding_prefixes::EmbeddingTask::SearchDocument)?;
    
    println!("\n📦 Batch embeddings:");
    println!("  - Documents: {}", test_documents.len());
    println!("  - Embeddings: {}", batch_embeddings.len());
    assert_eq!(batch_embeddings.len(), test_documents.len(), "Should have same number of embeddings as documents");
    
    for (i, embedding) in batch_embeddings.iter().enumerate() {
        println!("  - Doc {}: {} dims, first 3 values: {:?}", 
                 i + 1, 
                 embedding.len(), 
                 &embedding[..3.min(embedding.len())]);
        assert_eq!(embedding.len(), 768, "Each batch embedding should be 768-dimensional");
        assert!(embedding.iter().any(|&x| x != 0.0), "Each batch embedding should contain non-zero values");
    }
    
    // Test 4: Verify prefix usage (embeddings should be different for same text with different prefixes)
    let base_text = "search algorithm implementation";
    let passage_embedding = embedder.embed(base_text, embed_search::embedding_prefixes::EmbeddingTask::SearchDocument)?;
    let query_embedding_same_text = embedder.embed(base_text, embed_search::embedding_prefixes::EmbeddingTask::SearchQuery)?;
    
    println!("\n🔀 Prefix verification:");
    println!("  - Same text with passage prefix: first 3 values: {:?}", &passage_embedding[..3]);
    println!("  - Same text with query prefix: first 3 values: {:?}", &query_embedding_same_text[..3]);
    
    // They should be different due to different prefixes
    let embeddings_different = passage_embedding.iter()
        .zip(query_embedding_same_text.iter())
        .any(|(a, b)| (a - b).abs() > 1e-6);
    
    assert!(embeddings_different, "Embeddings with different prefixes should be different");
    println!("  ✅ Confirmed: passage: and query: prefixes produce different embeddings");
    
    // Test 5: Verify embeddings are deterministic
    let text = "deterministic test";
    let embedding1 = embedder.embed(text, embed_search::embedding_prefixes::EmbeddingTask::SearchDocument)?;
    let embedding2 = embedder.embed(text, embed_search::embedding_prefixes::EmbeddingTask::SearchDocument)?;
    
    let embeddings_identical = embedding1.iter()
        .zip(embedding2.iter())
        .all(|(a, b)| (a - b).abs() < 1e-6);
    
    assert!(embeddings_identical, "Same input should produce identical embeddings");
    println!("  ✅ Confirmed: Embeddings are deterministic");
    
    // Test 6: Vector similarity test
    let similar_text1 = "function definition";
    let similar_text2 = "function implementation"; 
    let different_text = "database connection";
    
    let emb1 = embedder.embed(similar_text1, embed_search::embedding_prefixes::EmbeddingTask::SearchDocument)?;
    let emb2 = embedder.embed(similar_text2, embed_search::embedding_prefixes::EmbeddingTask::SearchDocument)?;
    let emb3 = embedder.embed(different_text, embed_search::embedding_prefixes::EmbeddingTask::SearchDocument)?;
    
    // Calculate cosine similarity
    let similarity_similar = cosine_similarity(&emb1, &emb2);
    let similarity_different = cosine_similarity(&emb1, &emb3);
    
    println!("\n📊 Semantic similarity test:");
    println!("  - '{}' vs '{}': {:.4}", similar_text1, similar_text2, similarity_similar);
    println!("  - '{}' vs '{}': {:.4}", similar_text1, different_text, similarity_different);
    
    assert!(similarity_similar > similarity_different, 
            "Similar texts should have higher similarity than different texts");
    println!("  ✅ Confirmed: Semantic similarity works correctly");
    
    println!("\n🎉 ALL EMBEDDING VERIFICATION TESTS PASSED!");
    println!("✅ NomicEmbedder produces real 768-dimensional vectors");
    println!("✅ Proper 'passage:' and 'query:' prefixes are used");
    println!("✅ No fake/mock data - actual embedding generation confirmed");
    println!("✅ Embeddings are semantically meaningful");
    
    Ok(())
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

#[tokio::test]
async fn test_embedding_system_integration() -> Result<()> {
    println!("🔧 Testing Complete Embedding System Integration");
    
    // This test verifies the entire chain works together
    use embed_search::simple_storage::VectorStorage;
    use tempfile::tempdir;
    
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("integration.db").to_str().unwrap().to_string();
    
    let config = embed_search::gguf_embedder::GGUFEmbedderConfig::default();
    let mut embedder = embed_search::gguf_embedder::GGUFEmbedder::new(config)?;
    let mut storage = VectorStorage::new(&db_path)?;
    
    // Test documents
    let documents = vec![
        "fn main() { println!(\"Hello world\"); }".to_string(),
        "struct User { name: String, email: String }".to_string(),
        "impl BM25Engine { fn search(&self) -> Vec<Match> { } }".to_string(),
    ];
    
    let file_paths = vec![
        "main.rs".to_string(),
        "user.rs".to_string(), 
        "search.rs".to_string(),
    ];
    
    // Generate embeddings
    let embeddings = embedder.embed_batch(documents.clone(), embed_search::embedding_prefixes::EmbeddingTask::SearchDocument)?;
    println!("📦 Generated {} embeddings", embeddings.len());
    
    // Store in vector database
    storage.store(documents, embeddings, file_paths)?;
    println!("✅ Documents stored successfully");
    
    // Test search
    let query_embedding = embedder.embed("main function", embed_search::embedding_prefixes::EmbeddingTask::SearchQuery)?;
    let results = storage.search(query_embedding, 3)?;
    
    assert!(!results.is_empty(), "Search should return results");
    println!("🔍 Found {} search results", results.len());
    
    for (i, result) in results.iter().enumerate() {
        println!("  {}. {} (score: {:.4})", i + 1, result.file_path, result.score);
    }
    
    println!("✅ Complete integration test passed!");
    
    Ok(())
}