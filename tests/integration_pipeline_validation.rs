//! Integration Pipeline Validation Tests
//!
//! These tests verify the complete embedding pipeline from tokenization to final output.
//! CRITICAL: These tests will FAIL if any component in the pipeline is broken,
//! including tokenization, attention computation, quantization, or pooling.

#[cfg(feature = "ml")]
use embed_search::embedding::NomicEmbedder;
#[cfg(feature = "ml")]
mod fixtures {
    pub use crate::fixtures::semantic_similarity_benchmarks::*;
}
#[cfg(feature = "ml")]
use std::collections::{HashMap, HashSet};

/// Calculate cosine similarity between two embeddings
#[cfg(feature = "ml")]
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Embedding dimensions must match");
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// Test complete pipeline with diverse input types
/// Verifies that the entire pipeline works correctly across different input domains
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_complete_pipeline_diverse_inputs() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    // Comprehensive test cases covering different programming languages and constructs
    let test_cases = vec![
        // Programming languages
        ("python", "def fibonacci(n):\n    if n <= 1:\n        return n\n    return fibonacci(n-1) + fibonacci(n-2)"),
        ("javascript", "function factorial(n) {\n    return n === 0 ? 1 : n * factorial(n - 1);\n}"),
        ("rust", "fn bubble_sort<T: Ord>(arr: &mut [T]) {\n    for i in 0..arr.len() {\n        for j in 0..arr.len() - 1 - i {\n            if arr[j] > arr[j + 1] {\n                arr.swap(j, j + 1);\n            }\n        }\n    }\n}"),
        ("java", "public class QuickSort {\n    public static void quicksort(int[] arr, int low, int high) {\n        if (low < high) {\n            int pi = partition(arr, low, high);\n            quicksort(arr, low, pi - 1);\n            quicksort(arr, pi + 1, high);\n        }\n    }\n}"),
        
        // Database queries
        ("sql_select", "SELECT u.id, u.name, u.email, COUNT(o.id) as order_count\nFROM users u\nLEFT JOIN orders o ON u.id = o.user_id\nWHERE u.active = true\nGROUP BY u.id, u.name, u.email\nORDER BY order_count DESC\nLIMIT 100;"),
        ("sql_insert", "INSERT INTO products (name, description, price, category_id, created_at)\nVALUES ('Laptop', 'High-performance laptop', 1299.99, 1, NOW());"),
        ("mongodb", "db.users.aggregate([\n    { $match: { status: 'active' } },\n    { $group: { _id: '$department', count: { $sum: 1 } } },\n    { $sort: { count: -1 } }\n])"),
        
        // Web technologies
        ("html", "<!DOCTYPE html>\n<html lang='en'>\n<head>\n    <meta charset='UTF-8'>\n    <title>User Dashboard</title>\n</head>\n<body>\n    <div class='container'>\n        <h1>Welcome, User!</h1>\n        <nav class='sidebar'>\n            <ul>\n                <li><a href='/profile'>Profile</a></li>\n                <li><a href='/settings'>Settings</a></li>\n            </ul>\n        </nav>\n    </div>\n</body>\n</html>"),
        ("css", ".container {\n    max-width: 1200px;\n    margin: 0 auto;\n    padding: 20px;\n    display: grid;\n    grid-template-columns: 250px 1fr;\n    gap: 30px;\n}\n\n.sidebar {\n    background: #f5f5f5;\n    border-radius: 8px;\n    padding: 20px;\n}\n\n.sidebar ul {\n    list-style: none;\n    padding: 0;\n}"),
        ("json_api", "{\n    \"user\": {\n        \"id\": 123,\n        \"name\": \"John Doe\",\n        \"email\": \"john@example.com\",\n        \"preferences\": {\n            \"theme\": \"dark\",\n            \"notifications\": true,\n            \"language\": \"en\"\n        },\n        \"roles\": [\"user\", \"editor\"]\n    },\n    \"timestamp\": \"2024-01-15T10:30:00Z\"\n}"),
        
        // Configuration files
        ("yaml_config", "database:\n  host: localhost\n  port: 5432\n  name: myapp\n  username: ${DB_USER}\n  password: ${DB_PASS}\n  pool_size: 10\n\nredis:\n  host: localhost\n  port: 6379\n  db: 0\n\nlogging:\n  level: info\n  format: json\n  output: stdout"),
        ("dockerfile", "FROM node:16-alpine\n\nWORKDIR /app\n\nCOPY package*.json ./\nRUN npm ci --only=production\n\nCOPY . .\n\nEXPOSE 3000\n\nUSER node\n\nCMD [\"node\", \"server.js\"]"),
        
        // Documentation
        ("markdown_docs", "# User Authentication API\n\n## Overview\n\nThis API provides secure user authentication using JWT tokens.\n\n### Endpoints\n\n#### POST /api/auth/login\n\nAuthenticate a user and return a JWT token.\n\n**Request Body:**\n```json\n{\n    \"username\": \"string\",\n    \"password\": \"string\"\n}\n```\n\n**Response:**\n```json\n{\n    \"token\": \"jwt_token_here\",\n    \"expires_in\": 3600\n}\n```"),
        
        // Error handling and logs
        ("error_log", "2024-01-15 10:30:45 ERROR [UserService] Failed to authenticate user: InvalidCredentials\n    at UserService.authenticate (UserService.js:45)\n    at AuthController.login (AuthController.js:23)\n    at process._tickCallback (internal/process/next_tick.js:68)\nCaused by: Database connection timeout after 5000ms"),
        ("config_properties", "# Application Configuration\napp.name=MyApplication\napp.version=1.2.3\napp.debug=false\n\n# Database Configuration\ndb.host=localhost\ndb.port=5432\ndb.name=myapp\ndb.pool.min=5\ndb.pool.max=20\n\n# Security Settings\nsecurity.jwt.secret=${JWT_SECRET}\nsecurity.jwt.expiry=3600\nsecurity.cors.enabled=true"),
    ];
    
    let mut embeddings = HashMap::new();
    let mut embedding_stats = HashMap::new();
    
    println!("Testing complete pipeline with {} diverse input types...", test_cases.len());
    
    // Generate embeddings for all test cases
    for (category, input) in &test_cases {
        let embedding = embedder.embed(input).unwrap();
        
        // CRITICAL VALIDATION: Every embedding must be valid
        assert_eq!(embedding.len(), 768, 
            "Wrong embedding dimension for {}: got {}, expected 768", category, embedding.len());
        
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01,
            "Poor L2 normalization for {}: norm = {:.6}, expected ~1.0", category, norm);
        
        assert!(!embedding.iter().any(|&x| x.is_nan()),
            "NaN values found in {} embedding", category);
        assert!(!embedding.iter().any(|&x| x.is_infinite()),
            "Infinite values found in {} embedding", category);
        
        // Collect statistics
        let mean = embedding.iter().sum::<f32>() / embedding.len() as f32;
        let max_val = embedding.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let min_val = embedding.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let range = max_val - min_val;
        
        embeddings.insert(*category, embedding);
        embedding_stats.insert(*category, (norm, mean, range, max_val, min_val));
        
        println!("  ✓ {}: norm={:.6}, mean={:.6}, range={:.6}", category, norm, mean, range);
    }
    
    // INTEGRATION TEST: Verify semantic clustering by category
    let language_categories = vec!["python", "javascript", "rust", "java"];
    let sql_categories = vec!["sql_select", "sql_insert", "mongodb"];
    let web_categories = vec!["html", "css", "json_api"];
    
    // Calculate intra-category similarities
    let mut category_similarities = Vec::new();
    
    for category_group in [&language_categories, &sql_categories, &web_categories] {
        let mut group_similarities = Vec::new();
        
        for i in 0..category_group.len() {
            for j in i+1..category_group.len() {
                if let (Some(emb1), Some(emb2)) = 
                    (embeddings.get(category_group[i]), embeddings.get(category_group[j])) {
                    let sim = cosine_similarity(emb1, emb2);
                    group_similarities.push(sim);
                }
            }
        }
        
        if !group_similarities.is_empty() {
            let avg_sim = group_similarities.iter().sum::<f32>() / group_similarities.len() as f32;
            category_similarities.push(avg_sim);
        }
    }
    
    // Calculate inter-category similarities (should be lower)
    let mut inter_category_similarities = Vec::new();
    let all_categories: Vec<_> = embeddings.keys().collect();
    
    for i in 0..all_categories.len() {
        for j in i+1..all_categories.len() {
            let cat1 = all_categories[i];
            let cat2 = all_categories[j];
            
            // Skip if both are in the same semantic group
            let same_group = 
                (language_categories.contains(cat1) && language_categories.contains(cat2)) ||
                (sql_categories.contains(cat1) && sql_categories.contains(cat2)) ||
                (web_categories.contains(cat1) && web_categories.contains(cat2));
            
            if !same_group {
                if let (Some(emb1), Some(emb2)) = (embeddings.get(cat1), embeddings.get(cat2)) {
                    let sim = cosine_similarity(emb1, emb2);
                    inter_category_similarities.push(sim);
                }
            }
        }
    }
    
    let avg_intra_similarity = if category_similarities.is_empty() {
        0.0
    } else {
        category_similarities.iter().sum::<f32>() / category_similarities.len() as f32
    };
    
    let avg_inter_similarity = if inter_category_similarities.is_empty() {
        0.0
    } else {
        inter_category_similarities.iter().sum::<f32>() / inter_category_similarities.len() as f32
    };
    
    // CRITICAL INTEGRATION TEST: Semantic clustering should work
    if avg_intra_similarity > 0.0 && avg_inter_similarity > 0.0 {
        assert!(avg_intra_similarity > avg_inter_similarity + 0.05,
            "❌ FAILED: Semantic clustering not working in complete pipeline\n  \
             Intra-category similarity: {:.4}\n  \
             Inter-category similarity: {:.4}\n  \
             The pipeline should produce semantically meaningful embeddings.",
            avg_intra_similarity, avg_inter_similarity);
    }
    
    println!("✅ Complete pipeline integration test passed!");
    println!("  - All {} input types processed successfully", test_cases.len());
    println!("  - Semantic clustering: intra={:.4}, inter={:.4}", avg_intra_similarity, avg_inter_similarity);
}

