use embed_search::embedding::NomicEmbedder;
use embed_search::storage::{LanceDBStorage, LanceEmbeddingRecord};
use embed_search::chunking::{SimpleRegexChunker, Chunk};
use std::path::PathBuf;
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

/// Test the REAL embedding system with actual Nomic Embed Text v1.5 model
/// This requires internet connection to download the model

#[tokio::test]
async fn test_real_model_loading_and_basic_inference() {
    println!("🔬 Testing REAL Nomic Embed Text v1.5 model loading...");
    
    // Try to load the real model
    let embedder = match NomicEmbedder::get_global().await {
        Ok(model) => {
            println!("✅ Successfully loaded real Nomic Embed Text v1.5 model");
            model
        }
        Err(e) => {
            println!("⚠️  Skipping test - model loading failed: {}", e);
            println!("   This is expected if there's no internet connection or Hugging Face is unavailable");
            return;
        }
    };
    
    // Test basic inference
    let test_text = "This is a test sentence for the real embedding model.";
    let embedding = embedder.embed(test_text).expect("Should generate embedding");
    
    // Verify it's the correct dimensions
    assert_eq!(embedding.len(), 768, "Real model should produce 768-dimensional embeddings");
    
    // Verify it's normalized (L2 norm should be 1.0)
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 0.001, "Real embeddings should be L2 normalized, got norm: {}", norm);
    
    // Test that embeddings are not all zeros
    let non_zero_count = embedding.iter().filter(|&&x| x != 0.0).count();
    assert!(non_zero_count > 100, "Real embeddings should have many non-zero values, got: {}", non_zero_count);
    
    println!("✅ Real model basic inference test passed");
}

#[tokio::test]
async fn test_real_semantic_similarity() {
    println!("🧠 Testing REAL semantic similarity with actual model...");
    
    let embedder = match NomicEmbedder::get_global().await {
        Ok(model) => model,
        Err(_) => {
            println!("⏭️ Skipping semantic similarity test - model not available");
            return;
        }
    };
    
    // Test semantic similarity between related texts
    let similar_texts = vec![
        "def calculate_sum(a, b): return a + b",
        "def add_numbers(x, y): return x + y",
    ];
    
    let different_texts = vec![
        "def calculate_sum(a, b): return a + b",
        "class DatabaseConnection: pass",
    ];
    
    // Get embeddings
    let similar_embeddings: Vec<_> = similar_texts.iter()
        .map(|text| embedder.embed(text).unwrap())
        .collect();
    
    let different_embeddings: Vec<_> = different_texts.iter()
        .map(|text| embedder.embed(text).unwrap())
        .collect();
    
    // Calculate cosine similarities
    let similar_similarity = cosine_similarity(&similar_embeddings[0], &similar_embeddings[1]);
    let different_similarity = cosine_similarity(&different_embeddings[0], &different_embeddings[1]);
    
    println!("📊 Similar functions similarity: {:.4}", similar_similarity);
    println!("📊 Different code types similarity: {:.4}", different_similarity);
    
    // Similar code should have higher similarity than completely different code
    assert!(similar_similarity > different_similarity, 
            "Similar functions should have higher similarity ({:.4}) than different code types ({:.4})", 
            similar_similarity, different_similarity);
    
    // But they should still be different enough to distinguish
    assert!(similar_similarity < 0.99, "Even similar functions should not be identical: {:.4}", similar_similarity);
    assert!(different_similarity < 0.95, "Different code types should have low similarity: {:.4}", different_similarity);
    
    println!("✅ Real semantic similarity test passed");
}

#[tokio::test]
async fn test_real_embedding_determinism() {
    println!("🔁 Testing REAL embedding determinism...");
    
    let embedder = match NomicEmbedder::get_global().await {
        Ok(model) => model,
        Err(_) => {
            println!("⏭️ Skipping determinism test - model not available");
            return;
        }
    };
    
    let test_text = "function processData(data) { return data.filter(x => x > 0); }";
    
    // Generate embedding multiple times
    let embedding1 = embedder.embed(test_text).unwrap();
    let embedding2 = embedder.embed(test_text).unwrap();
    let embedding3 = embedder.embed(test_text).unwrap();
    
    // All should be identical
    assert_eq!(embedding1, embedding2, "Embeddings should be deterministic");
    assert_eq!(embedding1, embedding3, "Embeddings should be deterministic");
    assert_eq!(embedding2, embedding3, "Embeddings should be deterministic");
    
    println!("✅ Real embedding determinism test passed");
}

