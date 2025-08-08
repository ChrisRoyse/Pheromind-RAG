//! Reference Embeddings Test Fixtures
//!
//! This module contains reference embeddings for validation testing.
//! These embeddings represent known-good outputs that can be used to:
//! 1. Detect regressions in embedding generation
//! 2. Validate mathematical correctness of the transformer
//! 3. Ensure consistency across different model versions

use std::collections::HashMap;

/// Reference embedding test case
#[derive(Debug, Clone)]
pub struct ReferenceEmbedding {
    pub text: &'static str,
    pub embedding: Vec<f32>,
    pub description: &'static str,
    pub category: &'static str,
    pub expected_similarity_threshold: f32,
}

/// Critical mathematical validation vectors
/// These are synthetic vectors designed to test specific mathematical properties
pub fn get_mathematical_validation_vectors() -> Vec<(&'static str, Vec<f32>)> {
    vec![
        (
            "zero_vector",
            vec![0.0; 768]
        ),
        (
            "unit_vector_x",
            {
                let mut v = vec![0.0; 768];
                v[0] = 1.0;
                v
            }
        ),
        (
            "unit_vector_y", 
            {
                let mut v = vec![0.0; 768];
                v[1] = 1.0;
                v
            }
        ),
        (
            "normalized_random",
            {
                // Generate a normalized random-like vector for testing
                let values: Vec<f32> = (0..768).map(|i| {
                    let x = (i as f32 * 0.1).sin() * 0.5 + (i as f32 * 0.07).cos() * 0.3;
                    x
                }).collect();
                
                // Normalize to unit length
                let norm = values.iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm > 0.0 {
                    values.into_iter().map(|x| x / norm).collect()
                } else {
                    vec![0.0; 768]
                }
            }
        ),
        (
            "alternating_pattern",
            {
                let mut v = vec![0.0; 768];
                for i in 0..768 {
                    v[i] = if i % 2 == 0 { 0.5 } else { -0.5 };
                }
                // Normalize
                let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
                v.into_iter().map(|x| x / norm).collect()
            }
        ),
    ]
}

/// Get expected semantic similarity patterns
/// These define the relationships that should exist between different code types
pub fn get_semantic_similarity_patterns() -> HashMap<&'static str, Vec<(&'static str, f32, f32)>> {
    let mut patterns = HashMap::new();
    
    // Programming language similarities
    patterns.insert("programming_languages", vec![
        ("python_function", 0.70, 0.90), // Should be similar to other functions
        ("javascript_function", 0.70, 0.90),
        ("rust_function", 0.65, 0.85),
        ("java_method", 0.65, 0.85),
    ]);
    
    // Database query similarities
    patterns.insert("database_queries", vec![
        ("sql_select", 0.75, 0.95),
        ("sql_insert", 0.70, 0.90),
        ("sql_update", 0.70, 0.90),
        ("mongodb_query", 0.60, 0.85),
    ]);
    
    // Web technology similarities
    patterns.insert("web_technologies", vec![
        ("html_template", 0.60, 0.85),
        ("css_styles", 0.55, 0.80),
        ("json_data", 0.65, 0.85),
        ("yaml_config", 0.60, 0.80),
    ]);
    
    patterns
}

/// Regression test embeddings - these should remain stable across versions
pub fn get_regression_test_embeddings() -> Vec<ReferenceEmbedding> {
    vec![
        ReferenceEmbedding {
            text: "def hello_world(): return 'Hello, World!'",
            embedding: generate_stable_test_embedding("python_hello_world"),
            description: "Simple Python function",
            category: "python",
            expected_similarity_threshold: 0.95,
        },
        
        ReferenceEmbedding {
            text: "function helloWorld() { return 'Hello, World!'; }",
            embedding: generate_stable_test_embedding("js_hello_world"),
            description: "Simple JavaScript function",
            category: "javascript", 
            expected_similarity_threshold: 0.95,
        },
        
        ReferenceEmbedding {
            text: "SELECT * FROM users WHERE active = true",
            embedding: generate_stable_test_embedding("sql_select_users"),
            description: "Basic SQL SELECT query",
            category: "sql",
            expected_similarity_threshold: 0.95,
        },
        
        ReferenceEmbedding {
            text: "class User { constructor(name) { this.name = name; } }",
            embedding: generate_stable_test_embedding("js_class_user"),
            description: "Simple JavaScript class",
            category: "javascript",
            expected_similarity_threshold: 0.95,
        },
        
        ReferenceEmbedding {
            text: "const config = { host: 'localhost', port: 3000 };",
            embedding: generate_stable_test_embedding("js_config_object"),
            description: "JavaScript configuration object",
            category: "javascript",
            expected_similarity_threshold: 0.95,
        },
    ]
}

