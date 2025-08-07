#[cfg(all(feature = "ml", feature = "vectordb"))]
use embed_search::embedding::NomicEmbedder;
#[cfg(all(feature = "ml", feature = "vectordb"))]
use embed_search::storage::lancedb_storage::{LanceDBStorage, LanceEmbeddingRecord};
#[cfg(all(feature = "ml", feature = "vectordb"))]
use embed_search::chunking::SimpleRegexChunker;
#[cfg(all(feature = "ml", feature = "vectordb"))]
use embed_search::search::unified::UnifiedSearcher;
#[cfg(all(feature = "ml", feature = "vectordb"))]
use embed_search::git::watcher::{GitWatcher, VectorUpdater, WatchCommand, FileChange};
#[cfg(all(feature = "ml", feature = "vectordb"))]
use std::fs;
#[cfg(all(feature = "ml", feature = "vectordb"))]
use std::sync::Arc;
#[cfg(all(feature = "ml", feature = "vectordb"))]
use tokio::sync::RwLock;
#[cfg(all(feature = "ml", feature = "vectordb"))]
use tempfile::TempDir;

/// Test the REAL embedding system with actual Nomic Embed Text v1.5 model
/// This requires internet connection to download the model

#[cfg(all(feature = "ml", feature = "vectordb"))]
#[tokio::test]
async fn test_real_model_loading_and_basic_inference() {
    println!("🔬 Testing REAL Nomic Embed Text v1.5 model loading...");
    
    // Try to load the real model
    let embedder = match NomicEmbedder::get_global() {
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

#[cfg(all(feature = "ml", feature = "vectordb"))]
#[tokio::test]
async fn test_real_semantic_similarity() {
    println!("🧠 Testing REAL semantic similarity with actual model...");
    
    let embedder = match NomicEmbedder::get_global() {
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

#[cfg(all(feature = "ml", feature = "vectordb"))]
#[tokio::test]
async fn test_real_embedding_determinism() {
    println!("🔁 Testing REAL embedding determinism...");
    
    let embedder = match NomicEmbedder::get_global() {
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

#[cfg(all(feature = "ml", feature = "vectordb"))]
#[tokio::test]
async fn test_real_batch_processing() {
    println!("📦 Testing REAL batch processing...");
    
    let embedder = match NomicEmbedder::get_global() {
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

#[cfg(all(feature = "ml", feature = "vectordb"))]
#[tokio::test]
async fn test_real_lancedb_integration() {
    println!("🗄️ Testing REAL LanceDB integration with actual embeddings...");
    
    let embedder = match NomicEmbedder::get_global() {
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
    let chunker = SimpleRegexChunker::new().expect("Chunker creation must succeed in test");
    
    // Test code samples
    let test_codes = vec![
        ("python_function.py", "def calculate_area(radius):\n    return 3.14159 * radius ** 2"),
        ("javascript_class.js", "class Rectangle {\n    constructor(width, height) {\n        this.width = width;\n        this.height = height;\n    }\n}"),
        ("sql_query.sql", "SELECT users.name, COUNT(orders.id) as order_count\nFROM users\nLEFT JOIN orders ON users.id = orders.user_id\nGROUP BY users.id"),
    ];
    
    // Process and store embeddings
    let mut stored_records = Vec::new();
    for (filename, code) in &test_codes {
        let chunks = chunker.chunk_file(code);
        
        for (chunk_idx, chunk) in chunks.iter().enumerate() {
            let embedding = embedder.embed(&chunk.content).unwrap();
            
            let record = LanceEmbeddingRecord {
                id: format!("{}_{}", filename, chunk_idx),
                file_path: filename.to_string(),
                chunk_index: chunk_idx as u64,
                content: chunk.content.clone(),
                embedding,
                start_line: chunk.start_line as u64,
                end_line: chunk.end_line as u64,
                similarity_score: None,
            };
            
            storage.insert_batch(vec![record.clone()]).await.expect("Failed to store record");
            stored_records.push(record);
        }
    }
    
    println!("📊 Stored {} records in LanceDB", stored_records.len());
    
    // Test semantic search
    let query = "calculate mathematical operations";
    let query_embedding = embedder.embed(query).unwrap();
    
    let search_results = storage.search_similar(query_embedding, 3).await.expect("Failed to search");
    assert!(!search_results.is_empty(), "Should find some search results");
    
    println!("🔍 Found {} search results for query: '{}'", search_results.len(), query);
    for (i, result) in search_results.iter().enumerate() {
        match result.similarity_score {
            Some(score) => println!("  {}. {} (similarity: {:.4})", i+1, result.file_path, score),
            None => println!("  {}. {} (similarity: <unavailable>)", i+1, result.file_path),
        }
    }
    
    println!("✅ Real LanceDB integration test passed");
}

#[cfg(all(feature = "ml", feature = "vectordb"))]
#[tokio::test]
async fn test_real_code_differentiation() {
    println!("🎯 Testing REAL code differentiation capabilities...");
    
    let embedder = match NomicEmbedder::get_global() {
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

#[cfg(all(feature = "ml", feature = "vectordb"))]
#[tokio::test]
async fn test_git_tracking_with_768d_nomic_embeddings() {
    println!("\n🔍 TESTING GIT TRACKING WITH 768D NOMIC EMBEDDINGS");
    println!("{}", "=".repeat(60));
    
    // Step 1: Initialize Nomic embedder (768D)
    println!("\n1️⃣ Initializing Nomic embedder...");
    let _embedder = match NomicEmbedder::get_global() {
        Ok(model) => {
            assert_eq!(model.dimensions(), 768, "Should be 768D Nomic embeddings");
            println!("   ✅ Nomic embedder initialized with 768D embeddings");
            model
        },
        Err(e) => {
            println!("   ⚠️ Skipping git tracking test - model not available: {}", e);
            return;
        }
    };
    
    // Step 2: Create temporary git repository for testing
    println!("\n2️⃣ Setting up test git repository...");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().to_path_buf();
    let db_path = repo_path.join("test_vectors_768d.db");
    
    // Initialize git repository
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to initialize git repository");
    println!("   ✅ Git repository initialized at: {:?}", repo_path);
    
    // Step 3: Create test files
    println!("\n3️⃣ Creating test files...");
    let test_file1 = repo_path.join("example.py");
    fs::write(&test_file1, "def hello():\n    print('Hello World')\n").unwrap();
    
    let test_file2 = repo_path.join("data.sql");
    fs::write(&test_file2, "SELECT * FROM users WHERE active = true;\n").unwrap();
    
    let test_file3 = repo_path.join("app.js");
    fs::write(&test_file3, "function processData(data) { return data.map(x => x * 2); }\n").unwrap();
    
    println!("   ✅ Created 3 test files");
    
    // Step 4: Initialize UnifiedSearcher with 768D configuration
    println!("\n4️⃣ Initializing UnifiedSearcher with 768D support...");
    let searcher = Arc::new(
        UnifiedSearcher::new(repo_path.clone(), db_path.clone()).await
            .expect("Failed to create searcher")
    );
    let storage = Arc::new(RwLock::new(
        LanceDBStorage::new(db_path.clone()).await
            .expect("Failed to create storage")
    ));
    
    // Verify storage accepts 768D embeddings
    storage.write().await.init_table().await.expect("Failed to init table");
    println!("   ✅ Storage initialized for 768D embeddings");
    
    // Step 5: Index initial files
    println!("\n5️⃣ Indexing initial files...");
    for file in &[&test_file1, &test_file2, &test_file3] {
        searcher.index_file(file).await.expect("Failed to index file");
    }
    println!("   ✅ Indexed 3 files with 768D embeddings");
    
    // Step 6: Test GitWatcher detects changes
    println!("\n6️⃣ Testing GitWatcher change detection...");
    let watcher = GitWatcher::new(repo_path.clone());
    
    // Add files to git staging
    std::process::Command::new("git")
        .args(&["add", "."])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to add files to git");
    
    // Commit initial files
    std::process::Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to commit");
    
    // Modify a file
    fs::write(&test_file1, "def hello():\n    print('Hello 768D World!')\n    return 768\n").unwrap();
    
    // Check for changes
    let changes = watcher.get_changes().expect("Failed to get changes");
    assert_eq!(changes.len(), 1, "Should detect 1 modified file");
    
    match &changes[0] {
        FileChange::Modified(path) => {
            assert_eq!(path.file_name().unwrap().to_str().unwrap(), "example.py");
            println!("   ✅ Detected modification of example.py");
        },
        _ => panic!("Expected Modified change type"),
    }
    
    // Step 7: Test VectorUpdater with 768D embeddings
    println!("\n7️⃣ Testing VectorUpdater with 768D embeddings...");
    let updater = VectorUpdater::new(searcher.clone(), storage.clone());
    
    // Update the modified file
    for change in &changes {
        match change {
            FileChange::Modified(path) => {
                updater.update_file(path, change).await
                    .expect("Failed to update file embeddings");
            },
            _ => {},
        }
    }
    println!("   ✅ Updated embeddings for modified file");
    
    // Step 8: Verify new 768D embeddings were created
    println!("\n8️⃣ Verifying new 768D embeddings...");
    
    // Search for the updated content
    let search_results = searcher.search("768D World").await
        .expect("Failed to search");
    
    assert!(!search_results.is_empty(), "Should find results for updated content");
    assert!(search_results[0].file.contains("example.py"), 
            "Should find the updated Python file");
    println!("   ✅ Found updated content with 768D embeddings");
    
    // Step 9: Test batch update with multiple changes
    println!("\n9️⃣ Testing batch update with multiple files...");
    
    // Create more changes
    fs::write(&test_file2, "SELECT * FROM users WHERE active = true AND created > '2024-01-01';\n").unwrap();
    fs::write(&test_file3, "function processData(data) { return data.map(x => x * 768); }\n").unwrap();
    
    let batch_changes = watcher.get_changes().expect("Failed to get batch changes");
    assert!(batch_changes.len() >= 2, "Should detect multiple changes");
    
    let stats = updater.batch_update(batch_changes).await
        .expect("Failed to batch update");
    
    println!("   ✅ Batch update complete: {} files updated", stats.updated_files);
    assert!(stats.updated_files >= 2, "Should update at least 2 files");
    
    // Step 10: Test WatchCommand integration
    println!("\n🔟 Testing WatchCommand with 768D system...");
    let watch_command = WatchCommand::new(repo_path.clone(), searcher.clone(), storage.clone())?;
    
    // Create another change
    let new_file = repo_path.join("new_feature.rs");
    fs::write(&new_file, "fn process_embeddings() -> Vec<f32> {\n    vec![0.0; 768]\n}\n").unwrap();
    
    // Run once to detect and process the new file
    let final_stats = watch_command.run_once().await
        .expect("Failed to run watch command");
    
    assert!(final_stats.updated_files > 0, "Should detect and index new file");
    println!("   ✅ WatchCommand processed {} files", final_stats.updated_files);
    
    // Verify the new file was indexed with 768D embeddings
    let rust_results = searcher.search("process_embeddings 768").await
        .expect("Failed to search for new content");
    
    assert!(!rust_results.is_empty(), "Should find the new Rust file");
    println!("   ✅ New file indexed with 768D embeddings");
    
    println!("\n{}", "=".repeat(60));
    println!("✅ GIT TRACKING WITH 768D NOMIC EMBEDDINGS: ALL TESTS PASSED!");
    println!("{}", "=".repeat(60));
    println!("\n📊 Summary:");
    println!("  • GitWatcher detects file changes ✓");
    println!("  • VectorUpdater re-embeds with 768D ✓");
    println!("  • Batch updates work correctly ✓");
    println!("  • WatchCommand integrates properly ✓");
    println!("  • Search finds updated content ✓");
    println!("\n🚀 Git tracking system fully compatible with 768D Nomic embeddings!");
}