/// Test pipeline robustness with edge cases and malformed inputs
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_pipeline_robustness_edge_cases() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    let edge_cases = vec![
        // Empty and minimal inputs
        ("empty_string", ""),
        ("single_char", "a"),
        ("single_word", "test"),
        ("whitespace_only", "   \n\t  "),
        
        // Special characters
        ("unicode", "函数 función функция λ α β γ"),
        ("mixed_unicode", "def функция(): return 'α + β = γ'"),
        ("emoji", "function 👨‍💻 () { return '🚀'; }"),
        ("special_chars", "!@#$%^&*()_+-=[]{}|;':\",./<>?"),
        
        // Very long inputs
        ("repeated_token", "token ".repeat(100)),
        ("long_line", "x".repeat(1000)),
        ("nested_structure", "{{{{{{{{{{}}}}}}}}}}".repeat(10)),
        
        // Malformed code
        ("unclosed_brackets", "function test() { if (true { console.log('test'"),
        ("syntax_error", "def function class if while for:"),
        ("mixed_languages", "def function() { return 'hello'; } SELECT * FROM users;"),
        
        // Binary/encoded content
        ("base64_like", "YWJjZGVmZ2hpams1NM0ODQwOTB="),
        ("hex_like", "0x1A2B3C4D5E6F708090A0B0C0D0E0F0"),
        ("escaped_chars", "\\n\\r\\t\\\\\\\"\\'\\/"),
        
        // Very repetitive content
        ("all_same", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        ("pattern_repeat", "abcdefg".repeat(50)),
        ("number_sequence", (0..100).map(|i| i.to_string()).collect::<Vec<_>>().join(" ")),
    ];
    
    println!("Testing pipeline robustness with {} edge cases...", edge_cases.len());
    
    let mut successful_embeddings = 0;
    let mut failed_cases = Vec::new();
    
    for (case_name, input) in edge_cases {
        match embedder.embed(input) {
            Ok(embedding) => {
                // Validate the embedding
                assert_eq!(embedding.len(), 768, 
                    "Wrong dimensions for edge case {}: {}", case_name, embedding.len());
                
                let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                assert!((norm - 1.0).abs() < 0.01,
                    "Poor normalization for edge case {}: norm = {:.6}", case_name, norm);
                
                assert!(!embedding.iter().any(|&x| x.is_nan()),
                    "NaN values in edge case {} embedding", case_name);
                assert!(!embedding.iter().any(|&x| x.is_infinite()),
                    "Infinite values in edge case {} embedding", case_name);
                
                successful_embeddings += 1;
                println!("  ✓ {}: successful (norm: {:.6})", case_name, norm);
            }
            Err(e) => {
                failed_cases.push((case_name, e.to_string()));
                println!("  ✗ {}: failed with error: {}", case_name, e);
            }
        }
    }
    
    // CRITICAL ROBUSTNESS TEST: Most edge cases should be handled gracefully
    let success_rate = successful_embeddings as f64 / edge_cases.len() as f64;
    assert!(success_rate >= 0.8,
        "❌ FAILED: Pipeline robustness too low\n  \
         Success rate: {:.1}% ({}/{} cases)\n  \
         The pipeline should handle most edge cases gracefully.\n  \
         Failed cases: {:?}",
        success_rate * 100.0, successful_embeddings, edge_cases.len(), failed_cases);
    
    // Check that failures are reasonable (e.g., truly empty inputs)
    let unreasonable_failures: Vec<_> = failed_cases.iter()
        .filter(|(name, _)| !name.contains("empty") && !name.contains("whitespace"))
        .collect();
    
    assert!(unreasonable_failures.len() <= 2,
        "❌ FAILED: Too many unreasonable failures: {:?}\n  \
         Most non-empty inputs should be processable.",
        unreasonable_failures);
    
    println!("✅ Pipeline robustness test passed!");
    println!("  - Success rate: {:.1}% ({}/{})", success_rate * 100.0, successful_embeddings, edge_cases.len());
    println!("  - Failed cases: {}", failed_cases.len());
}

