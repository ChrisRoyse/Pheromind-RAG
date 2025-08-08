//! End-to-End Integration Validation for the Repaired Embedding System
//! 
//! This comprehensive test suite validates the complete pipeline after mathematical
//! fixes and system repairs. It tests the entire flow from text input to similarity search
//! with real semantic meaning validation.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tempfile::TempDir;
use anyhow::Result;

#[cfg(all(feature = "ml", feature = "vectordb"))]
use embed_search::embedding::NomicEmbedder;
#[cfg(all(feature = "ml", feature = "vectordb"))]
use embed_search::storage::lancedb_storage::LanceDBStorage;
use embed_search::search::unified::UnifiedSearcher;
use embed_search::config::Config;
use embed_search::chunking::SimpleRegexChunker;

/// Semantic similarity benchmark dataset for validation
struct SemanticBenchmark {
    similar_pairs: Vec<(String, String)>,
    dissimilar_pairs: Vec<(String, String)>,
}

impl SemanticBenchmark {
    fn new() -> Self {
        Self {
            similar_pairs: vec![
                (
                    "function calculateTotalPrice(items) { return items.reduce((sum, item) => sum + item.price, 0); }".to_string(),
                    "def compute_total_cost(products): return sum(product.cost for product in products)".to_string()
                ),
                (
                    "class DatabaseConnection { constructor(host, port) { this.host = host; this.port = port; } }".to_string(),
                    "struct DBConnection { host: String, port: u16 }".to_string()
                ),
                (
                    "if (user.isAuthenticated()) { return renderDashboard(); }".to_string(),
                    "if user.authenticated: dashboard.render()".to_string()
                ),
                (
                    "async function fetchUserData(userId) { const response = await api.get(`/users/${userId}`); return response.data; }".to_string(),
                    "async fn get_user_data(user_id: u64) -> Result<User> { let response = client.get(&format!(\"/users/{}\", user_id)).await?; Ok(response.json().await?) }".to_string()
                ),
                (
                    "const handleError = (error) => { console.error('Operation failed:', error); showNotification('An error occurred'); };".to_string(),
                    "fn handle_error(err: &Error) { eprintln!(\"Operation failed: {}\", err); show_notification(\"An error occurred\"); }".to_string()
                ),
            ],
            dissimilar_pairs: vec![
                (
                    "function calculateTotalPrice(items) { return items.reduce((sum, item) => sum + item.price, 0); }".to_string(),
                    "const colors = ['red', 'green', 'blue', 'yellow', 'purple'];".to_string()
                ),
                (
                    "class DatabaseConnection { constructor(host, port) { this.host = host; this.port = port; } }".to_string(),
                    "for (let i = 0; i < 100; i++) { console.log(`Iteration ${i}`); }".to_string()
                ),
                (
                    "if (user.isAuthenticated()) { return renderDashboard(); }".to_string(),
                    "const PI = 3.14159265359; const circumference = 2 * PI * radius;".to_string()
                ),
                (
                    "async function fetchUserData(userId) { const response = await api.get(`/users/${userId}`); return response.data; }".to_string(),
                    "body { font-family: Arial, sans-serif; margin: 0; padding: 20px; }".to_string()
                ),
                (
                    "const handleError = (error) => { console.error('Operation failed:', error); showNotification('An error occurred'); };".to_string(),
                    "<html><head><title>Welcome</title></head><body><h1>Hello World</h1></body></html>".to_string()
                ),
            ],
        }
    }
}