#[tokio::test]
async fn test_real_batch_processing() {
    println!("📦 Testing REAL batch processing...");
    
    let embedder = match NomicEmbedder::get_global().await {
        Ok(model) => model,
        Err(_) => {
            println!("⏭️ Skipping batch processing test - model not available");
            return;
        }
    };
    
    let test_texts = vec![
        "def hello_world(): print('Hello, World!')",
        "class Person: def __init__(self, name): self.name = name",
        "SELECT * FROM users WHERE active = true",
        "function fibonacci(n) { return n <= 1 ? n : fibonacci(n-1) + fibonacci(n-2); }",
    ];
    
    // Process individually
    let individual_embeddings: Vec<_> = test_texts.iter()
        .map(|text| embedder.embed(text).unwrap())
        .collect();
    
    // Process as batch
    let batch_embeddings = embedder.embed_batch(&test_texts).unwrap();
    
    // Results should be identical
    assert_eq!(individual_embeddings.len(), batch_embeddings.len());
    for (i, (individual, batch)) in individual_embeddings.iter().zip(batch_embeddings.iter()).enumerate() {
        assert_eq!(individual, batch, "Batch embedding {} should match individual embedding", i);
    }
    
    println!("✅ Real batch processing test passed");
}

#[tokio::test]
async fn test_real_lancedb_integration() {
    println!("🗄️ Testing REAL LanceDB integration with actual embeddings...");
    
    let embedder = match NomicEmbedder::get_global().await {
        Ok(model) => model,
        Err(_) => {
            println!("⏭️ Skipping LanceDB integration test - model not available");
            return;
        }
    };
    
    // Create temporary directory for test database
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_real_embeddings.db");
    
    let storage = LanceDBStorage::new(db_path.clone()).await.expect("Failed to create storage");
    let chunker = SimpleRegexChunker::new();
    
    // Test code samples
    let test_codes = vec![
        ("python_function.py", "def calculate_area(radius):\n    return 3.14159 * radius ** 2"),
        ("javascript_class.js", "class Rectangle {\n    constructor(width, height) {\n        this.width = width;\n        this.height = height;\n    }\n}"),
        ("sql_query.sql", "SELECT users.name, COUNT(orders.id) as order_count\nFROM users\nLEFT JOIN orders ON users.id = orders.user_id\nGROUP BY users.id"),
    ];
    
    // Process and store embeddings
    let mut stored_records = Vec::new();
    for (filename, code) in &test_codes {
        let chunks = chunker.chunk_text(code);
        
        for (chunk_idx, chunk) in chunks.iter().enumerate() {
            let embedding = embedder.embed(&chunk.content).unwrap();
            
            let record = LanceEmbeddingRecord {
                id: format!("{}_{}", filename, chunk_idx),
                file_path: filename.to_string(),
                chunk_index: chunk_idx,
                content: chunk.content.clone(),
                embedding,
                start_line: chunk.start_line,
                end_line: chunk.end_line,
                metadata: serde_json::Value::Object(serde_json::Map::new()),
            };
            
            storage.store(&record).await.expect("Failed to store record");
            stored_records.push(record);
        }
    }
    
    println!("📊 Stored {} records in LanceDB", stored_records.len());
    
    // Test semantic search
    let query = "calculate mathematical operations";
    let query_embedding = embedder.embed(query).unwrap();
    
    let search_results = storage.search(&query_embedding, 3).await.expect("Failed to search");
    assert!(!search_results.is_empty(), "Should find some search results");
    
    println!("🔍 Found {} search results for query: '{}'", search_results.len(), query);
    for (i, result) in search_results.iter().enumerate() {
        println!("  {}. {} (similarity: {:.4})", i+1, result.file_path, result.similarity);
    }
    
    println!("✅ Real LanceDB integration test passed");
}

#[tokio::test]
async fn test_real_code_differentiation() {
    println!("🎯 Testing REAL code differentiation capabilities...");
    
    let embedder = match NomicEmbedder::get_global().await {
        Ok(model) => model,
        Err(_) => {
            println!("⏭️ Skipping code differentiation test - model not available");
            return;
        }
    };
    
    // Test different types of code constructs
    let code_samples = vec![
        ("function_def", "def process_data(data): return sorted(data)"),
        ("class_def", "class DataProcessor: def __init__(self): pass"),
        ("sql_query", "SELECT * FROM users WHERE created_at > '2023-01-01'"),
        ("comment", "# This function processes user data efficiently"),
        ("variable_assignment", "user_count = len(active_users)"),
        ("control_flow", "if user.is_active: return user.profile"),
    ];
    
    let mut embeddings = Vec::new();
    for (code_type, code) in &code_samples {
        let embedding = embedder.embed(code).unwrap();
        embeddings.push((code_type, embedding));
    }
    
    // Verify that different code types produce different embeddings
    for i in 0..embeddings.len() {
        for j in i+1..embeddings.len() {
            let similarity = cosine_similarity(&embeddings[i].1, &embeddings[j].1);
            println!("📊 {} vs {}: {:.4} similarity", embeddings[i].0, embeddings[j].0, similarity);
            
            // Different code types should be distinguishable (not too similar)
            assert!(similarity < 0.95, 
                    "{} and {} are too similar: {:.4} - model may not be differentiating code types well", 
                    embeddings[i].0, embeddings[j].0, similarity);
        }
    }
    
    println!("✅ Real code differentiation test passed");
}

// Helper function
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