/// Generate a stable, deterministic embedding for testing purposes
/// This creates consistent embeddings based on the input text for regression testing
fn generate_stable_test_embedding(seed: &str) -> Vec<f32> {
    // Create a deterministic "embedding" based on the seed
    // This is NOT a real semantic embedding, but provides consistent test data
    let mut embedding = Vec::with_capacity(768);
    
    for i in 0..768 {
        // Generate deterministic values based on seed and position
        let mut hash_input = 0u64;
        for (j, byte) in seed.bytes().enumerate() {
            hash_input = hash_input.wrapping_mul(31).wrapping_add(byte as u64).wrapping_add(i as u64 * 17);
        }
        
        // Convert to float in range [-1, 1]
        let raw_value = (hash_input as f32) / (u64::MAX as f32) * 2.0 - 1.0;
        let scaled_value = raw_value * 0.3; // Scale down to reasonable range
        
        embedding.push(scaled_value);
    }
    
    // Normalize to unit length
    let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        embedding.iter_mut().for_each(|x| *x /= norm);
    }
    
    embedding
}

/// Quality thresholds for embedding validation
pub struct EmbeddingQualityThresholds {
    pub min_norm: f32,
    pub max_norm: f32,
    pub max_abs_value: f32,
    pub min_unique_values: usize,
    pub max_zero_ratio: f32,
}

impl Default for EmbeddingQualityThresholds {
    fn default() -> Self {
        Self {
            min_norm: 0.99,
            max_norm: 1.01,
            max_abs_value: 2.0,
            min_unique_values: 500, // At least 500 unique values in 768 dimensions
            max_zero_ratio: 0.1,    // At most 10% zero values
        }
    }
}

/// Validate embedding quality against thresholds
pub fn validate_embedding_quality(embedding: &[f32], thresholds: &EmbeddingQualityThresholds) -> Result<(), String> {
    if embedding.len() != 768 {
        return Err(format!("Wrong embedding dimension: {} (expected 768)", embedding.len()));
    }
    
    // Check normalization
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm < thresholds.min_norm || norm > thresholds.max_norm {
        return Err(format!("Poor normalization: {:.6} (expected {:.2}-{:.2})", 
                          norm, thresholds.min_norm, thresholds.max_norm));
    }
    
    // Check for NaN/Inf
    if embedding.iter().any(|&x| x.is_nan()) {
        return Err("Embedding contains NaN values".to_string());
    }
    if embedding.iter().any(|&x| x.is_infinite()) {
        return Err("Embedding contains infinite values".to_string());
    }
    
    // Check value range
    let max_abs = embedding.iter().map(|&x| x.abs()).fold(0.0f32, |a, b| a.max(b));
    if max_abs > thresholds.max_abs_value {
        return Err(format!("Value out of range: max_abs = {:.6} (expected <= {:.2})", 
                          max_abs, thresholds.max_abs_value));
    }
    
    // Check uniqueness (avoid degenerate embeddings)
    let mut unique_values = std::collections::HashSet::new();
    for &val in embedding {
        unique_values.insert((val * 10000.0) as i32); // Discretize for uniqueness check
    }
    
    if unique_values.len() < thresholds.min_unique_values {
        return Err(format!("Too few unique values: {} (expected >= {})", 
                          unique_values.len(), thresholds.min_unique_values));
    }
    
    // Check zero ratio
    let zero_count = embedding.iter().filter(|&&x| x.abs() < 1e-6).count();
    let zero_ratio = zero_count as f32 / embedding.len() as f32;
    if zero_ratio > thresholds.max_zero_ratio {
        return Err(format!("Too many near-zero values: {:.1}% (expected <= {:.1}%)", 
                          zero_ratio * 100.0, thresholds.max_zero_ratio * 100.0));
    }
    
    Ok(())
}

/// Test cases for attention mechanism validation
pub fn get_attention_validation_cases() -> Vec<(&'static str, &'static str, f32, f32)> {
    vec![
        // Cases where attention should create similar results
        ("def add(a, b): return a + b", "function add(a, b) { return a + b; }", 0.70, 0.95),
        ("user.authenticate()", "user.login()", 0.60, 0.90),
        ("SELECT name FROM users", "SELECT username FROM accounts", 0.65, 0.90),
        
        // Cases where attention should create different results  
        ("def add(a, b): return a + b", "DELETE FROM users", 0.0, 0.40),
        ("class User", "network timeout error", 0.0, 0.35),
        ("function calculate()", "<!DOCTYPE html>", 0.0, 0.40),
        
        // Position-sensitive cases (same tokens, different order)
        ("def authenticate user", "user authenticate def", 0.40, 0.80),
        ("SELECT FROM WHERE", "WHERE FROM SELECT", 0.30, 0.70),
        ("class method object", "object class method", 0.35, 0.75),
    ]
}