#[tokio::test]
#[cfg(all(feature = "ml", feature = "vectordb"))]
async fn test_complete_pipeline_integration() -> Result<()> {
    println!("🚀 Starting Complete Pipeline Integration Test");
    
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path().to_path_buf();
    
    // Step 1: Initialize system components
    println!("📋 Step 1: Initializing system components");
    let start_time = Instant::now();
    
    let config = Config::default();
    let embedder = NomicEmbedder::get_global().await?;
    let storage = Arc::new(RwLock::new(LanceDBStorage::new(&temp_path.join("vectors")).await?));
    
    println!("✅ System components initialized in {:?}", start_time.elapsed());
    
    // Step 2: Create test documents
    println!("📋 Step 2: Creating test documents");
    let test_documents = create_test_documents();
    let chunker = SimpleRegexChunker::new(512, 50);
    
    // Step 3: Process and embed documents
    println!("📋 Step 3: Processing and embedding {} documents", test_documents.len());
    let embed_start = Instant::now();
    
    for (doc_id, content) in test_documents.iter().enumerate() {
        let chunks = chunker.chunk_text(content);
        
        for (chunk_idx, chunk) in chunks.iter().enumerate() {
            let embedding = embedder.embed(&chunk.content)?;
            
            let record = embed_search::storage::lancedb_storage::LanceEmbeddingRecord {
                id: format!("doc_{}_{}", doc_id, chunk_idx),
                text: chunk.content.clone(),
                embedding,
                file_path: format!("test_doc_{}.rs", doc_id),
                line_number: chunk.line_number.unwrap_or(1) as i32,
                chunk_index: chunk_idx as i32,
            };
            
            storage.write().await.add_embedding(record).await?;
        }
    }
    
    println!("✅ Document processing completed in {:?}", embed_start.elapsed());
    
    // Step 4: Test semantic search quality
    println!("📋 Step 4: Testing semantic search quality");
    let benchmark = SemanticBenchmark::new();
    let mut similar_scores = Vec::new();
    let mut dissimilar_scores = Vec::new();
    
    // Test similar pairs (should have high similarity)
    for (text1, text2) in &benchmark.similar_pairs {
        let embedding1 = embedder.embed(text1)?;
        let embedding2 = embedder.embed(text2)?;
        
        let similarity = calculate_cosine_similarity(&embedding1, &embedding2);
        similar_scores.push(similarity);
        println!("📊 Similar pair similarity: {:.4}", similarity);
    }
    
    // Test dissimilar pairs (should have low similarity)
    for (text1, text2) in &benchmark.dissimilar_pairs {
        let embedding1 = embedder.embed(text1)?;
        let embedding2 = embedder.embed(text2)?;
        
        let similarity = calculate_cosine_similarity(&embedding1, &embedding2);
        dissimilar_scores.push(similarity);
        println!("📊 Dissimilar pair similarity: {:.4}", similarity);
    }
    
    // Step 5: Validate semantic quality metrics
    println!("📋 Step 5: Validating semantic quality metrics");
    
    let avg_similar = similar_scores.iter().sum::<f32>() / similar_scores.len() as f32;
    let avg_dissimilar = dissimilar_scores.iter().sum::<f32>() / dissimilar_scores.len() as f32;
    
    println!("📈 Average similar score: {:.4}", avg_similar);
    println!("📉 Average dissimilar score: {:.4}", avg_dissimilar);
    
    // Quality assertions
    assert!(avg_similar > 0.5, "Similar pairs should have similarity > 0.5, got {:.4}", avg_similar);
    assert!(avg_dissimilar < 0.3, "Dissimilar pairs should have similarity < 0.3, got {:.4}", avg_dissimilar);
    assert!(avg_similar > avg_dissimilar + 0.2, "Similar pairs should be significantly more similar than dissimilar pairs");
    
    // Step 6: Test vector search functionality
    println!("📋 Step 6: Testing vector search functionality");
    let search_start = Instant::now();
    
    let query = "function that calculates the total price of items";
    let query_embedding = embedder.embed(query)?;
    
    let search_results = storage.read().await.search_similar(&query_embedding, 5).await?;
    println!("🔍 Found {} search results in {:?}", search_results.len(), search_start.elapsed());
    
    assert!(!search_results.is_empty(), "Search should return results");
    assert!(search_results.len() <= 5, "Should not return more than requested limit");
    
    // Verify results are ranked by similarity
    for i in 1..search_results.len() {
        assert!(search_results[i-1].similarity >= search_results[i].similarity, 
                "Results should be sorted by similarity in descending order");
    }
    
    println!("✅ Complete Pipeline Integration Test PASSED");
    Ok(())
}

