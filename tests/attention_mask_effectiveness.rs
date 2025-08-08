//! Attention Mask Effectiveness Tests
//!
//! These tests verify that attention masks actually affect the output of the transformer.
//! CRITICAL: These tests will FAIL if the attention mechanism ignores masks or
//! if the broken Q6K quantization makes all weights meaningless.

#[cfg(feature = "ml")]
use embed_search::embedding::NomicEmbedder;
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

/// Test that different input lengths produce different attention patterns
/// This indirectly tests attention mask effectiveness since different lengths
/// create different effective attention masks
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_attention_length_sensitivity() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    // Create texts of different lengths with the same starting tokens
    let base_text = "class User";
    let extended_text = "class User { constructor(name) { this.name = name; } }";
    let very_extended_text = "class User { constructor(name, email, age) { this.name = name; this.email = email; this.age = age; } validateEmail() { return this.email.includes('@'); } }";
    
    let base_embedding = embedder.embed(base_text).unwrap();
    let extended_embedding = embedder.embed(extended_text).unwrap();
    let very_extended_embedding = embedder.embed(very_extended_text).unwrap();
    
    // Calculate similarities
    let base_vs_extended = cosine_similarity(&base_embedding, &extended_embedding);
    let base_vs_very_extended = cosine_similarity(&base_embedding, &very_extended_embedding);
    let extended_vs_very_extended = cosine_similarity(&extended_embedding, &very_extended_embedding);
    
    // CRITICAL TEST: Different lengths should produce different embeddings
    // If attention masks don't work, these might be too similar
    assert!(base_vs_extended < 0.98, 
        "Base vs extended too similar: {:.4}. Attention may not be processing length differences correctly.",
        base_vs_extended);
    
    assert!(base_vs_very_extended < 0.95, 
        "Base vs very extended too similar: {:.4}. Attention may be ignoring most of the input.",
        base_vs_very_extended);
    
    // Similarity should generally decrease with more differences
    assert!(base_vs_extended > base_vs_very_extended - 0.1,
        "Unexpected similarity pattern. Extended similarity: {:.4}, Very extended: {:.4}",
        base_vs_extended, base_vs_very_extended);
    
    println!("✓ Attention length sensitivity test passed");
    println!("  - Base vs Extended: {:.4}", base_vs_extended);
    println!("  - Base vs Very Extended: {:.4}", base_vs_very_extended);
    println!("  - Extended vs Very Extended: {:.4}", extended_vs_very_extended);
}

/// Test attention effectiveness with token position sensitivity
/// Different orderings of the same tokens should produce different results
/// if attention is working correctly
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_attention_position_sensitivity() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    // Same tokens in different orders
    let order1 = "def authenticate user password";
    let order2 = "password authenticate user def";  
    let order3 = "user def password authenticate";
    
    let emb1 = embedder.embed(order1).unwrap();
    let emb2 = embedder.embed(order2).unwrap();
    let emb3 = embedder.embed(order3).unwrap();
    
    let sim_1_2 = cosine_similarity(&emb1, &emb2);
    let sim_1_3 = cosine_similarity(&emb1, &emb3);
    let sim_2_3 = cosine_similarity(&emb2, &emb3);
    
    // CRITICAL TEST: Position should matter if attention is working
    assert!(sim_1_2 < 0.95, 
        "Different token orders too similar (1-2): {:.4}. Attention may not be position-sensitive.",
        sim_1_2);
    
    assert!(sim_1_3 < 0.95, 
        "Different token orders too similar (1-3): {:.4}. Attention may not be position-sensitive.",
        sim_1_3);
    
    assert!(sim_2_3 < 0.95, 
        "Different token orders too similar (2-3): {:.4}. Attention may not be position-sensitive.",
        sim_2_3);
    
    // But they should still be more similar than completely different text
    let different_text = "network socket connection timeout";
    let different_emb = embedder.embed(different_text).unwrap();
    let sim_different = cosine_similarity(&emb1, &different_emb);
    
    assert!(sim_1_2 > sim_different + 0.1,
        "Reordered tokens should be more similar to each other than to different tokens. \
         Reordered: {:.4}, Different: {:.4}", sim_1_2, sim_different);
    
    println!("✓ Attention position sensitivity test passed");
    println!("  - Order similarities: {:.4}, {:.4}, {:.4}", sim_1_2, sim_1_3, sim_2_3);
    println!("  - Different text similarity: {:.4}", sim_different);
}