/// Quantization validation test vectors
/// These test specific quantization scenarios
pub fn get_quantization_test_vectors() -> Vec<(&'static str, Vec<u8>, Vec<f32>)> {
    vec![
        (
            "q4_test_block",
            generate_q4_test_block(),
            vec![0.0, 0.125, -0.125, 0.25, -0.25, 0.375, -0.375, 0.5], // Expected dequantized values
        ),
        (
            "q6k_test_block", 
            generate_q6k_test_block(),
            // Expected values would be computed from correct Q6K dequantization
            // The broken implementation will produce different values
            vec![0.0; 256], // Placeholder - real implementation would have specific expected values
        ),
    ]
}

fn generate_q4_test_block() -> Vec<u8> {
    let mut block = vec![0u8; 18]; // Q4 block size
    
    // Set scale (f16 format for 1.0)
    block[0] = 0x00;
    block[1] = 0x3C; // 1.0 in f16
    
    // Set quantized values (4-bit pairs)
    for i in 0..16 {
        block[2 + i] = (i as u8) | ((i as u8) << 4);
    }
    
    block
}

fn generate_q6k_test_block() -> Vec<u8> {
    let mut block = vec![0u8; 210]; // Q6K block size
    
    // Set main scale (f16 format for 1.0)
    block[0] = 0x00;
    block[1] = 0x3C; // 1.0 in f16
    
    // Set scales array (12 bytes)
    for i in 0..12 {
        block[2 + i] = (i * 8) as u8; // Test pattern
    }
    
    // Set quantized values (remaining bytes)
    for i in 14..210 {
        block[i] = ((i - 14) % 64) as u8; // 6-bit pattern
    }
    
    block
}

/// Performance benchmarks for different input types
pub fn get_performance_benchmarks() -> Vec<(&'static str, String, u64)> {
    vec![
        ("short", "test".to_string(), 50), // Should process in ~50ms
        ("medium", "function test(param) { return param * 2; }".to_string(), 100),
        ("long", "class ComplexClass { constructor(args) { this.data = args; this.cache = new Map(); } process() { return this.data.map(x => x * 2); } }".repeat(3), 300),
        ("very_long", "x".repeat(1000), 500),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mathematical_validation_vectors() {
        let vectors = get_mathematical_validation_vectors();
        
        for (name, vector) in vectors {
            assert_eq!(vector.len(), 768, "Vector {} should have 768 dimensions", name);
            
            if name != "zero_vector" {
                let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
                assert!((norm - 1.0).abs() < 0.01, "Vector {} should be normalized: norm = {}", name, norm);
            }
        }
    }
    
    #[test]
    fn test_regression_embeddings_structure() {
        let embeddings = get_regression_test_embeddings();
        
        for embedding in embeddings {
            assert_eq!(embedding.embedding.len(), 768, 
                "Embedding for '{}' should have 768 dimensions", embedding.text);
            
            let thresholds = EmbeddingQualityThresholds::default();
            assert!(validate_embedding_quality(&embedding.embedding, &thresholds).is_ok(),
                "Embedding for '{}' should pass quality validation", embedding.text);
        }
    }
    
    #[test]
    fn test_quality_validation_function() {
        let thresholds = EmbeddingQualityThresholds::default();
        
        // Test with a good embedding
        let good_embedding = generate_stable_test_embedding("test_good");
        assert!(validate_embedding_quality(&good_embedding, &thresholds).is_ok());
        
        // Test with wrong dimensions
        let wrong_dim = vec![0.0; 100];
        assert!(validate_embedding_quality(&wrong_dim, &thresholds).is_err());
        
        // Test with NaN values
        let mut nan_embedding = generate_stable_test_embedding("test_nan");
        nan_embedding[0] = f32::NAN;
        assert!(validate_embedding_quality(&nan_embedding, &thresholds).is_err());
        
        // Test with infinite values
        let mut inf_embedding = generate_stable_test_embedding("test_inf");
        inf_embedding[0] = f32::INFINITY;
        assert!(validate_embedding_quality(&inf_embedding, &thresholds).is_err());
    }
    
    #[test]
    fn test_quantization_test_vectors() {
        let vectors = get_quantization_test_vectors();
        
        for (name, data, expected) in vectors {
            match name {
                "q4_test_block" => {
                    assert_eq!(data.len(), 18, "Q4 test block should be 18 bytes");
                    assert!(expected.len() > 0, "Q4 expected values should not be empty");
                }
                "q6k_test_block" => {
                    assert_eq!(data.len(), 210, "Q6K test block should be 210 bytes");
                    assert_eq!(expected.len(), 256, "Q6K should produce 256 values");
                }
                _ => {}
            }
        }
    }
}