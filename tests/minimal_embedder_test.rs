use embed_search::embedding::MinimalEmbedder;
use std::collections::HashSet;

#[test]
fn test_basic_embedding_generation() {
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
        (norm - 1.0).abs() < 1e-6,
        "Vector should be unit length, got norm: {}",
        norm
    );
}

#[test]
fn test_determinism() {
    let embedder = MinimalEmbedder::new();
    let text = "deterministic test";
    
    // Generate embedding for same text twice
    let embedding1 = embedder.embed(text);
    let embedding2 = embedder.embed(text);
    
    // Verify identical output
    assert_eq!(
        embedding1.len(),
        embedding2.len(),
        "Embeddings should have same length"
    );
    
    for (i, (&v1, &v2)) in embedding1.iter().zip(embedding2.iter()).enumerate() {
        assert_eq!(
            v1, v2,
            "Embeddings should be identical at index {}: {} != {}",
            i, v1, v2
        );
    }
}

#[test]
fn test_different_texts_produce_different_embeddings() {
    let embedder = MinimalEmbedder::new();
    
    let embedding1 = embedder.embed("first text");
    let embedding2 = embedder.embed("second text");
    let embedding3 = embedder.embed("completely different content");
    
    // Verify embeddings are different
    assert_ne!(embedding1, embedding2, "Different texts should produce different embeddings");
    assert_ne!(embedding1, embedding3, "Different texts should produce different embeddings");
    assert_ne!(embedding2, embedding3, "Different texts should produce different embeddings");
}

#[test]
fn test_memory_safety_stress() {
    let embedder = MinimalEmbedder::new();
    let mut embeddings = Vec::new();
    
    // Generate 1000 embeddings
    for i in 0..1000 {
        let text = format!("test text number {}", i);
        let embedding = embedder.embed(&text);
        
        // Verify each embedding is valid
        assert_eq!(embedding.len(), 768);
        
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6, "Vector {} should be unit length", i);
        
        // Store first 10 for additional verification
        if i < 10 {
            embeddings.push(embedding);
        }
    }
    
    // Verify stored embeddings are still valid
    for (i, embedding) in embeddings.iter().enumerate() {
        assert_eq!(embedding.len(), 768);
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-6,
            "Stored embedding {} should still be unit length",
            i
        );
    }
}

#[test]
fn test_edge_cases() {
    let embedder = MinimalEmbedder::new();
    
    // Test empty string
    let empty_embedding = embedder.embed("");
    assert_eq!(empty_embedding.len(), 768);
    let norm = empty_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-6, "Empty string embedding should be unit length");
    
    // Test very long string
    let long_text = "a".repeat(10000);
    let long_embedding = embedder.embed(&long_text);
    assert_eq!(long_embedding.len(), 768);
    let norm = long_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-6, "Long string embedding should be unit length");
    
    // Test unicode
    let unicode_text = "Hello 世界 🌍 αβγ";
    let unicode_embedding = embedder.embed(unicode_text);
    assert_eq!(unicode_embedding.len(), 768);
    let norm = unicode_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-6, "Unicode embedding should be unit length");
    
    // Test special characters
    let special_text = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
    let special_embedding = embedder.embed(special_text);
    assert_eq!(special_embedding.len(), 768);
    let norm = special_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-6, "Special chars embedding should be unit length");
}

#[test]
fn test_consistency_across_similar_texts() {
    let embedder = MinimalEmbedder::new();
    
    // Test that similar texts produce different but reasonable embeddings
    let base_text = "the quick brown fox";
    let similar_texts = vec![
        "the quick brown fox",
        "the quick brown fox jumps",
        "the quick brown fox jumps over",
        "the quick brown fox jumps over the lazy dog",
    ];
    
    let mut embeddings = Vec::new();
    for text in similar_texts {
        let embedding = embedder.embed(text);
        assert_eq!(embedding.len(), 768);
        
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6, "Embedding should be unit length for: {}", text);
        
        embeddings.push(embedding);
    }
    
    // Verify first embedding is identical to itself (determinism check)
    let duplicate = embedder.embed(similar_texts[0]);
    assert_eq!(embeddings[0], duplicate, "Same text should produce identical embeddings");
    
    // Verify different texts produce different embeddings
    for i in 1..embeddings.len() {
        assert_ne!(
            embeddings[0], embeddings[i],
            "Different texts should produce different embeddings"
        );
    }
}

#[test]
fn test_hash_distribution() {
    let embedder = MinimalEmbedder::new();
    let mut value_sets = vec![HashSet::new(); 768];
    
    // Generate embeddings for various texts
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
            value_sets[i].insert((value * 1000000.0) as i32); // Convert to int for deduplication
        }
    }
    
    // Verify we get reasonable distribution (not all same values)
    for (i, set) in value_sets.iter().enumerate() {
        assert!(
            set.len() > 1,
            "Dimension {} should have varied values across different texts, got {} unique values",
            i, set.len()
        );
    }
}

#[test]
fn test_performance_baseline() {
    let embedder = MinimalEmbedder::new();
    let text = "performance test text for benchmarking purposes";
    
    let start = std::time::Instant::now();
    
    // Generate 100 embeddings
    for _ in 0..100 {
        let embedding = embedder.embed(text);
        assert_eq!(embedding.len(), 768);
    }
    
    let duration = start.elapsed();
    
    // Should be very fast - less than 100ms for 100 embeddings
    assert!(
        duration.as_millis() < 100,
        "100 embeddings should complete in under 100ms, took {}ms",
        duration.as_millis()
    );
    
    println!("Performance: 100 embeddings in {}ms", duration.as_millis());
}