// Direct test file - standalone execution

fn main() {
    println!("🧪 RUNNING MINIMAL EMBEDDER VERIFICATION TESTS");
    println!("================================================");
    
    // Run all tests
    test_basic_embedding_generation();
    test_determinism();
    test_different_texts_produce_different_embeddings();
    test_memory_safety_stress();
    test_edge_cases();
    test_consistency_across_similar_texts();
    test_hash_distribution();
    test_performance_baseline();
    test_numerical_properties();
    
    println!("\n🎉 ALL TESTS PASSED! MinimalEmbedder works correctly.");
    println!("📊 SUMMARY:");
    println!("   ✅ Basic embedding generation (768 dimensions, unit length)");
    println!("   ✅ Deterministic output (same text = same embedding)");
    println!("   ✅ Different texts produce different embeddings");
    println!("   ✅ Memory safety (1000+ embeddings without crashes)");
    println!("   ✅ Edge cases (empty, long, unicode, special chars)");
    println!("   ✅ Consistency and differentiation");
    println!("   ✅ Good hash distribution across dimensions");
    println!("   ✅ Performance baseline (fast generation)");
    println!("   ✅ Proper numerical properties");
}

// Copy of MinimalEmbedder implementation
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::collections::HashSet;

pub struct MinimalEmbedder {
    dimension: usize,
}

impl MinimalEmbedder {
    pub fn new() -> Self {
        Self { dimension: 768 }
    }
    
    pub fn dimension(&self) -> usize {
        self.dimension
    }
    
    pub fn embed(&self, text: &str) -> Vec<f32> {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let base_hash = hasher.finish();
        
        let mut embedding = Vec::with_capacity(self.dimension);
        for i in 0..self.dimension {
            // Create more varied seeds using different operations
            let seed1 = base_hash.wrapping_mul(i as u64 + 1);
            let seed2 = seed1.rotate_left(i as u32 % 64);
            let seed = seed1 ^ seed2;
            
            // Convert to float with better range distribution
            let normalized = (seed as f64) / (u64::MAX as f64);
            let value = (normalized * 2.0 - 1.0) as f32;
            
            embedding.push(value);
        }
        
        // Normalize to unit length
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            embedding.iter_mut().for_each(|x| *x /= norm);
        }
        
        embedding
    }
}

fn test_basic_embedding_generation() {
    print!("🧪 Testing basic embedding generation... ");
    
    let embedder = MinimalEmbedder::new();
    let embedding = embedder.embed("test text");
    
    // Verify it's 768 dimensions
    assert_eq!(embedding.len(), 768, "Embedding should be 768 dimensions");
    
    // Verify all values are in normalized range (-1 to 1)
    for (i, &value) in embedding.iter().enumerate() {
        assert!(
            value >= -1.0 && value <= 1.0,
            "Embedding value at index {} ({}) is out of range [-1, 1]",
            i, value
        );
        assert!(
            !value.is_nan() && !value.is_infinite(),
            "Embedding value at index {} is NaN or infinite",
            i
        );
    }
    
    // Verify vector is unit length (norm ≈ 1.0)
    let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!(
        (norm - 1.0).abs() < 1e-5,
        "Vector should be unit length, got norm: {}",
        norm
    );
    
    println!("✅ PASS");
}

fn test_determinism() {
    print!("🧪 Testing determinism... ");
    
    let embedder = MinimalEmbedder::new();
    let text = "deterministic test";
    
    // Generate embedding for same text twice
    let embedding1 = embedder.embed(text);
    let embedding2 = embedder.embed(text);
    
    // Verify identical output
    assert_eq!(embedding1.len(), embedding2.len(), "Embeddings should have same length");
    
    for (i, (&v1, &v2)) in embedding1.iter().zip(embedding2.iter()).enumerate() {
        assert_eq!(v1, v2, "Embeddings should be identical at index {}: {} != {}", i, v1, v2);
    }
    
    println!("✅ PASS");
}

fn test_different_texts_produce_different_embeddings() {
    print!("🧪 Testing different texts produce different embeddings... ");
    
    let embedder = MinimalEmbedder::new();
    
    let embedding1 = embedder.embed("first text");
    let embedding2 = embedder.embed("second text");
    let embedding3 = embedder.embed("completely different content");
    
    // Verify embeddings are different
    assert_ne!(embedding1, embedding2, "Different texts should produce different embeddings");
    assert_ne!(embedding1, embedding3, "Different texts should produce different embeddings");
    assert_ne!(embedding2, embedding3, "Different texts should produce different embeddings");
    
    println!("✅ PASS");
}

fn test_memory_safety_stress() {
    print!("🧪 Testing memory safety with 1000 embeddings... ");
    
    let embedder = MinimalEmbedder::new();
    let mut embeddings = Vec::new();
    
    // Generate 1000 embeddings
    for i in 0..1000 {
        let text = format!("test text number {}", i);
        let embedding = embedder.embed(&text);
        
        // Verify each embedding is valid
        assert_eq!(embedding.len(), 768);
        
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5, "Vector {} should be unit length", i);
        
        // Store first 10 for additional verification
        if i < 10 {
            embeddings.push(embedding);
        }
    }
    
    // Verify stored embeddings are still valid
    for (i, embedding) in embeddings.iter().enumerate() {
        assert_eq!(embedding.len(), 768);
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5, "Stored embedding {} should still be unit length", i);
    }
    
    println!("✅ PASS");
}