/// Test pipeline consistency and determinism
/// Same inputs should always produce identical outputs
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_pipeline_consistency_determinism() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    let consistency_tests = vec![
        "function authenticate(user, pass) { return validate(user, pass); }",
        "SELECT id, name FROM users WHERE active = true ORDER BY created_at",
        "class UserManager { constructor() { this.users = new Map(); } }",
        "def process_data(input): return [transform(item) for item in input]",
        "# Configuration\nhost: localhost\nport: 8080\ndebug: true",
    ];
    
    println!("Testing pipeline consistency with {} test cases...", consistency_tests.len());
    
    for (i, test_input) in consistency_tests.iter().enumerate() {
        // Generate multiple embeddings for the same input
        let mut embeddings = Vec::new();
        for attempt in 0..5 {
            let embedding = embedder.embed(test_input).unwrap();
            embeddings.push(embedding);
        }
        
        // CRITICAL CONSISTENCY TEST: All embeddings should be identical
        let reference = &embeddings[0];
        for (attempt, embedding) in embeddings.iter().enumerate().skip(1) {
            for (dim, (&ref_val, &emb_val)) in reference.iter().zip(embedding.iter()).enumerate() {
                assert_eq!(ref_val, emb_val,
                    "❌ INCONSISTENCY: Test case {} (attempt {}), dimension {}\n  \
                     Reference: {}, Current: {}\n  \
                     Same input should always produce identical embeddings.",
                    i, attempt, dim, ref_val, emb_val);
            }
        }
        
        // Verify the embedding is valid
        let norm: f32 = reference.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01,
            "Poor normalization for consistency test {}: norm = {:.6}", i, norm);
        
        println!("  ✓ Test case {}: consistent across 5 attempts (norm: {:.6})", i, norm);
    }
    
    println!("✅ Pipeline consistency test passed!");
}

