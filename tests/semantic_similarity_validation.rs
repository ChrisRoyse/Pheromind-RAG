//! Semantic Similarity Validation Tests
//!
//! These tests use curated benchmark datasets to verify that the embedding model
//! preserves semantic relationships correctly. CRITICAL: These tests will FAIL
//! if the broken Q6K quantization produces semantically meaningless embeddings.

#[cfg(feature = "ml")]
use embed_search::embedding::NomicEmbedder;
#[cfg(feature = "ml")]
mod fixtures {
    pub use crate::fixtures::semantic_similarity_benchmarks::*;
}
#[cfg(feature = "ml")]
use std::collections::HashMap;

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

/// Test semantic similarity preservation using curated benchmark datasets
/// CRITICAL: This test will FAIL if embeddings don't preserve semantic meaning
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_semantic_similarity_benchmark_suite() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    let benchmarks = fixtures::get_semantic_similarity_benchmarks();
    
    let mut results = Vec::new();
    let mut category_results: HashMap<String, Vec<f32>> = HashMap::new();
    
    println!("Running semantic similarity benchmark suite ({} test cases)...", benchmarks.len());
    
    for (i, benchmark) in benchmarks.iter().enumerate() {
        let emb1 = embedder.embed(benchmark.text1).unwrap();
        let emb2 = embedder.embed(benchmark.text2).unwrap();
        
        let similarity = cosine_similarity(&emb1, &emb2);
        
        // CRITICAL VALIDATION: Similarity must be within expected bounds
        assert!(similarity >= benchmark.expected_similarity_min,
            "❌ FAILED: {} ({})\n  \
             Similarity too low: {:.4} < {:.4}\n  \
             Text1: '{}'\n  \
             Text2: '{}'\n  \
             This indicates broken semantic embedding generation.",
            benchmark.description, benchmark.category,
            similarity, benchmark.expected_similarity_min,
            benchmark.text1, benchmark.text2);
        
        assert!(similarity <= benchmark.expected_similarity_max,
            "❌ FAILED: {} ({})\n  \
             Similarity too high: {:.4} > {:.4}\n  \
             Text1: '{}'\n  \
             Text2: '{}'\n  \
             This indicates the model is not discriminating between different concepts.",
            benchmark.description, benchmark.category,
            similarity, benchmark.expected_similarity_max,
            benchmark.text1, benchmark.text2);
        
        // Track results by category
        category_results.entry(benchmark.category.to_string())
            .or_default()
            .push(similarity);
        
        results.push((benchmark, similarity));
        
        if i % 5 == 0 {
            println!("  Progress: {}/{}", i + 1, benchmarks.len());
        }
    }
    
    // Print detailed results
    println!("\n✅ Semantic similarity benchmark results:");
    for (benchmark, similarity) in &results {
        let status = if similarity >= &benchmark.expected_similarity_min && 
                        similarity <= &benchmark.expected_similarity_max {
            "✓"
        } else {
            "✗"
        };
        
        println!("  {} {} ({:.4}) - {} [expected: {:.4}-{:.4}]",
                status, benchmark.category, similarity, benchmark.description,
                benchmark.expected_similarity_min, benchmark.expected_similarity_max);
    }
    
    // Category analysis
    println!("\n📊 Category Analysis:");
    for (category, similarities) in category_results {
        let mean_sim = similarities.iter().sum::<f32>() / similarities.len() as f32;
        let min_sim = similarities.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_sim = similarities.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        
        println!("  {}: mean={:.4}, range=[{:.4}, {:.4}], count={}",
                category, mean_sim, min_sim, max_sim, similarities.len());
    }
    
    println!("\n✓ All semantic similarity benchmarks passed!");
}