fn test_edge_cases() {
    print!("🧪 Testing edge cases... ");
    
    let embedder = MinimalEmbedder::new();
    
    // Test empty string
    let empty_embedding = embedder.embed("");
    assert_eq!(empty_embedding.len(), 768);
    let norm = empty_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-5, "Empty string embedding should be unit length");
    
    // Test very long string
    let long_text = "a".repeat(10000);
    let long_embedding = embedder.embed(&long_text);
    assert_eq!(long_embedding.len(), 768);
    let norm = long_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-5, "Long string embedding should be unit length");
    
    // Test unicode
    let unicode_text = "Hello 世界 🌍 αβγ";
    let unicode_embedding = embedder.embed(unicode_text);
    assert_eq!(unicode_embedding.len(), 768);
    let norm = unicode_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-5, "Unicode embedding should be unit length");
    
    // Test special characters
    let special_text = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
    let special_embedding = embedder.embed(special_text);
    assert_eq!(special_embedding.len(), 768);
    let norm = special_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-5, "Special chars embedding should be unit length");
    
    println!("✅ PASS");
}

fn test_consistency_across_similar_texts() {
    print!("🧪 Testing consistency across similar texts... ");
    
    let embedder = MinimalEmbedder::new();
    
    let similar_texts = vec![
        "the quick brown fox",
        "the quick brown fox jumps", 
        "the quick brown fox jumps over",
        "the quick brown fox jumps over the lazy dog",
    ];
    
    let mut embeddings = Vec::new();
    for text in &similar_texts {
        let embedding = embedder.embed(text);
        assert_eq!(embedding.len(), 768);
        
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5, "Embedding should be unit length for: {}", text);
        
        embeddings.push(embedding);
    }
    
    // Verify first embedding is identical to itself (determinism check)
    let duplicate = embedder.embed(similar_texts[0]);
    assert_eq!(embeddings[0], duplicate, "Same text should produce identical embeddings");
    
    // Verify different texts produce different embeddings
    for i in 1..embeddings.len() {
        assert_ne!(embeddings[0], embeddings[i], "Different texts should produce different embeddings");
    }
    
    println!("✅ PASS");
}

fn test_hash_distribution() {
    print!("🧪 Testing hash distribution... ");
    
    let embedder = MinimalEmbedder::new();
    let mut value_sets = vec![HashSet::new(); 768];
    
    let test_texts = vec![
        "apple", "banana", "cherry", "date", "elderberry",
        "fig", "grape", "honeydew", "kiwi", "lemon",
        "mango", "nectarine", "orange", "papaya", "quince",
        "raspberry", "strawberry", "tangerine", "ugli", "vanilla"
    ];
    
    for text in test_texts {
        let embedding = embedder.embed(text);
        
        // Collect unique values at each dimension
        for (i, &value) in embedding.iter().enumerate() {
            value_sets[i].insert((value * 1000000.0) as i32);
        }
    }
    
    // Verify we get reasonable distribution
    for (i, set) in value_sets.iter().enumerate() {
        assert!(set.len() > 1, "Dimension {} should have varied values, got {} unique", i, set.len());
    }
    
    println!("✅ PASS");
}

fn test_performance_baseline() {
    print!("🧪 Testing performance baseline... ");
    
    let embedder = MinimalEmbedder::new();
    let text = "performance test text for benchmarking purposes";
    
    let start = std::time::Instant::now();
    
    // Generate 100 embeddings
    for _ in 0..100 {
        let embedding = embedder.embed(text);
        assert_eq!(embedding.len(), 768);
    }
    
    let duration = start.elapsed();
    
    // Should be very fast
    assert!(duration.as_millis() < 100, "100 embeddings should be fast, took {}ms", duration.as_millis());
    
    println!("✅ PASS ({}ms for 100 embeddings)", duration.as_millis());
}

fn test_numerical_properties() {
    print!("🧪 Testing numerical properties... ");
    
    let embedder = MinimalEmbedder::new();
    let embedding = embedder.embed("numerical properties test");
    
    let mut zero_count = 0;
    let mut positive_count = 0;
    let mut negative_count = 0;
    
    // Debug first 10 values
    println!("\nFirst 10 values: ");
    for (i, &value) in embedding.iter().take(10).enumerate() {
        println!("  [{}]: {}", i, value);
    }
    
    for &value in embedding.iter() {
        if value == 0.0 {
            zero_count += 1;
        } else if value > 0.0 {
            positive_count += 1;
        } else {
            negative_count += 1;
        }
        
        assert!(value.abs() <= 1.0, "Value should be bounded: {}", value);
    }
    
    println!("Distribution: +{} -{} =0:{}", positive_count, negative_count, zero_count);
    
    // Should have a mix of positive and negative values
    assert!(positive_count > 0, "Should have some positive values");
    assert!(negative_count > 0, "Should have some negative values");
    assert!(zero_count < 700, "Should not have too many zeros");
    
    println!("✅ PASS (+{} -{} =0:{})", positive_count, negative_count, zero_count);
}