/// Test that attention properly handles context relationships
/// Tokens should influence each other through attention if working correctly
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_attention_context_influence() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    // Test word in different contexts
    let context_tests = vec![
        ("bank river", "bank by the river for fishing"),
        ("bank money", "bank account with money deposit"),
        ("bank turn", "sharp bank turn in racing"),
    ];
    
    let mut embeddings = Vec::new();
    let mut context_embeddings = Vec::new();
    
    for (word_context, full_context) in &context_tests {
        let word_emb = embedder.embed(word_context).unwrap();
        let full_emb = embedder.embed(full_context).unwrap();
        
        embeddings.push(word_emb);
        context_embeddings.push(full_emb);
    }
    
    // CRITICAL TEST: Context should influence word meaning if attention works
    for i in 0..embeddings.len() {
        let word_to_full_sim = cosine_similarity(&embeddings[i], &context_embeddings[i]);
        
        // Word should be somewhat similar to its full context
        assert!(word_to_full_sim > 0.3,
            "Word '{}' too dissimilar from its context: {:.4}. Context not influencing meaning.",
            context_tests[i].0, word_to_full_sim);
        
        // But different contexts should create different word meanings
        for j in 0..embeddings.len() {
            if i != j {
                let cross_sim = cosine_similarity(&embeddings[i], &context_embeddings[j]);
                assert!(word_to_full_sim > cross_sim + 0.05,
                    "Context not disambiguating word meaning. Own context: {:.4}, Other context: {:.4}",
                    word_to_full_sim, cross_sim);
            }
        }
    }
    
    println!("✓ Attention context influence test passed");
    for (i, (word_context, _)) in context_tests.iter().enumerate() {
        let sim = cosine_similarity(&embeddings[i], &context_embeddings[i]);
        println!("  - '{}' context similarity: {:.4}", word_context, sim);
    }
}

/// Test attention effectiveness with increasing context complexity
/// More complex contexts should create more nuanced embeddings
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_attention_complexity_scaling() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    // Progressive complexity tests
    let complexity_levels = vec![
        ("simple", "user login"),
        ("medium", "user login authentication system"),
        ("complex", "user login authentication system with password validation and security checks"),
        ("very_complex", "user login authentication system with password validation, security checks, two-factor authentication, session management, and audit logging capabilities"),
    ];
    
    let mut embeddings = Vec::new();
    let mut complexities = Vec::new();
    
    for (level, text) in &complexity_levels {
        let emb = embedder.embed(text).unwrap();
        
        // Measure "complexity" as distribution entropy
        let mean = emb.iter().sum::<f32>() / emb.len() as f32;
        let variance = emb.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / emb.len() as f32;
        let complexity_measure = variance.sqrt();
        
        embeddings.push(emb);
        complexities.push(complexity_measure);
        
        println!("  - {}: complexity measure = {:.6}", level, complexity_measure);
    }
    
    // CRITICAL TEST: More complex text should engage more of the model
    // (though this relationship isn't strictly monotonic)
    let simple_complex_sim = cosine_similarity(&embeddings[0], &embeddings[3]);
    let medium_complex_sim = cosine_similarity(&embeddings[1], &embeddings[3]);
    
    assert!(simple_complex_sim < 0.90,
        "Simple vs very complex too similar: {:.4}. Attention may not be processing complexity.",
        simple_complex_sim);
    
    // Verify embeddings are meaningful (not degenerate)
    for (i, (level, _)) in complexity_levels.iter().enumerate() {
        let norm: f32 = embeddings[i].iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01,
            "Poor normalization for {}: {:.6}", level, norm);
        
        assert!(!embeddings[i].iter().any(|&x| x.is_nan()),
            "NaN values in {} embedding", level);
    }
    
    println!("✓ Attention complexity scaling test passed");
    println!("  - Simple vs Very Complex similarity: {:.4}", simple_complex_sim);
    println!("  - Medium vs Very Complex similarity: {:.4}", medium_complex_sim);
}

