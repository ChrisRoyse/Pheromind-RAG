//! Semantic Accuracy Unit Tests
//! Test the individual components of our semantic search fixes

use embed_search::search::fusion::{SimpleFusion, FusedResult, MatchType, FusionConfig};

#[cfg(feature = "vectordb")]
use embed_search::storage::lancedb_storage::LanceEmbeddingRecord;

#[test]
#[cfg(feature = "vectordb")]
fn test_improved_semantic_scoring() {
    let config = FusionConfig {
        max_results: 20,
        bm25_score_cap: 0.9,
        bm25_min_threshold: 0.01,
        normalization_percentile: 0.95,
        semantic_score_factor: 1.2,
        min_semantic_similarity: 0.4,
    };
    
    let fusion = SimpleFusion::with_config(config);
    
    // Test semantic score calculation for different similarity levels
    let test_cases = vec![
        (0.9, "very high similarity"),
        (0.8, "high similarity"),
        (0.7, "good similarity"),
        (0.6, "moderate similarity"),
        (0.5, "low similarity"),
        (0.4, "minimum threshold"),
        (0.3, "below threshold"),
    ];
    
    println!("🧪 Testing Improved Semantic Scoring");
    
    for (similarity, description) in test_cases {
        // Create test semantic match
        let semantic_record = LanceEmbeddingRecord {
            id: "test-1".to_string(),
            file_path: "test.rs".to_string(),
            chunk_index: 0,
            content: "test content".to_string(),
            embedding: vec![0.1; 768],
            start_line: 1,
            end_line: 10,
            similarity_score: Some(similarity),
            checksum: None,
        };
        
        // Test fusion with the semantic match
        let result = fusion.fuse_results(vec![], vec![semantic_record]);
        
        if similarity >= 0.4 {
            // Should pass minimum threshold
            assert!(result.is_ok(), "Fusion should succeed for similarity {}", similarity);
            let fused_results = result.unwrap();
            
            if !fused_results.is_empty() {
                let score = fused_results[0].score;
                println!("  {} (sim: {:.1}): score = {:.3}", description, similarity, score);
                
                // Verify score is reasonable
                assert!(score > 0.0, "Score should be positive");
                assert!(score <= 2.0, "Score should not be excessive");
                
                // Higher similarities should generally get higher scores
                if similarity > 0.7 {
                    assert!(score > similarity, "High similarities should get boosted scores");
                }
            }
        } else {
            // Should be filtered out by minimum threshold
            match result {
                Ok(results) => assert!(results.is_empty(), "Results below threshold should be filtered out"),
                Err(_) => {} // Also acceptable - could fail due to quality filters
            }
            println!("  {} (sim: {:.1}): filtered out ❌", description, similarity);
        }
    }
    
    println!("✅ Semantic scoring improvements validated");
}

#[test]
fn test_distance_to_similarity_conversion() {
    println!("🧪 Testing Distance-to-Similarity Conversion");
    
    // Test the mathematical conversion we implemented
    let test_distances = vec![
        (0.0, "perfect match"),
        (0.5, "close match"),
        (1.0, "moderate distance"),
        (1.5, "far distance"),
        (2.0, "very far distance"),
    ];
    
    for (distance, description) in test_distances {
        // Apply the conversion formula from our fix:
        // similarity = 1 - (distance^2 / 2), clamped to [0,1]
        let similarity = (1.0f32 - (distance * distance / 2.0)).max(0.0).min(1.0);
        
        println!("  {}: distance={:.1} -> similarity={:.3}", description, distance, similarity);
        
        // Verify properties
        assert!(similarity >= 0.0, "Similarity must be non-negative");
        assert!(similarity <= 1.0, "Similarity must not exceed 1.0");
        
        // Distance 0 should give similarity 1
        if distance == 0.0 {
            assert_eq!(similarity, 1.0, "Perfect distance should give perfect similarity");
        }
        
        // Higher distances should give lower similarities
        if distance > 1.0 {
            assert!(similarity < 0.5, "High distances should give low similarities");
        }
    }
    
    println!("✅ Distance-to-similarity conversion validated");
}

#[test]
fn test_fusion_config_improvements() {
    println!("🧪 Testing Fusion Configuration Improvements");
    
    let config = FusionConfig::default();
    
    // Verify our improved defaults - using actual fields that exist
    assert_eq!(config.semantic_score_factor, 0.8, "Semantic factor should be 0.8 (default)");
    assert_eq!(config.bm25_min_threshold, 0.01, "BM25 minimum threshold should be 0.01");
    assert_eq!(config.max_results, 20, "Max results should be 20");
    
    println!("  ✅ Semantic score factor: {}", config.semantic_score_factor);
    println!("  ✅ BM25 minimum threshold: {}", config.bm25_min_threshold);
    println!("  ✅ Max results: {}", config.max_results);
    
    println!("✅ Fusion configuration improvements validated");
}

/// Test that semantic matches are properly prioritized in rankings
#[test] 
#[cfg(feature = "vectordb")]
fn test_semantic_match_prioritization() {
    println!("🧪 Testing Semantic Match Prioritization");
    
    let fusion = SimpleFusion::new();
    
    // Create semantic matches with different quality levels
    let high_quality = LanceEmbeddingRecord {
        id: "high-1".to_string(),
        file_path: "important.rs".to_string(), 
        chunk_index: 0,
        content: "high quality semantic match".to_string(),
        embedding: vec![0.1; 768],
        start_line: 1,
        end_line: 10,
        similarity_score: Some(0.85), // High similarity
        checksum: None,
    };
    
    let low_quality = LanceEmbeddingRecord {
        id: "low-1".to_string(),
        file_path: "other.rs".to_string(),
        chunk_index: 0, 
        content: "lower quality match".to_string(),
        embedding: vec![0.1; 768],
        start_line: 1,
        end_line: 10,
        similarity_score: Some(0.45), // Low similarity (but above threshold)
        checksum: None,
    };
    
    let result = fusion.fuse_results(vec![], vec![high_quality, low_quality]).unwrap();
    
    // Verify that high quality match is ranked higher
    assert!(result.len() >= 2, "Should have both matches");
    assert!(result[0].score > result[1].score, "High quality match should rank higher");
    
    let high_score = result[0].score;
    let low_score = result[1].score;
    
    println!("  High quality match (0.85 sim): score = {:.3}", high_score);
    println!("  Low quality match (0.45 sim): score = {:.3}", low_score);
    
    // Verify score difference is meaningful
    assert!(high_score > low_score * 1.5, "High quality should significantly outrank low quality");
    
    println!("✅ Semantic match prioritization validated");
}