/// Test intra-category similarity clustering  
/// Items in the same semantic category should be more similar to each other
/// than to items in different categories
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_intra_category_similarity_clustering() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    let by_category = fixtures::get_benchmarks_by_category();
    
    let mut intra_category_sims = Vec::new();
    let mut inter_category_sims = Vec::new();
    
    // Calculate intra-category similarities
    for (category, benchmarks) in &by_category {
        if benchmarks.len() < 2 {
            continue;
        }
        
        println!("Testing category clustering: {}", category);
        
        // Get embeddings for all texts in this category
        let mut category_embeddings = Vec::new();
        let mut category_texts = Vec::new();
        
        for benchmark in benchmarks {
            let emb1 = embedder.embed(benchmark.text1).unwrap();
            let emb2 = embedder.embed(benchmark.text2).unwrap();
            
            category_embeddings.push(emb1);
            category_embeddings.push(emb2);
            category_texts.push(benchmark.text1);
            category_texts.push(benchmark.text2);
        }
        
        // Calculate all pairwise similarities within category
        for i in 0..category_embeddings.len() {
            for j in i+1..category_embeddings.len() {
                let sim = cosine_similarity(&category_embeddings[i], &category_embeddings[j]);
                intra_category_sims.push(sim);
            }
        }
    }
    
    // Calculate inter-category similarities
    let category_names: Vec<_> = by_category.keys().collect();
    for i in 0..category_names.len() {
        for j in i+1..category_names.len() {
            let cat1 = category_names[i];
            let cat2 = category_names[j];
            
            if let (Some(benchmarks1), Some(benchmarks2)) = 
                (by_category.get(cat1), by_category.get(cat2)) {
                
                // Compare first benchmark from each category
                if !benchmarks1.is_empty() && !benchmarks2.is_empty() {
                    let emb1 = embedder.embed(benchmarks1[0].text1).unwrap();
                    let emb2 = embedder.embed(benchmarks2[0].text1).unwrap();
                    
                    let sim = cosine_similarity(&emb1, &emb2);
                    inter_category_sims.push(sim);
                }
            }
        }
    }
    
    // Statistical analysis
    let intra_mean = intra_category_sims.iter().sum::<f32>() / intra_category_sims.len() as f32;
    let inter_mean = inter_category_sims.iter().sum::<f32>() / inter_category_sims.len() as f32;
    
    let intra_max = intra_category_sims.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    let intra_min = intra_category_sims.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let inter_max = inter_category_sims.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    let inter_min = inter_category_sims.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    
    println!("\n📊 Category Clustering Analysis:");
    println!("  Intra-category: mean={:.4}, range=[{:.4}, {:.4}], samples={}",
            intra_mean, intra_min, intra_max, intra_category_sims.len());
    println!("  Inter-category: mean={:.4}, range=[{:.4}, {:.4}], samples={}",
            inter_mean, inter_min, inter_max, inter_category_sims.len());
    
    // CRITICAL TEST: Intra-category should be more similar than inter-category
    assert!(intra_mean > inter_mean + 0.05,
        "❌ FAILED: Categories not clustering properly.\n  \
         Intra-category mean: {:.4}, Inter-category mean: {:.4}\n  \
         Semantic categories should show clustering behavior.\n  \
         This indicates broken semantic embedding generation.",
        intra_mean, inter_mean);
    
    println!("✅ Category clustering test passed!");
    println!("  Clustering strength: {:.4}", intra_mean - inter_mean);
}