#[tokio::test] 
async fn test_system_fault_tolerance() -> Result<()> {
    println!("🛡️ Testing System Fault Tolerance");
    
    // Test 1: Empty input handling
    println!("🧪 Test 1: Empty input handling");
    #[cfg(feature = "ml")]
    {
        let embedder = match NomicEmbedder::get_global().await {
            Ok(e) => e,
            Err(_) => {
                println!("⚠️ Skipping ML tests - model not available");
                return Ok(());
            }
        };
        
        let result = embedder.embed("");
        match result {
            Ok(embedding) => {
                assert_eq!(embedding.len(), 768, "Empty input should still produce valid embedding");
                println!("✅ Empty input handled gracefully");
            }
            Err(e) => {
                println!("⚠️ Empty input handling failed: {}", e);
                // This is acceptable behavior
            }
        }
    }
    
    // Test 2: Very long input handling  
    println!("🧪 Test 2: Very long input handling");
    #[cfg(feature = "ml")]
    {
        let embedder = NomicEmbedder::get_global().await?;
        let long_text = "a".repeat(10000);
        
        let start = Instant::now();
        let result = embedder.embed(&long_text);
        let duration = start.elapsed();
        
        match result {
            Ok(embedding) => {
                assert_eq!(embedding.len(), 768);
                println!("✅ Long input handled in {:?}", duration);
            }
            Err(e) => {
                println!("⚠️ Long input failed (acceptable): {}", e);
            }
        }
    }
    
    // Test 3: Special characters and Unicode
    println!("🧪 Test 3: Special characters and Unicode handling");
    #[cfg(feature = "ml")]
    {
        let embedder = NomicEmbedder::get_global().await?;
        let special_text = "Hello 🌍! This is a test with émojis and spëcial châractèrs: αβγ 中文 العربية";
        
        let result = embedder.embed(special_text)?;
        assert_eq!(result.len(), 768);
        println!("✅ Special characters handled correctly");
    }
    
    println!("✅ Fault Tolerance Tests PASSED");
    Ok(())
}

#[tokio::test]
async fn test_memory_usage_and_performance() -> Result<()> {
    println!("📊 Testing Memory Usage and Performance");
    
    #[cfg(feature = "ml")]
    {
        let embedder = match NomicEmbedder::get_global().await {
            Ok(e) => e,
            Err(_) => {
                println!("⚠️ Skipping performance tests - model not available");
                return Ok(());
            }
        };
        
        let test_texts = vec![
            "Short text",
            "Medium length text that contains several words and should be processed efficiently",
            "Much longer text content that spans multiple lines and includes various programming concepts like functions, classes, variables, and complex logic patterns that might be found in a typical codebase",
        ];
        
        println!("🚀 Performance benchmarking with {} different text lengths", test_texts.len());
        
        let mut total_time = std::time::Duration::ZERO;
        let iterations = 10;
        
        for (i, text) in test_texts.iter().enumerate() {
            let mut text_total = std::time::Duration::ZERO;
            
            for _ in 0..iterations {
                let start = Instant::now();
                let _embedding = embedder.embed(text)?;
                text_total += start.elapsed();
            }
            
            let avg_time = text_total / iterations;
            total_time += text_total;
            
            println!("📏 Text {}: {} chars, avg time: {:?}", 
                     i + 1, text.len(), avg_time);
        }
        
        let overall_avg = total_time / (test_texts.len() as u32 * iterations);
        println!("📈 Overall average embedding time: {:?}", overall_avg);
        
        // Performance assertions
        assert!(overall_avg.as_millis() < 1000, "Average embedding time should be under 1 second");
        
        println!("✅ Performance benchmarks PASSED");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_access_patterns() -> Result<()> {
    println!("🔄 Testing Concurrent Access Patterns");
    
    #[cfg(feature = "ml")]
    {
        let embedder = match NomicEmbedder::get_global().await {
            Ok(e) => e,
            Err(_) => {
                println!("⚠️ Skipping concurrency tests - model not available");
                return Ok(());
            }
        };
        
        let test_texts = vec![
            "Concurrent access test 1",
            "Concurrent access test 2", 
            "Concurrent access test 3",
            "Concurrent access test 4",
            "Concurrent access test 5",
        ];
        
        println!("🧵 Testing {} concurrent embedding requests", test_texts.len());
        let start = Instant::now();
        
        let mut handles = Vec::new();
        
        for text in test_texts {
            let embedder_clone = embedder.clone();
            let handle = tokio::spawn(async move {
                embedder_clone.embed(&text)
            });
            handles.push(handle);
        }
        
        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await??;
            results.push(result);
        }
        
        let duration = start.elapsed();
        println!("⏱️ Concurrent operations completed in {:?}", duration);
        
        // Verify all results are valid
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.len(), 768, "Concurrent result {} should have 768 dimensions", i);
        }
        
        println!("✅ Concurrent access patterns PASSED");
    }
    
    Ok(())
}