/// Test that attention masks properly handle padding scenarios  
/// This tests the attention mask validation and application
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_attention_mask_padding_behavior() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    // Test sequences that would naturally have different padding
    // (internally, tokenization creates different effective masks)
    
    let short_sequence = "test";
    let medium_sequence = "test sequence with multiple tokens here";  
    let long_sequence = "test sequence with multiple tokens here and even more tokens to create a longer sequence that would require different attention mask handling in the transformer layers";
    
    let short_emb = embedder.embed(short_sequence).unwrap();
    let medium_emb = embedder.embed(medium_sequence).unwrap();
    let long_emb = embedder.embed(long_sequence).unwrap();
    
    // CRITICAL TESTS: Different sequence lengths should produce different results
    let short_medium_sim = cosine_similarity(&short_emb, &medium_emb);
    let short_long_sim = cosine_similarity(&short_emb, &long_emb);
    let medium_long_sim = cosine_similarity(&medium_emb, &long_emb);
    
    assert!(short_medium_sim < 0.95,
        "Short vs medium too similar: {:.4}. Attention masks may not be working.",
        short_medium_sim);
    
    assert!(short_long_sim < 0.90,
        "Short vs long too similar: {:.4}. Attention masks may not be working.",
        short_long_sim);
    
    // Verify no degradation with longer sequences (common mask bug)
    let long_norm: f32 = long_emb.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((long_norm - 1.0).abs() < 0.01,
        "Long sequence produced poor normalization: {:.6}. Mask handling may be broken.",
        long_norm);
    
    assert!(!long_emb.iter().any(|&x| x.is_nan()),
        "Long sequence produced NaN values. Mask handling is broken.");
    
    println!("✓ Attention mask padding behavior test passed");
    println!("  - Short-Medium similarity: {:.4}", short_medium_sim);
    println!("  - Short-Long similarity: {:.4}", short_long_sim);
    println!("  - Medium-Long similarity: {:.4}", medium_long_sim);
    println!("  - Long sequence norm: {:.6}", long_norm);
}

/// Test attention mechanism with repeated tokens
/// This exposes issues where attention doesn't properly distribute over repetitions
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_attention_repeated_token_handling() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    // Different repetition patterns  
    let single = "function";
    let double = "function function";
    let many = "function function function function function";
    let with_context = "function authenticate user function validate function";
    
    let single_emb = embedder.embed(single).unwrap();
    let double_emb = embedder.embed(double).unwrap();
    let many_emb = embedder.embed(many).unwrap();
    let context_emb = embedder.embed(with_context).unwrap();
    
    // CRITICAL TEST: Repetition should change the embedding
    let single_double_sim = cosine_similarity(&single_emb, &double_emb);
    let single_many_sim = cosine_similarity(&single_emb, &many_emb);
    
    assert!(single_double_sim < 0.98,
        "Single vs double repetition too similar: {:.4}. Attention may not handle repetition.",
        single_double_sim);
    
    assert!(single_many_sim < 0.95,
        "Single vs many repetitions too similar: {:.4}. Attention may not handle repetition.",
        single_many_sim);
    
    // Context with repetitions should be different from pure repetition
    let many_context_sim = cosine_similarity(&many_emb, &context_emb);
    assert!(many_context_sim < 0.90,
        "Pure repetition vs contextual repetition too similar: {:.4}. Attention not contextual.",
        many_context_sim);
    
    // Verify no degenerate behavior with repetitions
    for (name, emb) in [("single", &single_emb), ("double", &double_emb), 
                       ("many", &many_emb), ("context", &context_emb)] {
        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01,
            "Poor normalization for {}: {:.6}", name, norm);
        
        assert!(!emb.iter().any(|&x| x.is_nan()),
            "NaN values in {} embedding", name);
    }
    
    println!("✓ Attention repeated token handling test passed");
    println!("  - Single-Double similarity: {:.4}", single_double_sim);
    println!("  - Single-Many similarity: {:.4}", single_many_sim);
    println!("  - Many-Context similarity: {:.4}", many_context_sim);
}