/// Test semantic similarity with programming language equivalents
/// Same algorithms in different languages should have high similarity
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_cross_language_semantic_equivalence() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    let equivalence_tests = vec![
        (
            "fibonacci_python",
            "def fibonacci(n): return n if n <= 1 else fibonacci(n-1) + fibonacci(n-2)",
            "fibonacci_js", 
            "function fibonacci(n) { return n <= 1 ? n : fibonacci(n-1) + fibonacci(n-2); }",
            0.70, 0.95
        ),
        (
            "quicksort_python",
            "def quicksort(arr): return arr if len(arr) <= 1 else quicksort([x for x in arr[1:] if x < arr[0]]) + [arr[0]] + quicksort([x for x in arr[1:] if x >= arr[0]])",
            "quicksort_java",
            "public static void quicksort(int[] arr, int low, int high) { if (low < high) { int pi = partition(arr, low, high); quicksort(arr, low, pi - 1); quicksort(arr, pi + 1, high); } }",
            0.60, 0.85
        ),
        (
            "class_python",
            "class User: def __init__(self, name): self.name = name",
            "class_java",
            "public class User { private String name; public User(String name) { this.name = name; } }",
            0.65, 0.90
        ),
        (
            "loop_python", 
            "for i in range(10): print(i)",
            "loop_c",
            "for (int i = 0; i < 10; i++) { printf(\"%d\\n\", i); }",
            0.70, 0.95
        ),
    ];
    
    println!("Testing cross-language semantic equivalence...");
    
    for (name1, code1, name2, code2, min_sim, max_sim) in equivalence_tests {
        let emb1 = embedder.embed(code1).unwrap();
        let emb2 = embedder.embed(code2).unwrap();
        
        let similarity = cosine_similarity(&emb1, &emb2);
        
        assert!(similarity >= min_sim,
            "❌ FAILED: Cross-language equivalence too low\n  \
             {} vs {}: {:.4} < {:.4}\n  \
             Code1: '{}'\n  \
             Code2: '{}'\n  \
             This indicates the model doesn't understand algorithmic equivalence.",
            name1, name2, similarity, min_sim, code1, code2);
        
        assert!(similarity <= max_sim,
            "❌ FAILED: Cross-language equivalence too high\n  \
             {} vs {}: {:.4} > {:.4}\n  \
             This might indicate the model is not sensitive to language differences.",
            name1, name2, similarity, max_sim);
        
        println!("  ✓ {} vs {}: {:.4} [expected: {:.4}-{:.4}]",
                name1, name2, similarity, min_sim, max_sim);
    }
    
    println!("✅ Cross-language semantic equivalence test passed!");
}

/// Test semantic gradients - similarity should correlate with semantic distance
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_semantic_gradient_preservation() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    // Create semantic gradients from specific to general concepts
    let gradients = vec![
        vec![
            "authenticate_user",
            "user_auth", 
            "authentication",
            "security",
            "system"
        ],
        vec![
            "mysql_query",
            "sql_query",
            "database_query", 
            "database",
            "storage"
        ],
        vec![
            "http_get_request",
            "http_request",
            "network_request",
            "network",
            "communication"
        ],
    ];
    
    for (gradient_idx, gradient) in gradients.iter().enumerate() {
        println!("Testing semantic gradient {}: {:?}", gradient_idx, gradient);
        
        // Generate embeddings for the gradient
        let mut embeddings = Vec::new();
        for term in gradient {
            let emb = embedder.embed(term).unwrap();
            embeddings.push(emb);
        }
        
        // Test gradient properties
        for i in 0..embeddings.len() - 2 {
            let sim_adjacent = cosine_similarity(&embeddings[i], &embeddings[i + 1]);
            let sim_distant = cosine_similarity(&embeddings[i], &embeddings[i + 2]);
            
            // CRITICAL TEST: Adjacent concepts should be more similar than distant ones
            assert!(sim_adjacent > sim_distant + 0.02,
                "❌ FAILED: Semantic gradient not preserved\n  \
                 '{}' vs '{}': {:.4}\n  \
                 '{}' vs '{}': {:.4}\n  \
                 Adjacent concepts should be more similar than distant ones.\n  \
                 This indicates broken semantic distance preservation.",
                gradient[i], gradient[i + 1], sim_adjacent,
                gradient[i], gradient[i + 2], sim_distant);
            
            println!("    {} -> {}: {:.4}, {} -> {}: {:.4} ✓",
                    gradient[i], gradient[i + 1], sim_adjacent,
                    gradient[i], gradient[i + 2], sim_distant);
        }
    }
    
    println!("✅ Semantic gradient preservation test passed!");
}