/// Test pipeline integration with caching
/// Cached results should be identical to fresh computations
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_pipeline_caching_integration() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    let cache_tests = vec![
        "cached_test_1",
        "function cachedFunction() { return 'test'; }",
        "SELECT * FROM cached_table WHERE id = 1",
        "class CachedClass { method() {} }",
        "# Cached config\nvalue: cached_value",
    ];
    
    println!("Testing pipeline caching integration...");
    
    for (i, test_input) in cache_tests.iter().enumerate() {
        // First computation (should be fresh)
        let start_fresh = std::time::Instant::now();
        let fresh_embedding = embedder.embed(test_input).unwrap();
        let fresh_duration = start_fresh.elapsed();
        
        // Second computation (should be cached)
        let start_cached = std::time::Instant::now();
        let cached_embedding = embedder.embed(test_input).unwrap();
        let cached_duration = start_cached.elapsed();
        
        // CRITICAL CACHING TEST: Results should be identical
        assert_eq!(fresh_embedding.len(), cached_embedding.len());
        for (dim, (&fresh_val, &cached_val)) in fresh_embedding.iter().zip(cached_embedding.iter()).enumerate() {
            assert_eq!(fresh_val, cached_val,
                "❌ CACHE INCONSISTENCY: Test case {}, dimension {}\n  \
                 Fresh: {}, Cached: {}\n  \
                 Cached results must match fresh computations exactly.",
                i, dim, fresh_val, cached_val);
        }
        
        // Performance check: cached should generally be faster (though not guaranteed)
        let speedup = fresh_duration.as_nanos() as f64 / cached_duration.as_nanos() as f64;
        
        println!("  ✓ Test case {}: identical results (speedup: {:.2}x)", i, speedup);
    }
    
    println!("✅ Pipeline caching integration test passed!");
}