/// Comprehensive attention mask effectiveness validation
/// Tests multiple scenarios to ensure masks are actually being used
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_comprehensive_attention_mask_validation() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    // Create test cases that should definitely produce different attention patterns
    let test_scenarios = vec![
        ("prefix_only", "class User"),
        ("prefix_extended", "class User extends"),
        ("full_minimal", "class User extends BaseUser"),
        ("full_extended", "class User extends BaseUser { constructor() { super(); } }"),
        ("different_start", "function createUser() { return new User(); }"),
    ];
    
    let mut scenario_embeddings = HashMap::new();
    let mut scenario_stats = HashMap::new();
    
    for (scenario, text) in &test_scenarios {
        let emb = embedder.embed(text).unwrap();
        
        // Calculate embedding statistics
        let mean = emb.iter().sum::<f32>() / emb.len() as f32;
        let max_val = emb.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let min_val = emb.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let range = max_val - min_val;
        
        scenario_embeddings.insert(*scenario, emb);
        scenario_stats.insert(*scenario, (mean, range, max_val, min_val));
        
        println!("  - {}: mean={:.4}, range={:.4}", scenario, mean, range);
    }
    
    // Cross-validation: related scenarios should be more similar than unrelated
    let prefix_only = scenario_embeddings.get("prefix_only").unwrap();
    let prefix_extended = scenario_embeddings.get("prefix_extended").unwrap();
    let different_start = scenario_embeddings.get("different_start").unwrap();
    
    let related_sim = cosine_similarity(prefix_only, prefix_extended);
    let unrelated_sim = cosine_similarity(prefix_only, different_start);
    
    // CRITICAL TEST: Attention should create meaningful relationships
    assert!(related_sim > unrelated_sim + 0.05,
        "Related texts should be more similar than unrelated texts. \
         Related: {:.4}, Unrelated: {:.4}. Attention may not be working correctly.",
        related_sim, unrelated_sim);
    
    // CRITICAL TEST: All embeddings should be valid and different
    let scenarios: Vec<&str> = test_scenarios.iter().map(|(name, _)| *name).collect();
    for i in 0..scenarios.len() {
        for j in i+1..scenarios.len() {
            let emb_i = scenario_embeddings.get(scenarios[i]).unwrap();
            let emb_j = scenario_embeddings.get(scenarios[j]).unwrap();
            let sim = cosine_similarity(emb_i, emb_j);
            
            // Should not be identical (unless they're very similar inputs)
            assert!(sim < 0.999,
                "Scenarios '{}' and '{}' produced nearly identical embeddings: {:.6}. \
                 This suggests broken attention or quantization.",
                scenarios[i], scenarios[j], sim);
        }
    }
    
    println!("✓ Comprehensive attention mask validation passed");
    println!("  - Related similarity: {:.4}", related_sim);
    println!("  - Unrelated similarity: {:.4}", unrelated_sim);
}

/// Test the attention mask validation function directly
#[cfg(feature = "ml")]
#[test] 
fn test_attention_mask_validation_function() {
    use embed_search::embedding::NomicEmbedder;
    
    // Test comprehensive validation scenarios
    let test_cases = vec![
        // Valid cases
        (vec![1, 1, 1, 0, 0], 5, true, "standard padding"),
        (vec![1; 10], 10, true, "no padding"),
        (vec![1, 0, 1, 0, 1], 5, true, "sparse attention"),
        (vec![1], 1, true, "single token"),
        
        // Invalid cases  
        (vec![0, 0, 0], 3, false, "all zeros"),
        (vec![], 0, false, "empty mask"),
        (vec![1, 1, 1], 5, false, "dimension mismatch short"),
        (vec![1; 10], 5, false, "dimension mismatch long"),
    ];
    
    for (mask, expected_len, should_pass, description) in test_cases {
        let result = NomicEmbedder::validate_attention_mask(&mask, expected_len);
        
        if should_pass {
            assert!(result.is_ok(), 
                "Validation should pass for {}: {:?} with length {}", 
                description, mask, expected_len);
        } else {
            assert!(result.is_err(), 
                "Validation should fail for {}: {:?} with length {}", 
                description, mask, expected_len);
            
            let error_msg = result.unwrap_err().to_string();
            println!("  - Expected error for '{}': {}", description, error_msg);
        }
    }
    
    println!("✓ Attention mask validation function test passed");
}

#[cfg(not(feature = "ml"))]
mod no_ml_tests {
    #[test]
    fn test_attention_mask_effectiveness_requires_ml_feature() {
        println!("Attention mask effectiveness tests require 'ml' feature to be enabled");
        println!("Run with: cargo test --features ml");
    }
}