/// Test semantic compositionality - compound concepts should relate to components
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_semantic_compositionality() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    let compositionality_tests = vec![
        (
            "user_authentication",
            vec!["user", "authentication"],
            0.4, 0.8, "compound concept should relate to components"
        ),
        (
            "database_connection",
            vec!["database", "connection"],
            0.4, 0.8, "compound concept should relate to components"  
        ),
        (
            "error_handling",
            vec!["error", "handling"],
            0.4, 0.8, "compound concept should relate to components"
        ),
        (
            "network_security",
            vec!["network", "security"],
            0.4, 0.8, "compound concept should relate to components"
        ),
    ];
    
    println!("Testing semantic compositionality...");
    
    for (compound, components, min_sim, max_sim, description) in compositionality_tests {
        let compound_emb = embedder.embed(compound).unwrap();
        
        for component in &components {
            let component_emb = embedder.embed(component).unwrap();
            let similarity = cosine_similarity(&compound_emb, &component_emb);
            
            assert!(similarity >= min_sim,
                "❌ FAILED: Compositionality too weak\n  \
                 '{}' vs component '{}': {:.4} < {:.4}\n  \
                 {}\n  \
                 This indicates broken semantic compositionality.",
                compound, component, similarity, min_sim, description);
            
            assert!(similarity <= max_sim,
                "❌ FAILED: Compositionality too strong\n  \
                 '{}' vs component '{}': {:.4} > {:.4}\n  \
                 Components should be related but distinct from compounds.",
                compound, component, similarity, max_sim);
            
            println!("  ✓ '{}' -> '{}': {:.4} [expected: {:.4}-{:.4}]",
                    compound, component, similarity, min_sim, max_sim);
        }
        
        // Test that compound is different from individual components
        if components.len() >= 2 {
            let comp1_emb = embedder.embed(components[0]).unwrap();
            let comp2_emb = embedder.embed(components[1]).unwrap();
            let comp_sim = cosine_similarity(&comp1_emb, &comp2_emb);
            let compound_comp1_sim = cosine_similarity(&compound_emb, &comp1_emb);
            
            // Compound should be distinct from its components
            assert!(compound_comp1_sim < 0.95,
                "Compound '{}' too similar to component '{}': {:.4}. Should be related but distinct.",
                compound, components[0], compound_comp1_sim);
        }
    }
    
    println!("✅ Semantic compositionality test passed!");
}

/// Test robustness to syntactic variations
/// Semantically equivalent expressions with different syntax should be similar
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_syntactic_variation_robustness() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    let variation_tests = vec![
        (
            "camelCase function",
            "getUserById",
            "snake_case function", 
            "get_user_by_id",
            0.70, 0.95, "naming convention variations"
        ),
        (
            "abbreviated",
            "auth_user",
            "expanded",
            "authenticate_user", 
            0.75, 0.95, "abbreviation variations"
        ),
        (
            "with_spaces",
            "user login system",
            "underscore",
            "user_login_system",
            0.80, 0.98, "spacing variations"
        ),
        (
            "singular",
            "create_user",
            "plural",
            "create_users",
            0.70, 0.95, "singular/plural variations"
        ),
    ];
    
    println!("Testing robustness to syntactic variations...");
    
    for (type1, text1, type2, text2, min_sim, max_sim, description) in variation_tests {
        let emb1 = embedder.embed(text1).unwrap();
        let emb2 = embedder.embed(text2).unwrap();
        
        let similarity = cosine_similarity(&emb1, &emb2);
        
        assert!(similarity >= min_sim,
            "❌ FAILED: Not robust to syntactic variation\n  \
             {} ('{}') vs {} ('{}'): {:.4} < {:.4}\n  \
             {}\n  \
             This indicates the model is too sensitive to syntax.",
            type1, text1, type2, text2, similarity, min_sim, description);
        
        assert!(similarity <= max_sim,
            "❌ FAILED: Syntactic variations too similar\n  \
             This might indicate insufficient sensitivity to meaningful differences.",
            );
        
        println!("  ✓ {} vs {}: {:.4} [expected: {:.4}-{:.4}] - {}",
                type1, type2, similarity, min_sim, max_sim, description);
    }
    
    println!("✅ Syntactic variation robustness test passed!");
}

