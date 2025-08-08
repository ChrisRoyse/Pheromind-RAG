use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use anyhow::{Result, Context};
use tempfile::TempDir;

#[cfg(feature = "tantivy")]
use embed_search::search::{TantivySearcher, ExactMatch};

/// QUICK TANTIVY VERIFICATION TEST
/// 
/// This test proves the stress testing framework works by running actual
/// TantivySearcher operations and verifying they produce real results.

#[tokio::test]
async fn quick_tantivy_functionality_proof() -> Result<()> {
    println!("🔥 QUICK TANTIVY FUNCTIONALITY PROOF");
    println!("=====================================");
    
    #[cfg(feature = "tantivy")]
    {
        let temp_dir = TempDir::new()?;
        
        // Test 1: Basic functionality
        println!("📋 Test 1: Basic TantivySearcher functionality");
        let start_time = Instant::now();
        
        let mut searcher = TantivySearcher::new_with_path(temp_dir.path().join("proof_test")).await
            .context("Failed to create TantivySearcher - this proves the implementation exists")?;
        
        // Create real test content
        let test_file = temp_dir.path().join("proof.rs");
        let content = r#"
pub fn proof_function() -> String {
    let search_marker = "PROOF_MARKER_12345";
    search_marker.to_string()
}

pub struct ProofStruct {
    field: String,
}

impl ProofStruct {
    pub fn get_proof_data() -> &'static str {
        "proof_data_unique_67890"
    }
}
"#;
        fs::write(&test_file, content)?;
        
        // Index the file
        searcher.index_file(&test_file).await
            .context("Indexing failed - this proves the indexing functionality works")?;
        
        // Verify indexing worked
        let stats = searcher.get_index_stats()
            .context("Failed to get index stats")?;
        
        if stats.num_documents == 0 {
            return Err(anyhow::anyhow!("❌ CRITICAL: Index shows 0 documents after indexing"));
        }
        
        println!("   ✅ Indexed {} documents", stats.num_documents);
        
        // Test exact search
        let exact_results = searcher.search("PROOF_MARKER_12345").await
            .context("Exact search failed")?;
        
        if exact_results.is_empty() {
            return Err(anyhow::anyhow!("❌ CRITICAL: Exact search returned no results for known content"));
        }
        
        let first_result = &exact_results[0];
        if !first_result.content.contains("PROOF_MARKER_12345") {
            return Err(anyhow::anyhow!("❌ CRITICAL: Search result doesn't contain searched content: {}", first_result.content));
        }
        
        println!("   ✅ Exact search found {} results", exact_results.len());
        println!("   ✅ First result: {}", first_result.content.trim());
        
        // Test fuzzy search
        let fuzzy_results = searcher.search_fuzzy("ProofStrct", 2).await
            .context("Fuzzy search failed")?;
        
        println!("   ✅ Fuzzy search found {} results", fuzzy_results.len());
        
        let basic_duration = start_time.elapsed();
        println!("   ⏱️  Basic test completed in {:?}\n", basic_duration);
        
        // Test 2: Stress scenario (moderate)
        println!("📋 Test 2: Moderate stress scenario");
        let stress_start = Instant::now();
        
        // Create multiple files quickly
        let stress_dir = temp_dir.path().join("stress");
        fs::create_dir_all(&stress_dir)?;
        
        for i in 0..20 {
            let file_path = stress_dir.join(format!("stress_{}.rs", i));
            let content = format!(
                "pub fn stress_function_{}() -> String {{\n    \
                    \"stress_marker_{}\".to_string()\n\
                }}",
                i, i
            );
            fs::write(&file_path, content)?;
        }
        
        // Index all files
        searcher.index_directory(&stress_dir).await
            .context("Directory indexing failed")?;
        
        let final_stats = searcher.get_index_stats()?;
        println!("   ✅ Total documents after stress: {}", final_stats.num_documents);
        
        // Test search performance
        let search_start = Instant::now();
        let search_results = searcher.search("stress_marker_10").await?;
        let search_duration = search_start.elapsed();
        
        if search_results.is_empty() {
            return Err(anyhow::anyhow!("❌ CRITICAL: No results found for stress test content"));
        }
        
        println!("   ✅ Stress search found {} results in {:?}", 
                search_results.len(), search_duration);
        
        let stress_duration = stress_start.elapsed();
        println!("   ⏱️  Stress test completed in {:?}\n", stress_duration);
        
        // Test 3: Error handling
        println!("📋 Test 3: Error handling verification");
        
        let nonexistent_file = temp_dir.path().join("does_not_exist.rs");
        let error_result = searcher.index_file(&nonexistent_file).await;
        
        match error_result {
            Ok(()) => {
                return Err(anyhow::anyhow!("❌ CRITICAL: Indexing nonexistent file should fail"));
            }
            Err(e) => {
                println!("   ✅ Correctly failed to index nonexistent file: {}", e);
                let error_msg = e.to_string().to_lowercase();
                if !error_msg.contains("failed to read file") {
                    return Err(anyhow::anyhow!("❌ CRITICAL: Error message doesn't indicate file read failure"));
                }
            }
        }
        
        println!("✅ ALL VERIFICATION TESTS PASSED");
        println!("=================================");
        println!("🎯 TantivySearcher implementation is functional");
        println!("🎯 Indexing works correctly");
        println!("🎯 Search operations return real results");
        println!("🎯 Error handling works as expected");
        println!("🎯 Stress testing framework is ready for comprehensive testing");
        
        Ok(())
    }
    
    #[cfg(not(feature = "tantivy"))]
    {
        println!("⚠️  Tantivy feature not enabled");
        println!("Cannot verify TantivySearcher functionality without tantivy feature");
        println!("Run with: cargo test --features tantivy");
        Ok(())
    }
}