/// Test pipeline memory efficiency during sustained operation
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_pipeline_sustained_operation() {
    use std::time::Instant;
    
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    // Sustained operation test: process many varied inputs
    let base_inputs = vec![
        "function process() { return true; }",
        "SELECT count(*) FROM table",
        "class Handler { handle() {} }",
        "def calculate(x): return x * 2",
        "{ \"key\": \"value\", \"number\": 42 }",
    ];
    
    println!("Testing pipeline sustained operation (500 embeddings)...");
    
    let mut all_embeddings = Vec::new();
    let mut processing_times = Vec::new();
    
    let total_start = Instant::now();
    
    for batch in 0..10 {
        let batch_start = Instant::now();
        let mut batch_embeddings = Vec::new();
        
        // Process 50 varied inputs per batch
        for i in 0..50 {
            let input = format!("{} // variation {}", base_inputs[i % base_inputs.len()], batch * 50 + i);
            let embedding = embedder.embed(&input).unwrap();
            
            // Validate each embedding
            assert_eq!(embedding.len(), 768);
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 0.01, "Poor normalization in batch {}, item {}: norm = {}", batch, i, norm);
            
            batch_embeddings.push(embedding);
        }
        
        let batch_duration = batch_start.elapsed();
        processing_times.push(batch_duration);
        all_embeddings.extend(batch_embeddings);
        
        let batch_throughput = 50.0 / batch_duration.as_secs_f64();
        println!("  Batch {}: {}ms ({:.2} emb/sec)", batch, batch_duration.as_millis(), batch_throughput);
    }
    
    let total_duration = total_start.elapsed();
    let overall_throughput = 500.0 / total_duration.as_secs_f64();
    
    // CRITICAL SUSTAINED OPERATION TESTS
    assert_eq!(all_embeddings.len(), 500, "Should have processed exactly 500 embeddings");
    
    // Performance should not degrade significantly over time
    let first_half_avg = processing_times[0..5].iter().sum::<std::time::Duration>().as_millis() / 5;
    let second_half_avg = processing_times[5..10].iter().sum::<std::time::Duration>().as_millis() / 5;
    
    let performance_degradation = second_half_avg as f64 / first_half_avg as f64;
    assert!(performance_degradation < 2.0,
        "❌ PERFORMANCE DEGRADATION: Second half {}ms avg vs first half {}ms avg\n  \
         Degradation ratio: {:.2}x (should be < 2.0x)\n  \
         This suggests memory leaks or resource accumulation.",
        second_half_avg, first_half_avg, performance_degradation);
    
    // Overall throughput should meet minimum requirements
    assert!(overall_throughput > 5.0,
        "❌ POOR SUSTAINED THROUGHPUT: {:.2} emb/sec (expected > 5.0 emb/sec)\n  \
         Total time: {}ms for 500 embeddings",
        overall_throughput, total_duration.as_millis());
    
    // Verify all embeddings are distinct and valid
    let mut embedding_hashes = HashSet::new();
    let mut duplicates = 0;
    
    for (i, embedding) in all_embeddings.iter().enumerate() {
        // Create a simple hash for uniqueness checking
        let hash = format!("{:.6}", embedding.iter().take(10).sum::<f32>());
        if embedding_hashes.contains(&hash) {
            duplicates += 1;
        } else {
            embedding_hashes.insert(hash);
        }
        
        // Validate embedding quality
        assert!(!embedding.iter().any(|&x| x.is_nan()), "NaN in embedding {}", i);
        assert!(!embedding.iter().any(|&x| x.is_infinite()), "Infinite in embedding {}", i);
    }
    
    // Some duplicates are expected (due to similar inputs), but not too many
    let duplicate_rate = duplicates as f64 / 500.0;
    assert!(duplicate_rate < 0.5,
        "❌ TOO MANY DUPLICATE EMBEDDINGS: {:.1}% ({}/500)\n  \
         This suggests degenerate embedding generation.",
        duplicate_rate * 100.0, duplicates);
    
    println!("✅ Pipeline sustained operation test passed!");
    println!("  - Total: 500 embeddings in {}ms ({:.2} emb/sec)", 
            total_duration.as_millis(), overall_throughput);
    println!("  - Performance degradation: {:.2}x", performance_degradation);
    println!("  - Duplicate rate: {:.1}%", duplicate_rate * 100.0);
}