/// Comprehensive semantic validation using all benchmark categories
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_comprehensive_semantic_validation() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    println!("Running comprehensive semantic validation...");
    
    // Test 1: Basic functionality preservation
    let basic_tests = vec![
        ("print('hello')", "console.log('hello')", 0.60, 0.90),
        ("x = x + 1", "x++", 0.65, 0.90),
        ("if x > 0:", "if (x > 0) {", 0.70, 0.95),
    ];
    
    for (code1, code2, min_sim, max_sim) in basic_tests {
        let emb1 = embedder.embed(code1).unwrap();
        let emb2 = embedder.embed(code2).unwrap();
        let sim = cosine_similarity(&emb1, &emb2);
        
        assert!(sim >= min_sim && sim <= max_sim,
            "Basic functionality test failed: '{}' vs '{}' = {:.4} (expected {:.4}-{:.4})",
            code1, code2, sim, min_sim, max_sim);
    }
    
    // Test 2: Domain clustering
    let auth_texts = vec![
        "login_user", "authenticate", "verify_password", "check_credentials"
    ];
    let db_texts = vec![
        "select_query", "insert_record", "update_table", "delete_row"  
    ];
    
    // Calculate intra-domain similarities
    let auth_embeddings: Vec<_> = auth_texts.iter()
        .map(|text| embedder.embed(text).unwrap())
        .collect();
    let db_embeddings: Vec<_> = db_texts.iter()
        .map(|text| embedder.embed(text).unwrap())
        .collect();
    
    let mut intra_auth_sims = Vec::new();
    let mut intra_db_sims = Vec::new();
    let mut inter_domain_sims = Vec::new();
    
    // Intra-domain similarities
    for i in 0..auth_embeddings.len() {
        for j in i+1..auth_embeddings.len() {
            intra_auth_sims.push(cosine_similarity(&auth_embeddings[i], &auth_embeddings[j]));
        }
    }
    
    for i in 0..db_embeddings.len() {
        for j in i+1..db_embeddings.len() {
            intra_db_sims.push(cosine_similarity(&db_embeddings[i], &db_embeddings[j]));
        }
    }
    
    // Inter-domain similarities
    for auth_emb in &auth_embeddings {
        for db_emb in &db_embeddings {
            inter_domain_sims.push(cosine_similarity(auth_emb, db_emb));
        }
    }
    
    let intra_mean = (intra_auth_sims.iter().sum::<f32>() + intra_db_sims.iter().sum::<f32>()) 
        / (intra_auth_sims.len() + intra_db_sims.len()) as f32;
    let inter_mean = inter_domain_sims.iter().sum::<f32>() / inter_domain_sims.len() as f32;
    
    assert!(intra_mean > inter_mean + 0.05,
        "❌ FAILED: Domain clustering not working. Intra: {:.4}, Inter: {:.4}",
        intra_mean, inter_mean);
    
    // Test 3: Semantic stability (same text should produce identical embeddings)
    let test_text = "function authenticate_user(username, password)";
    let emb_a = embedder.embed(test_text).unwrap();
    let emb_b = embedder.embed(test_text).unwrap();
    let stability = cosine_similarity(&emb_a, &emb_b);
    
    assert!(stability > 0.9999,
        "❌ FAILED: Semantic stability broken: {:.6}. Same text should produce identical embeddings.",
        stability);
    
    println!("✅ Comprehensive semantic validation passed!");
    println!("  - Basic functionality: ✓");
    println!("  - Domain clustering strength: {:.4}", intra_mean - inter_mean);
    println!("  - Semantic stability: {:.6}", stability);
}

#[cfg(not(feature = "ml"))]
mod no_ml_tests {
    #[test] 
    fn test_semantic_similarity_requires_ml_feature() {
        println!("Semantic similarity validation tests require 'ml' feature to be enabled");
        println!("Run with: cargo test --features ml");
    }
}