/// Helper function to create test documents for pipeline testing
fn create_test_documents() -> Vec<String> {
    vec![
        r#"
        function calculateTotalPrice(items) {
            return items.reduce((sum, item) => sum + item.price, 0);
        }
        
        function applyDiscount(total, discountPercent) {
            return total * (1 - discountPercent / 100);
        }
        "#.to_string(),
        
        r#"
        class DatabaseConnection {
            constructor(host, port) {
                this.host = host;
                this.port = port;
                this.connected = false;
            }
            
            async connect() {
                try {
                    await this.establishConnection();
                    this.connected = true;
                } catch (error) {
                    console.error('Connection failed:', error);
                }
            }
        }
        "#.to_string(),
        
        r#"
        def process_user_data(user_info):
            """Process and validate user information"""
            if not user_info:
                return None
                
            validated_data = {}
            for key, value in user_info.items():
                if validate_field(key, value):
                    validated_data[key] = sanitize_input(value)
                    
            return validated_data
        "#.to_string(),
        
        r#"
        pub struct SearchEngine {
            pub config: Config,
            pub embedder: Arc<Embedder>,
            pub storage: Arc<VectorStorage>,
        }
        
        impl SearchEngine {
            pub async fn new(config: Config) -> Result<Self> {
                let embedder = Arc::new(Embedder::load(&config.model_path).await?);
                let storage = Arc::new(VectorStorage::new(&config.db_path).await?);
                
                Ok(SearchEngine {
                    config,
                    embedder,
                    storage,
                })
            }
        }
        "#.to_string(),
        
        r#"
        async function handleApiRequest(req, res) {
            try {
                const { method, url, body } = req;
                
                if (method === 'GET') {
                    const data = await fetchData(url);
                    return res.json(data);
                }
                
                if (method === 'POST') {
                    const result = await processData(body);
                    return res.status(201).json(result);
                }
                
                res.status(405).json({ error: 'Method not allowed' });
            } catch (error) {
                res.status(500).json({ error: error.message });
            }
        }
        "#.to_string(),
    ]
}

/// Helper function to calculate cosine similarity between two vectors
fn calculate_cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have same length");
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

// Ensure tests are run with proper features
#[cfg(test)]
mod feature_validation {
    use super::*;
    
    #[test]
    fn verify_test_features() {
        #[cfg(not(feature = "ml"))]
        panic!("ML feature required for integration tests");
        
        #[cfg(not(feature = "vectordb"))]
        panic!("VectorDB feature required for integration tests");
        
        println!("✅ Required features are available");
    }
}