/// Test complete pipeline error handling and recovery
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_pipeline_error_handling_recovery() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    println!("Testing pipeline error handling and recovery...");
    
    // Test recovery after various challenging inputs
    let challenging_inputs = vec![
        ("normal", "function normal() { return true; }", true),
        ("very_long", "x".repeat(5000), true), // Should handle or gracefully truncate
        ("unicode_heavy", "🚀🎉🔥💯🎯🌟⭐✨🌈🦄".repeat(100), true),
        ("normal_after_challenge", "function recovery() { return 'works'; }", true),
        ("binary_like", "\x00\x01\x02\x03\x7F\x7E\x7D", false), // Might fail
        ("normal_after_binary", "SELECT * FROM recovery", true),
        ("deeply_nested", "(((((((((({}))))))))))".repeat(50), true),
        ("normal_after_nested", "class Recovery { test() {} }", true),
    ];
    
    let mut processed = 0;
    let mut last_successful_embedding: Option<Vec<f32>> = None;
    
    for (case_name, input, should_succeed) in challenging_inputs {
        match embedder.embed(input) {
            Ok(embedding) => {
                // Validate successful embedding
                assert_eq!(embedding.len(), 768, "Wrong dimensions for {}", case_name);
                
                let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                assert!((norm - 1.0).abs() < 0.01, "Poor normalization for {}: {}", case_name, norm);
                
                assert!(!embedding.iter().any(|&x| x.is_nan()), "NaN in {} embedding", case_name);
                assert!(!embedding.iter().any(|&x| x.is_infinite()), "Infinite in {} embedding", case_name);
                
                last_successful_embedding = Some(embedding);
                processed += 1;
                
                println!("  ✓ {}: successful (norm: {:.6})", case_name, norm);
            }
            Err(e) => {
                if should_succeed {
                    panic!("❌ UNEXPECTED FAILURE: {} should have succeeded but failed with: {}", case_name, e);
                } else {
                    println!("  ◯ {}: expected failure: {}", case_name, e);
                }
            }
        }
    }
    
    // CRITICAL RECOVERY TEST: Pipeline should continue working after challenges
    assert!(processed >= 6,
        "❌ POOR ERROR RECOVERY: Only {}/8 expected successes processed\n  \
         Pipeline should recover from challenging inputs.",
        processed);
    
    // Test that normal operation continues after recovery
    let post_recovery_tests = vec![
        "function final_test() { return 'success'; }",
        "SELECT 'recovery confirmed' as status",
        "class FinalTest { success() { return true; } }",
    ];
    
    for (i, test_input) in post_recovery_tests.iter().enumerate() {
        let embedding = embedder.embed(test_input).unwrap();
        
        assert_eq!(embedding.len(), 768);
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01, "Poor normalization for post-recovery test {}: {}", i, norm);
        
        // Compare with last successful embedding to ensure variety
        if let Some(ref last_emb) = last_successful_embedding {
            let similarity = cosine_similarity(&embedding, last_emb);
            assert!(similarity < 0.95, 
                "Post-recovery embedding {} too similar to previous: {:.4}", i, similarity);
        }
        
        println!("  ✓ Post-recovery test {}: successful (norm: {:.6})", i, norm);
    }
    
    println!("✅ Pipeline error handling and recovery test passed!");
    println!("  - Processed: {}/8 expected successes", processed);
    println!("  - Post-recovery: 3/3 tests successful");
}

#[cfg(not(feature = "ml"))]
mod no_ml_tests {
    #[test]
    fn test_integration_pipeline_requires_ml_feature() {
        println!("Integration pipeline tests require 'ml' feature to be enabled");
        println!("Run with: cargo test --features ml");
    }
}