#[cfg(test)]
mod bm25_idf_ranking_verification {
    use embed_search::search::bm25_fixed::BM25Engine;
    use anyhow::Result;

    #[test]
    fn test_idf_affects_search_ranking_order() -> Result<()> {
        let mut engine = BM25Engine::new()?;
        
        // Index documents with strategic term distribution
        engine.index_document("doc1", "function implements rare_algorithm optimization");
        engine.index_document("doc2", "function basic implementation");
        engine.index_document("doc3", "function unique_method processing");
        engine.index_document("doc4", "function function function common");

        // Test Query 1: "function rare_algorithm" 
        // Expected: doc1 should rank highest (has rare term "rare_algorithm")
        let results1 = engine.search("function rare_algorithm", 10)?;
        
        println!("\n=== Query: 'function rare_algorithm' ===");
        for (i, result) in results1.iter().enumerate() {
            println!("Rank {}: {} (Score: {:.6})", i+1, result.path, result.score);
        }
        
        assert!(results1.len() >= 1, "Should return at least one result");
        assert_eq!(results1[0].path, "doc1", 
                  "Document with rare term 'rare_algorithm' should rank highest. Got: {}", 
                  results1[0].path);

        // Test Query 2: "function unique_method"
        // Expected: doc3 should rank highest (has rare term "unique_method")  
        let results2 = engine.search("function unique_method", 10)?;
        
        println!("\n=== Query: 'function unique_method' ===");
        for (i, result) in results2.iter().enumerate() {
            println!("Rank {}: {} (Score: {:.6})", i+1, result.path, result.score);
        }
        
        assert!(results2.len() >= 1, "Should return at least one result");
        assert_eq!(results2[0].path, "doc3",
                  "Document with rare term 'unique_method' should rank highest. Got: {}",
                  results2[0].path);

        // Test Query 3: "rare_algorithm unique_method" 
        // Expected: doc1 and doc3 should rank higher than doc2 and doc4
        let results3 = engine.search("rare_algorithm unique_method", 10)?;
        
        println!("\n=== Query: 'rare_algorithm unique_method' ===");
        for (i, result) in results3.iter().enumerate() {
            println!("Rank {}: {} (Score: {:.6})", i+1, result.path, result.score);
        }
        
        // Both doc1 and doc3 should appear before doc2 and doc4
        let top_results: Vec<&str> = results3.iter().take(2).map(|r| r.path.as_str()).collect();
        assert!(top_results.contains(&"doc1"), "doc1 should be in top 2 results");
        assert!(top_results.contains(&"doc3"), "doc3 should be in top 2 results");
        
        Ok(())
    }

    #[test]
    fn test_idf_boost_vs_term_frequency() -> Result<()> {
        let mut engine = BM25Engine::new()?;
        
        // Create documents where TF vs IDF impact can be measured
        engine.index_document("high_tf_common", "function function function function common");
        engine.index_document("low_tf_rare", "function specialized_algorithm");

        // Query for both terms
        let results = engine.search("function specialized_algorithm", 10)?;
        
        println!("\n=== IDF Boost vs TF Test ===");
        for (i, result) in results.iter().enumerate() {
            println!("Rank {}: {} (Score: {:.6})", i+1, result.path, result.score);
        }
        
        assert!(results.len() >= 2, "Should return both documents");
        
        // The document with rare term should score higher despite lower TF for "function"
        assert_eq!(results[0].path, "low_tf_rare",
                  "Document with rare term should rank higher despite lower term frequency. Got: {}",
                  results[0].path);
        
        Ok(())
    }

