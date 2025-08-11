use anyhow::Result;
use embed_search::llama_wrapper::{GGUFModel, GGUFContext};
use embed_search::simple_embedder::NomicEmbedder;
use std::sync::Arc;
use std::path::Path;

#[test]
fn test_gguf_model_loading() -> Result<()> {
    // Test loading the GGUF model
    let model_path = "./src/model/nomic-embed-code.Q4_K_M.gguf";
    
    // Skip test if model doesn't exist
    if !Path::new(model_path).exists() {
        println!("Skipping test: Model file not found at {}", model_path);
        return Ok(());
    }
    
    let model = GGUFModel::load_from_file(model_path, 0)?;
    
    // Verify model loaded correctly
    assert!(model.embedding_dim() > 0, "Model should have valid embedding dimension");
    println!("Model embedding dimension: {}", model.embedding_dim());
    
    Ok(())
}

#[test]
fn test_gguf_context_creation() -> Result<()> {
    // Test creating context from model
    let model_path = "./src/model/nomic-embed-code.Q4_K_M.gguf";
    
    if !Path::new(model_path).exists() {
        println!("Skipping test: Model file not found");
        return Ok(());
    }
    
    let model = Arc::new(GGUFModel::load_from_file(model_path, 0)?);
    let mut context = GGUFContext::new_with_model(model, 2048)?;
    
    // Test single embedding
    let text = "This is a test sentence for embeddings";
    let embedding = context.embed(text)?;
    
    assert!(!embedding.is_empty(), "Embedding should not be empty");
    
    // Check normalization
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 0.01, "Embeddings should be L2 normalized");
    
    println!("Generated embedding of dimension: {}", embedding.len());
    
    Ok(())
}

#[test]
fn test_batch_embeddings() -> Result<()> {
    let model_path = "./src/model/nomic-embed-code.Q4_K_M.gguf";
    
    if !Path::new(model_path).exists() {
        println!("Skipping test: Model file not found");
        return Ok(());
    }
    
    let model = Arc::new(GGUFModel::load_from_file(model_path, 0)?);
    let mut context = GGUFContext::new_with_model(model, 2048)?;
    
    // Test batch embedding
    let texts = vec![
        "First test sentence".to_string(),
        "Second test sentence".to_string(),
        "Third test sentence".to_string(),
    ];
    
    let embeddings = context.embed_batch(texts.clone())?;
    
    assert_eq!(embeddings.len(), texts.len(), "Should generate one embedding per text");
    
    for (i, embedding) in embeddings.iter().enumerate() {
        assert!(!embedding.is_empty(), "Embedding {} should not be empty", i);
        
        // Check normalization
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01, "Embedding {} should be normalized", i);
    }
    
    println!("Successfully generated {} embeddings", embeddings.len());
    
    Ok(())
}

#[test]
fn test_nomic_embedder_integration() -> Result<()> {
    let model_path = "./src/model/nomic-embed-code.Q4_K_M.gguf";
    
    if !Path::new(model_path).exists() {
        println!("Skipping test: Model file not found");
        return Ok(());
    }
    
    // Test the integrated NomicEmbedder
    let mut embedder = NomicEmbedder::new()?;
    
    // Test document embedding (with "passage:" prefix)
    let doc_text = "This is a document about Rust programming";
    let doc_embedding = embedder.embed(doc_text)?;
    
    assert!(!doc_embedding.is_empty(), "Document embedding should not be empty");
    assert_eq!(doc_embedding.len(), 768, "Should be 768-dimensional");
    
    // Test query embedding (with "query:" prefix)
    let query_text = "How to program in Rust?";
    let query_embedding = embedder.embed_query(query_text)?;
    
    assert!(!query_embedding.is_empty(), "Query embedding should not be empty");
    assert_eq!(query_embedding.len(), 768, "Should be 768-dimensional");
    
    // Calculate cosine similarity
    let similarity: f32 = doc_embedding.iter()
        .zip(query_embedding.iter())
        .map(|(a, b)| a * b)
        .sum();
    
    println!("Document-Query similarity: {}", similarity);
    
    // Test batch processing
    let documents = vec![
        "Rust memory safety".to_string(),
        "Python data science".to_string(),
        "JavaScript web development".to_string(),
    ];
    
    let batch_embeddings = embedder.embed_batch(documents.clone())?;
    
    assert_eq!(batch_embeddings.len(), documents.len());
    for embedding in &batch_embeddings {
        assert_eq!(embedding.len(), 768);
    }
    
    println!("Successfully integrated NomicEmbedder with llama-cpp-2");
    
    Ok(())
}

#[test]
fn test_embedding_consistency() -> Result<()> {
    let model_path = "./src/model/nomic-embed-code.Q4_K_M.gguf";
    
    if !Path::new(model_path).exists() {
        println!("Skipping test: Model file not found");
        return Ok(());
    }
    
    let mut embedder = NomicEmbedder::new()?;
    
    // Test that same text produces same embedding
    let text = "Consistent test text";
    let embedding1 = embedder.embed(text)?;
    let embedding2 = embedder.embed(text)?;
    
    // Calculate similarity (should be very close to 1.0)
    let similarity: f32 = embedding1.iter()
        .zip(embedding2.iter())
        .map(|(a, b)| a * b)
        .sum();
    
    assert!(similarity > 0.99, "Same text should produce nearly identical embeddings");
    
    println!("Embedding consistency test passed: similarity = {}", similarity);
    
    Ok(())
}

#[test]
fn test_memory_safety() -> Result<()> {
    let model_path = "./src/model/nomic-embed-code.Q4_K_M.gguf";
    
    if !Path::new(model_path).exists() {
        println!("Skipping test: Model file not found");
        return Ok(());
    }
    
    // Test that model can be shared across threads (Arc)
    let model = Arc::new(GGUFModel::load_from_file(model_path, 0)?);
    
    // Create multiple contexts from same model
    let mut context1 = GGUFContext::new_with_model(model.clone(), 2048)?;
    let mut context2 = GGUFContext::new_with_model(model.clone(), 2048)?;
    
    // Use both contexts
    let embedding1 = context1.embed("Test text 1")?;
    let embedding2 = context2.embed("Test text 2")?;
    
    assert!(!embedding1.is_empty());
    assert!(!embedding2.is_empty());
    
    println!("Memory safety test passed: multiple contexts work correctly");
    
    Ok(())
}