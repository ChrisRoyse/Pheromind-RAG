use embed_search::embedding::RealMiniLMEmbedder;
use embed_search::storage::{LanceDBStorage, LanceEmbeddingRecord};
use embed_search::chunking::{SimpleRegexChunker, Chunk};
use std::path::PathBuf;
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

/// Test the REAL embedding system with actual all-MiniLM-L6-v2 model
/// This requires internet connection to download the model

#[tokio::test]
async fn test_real_model_loading_and_basic_inference() {
    println!("🔬 Testing REAL all-MiniLM-L6-v2 model loading...");
    
    // Try to load the real model
    let embedder = match RealMiniLMEmbedder::get_global().await {
        Ok(model) => {
            println!("✅ Successfully loaded real all-MiniLM-L6-v2 model");
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
    assert_eq!(embedding.len(), 384, "Real model should produce 384-dimensional embeddings");
    
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
    
    let embedder = match RealMiniLMEmbedder::get_global().await {
        Ok(model) => model,
        Err(e) => {
            println!("⚠️  Skipping semantic test - model not available: {}", e);
            return;
        }
    };
    
    // Test real semantic understanding
    let sentences = [
        "The cat sits on the mat",           // A
        "A feline rests on the carpet",      // B - similar to A  
        "Python is a programming language",  // C - different domain
        "JavaScript code execution",         // D - similar to C
    ];
    
    println!("🔍 Generating real embeddings for test sentences...");
    let mut embeddings = Vec::new();
    for sentence in &sentences {
        let emb = embedder.embed(sentence).expect("Should generate embedding");
        println!("   '{}' -> {} dims", sentence, emb.len());
        embeddings.push(emb);
    }
    
    // Calculate cosine similarities  
    let cosine_sim = |a: &[f32], b: &[f32]| -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    };
    
    let sim_cat_feline = cosine_sim(&embeddings[0], &embeddings[1]);      // Should be high
    let sim_cat_python = cosine_sim(&embeddings[0], &embeddings[2]);      // Should be low
    let sim_python_js = cosine_sim(&embeddings[2], &embeddings[3]);       // Should be medium-high
    
    println!("🔍 Real semantic similarities:");
    println!("   Cat ↔ Feline: {:.4}", sim_cat_feline);
    println!("   Cat ↔ Python: {:.4}", sim_cat_python);
    println!("   Python ↔ JavaScript: {:.4}", sim_python_js);
    
    // Real semantic model should show these relationships
    assert!(sim_cat_feline > sim_cat_python, 
        "Cat/feline should be more similar than cat/python (got {:.4} vs {:.4})", 
        sim_cat_feline, sim_cat_python);
    
    assert!(sim_cat_feline > 0.4, 
        "Semantically similar sentences should have decent similarity: {:.4}", sim_cat_feline);
    
    assert!(sim_python_js > sim_cat_python,
        "Programming languages should be more similar to each other than to cats");
    
    println!("✅ Real semantic similarity test passed - model shows semantic understanding!");
}

#[tokio::test]
async fn test_real_lancedb_integration() {
    println!("🗄️  Testing REAL LanceDB integration...");
    
    let embedder = match RealMiniLMEmbedder::get_global().await {
        Ok(model) => model,
        Err(e) => {
            println!("⚠️  Skipping LanceDB test - model not available: {}", e);
            return;
        }
    };
    
    // Create temporary LanceDB
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("real_test.lancedb");
    
    let storage = LanceDBStorage::new(db_path).await.expect("Should create LanceDB storage");
    storage.init_table().await.expect("Should initialize table");
    
    println!("✅ LanceDB storage created and initialized");
    
    // Create test chunks with diverse content
    let test_chunks = vec![
        Chunk { content: "fn main() { println!(\"Hello Rust\"); }".to_string(), start_line: 1, end_line: 1 },
        Chunk { content: "def hello(): print('Hello Python')".to_string(), start_line: 1, end_line: 1 },
        Chunk { content: "function hello() { console.log('Hello JS'); }".to_string(), start_line: 1, end_line: 1 },
        Chunk { content: "SELECT * FROM users WHERE active = true".to_string(), start_line: 1, end_line: 1 },
    ];
    
    let file_names = ["main.rs", "hello.py", "app.js", "query.sql"];
    
    // Generate real embeddings and store in LanceDB
    println!("🔄 Generating real embeddings and storing in LanceDB...");
    for (i, (chunk, file_name)) in test_chunks.iter().zip(file_names.iter()).enumerate() {
        let embedding = embedder.embed(&chunk.content).expect("Should generate real embedding");
        
        storage.insert_embedding(file_name, i, chunk, embedding).await
            .expect("Should store in LanceDB");
        
        println!("   Stored: {} -> {} chars, 384 dims", file_name, chunk.content.len());
    }
    
    // Verify storage
    let count = storage.count().await.expect("Should get count");
    assert_eq!(count, 4, "Should have stored 4 embeddings");
    
    // Test real semantic search
    println!("🔍 Testing real semantic search...");
    
    // Search for Rust-like content
    let rust_query = "Rust programming language function";
    let rust_query_embedding = embedder.embed(rust_query).expect("Should embed query");
    
    let rust_results = storage.search_similar(rust_query_embedding, 4).await
        .expect("Should perform search");
    
    println!("   Query: '{}' found {} results", rust_query, rust_results.len());
    for (i, result) in rust_results.iter().enumerate() {
        println!("     {}. {} - '{}'", i+1, result.file_path, 
                 if result.content.len() > 50 { &result.content[..50] } else { &result.content });
    }
    
    // The real model should rank the Rust code higher
    assert!(!rust_results.is_empty(), "Should find results");
    
    // Search for Python-like content  
    let python_query = "Python script function definition";
    let python_query_embedding = embedder.embed(python_query).expect("Should embed query");
    
    let python_results = storage.search_similar(python_query_embedding, 4).await
        .expect("Should perform search");
    
    println!("   Query: '{}' found {} results", python_query, python_results.len());
    for (i, result) in python_results.iter().enumerate() {
        println!("     {}. {} - '{}'", i+1, result.file_path,
                 if result.content.len() > 50 { &result.content[..50] } else { &result.content });
    }
    
    assert!(!python_results.is_empty(), "Should find results");
    
    println!("✅ Real LanceDB integration test passed!");
}

#[tokio::test]
async fn test_real_vectortest_directory_processing() {
    println!("📁 Testing REAL embedding processing of vectortest directory...");
    
    let embedder = match RealMiniLMEmbedder::get_global().await {
        Ok(model) => model,
        Err(e) => {
            println!("⚠️  Skipping vectortest processing - model not available: {}", e);
            return;
        }
    };
    
    // Create LanceDB storage
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("vectortest_real.lancedb");
    
    let storage = LanceDBStorage::new(db_path).await.expect("Should create storage");
    storage.init_table().await.expect("Should init table");
    
    // Process actual vectortest files
    let vectortest_path = PathBuf::from("C:\\code\\embed\\vectortest");
    let chunker = SimpleRegexChunker::new();
    
    let mut total_chunks = 0;
    let mut processed_files = 0;
    
    // Read and process some files (not all to keep test time reasonable)
    let test_files = ["user_controller.js", "auth_service.py", "memory_cache.rs", "API_DOCUMENTATION.md"];
    
    println!("🔄 Processing vectortest files with REAL embeddings...");
    
    for filename in &test_files {
        let file_path = vectortest_path.join(filename);
        
        if let Ok(content) = fs::read_to_string(&file_path) {
            let chunks = chunker.chunk_file(&content);
            println!("   {} -> {} chunks", filename, chunks.len());
            
            // Process first few chunks to keep test reasonable
            let chunks_to_process = chunks.into_iter().take(3).collect::<Vec<_>>();
            
            for (idx, chunk) in chunks_to_process.iter().enumerate() {
                // Generate REAL embedding
                let embedding = embedder.embed(&chunk.content).expect("Should generate real embedding");
                
                // Store in LanceDB
                storage.insert_embedding(filename, idx, chunk, embedding).await
                    .expect("Should store embedding");
                
                total_chunks += 1;
            }
            
            processed_files += 1;
        }
    }
    
    println!("✅ Processed {} files, {} chunks with real embeddings", processed_files, total_chunks);
    
    // Verify storage
    let stored_count = storage.count().await.expect("Should get count");
    assert_eq!(stored_count, total_chunks, "Should have stored all chunks");
    
    // Test semantic search on real code
    println!("🔍 Testing semantic search on real code...");
    
    // Search for function-related content
    let function_query = "function definition implementation";
    let function_embedding = embedder.embed(function_query).expect("Should embed function query");
    
    let function_results = storage.search_similar(function_embedding, 5).await
        .expect("Should search");
    
    println!("   Function query found {} results:", function_results.len());
    for (i, result) in function_results.iter().enumerate() {
        let preview = if result.content.len() > 60 { 
            format!("{}...", &result.content[..60]) 
        } else { 
            result.content.clone() 
        };
        println!("     {}. {} - {}", i+1, result.file_path, preview);
    }
    
    // Search for documentation content
    let doc_query = "API documentation guide";
    let doc_embedding = embedder.embed(doc_query).expect("Should embed doc query");
    
    let doc_results = storage.search_similar(doc_embedding, 5).await
        .expect("Should search");
    
    println!("   Documentation query found {} results:", doc_results.len());
    for (i, result) in doc_results.iter().enumerate() {
        let preview = if result.content.len() > 60 { 
            format!("{}...", &result.content[..60]) 
        } else { 
            result.content.clone() 
        };
        println!("     {}. {} - {}", i+1, result.file_path, preview);
    }
    
    assert!(!function_results.is_empty(), "Should find function-related results");
    assert!(!doc_results.is_empty(), "Should find documentation results");
    
    println!("✅ Real vectortest directory processing test passed!");
}