    #[test]
    fn test_idf_calculation_correctness() -> Result<()> {
        let mut engine = BM25Engine::new()?;
        
        // Verify IDF calculations are mathematically correct
        engine.index_document("doc1", "common common unique1");
        engine.index_document("doc2", "common common unique2");
        engine.index_document("doc3", "common rare_term");

        // Test single rare term query
        let rare_results = engine.search("rare_term", 10)?;
        
        println!("\n=== Single Rare Term Query ===");
        for (i, result) in rare_results.iter().enumerate() {
            println!("Rank {}: {} (Score: {:.6})", i+1, result.path, result.score);
        }
        
        assert_eq!(rare_results.len(), 1, "Only one document should match rare term");
        assert_eq!(rare_results[0].path, "doc3", "doc3 should be the only match for rare term");
        
        // Test common term query - should return all documents but with lower scores
        let common_results = engine.search("common", 10)?;
        
        println!("\n=== Common Term Query ===");
        for (i, result) in common_results.iter().enumerate() {
            println!("Rank {}: {} (Score: {:.6})", i+1, result.path, result.score);
        }
        
        assert_eq!(common_results.len(), 3, "All documents should match common term");
        
        // Rare term should have higher score than common term
        assert!(rare_results[0].score > common_results[0].score,
               "Rare term score ({:.6}) should be higher than common term score ({:.6})",
               rare_results[0].score, common_results[0].score);
        
        Ok(())
    }

    #[test]
    fn test_compound_query_idf_impact() -> Result<()> {
        let mut engine = BM25Engine::new()?;
        
        // Test how IDF affects ranking in multi-term queries
        engine.index_document("doc1", "programming language rust specialized");
        engine.index_document("doc2", "programming language python common");
        engine.index_document("doc3", "programming language java standard");

        // Query with common + rare terms
        let results = engine.search("programming specialized", 10)?;
        
        println!("\n=== Compound Query: 'programming specialized' ===");
        for (i, result) in results.iter().enumerate() {
            println!("Rank {}: {} (Score: {:.6})", i+1, result.path, result.score);
        }
        
        assert!(results.len() >= 1, "Should return at least one result");
        assert_eq!(results[0].path, "doc1",
                  "Document with rare term 'specialized' should rank highest. Got: {}",
                  results[0].path);
        
        // Verify that documents with only common terms score lower
        if results.len() > 1 {
            let top_score = results[0].score;
            let second_score = results[1].score;
            assert!(top_score > second_score,
                   "Document with rare term should score higher ({:.6}) than documents with only common terms ({:.6})",
                   top_score, second_score);
        }
        
        Ok(())
    }

    #[test]
    fn test_idf_mathematical_properties() -> Result<()> {
        let mut engine = BM25Engine::new()?;
        
        // Verify IDF mathematical properties hold in search results
        // Use same document length to isolate IDF effect
        engine.index_document("single", "ultra_rare_term extra_word");
        engine.index_document("multi1", "moderately_rare_term extra_word");
        engine.index_document("multi2", "moderately_rare_term extra_word");
        engine.index_document("multi3", "moderately_rare_term extra_word");

        // Search for ultra rare term (appears in 1/4 documents)
        let ultra_rare_results = engine.search("ultra_rare_term", 1)?;
        
        // Search for moderately rare term (appears in 3/4 documents) 
        let mod_rare_results = engine.search("moderately_rare_term", 3)?;
        
        println!("\n=== IDF Mathematical Properties ===");
        println!("Ultra rare term score: {:.6}", ultra_rare_results[0].score);
        println!("Moderately rare term score: {:.6}", mod_rare_results[0].score);
        
        // Ultra rare term should have higher score due to higher IDF
        assert!(ultra_rare_results[0].score > mod_rare_results[0].score,
               "Ultra rare term (1/4 docs) should score higher ({:.6}) than moderately rare term (3/4 docs) ({:.6})",
               ultra_rare_results[0].score, mod_rare_results[0].score);
        
        // Verify that IDF difference is significant enough to distinguish rarity
        let score_diff = ultra_rare_results[0].score - mod_rare_results[0].score;
        assert!(score_diff > 0.5,
               "IDF difference should be significant. Ultra rare: {:.6}, Moderately rare: {:.6}, Diff: {:.6}",
               ultra_rare_results[0].score, mod_rare_results[0].score, score_diff);
        
        Ok(())
    }
}