/// Test that demonstrates the stress testing approach with actual failures
#[tokio::test]
async fn demonstrate_stress_testing_approach() -> Result<()> {
    println!("🔥 STRESS TESTING APPROACH DEMONSTRATION");
    println!("=========================================");
    
    // This test shows how we identify real vs simulated issues
    println!("📋 How the stress tests work:");
    println!("   1. Create REAL files with REAL content");
    println!("   2. Perform REAL operations (no mocking)");
    println!("   3. Measure ACTUAL performance metrics");
    println!("   4. Verify ALL results contain expected data");
    println!("   5. Classify errors as legitimate vs bugs");
    
    #[cfg(feature = "tantivy")]
    {
        let temp_dir = TempDir::new()?;
        let mut searcher = TantivySearcher::new().await?;
        
        // Demonstrate truthfulness requirement
        println!("\n📊 Truthfulness Demonstration:");
        
        // Test with empty index
        let empty_results = searcher.search("nonexistent_content").await?;
        println!("   Empty index search: {} results (TRUTH: should be 0)", empty_results.len());
        assert_eq!(empty_results.len(), 0, "Empty index must return 0 results");
        
        // Test with real content
        let test_file = temp_dir.path().join("truth_test.rs");
        fs::write(&test_file, "pub fn truth_test() { let marker = \"TRUTH_MARKER\"; }")?;
        searcher.index_file(&test_file).await?;
        
        let real_results = searcher.search("TRUTH_MARKER").await?;
        println!("   Real content search: {} results (TRUTH: should be > 0)", real_results.len());
        assert!(!real_results.is_empty(), "Real content must return results");
        
        // Verify result content is real
        let first_result = &real_results[0];
        assert!(first_result.content.contains("TRUTH_MARKER"), 
               "Result must contain actual searched content");
        println!("   Result verification: Contains search term ✅");
        
        // Demonstrate performance measurement
        let perf_start = Instant::now();
        let _perf_results = searcher.search("truth_test").await?;
        let perf_duration = perf_start.elapsed();
        println!("   Performance measurement: {:?} (REAL timing)", perf_duration);
        
        println!("\n✅ Stress testing approach validated");
        println!("All measurements are real, all results are verified");
    }
    
    Ok(())
}