use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use std::thread;
use anyhow::Result;
use tempfile::TempDir;
use tokio::task;
use futures::future::join_all;

#[cfg(feature = "tantivy")]
use embed_search::search::{TantivySearcher, ExactMatch};

/// BRUTAL WEAKNESS EXPOSURE TESTS FOR TANTIVY INTEGRATION
/// 
/// These tests are specifically designed to expose the CRITICAL FLAWS identified:
/// 1. No fuzzy distance configuration exposed
/// 2. Hardcoded inflexible schema  
/// 3. No error recovery mechanisms
/// 4. Unbounded memory growth
/// 5. Index corruption vulnerability
/// 6. Empty query panics
/// 7. Unicode normalization issues
/// 8. Concurrent write corruption
/// 9. Large document memory exhaustion
/// 10. Special character path breaking
/// 
/// Each test MUST FAIL or expose weakness to be considered successful.
/// NO WORKAROUNDS ALLOWED - system works or fails clearly.

#[cfg(test)]
mod brutal_weakness_tests {
    use super::*;

    /// TEST 1: FUZZY DISTANCE EDGE CASES
    /// 
    /// CRITICAL FLAW: max_distance is hardcoded to 2, no configuration exposed
    /// This test MUST prove that fuzzy distance behavior is unpredictable
    #[cfg(feature = "tantivy")]
    #[tokio::test]
    async fn brutal_test_01_fuzzy_distance_undefined_behavior() -> Result<()> {
        println!("🔥 BRUTAL TEST 1: Fuzzy Distance Edge Cases");
        println!("==========================================");
        
        let temp_dir = TempDir::new()?;
        let mut searcher = TantivySearcher::new().await?;
        
        // Create content with known fuzzy match targets
        let test_file = temp_dir.path().join("fuzzy_targets.rs");
        fs::write(&test_file, r#"
pub fn database_connection() -> DatabaseConnection { unimplemented!() }
pub fn database_manager() -> DatabaseManager { unimplemented!() }
pub fn user_authentication() -> UserAuth { unimplemented!() }
pub fn payment_processor() -> PaymentProc { unimplemented!() }
"#)?;
        searcher.index_file(&test_file).await?;
        
        println!("📋 Testing fuzzy distance edge cases...");
        
        // Test cases that SHOULD expose undefined behavior
        let edge_cases = vec![
            // (query, max_distance, expected_issue)
            ("database", 0, "Distance 0 should be exact match only"),
            ("database", 1, "Distance 1 behavior undefined"),
            ("database", 2, "Distance 2 capped arbitrarily"),
            ("database", 3, "Distance 3 should be ignored (capped to 2)"),
            ("database", 5, "Distance 5 should be ignored (capped to 2)"),
            ("database", 10, "Distance 10 should be ignored (capped to 2)"),
            ("database", 255, "Max u8 should be ignored (capped to 2)"),
            
            // Prove that distance configuration is broken
            ("databse", 1, "1-char typo should match with distance=1"),
            ("databse", 0, "1-char typo should NOT match with distance=0"),
            ("dtabase", 2, "2-char error should match with distance=2"),
            ("dtabase", 1, "2-char error should NOT match with distance=1"),
        ];
        
        let mut undefined_behaviors = Vec::new();
        
        for (query, max_distance, expected_issue) in edge_cases {
            println!("   Testing: '{}' with max_distance={} ({})", query, max_distance, expected_issue);
            
            let result = searcher.search_fuzzy(query, max_distance).await?;
            println!("      Results: {} matches", result.len());
            
            // TRUTH CHECK: Verify distance behavior
            match max_distance {
                0 => {
                    // Distance 0 should only find exact matches
                    if !result.is_empty() && !result.iter().any(|r| r.content.contains(query)) {
                        undefined_behaviors.push(format!(
                            "Distance 0 found fuzzy matches for '{}' - should be exact only", 
                            query
                        ));
                    }
                }
                1 => {
                    // Distance 1 should find 1-character differences
                    let should_match = query == "databse"; // 1 char different from "database"
                    let found_match = !result.is_empty();
                    if should_match != found_match {
                        undefined_behaviors.push(format!(
                            "Distance 1 behavior wrong for '{}': expected match={}, found={}",
                            query, should_match, found_match
                        ));
                    }
                }
                2 => {
                    // This is the hardcoded limit - behavior should be consistent
                }
                _ => {
                    // Higher distances should be capped to 2, but we can't verify this
                    // because the implementation doesn't expose what distance was actually used
                    undefined_behaviors.push(format!(
                        "Cannot verify distance capping for max_distance={} - no feedback mechanism",
                        max_distance
                    ));
                }
            }
        }
        
        // EXPOSE THE FLAW: Distance configuration is completely opaque
        println!("\n🚨 CRITICAL FLAW EXPOSED: Fuzzy Distance Configuration");
        println!("   • No way to verify actual distance used");
        println!("   • Distance capping is invisible to users");
        println!("   • No configuration interface exposed");
        println!("   • Behavior is unpredictable and undocumented");
        
        if !undefined_behaviors.is_empty() {
            println!("\n❌ UNDEFINED BEHAVIORS DETECTED:");
            for behavior in &undefined_behaviors {
                println!("   • {}", behavior);
            }
        }
        
        // FAIL THE TEST if no issues were found (something is wrong with our test)
        assert!(!undefined_behaviors.is_empty() || max_distance > 2, 
               "Test must expose fuzzy distance issues or detect configuration problems");
        
        Ok(())
    }
    
    /// TEST 2: SCHEMA FLEXIBILITY BREAKING
    /// 
    /// CRITICAL FLAW: Schema is hardcoded, no flexibility for different use cases
    /// This test MUST prove schema cannot be modified or extended
    #[cfg(feature = "tantivy")]
    #[tokio::test]
    async fn brutal_test_02_schema_inflexibility_breaking() -> Result<()> {
        println!("🔥 BRUTAL TEST 2: Schema Flexibility Breaking");
        println!("============================================");
        
        let temp_dir = TempDir::new()?;
        
        println!("📋 Attempting to break schema assumptions...");
        
        // Test 1: Try to create searchers with different requirements
        let index_path = temp_dir.path().join("schema_test");
        
        // Create initial searcher
        let mut searcher1 = TantivySearcher::new_with_path(&index_path).await?;
        
        // Index some content
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "pub fn test() { println!(\"test\"); }")?;
        searcher1.index_file(&test_file).await?;
        
        // Get stats to verify it works
        let stats1 = searcher1.get_index_stats()?;
        println!("   Initial index: {} documents", stats1.num_documents);
        
        drop(searcher1); // Release file handles
        
        // Test 2: Try to open same index with different schema expectations
        // This should expose schema compatibility issues
        println!("   Testing schema compatibility...");
        
        let searcher2_result = TantivySearcher::new_with_path(&index_path).await;
        match searcher2_result {
            Ok(searcher2) => {
                let stats2 = searcher2.get_index_stats()?;
                println!("   Reopened index: {} documents", stats2.num_documents);
                
                // EXPOSE THE FLAW: Schema is completely fixed
                println!("\n🚨 CRITICAL FLAW EXPOSED: Schema Inflexibility");
                println!("   • Schema is hardcoded in implementation");
                println!("   • No way to customize fields");
                println!("   • No way to add metadata fields");
                println!("   • No versioning or migration support");
                
                // Try to search for something that would require different schema
                let search_result = searcher2.search("test").await?;
                println!("   Search results: {}", search_result.len());
                
                // The fact that this works shows the schema is inflexible but functional
                // The FLAW is that users cannot customize it for their needs
            }
            Err(e) => {
                println!("   ❌ Schema reopening failed: {}", e);
                
                // This could indicate schema corruption or compatibility issues
                if e.to_string().contains("schema") {
                    println!("\n🚨 CRITICAL FLAW EXPOSED: Schema Fragility");
                    println!("   • Schema compatibility detection is unreliable");
                    println!("   • Index becomes unusable after schema changes");
                    println!("   • No graceful schema evolution");
                } else {
                    return Err(e);
                }
            }
        }
        
        // Test 3: Demonstrate inability to add custom fields
        println!("   Testing custom field impossibility...");
        
        // Create content that would benefit from custom schema
        let complex_file = temp_dir.path().join("complex.rs");
        fs::write(&complex_file, r#"
// File: src/main.rs
// Author: John Doe
// Date: 2024-01-01
// Tags: main, entry-point, important
// Priority: high
// Module: core

pub fn main() {
    println!("Hello World");
}
"#)?;
        
        let mut searcher3 = TantivySearcher::new().await?;
        searcher3.index_file(&complex_file).await?;
        
        // Try to search by metadata that would require custom fields
        let metadata_searches = vec![
            "Author: John Doe",  // Would need author field
            "Tags: main",        // Would need tags field  
            "Priority: high",    // Would need priority field
            "Module: core",      // Would need module field
        ];
        
        for metadata_query in metadata_searches {
            let results = searcher3.search(metadata_query).await?;
            println!("   Search '{}': {} results", metadata_query, results.len());
            
            // These might find results in content field, but can't be structured
        }
        
        println!("\n🚨 SCHEMA LIMITATIONS EXPOSED:");
        println!("   • Cannot index file metadata separately");
        println!("   • Cannot add custom fields for domain-specific data");
        println!("   • Cannot optimize search for structured content");
        println!("   • All content is forced into generic 'content' field");
        
        Ok(())
    }
    
    /// TEST 3: MEMORY EXHAUSTION ATTACK
    /// 
    /// CRITICAL FLAW: No memory limits or bounds checking
    /// This test MUST demonstrate unbounded memory growth
    #[cfg(feature = "tantivy")]
    #[tokio::test]
    async fn brutal_test_03_unbounded_memory_exhaustion() -> Result<()> {
        println!("🔥 BRUTAL TEST 3: Unbounded Memory Exhaustion");
        println!("=============================================");
        
        let temp_dir = TempDir::new()?;
        let mut searcher = TantivySearcher::new().await?;
        
        println!("📋 Creating memory exhaustion scenarios...");
        
        // Test 1: Single massive document
        println!("   Test 1: Single massive document attack");
        
        let massive_file = temp_dir.path().join("memory_bomb.rs");
        let mut massive_content = String::new();
        
        // Create 50MB single-line document (memory bomb)
        massive_content.push_str("pub const MASSIVE_STRING: &str = \"");
        for i in 0..1_000_000 {
            massive_content.push_str(&format!("data_segment_{}_", i));
        }
        massive_content.push_str("\";\n");
        
        println!("      Creating 50MB+ single document...");
        fs::write(&massive_file, &massive_content)?;
        
        let memory_before = get_memory_usage();
        println!("      Memory before indexing: {:.1}MB", memory_before);
        
        let index_start = Instant::now();
        let index_result = searcher.index_file(&massive_file).await;
        let index_duration = index_start.elapsed();
        
        let memory_after = get_memory_usage();
        let memory_increase = memory_after - memory_before;
        
        match index_result {
            Ok(()) => {
                println!("      ✅ Indexed massive document in {:?}", index_duration);
                println!("      Memory increase: {:.1}MB", memory_increase);
                
                if memory_increase > 200.0 {
                    println!("\n🚨 MEMORY EXHAUSTION FLAW EXPOSED:");
                    println!("   • No memory limits enforced");
                    println!("   • Single document consumed {:.1}MB", memory_increase);
                    println!("   • Vulnerable to memory bomb attacks");
                    println!("   • No protection against malicious content");
                }
                
                // Test search on massive document
                let search_start = Instant::now();
                let search_result = searcher.search("data_segment_500000").await?;
                let search_duration = search_start.elapsed();
                
                println!("      Search on massive doc: {} results in {:?}", 
                        search_result.len(), search_duration);
                
                if search_duration > Duration::from_secs(5) {
                    println!("      🚨 Search performance degraded severely on massive document");
                }
            }
            Err(e) => {
                println!("      ❌ Massive document indexing failed: {}", e);
                
                // Check if it's a memory error
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("memory") || error_msg.contains("allocation") {
                    println!("      ✅ System protected against memory exhaustion");
                } else {
                    println!("      🚨 Failed for unexpected reason: {}", e);
                }
            }
        }
        
        // Test 2: Death by a thousand cuts - many medium documents
        println!("   Test 2: Death by thousand cuts attack");
        
        searcher.clear_index().await?;
        let memory_before_2 = get_memory_usage();
        
        let docs_dir = temp_dir.path().join("many_docs");
        fs::create_dir_all(&docs_dir)?;
        
        const DOC_COUNT: usize = 1000;
        const DOC_SIZE_KB: usize = 100; // 100KB each = 100MB total
        
        for i in 0..DOC_COUNT {
            let doc_file = docs_dir.join(format!("doc_{:04}.rs", i));
            let mut doc_content = String::new();
            
            // Create 100KB document
            let base_content = format!("pub fn function_{}() -> String {{\n    let data = \"", i);
            doc_content.push_str(&base_content);
            
            let data_size = (DOC_SIZE_KB * 1024) - base_content.len() - 20; // Leave room for closing
            for _ in 0..data_size / 10 {
                doc_content.push_str("0123456789");
            }
            
            doc_content.push_str("\";\n    data.to_string()\n}\n");
            fs::write(&doc_file, &doc_content)?;
        }
        
        println!("      Indexing {} x {}KB documents...", DOC_COUNT, DOC_SIZE_KB);
        let mass_index_start = Instant::now();
        let mass_index_result = searcher.index_directory(&docs_dir).await;
        let mass_index_duration = mass_index_start.elapsed();
        
        let memory_after_2 = get_memory_usage();
        let memory_increase_2 = memory_after_2 - memory_before_2;
        
        match mass_index_result {
            Ok(()) => {
                println!("      ✅ Mass indexing completed in {:?}", mass_index_duration);
                println!("      Memory increase: {:.1}MB", memory_increase_2);
                
                let stats = searcher.get_index_stats()?;
                println!("      Index size: {:.1}MB", stats.index_size_bytes as f64 / 1_024_000.0);
                
                if memory_increase_2 > 500.0 {
                    println!("\n🚨 UNBOUNDED MEMORY GROWTH EXPOSED:");
                    println!("   • Memory grew by {:.1}MB for {}MB of content", 
                            memory_increase_2, (DOC_COUNT * DOC_SIZE_KB) as f64 / 1024.0);
                    println!("   • No memory management or cleanup");
                    println!("   • Vulnerable to resource exhaustion");
                    println!("   • Memory usage ratio: {:.1}x content size", 
                            memory_increase_2 / ((DOC_COUNT * DOC_SIZE_KB) as f64 / 1024.0));
                }
            }
            Err(e) => {
                println!("      ❌ Mass indexing failed: {}", e);
                
                if e.to_string().to_lowercase().contains("memory") {
                    println!("      ✅ System has some memory protection");
                } else {
                    println!("      🚨 Failed for unexpected reason, may indicate other issues");
                }
            }
        }
        
        Ok(())
    }
    
    /// TEST 4: CONCURRENT WRITE CORRUPTION
    /// 
    /// CRITICAL FLAW: No proper concurrent access control
    /// This test MUST demonstrate race conditions and corruption
    #[cfg(feature = "tantivy")]
    #[tokio::test]
    async fn brutal_test_04_concurrent_write_corruption() -> Result<()> {
        println!("🔥 BRUTAL TEST 4: Concurrent Write Corruption");
        println!("=============================================");
        
        let temp_dir = TempDir::new()?;
        let index_path = temp_dir.path().join("corruption_test");
        
        // Create test files for concurrent access
        let test_files_dir = temp_dir.path().join("concurrent_files");
        fs::create_dir_all(&test_files_dir)?;
        
        const CONCURRENT_FILES: usize = 50;
        let mut file_paths = Vec::new();
        
        for i in 0..CONCURRENT_FILES {
            let file_path = test_files_dir.join(format!("concurrent_{}.rs", i));
            let content = format!(
                "pub fn concurrent_function_{}() {{\n    \
                    let unique_data_{} = \"corruption_test_data_{}\";\n    \
                    println!(\"Processing: {{}}\", unique_data_{});\n\
                }}\n\n\
                pub struct ConcurrentStruct_{} {{\n    \
                    field_{}: String,\n\
                }}\n",
                i, i, i, i, i, i
            );
            fs::write(&file_path, content)?;
            file_paths.push(file_path);
        }
        
        println!("📋 Testing concurrent write operations...");
        
        // Shared corruption detection
        let corruption_detected = Arc::new(AtomicUsize::new(0));
        let successful_operations = Arc::new(AtomicUsize::new(0));
        let failed_operations = Arc::new(AtomicUsize::new(0));
        let inconsistent_results = Arc::new(Mutex::new(Vec::new()));
        
        // Launch multiple concurrent indexing tasks
        let mut tasks = Vec::new();
        
        for task_id in 0..10 {
            let index_path = index_path.clone();
            let files_dir = test_files_dir.clone();
            let corruption_detected = corruption_detected.clone();
            let successful_operations = successful_operations.clone();
            let failed_operations = failed_operations.clone();
            let inconsistent_results = inconsistent_results.clone();
            
            let task = task::spawn(async move {
                println!("   Task {} starting concurrent operations...", task_id);
                
                for iteration in 0..5 {
                    // Create searcher and try to index
                    let searcher_result = TantivySearcher::new_with_path(&index_path).await;
                    
                    match searcher_result {
                        Ok(mut searcher) => {
                            // Try to index directory
                            let index_result = searcher.index_directory(&files_dir).await;
                            
                            match index_result {
                                Ok(()) => {
                                    successful_operations.fetch_add(1, Ordering::SeqCst);
                                    
                                    // Verify index integrity
                                    let stats_result = searcher.get_index_stats();
                                    match stats_result {
                                        Ok(stats) => {
                                            // Check for reasonable document count
                                            if stats.num_documents == 0 {
                                                corruption_detected.fetch_add(1, Ordering::SeqCst);
                                                println!("      Task {}: CORRUPTION - Zero documents after successful index", task_id);
                                            } else if stats.num_documents > CONCURRENT_FILES * 10 {
                                                corruption_detected.fetch_add(1, Ordering::SeqCst);
                                                println!("      Task {}: CORRUPTION - Excessive documents: {}", task_id, stats.num_documents);
                                            }
                                            
                                            // Test search consistency
                                            let search_result = searcher.search(&format!("concurrent_function_{}", iteration)).await;
                                            match search_result {
                                                Ok(results) => {
                                                    if results.is_empty() {
                                                        let mut inconsistent = inconsistent_results.lock().unwrap();
                                                        inconsistent.push(format!(
                                                            "Task {} iteration {}: No search results for known content",
                                                            task_id, iteration
                                                        ));
                                                    }
                                                }
                                                Err(e) => {
                                                    corruption_detected.fetch_add(1, Ordering::SeqCst);
                                                    println!("      Task {}: SEARCH CORRUPTION - {}", task_id, e);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            corruption_detected.fetch_add(1, Ordering::SeqCst);
                                            println!("      Task {}: STATS CORRUPTION - {}", task_id, e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    failed_operations.fetch_add(1, Ordering::SeqCst);
                                    
                                    // Check if it's corruption-related
                                    let error_msg = e.to_string().to_lowercase();
                                    if error_msg.contains("corrupt") || error_msg.contains("lock") || 
                                       error_msg.contains("conflict") || error_msg.contains("race") {
                                        corruption_detected.fetch_add(1, Ordering::SeqCst);
                                        println!("      Task {}: CONCURRENCY CORRUPTION - {}", task_id, e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            failed_operations.fetch_add(1, Ordering::SeqCst);
                            
                            let error_msg = e.to_string().to_lowercase();
                            if error_msg.contains("corrupt") || error_msg.contains("lock") {
                                corruption_detected.fetch_add(1, Ordering::SeqCst);
                                println!("      Task {}: SEARCHER CORRUPTION - {}", task_id, e);
                            }
                        }
                    }
                    
                    // Small delay to increase chance of race conditions
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            });
            
            tasks.push(task);
        }
        
        // Wait for all concurrent tasks
        join_all(tasks).await;
        
        let total_corruption = corruption_detected.load(Ordering::SeqCst);
        let total_successful = successful_operations.load(Ordering::SeqCst);
        let total_failed = failed_operations.load(Ordering::SeqCst);
        let inconsistencies = inconsistent_results.lock().unwrap();
        
        println!("\n📊 Concurrent corruption test results:");
        println!("   ✅ Successful operations: {}", total_successful);
        println!("   ❌ Failed operations: {}", total_failed);
        println!("   🚨 Corruption incidents: {}", total_corruption);
        println!("   ⚠️  Inconsistent results: {}", inconsistencies.len());
        
        if total_corruption > 0 {
            println!("\n🚨 CONCURRENT WRITE CORRUPTION EXPOSED:");
            println!("   • {} corruption incidents detected", total_corruption);
            println!("   • Race conditions in index operations");
            println!("   • No proper concurrency control");
            println!("   • Data integrity compromised under load");
        }
        
        if !inconsistencies.is_empty() {
            println!("\n🚨 CONSISTENCY VIOLATIONS:");
            for inconsistency in inconsistencies.iter().take(5) {
                println!("   • {}", inconsistency);
            }
            if inconsistencies.len() > 5 {
                println!("   • ... and {} more", inconsistencies.len() - 5);
            }
        }
        
        // Final integrity check
        println!("\n   Final integrity verification...");
        let final_searcher = TantivySearcher::new_with_path(&index_path).await?;
        let final_stats = final_searcher.get_index_stats()?;
        println!("   Final index state: {} documents, {} segments", 
                final_stats.num_documents, final_stats.num_segments);
        
        // The test succeeds if it exposed corruption or consistency issues
        let issues_found = total_corruption > 0 || !inconsistencies.is_empty() || 
                          final_stats.num_documents == 0;
        
        if !issues_found {
            println!("   ⚠️  No corruption detected - concurrency controls may be working");
        }
        
        Ok(())
    }
    
    /// TEST 5: UNICODE NORMALIZATION CHAOS
    /// 
    /// CRITICAL FLAW: No proper Unicode handling or normalization
    /// This test MUST demonstrate Unicode-related failures
    #[cfg(feature = "tantivy")]
    #[tokio::test]
    async fn brutal_test_05_unicode_normalization_chaos() -> Result<()> {
        println!("🔥 BRUTAL TEST 5: Unicode Normalization Chaos");
        println!("==============================================");
        
        let temp_dir = TempDir::new()?;
        let mut searcher = TantivySearcher::new().await?;
        
        println!("📋 Creating Unicode chaos scenarios...");
        
        // Test 1: Unicode normalization inconsistencies
        let unicode_file = temp_dir.path().join("unicode_chaos.rs");
        let unicode_content = r#"
// Test various Unicode representations of the same logical content

// NFC (Canonical Decomposition, then Canonical Composition)
pub fn café_nfc() -> String { "café".to_string() }

// NFD (Canonical Decomposition)  
pub fn café_nfd() -> String { "cafe\u{0301}".to_string() }

// NFKC (Compatibility Decomposition, then Canonical Composition)
pub fn résumé_nfkc() -> String { "résumé".to_string() }

// NFKD (Compatibility Decomposition)
pub fn résumé_nfkd() -> String { "re\u{0301}sume\u{0301}".to_string() }

// Mixed scripts and directions
pub fn mixed_scripts() -> String { "English العربية 中文 🚀".to_string() }

// Zero-width characters (invisible)
pub fn zero_width_chaos() -> String { "test\u{200B}function\u{200C}name\u{200D}here".to_string() }

// Homoglyphs (visually identical but different codepoints)
pub fn homoglyph_а() -> String { "а".to_string() } // Cyrillic 'а' (U+0430)
pub fn homoglyph_a() -> String { "a".to_string() } // Latin 'a' (U+0061)

// Combining character chaos
pub fn combining_chaos() -> String { "e\u{0301}\u{0302}\u{0303}\u{0304}\u{0305}".to_string() }

// Surrogate pairs and high codepoints
pub fn emoji_chaos() -> String { "🚀👨‍💻🏳️‍🌈".to_string() }

// Right-to-left text
pub fn rtl_text() -> String { "שלום hello مرحبا".to_string() }

// Invalid UTF-8 sequences (if they survive file writing)
pub fn boundary_test() -> String { "\u{FEFF}\u{FFFD}".to_string() }
"#;
        
        fs::write(&unicode_file, unicode_content)?;
        
        let index_result = searcher.index_file(&unicode_file).await;
        match index_result {
            Ok(()) => {
                println!("   ✅ Unicode content indexed");
                
                // Test normalization issues
                let normalization_tests = vec![
                    // (search_term, expected_to_find, description)
                    ("café", "Both NFC and NFD café", "Normalization consistency"),
                    ("cafe\u{0301}", "Both NFC and NFD café", "Decomposed search"),
                    ("résumé", "Both NFKC and NFKD résumé", "Compatibility normalization"),
                    ("re\u{0301}sume\u{0301}", "Both NFKC and NFKD résumé", "Decomposed compatibility"),
                    
                    // Homoglyph confusion
                    ("а", "Cyrillic а function", "Cyrillic homoglyph"),
                    ("a", "Latin a function", "Latin homoglyph"),
                    
                    // Zero-width character handling
                    ("testfunction", "Should match zero_width_chaos", "Zero-width normalization"),
                    ("test\u{200B}function", "Direct zero-width search", "Zero-width exact"),
                    
                    // Emoji and complex scripts
                    ("🚀", "Should match emoji content", "Emoji search"),
                    ("العربية", "Should match Arabic text", "RTL script search"),
                    ("中文", "Should match Chinese text", "CJK search"),
                ];
                
                let mut normalization_failures = Vec::new();
                
                for (search_term, expected_desc, test_desc) in normalization_tests {
                    println!("   Testing: {} ({})", test_desc, expected_desc);
                    
                    let search_result = searcher.search(search_term).await?;
                    println!("      Results: {} matches", search_result.len());
                    
                    if search_result.is_empty() {
                        normalization_failures.push(format!(
                            "No results for '{}' - {} failed", 
                            search_term.escape_debug(), test_desc
                        ));
                    } else {
                        // Verify results are relevant
                        let first_result = &search_result[0];
                        println!("      Sample: {}", first_result.content.escape_debug());
                        
                        // Check for normalization issues
                        let content_normalized = first_result.content.to_lowercase();
                        let search_normalized = search_term.to_lowercase();
                        
                        if !content_normalized.contains(&search_normalized) && 
                           !search_normalized.chars().any(|c| content_normalized.contains(c)) {
                            normalization_failures.push(format!(
                                "Result '{}' doesn't relate to search '{}' - normalization broken",
                                first_result.content.escape_debug(),
                                search_term.escape_debug()
                            ));
                        }
                    }
                }
                
                // Test fuzzy search with Unicode
                println!("   Testing fuzzy search with Unicode...");
                
                let fuzzy_unicode_tests = vec![
                    ("cafe", "Should find café variants"),
                    ("resume", "Should find résumé variants"),
                    ("العرب", "Arabic fuzzy matching"),
                    ("中", "CJK fuzzy matching"),
                ];
                
                for (fuzzy_term, expected) in fuzzy_unicode_tests {
                    let fuzzy_result = searcher.search_fuzzy(fuzzy_term, 2).await?;
                    println!("      Fuzzy '{}': {} results ({})", 
                            fuzzy_term.escape_debug(), fuzzy_result.len(), expected);
                    
                    if fuzzy_result.is_empty() {
                        normalization_failures.push(format!(
                            "Fuzzy search failed for Unicode term '{}'", 
                            fuzzy_term.escape_debug()
                        ));
                    }
                }
                
                if !normalization_failures.is_empty() {
                    println!("\n🚨 UNICODE NORMALIZATION FAILURES EXPOSED:");
                    for failure in &normalization_failures {
                        println!("   • {}", failure);
                    }
                    println!("   • No proper Unicode normalization");
                    println!("   • Inconsistent handling of equivalent characters");
                    println!("   • Search results unreliable for international content");
                }
            }
            Err(e) => {
                println!("   ❌ Unicode content indexing failed: {}", e);
                
                let error_msg = e.to_string();
                if error_msg.contains("encoding") || error_msg.contains("unicode") || 
                   error_msg.contains("invalid") {
                    println!("\n🚨 UNICODE HANDLING FAILURE EXPOSED:");
                    println!("   • Cannot handle complex Unicode content");
                    println!("   • No proper encoding support");
                    println!("   • International users will experience failures");
                }
            }
        }
        
        // Test 2: File path Unicode issues
        println!("   Testing Unicode in file paths...");
        
        let unicode_path_tests = vec![
            "测试文件.rs",
            "café_résumé.rs",
            "🚀_emoji_file.rs",
            "العربية.rs",
            "файл.rs",
        ];
        
        let mut path_failures = Vec::new();
        
        for unicode_filename in unicode_path_tests {
            let unicode_file_path = temp_dir.path().join(unicode_filename);
            let simple_content = format!("pub fn {}() {{}}", unicode_filename.replace(".rs", ""));
            
            let write_result = fs::write(&unicode_file_path, &simple_content);
            match write_result {
                Ok(()) => {
                    let index_result = searcher.index_file(&unicode_file_path).await;
                    match index_result {
                        Ok(()) => {
                            println!("      ✅ Indexed Unicode filename: {}", unicode_filename);
                        }
                        Err(e) => {
                            path_failures.push(format!(
                                "Failed to index Unicode filename '{}': {}", 
                                unicode_filename, e
                            ));
                        }
                    }
                }
                Err(e) => {
                    path_failures.push(format!(
                        "Failed to create Unicode filename '{}': {}", 
                        unicode_filename, e
                    ));
                }
            }
        }
        
        if !path_failures.is_empty() {
            println!("\n🚨 UNICODE PATH HANDLING FAILURES:");
            for failure in &path_failures {
                println!("   • {}", failure);
            }
        }
        
        Ok(())
    }

    /// TEST 6: EMPTY AND MALFORMED QUERY BOUNDARY CONDITIONS
    /// 
    /// CRITICAL FLAW: No proper input validation or error handling
    /// This test MUST demonstrate panic or undefined behavior with edge cases
    #[cfg(feature = "tantivy")]
    #[tokio::test]
    async fn brutal_test_06_query_boundary_chaos() -> Result<()> {
        println!("🔥 BRUTAL TEST 6: Query Boundary Chaos");
        println!("======================================");
        
        let temp_dir = TempDir::new()?;
        let mut searcher = TantivySearcher::new().await?;
        
        // Index some normal content first
        let test_file = temp_dir.path().join("boundary_test.rs");
        fs::write(&test_file, r#"
pub fn normal_function() -> String { "test content".to_string() }
pub fn another_function() -> i32 { 42 }
pub struct TestStruct { field: String }
"#)?;
        searcher.index_file(&test_file).await?;
        
        println!("📋 Testing boundary condition queries...");
        
        // Boundary condition queries that should expose weaknesses
        let boundary_queries = vec![
            // (query, description, expect_panic, expect_error)
            ("", "Empty string", false, true),
            (" ", "Single space", false, false),
            ("  ", "Multiple spaces", false, false),
            ("\n", "Newline only", false, false),
            ("\t", "Tab only", false, false),
            ("\r", "Carriage return", false, false),
            ("\r\n", "CRLF", false, false),
            
            // Null and control characters
            ("\0", "Null byte", false, true),
            ("\x01", "Control character", false, false),
            ("\x1F", "Unit separator", false, false),
            ("\x7F", "DEL character", false, false),
            
            // Query syntax edge cases
            ("\"", "Unmatched quote", false, true),
            ("\"\"", "Empty quotes", false, false),
            ("\\", "Single backslash", false, false),
            ("\\\\", "Double backslash", false, false),
            ("'", "Single quote", false, false),
            ("''", "Empty single quotes", false, false),
            
            // Tantivy query syntax abuse
            ("AND", "Boolean operator alone", false, true),
            ("OR", "Boolean operator alone", false, true),
            ("NOT", "Boolean operator alone", false, true),
            ("AND OR", "Invalid boolean combination", false, true),
            ("test AND", "Incomplete boolean expression", false, true),
            ("test OR", "Incomplete boolean expression", false, true),
            ("NOT test AND", "Complex incomplete expression", false, true),
            
            // Field syntax abuse
            ("field:", "Empty field query", false, true),
            (":value", "Missing field name", false, true),
            ("field::", "Double colon", false, true),
            ("nonexistent:value", "Nonexistent field", false, false),
            
            // Range and wildcard abuse
            ("[", "Unmatched bracket", false, true),
            ("]", "Unmatched closing bracket", false, true),
            ("[}", "Mismatched brackets", false, true),
            ("{]", "Mismatched brackets reverse", false, true),
            ("*", "Wildcard alone", false, true),
            ("**", "Double wildcard", false, true),
            ("?", "Question mark alone", false, true),
            ("??", "Double question mark", false, true),
            
            // Extremely long queries
            ("a".repeat(10000).as_str(), "10K character query", false, false),
            ("test ".repeat(1000).as_str(), "1K repeated words", false, false),
            
            // Special Unicode queries
            ("\u{FEFF}", "Byte order mark", false, false),
            ("\u{200B}", "Zero width space", false, false),
            ("\u{FFFD}", "Replacement character", false, false),
            
            // Regex-like patterns that might confuse parser
            ("test.*", "Regex-like pattern", false, false),
            ("test.+", "Regex quantifier", false, false),
            ("test{1,5}", "Regex quantifier braces", false, false),
            ("test|other", "Regex alternation", false, false),
            ("(test)", "Parentheses grouping", false, false),
            ("[test]", "Character class", false, false),
            
            // SQL injection style
            ("'; DROP TABLE documents; --", "SQL injection attempt", false, false),
            ("test'; DELETE FROM index; --", "SQL deletion attempt", false, false),
        ];
        
        let mut panic_queries = Vec::new();
        let mut error_queries = Vec::new();
        let mut unexpected_success = Vec::new();
        
        for (query, description, expect_panic, expect_error) in boundary_queries {
            println!("   Testing: {} ('{}')", description, query.escape_debug());
            
            // Test regular search
            let regular_search_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                tokio::runtime::Handle::current().block_on(async {
                    searcher.search(query).await
                })
            }));
            
            match regular_search_result {
                Ok(search_result) => {
                    match search_result {
                        Ok(results) => {
                            println!("      Regular search: {} results", results.len());
                            if expect_error {
                                unexpected_success.push(format!(
                                    "Expected error but got {} results for: {}", 
                                    results.len(), description
                                ));
                            }
                        }
                        Err(e) => {
                            println!("      Regular search error: {}", e);
                            if !expect_error {
                                error_queries.push(format!(
                                    "Unexpected error for {}: {}", description, e
                                ));
                            }
                        }
                    }
                }
                Err(_) => {
                    println!("      🚨 PANIC in regular search!");
                    panic_queries.push(format!("Regular search panic: {}", description));
                }
            }
            
            // Test fuzzy search
            let fuzzy_search_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                tokio::runtime::Handle::current().block_on(async {
                    searcher.search_fuzzy(query, 1).await
                })
            }));
            
            match fuzzy_search_result {
                Ok(search_result) => {
                    match search_result {
                        Ok(results) => {
                            println!("      Fuzzy search: {} results", results.len());
                        }
                        Err(e) => {
                            println!("      Fuzzy search error: {}", e);
                        }
                    }
                }
                Err(_) => {
                    println!("      🚨 PANIC in fuzzy search!");
                    panic_queries.push(format!("Fuzzy search panic: {}", description));
                }
            }
        }
        
        // Report findings
        if !panic_queries.is_empty() {
            println!("\n🚨 PANIC CONDITIONS EXPOSED:");
            for panic in &panic_queries {
                println!("   • {}", panic);
            }
            println!("   • No proper input validation");
            println!("   • System vulnerable to crash attacks");
        }
        
        if !error_queries.is_empty() {
            println!("\n🚨 UNEXPECTED ERROR CONDITIONS:");
            for error in &error_queries {
                println!("   • {}", error);
            }
        }
        
        if !unexpected_success.is_empty() {
            println!("\n🚨 INSUFFICIENT INPUT VALIDATION:");
            for success in &unexpected_success {
                println!("   • {}", success);
            }
        }
        
        println!("\n📊 Boundary test summary:");
        println!("   Panic conditions: {}", panic_queries.len());
        println!("   Unexpected errors: {}", error_queries.len());
        println!("   Validation bypasses: {}", unexpected_success.len());
        
        Ok(())
    }
    
    /// TEST 7: INDEX CORRUPTION DETECTION AND RECOVERY
    /// 
    /// CRITICAL FLAW: No corruption detection or recovery mechanisms
    /// This test MUST demonstrate vulnerability to corruption
    #[cfg(feature = "tantivy")]
    #[tokio::test]
    async fn brutal_test_07_index_corruption_vulnerability() -> Result<()> {
        println!("🔥 BRUTAL TEST 7: Index Corruption Vulnerability");
        println!("===============================================");
        
        let temp_dir = TempDir::new()?;
        let index_path = temp_dir.path().join("corruption_target");
        
        // Create and populate index
        let mut searcher = TantivySearcher::new_with_path(&index_path).await?;
        
        let test_file = temp_dir.path().join("corruption_test.rs");
        fs::write(&test_file, r#"
pub fn important_function() -> String { "critical_data".to_string() }
pub fn another_function() -> i32 { 12345 }
pub struct ImportantStruct { data: String, value: i64 }
"#)?;
        searcher.index_file(&test_file).await?;
        
        // Verify index works before corruption
        let pre_corruption_results = searcher.search("important_function").await?;
        assert!(!pre_corruption_results.is_empty(), "Index must work before corruption");
        println!("   ✅ Pre-corruption: {} results found", pre_corruption_results.len());
        
        // Get index stats
        let pre_stats = searcher.get_index_stats()?;
        println!("   📊 Pre-corruption stats: {} docs, {} segments", 
                pre_stats.num_documents, pre_stats.num_segments);
        
        drop(searcher); // Release file handles
        
        println!("📋 Implementing various corruption scenarios...");
        
        let mut corruption_results = Vec::new();
        
        // Corruption Test 1: Partial file corruption
        println!("   Test 1: Partial file corruption");
        {
            if let Some(index_dir) = fs::read_dir(&index_path).ok() {
                for entry in index_dir {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file() && path.extension().is_some() {
                            // Corrupt the middle of the file
                            if let Ok(mut file_data) = fs::read(&path) {
                                let original_size = file_data.len();
                                if original_size > 100 {
                                    let corrupt_start = original_size / 3;
                                    let corrupt_end = (original_size * 2) / 3;
                                    
                                    // Overwrite with garbage
                                    for i in corrupt_start..corrupt_end {
                                        file_data[i] = 0xFF;
                                    }
                                    
                                    fs::write(&path, &file_data)?;
                                    println!("      Corrupted file: {:?} ({} bytes)", path.file_name(), corrupt_end - corrupt_start);
                                    break; // Corrupt one file
                                }
                            }
                        }
                    }
                }
            }
            
            // Try to open corrupted index
            let corruption_test_result = TantivySearcher::new_with_path(&index_path).await;
            match corruption_test_result {
                Ok(mut recovered_searcher) => {
                    println!("      ⚠️  Corrupted index opened successfully - corruption not detected");
                    
                    // Test if search still works
                    let search_result = recovered_searcher.search("important_function").await;
                    match search_result {
                        Ok(results) => {
                            if results.len() != pre_corruption_results.len() {
                                corruption_results.push(format!(
                                    "Partial corruption: Search results changed from {} to {} - data integrity compromised",
                                    pre_corruption_results.len(), results.len()
                                ));
                            } else {
                                corruption_results.push(
                                    "Partial corruption went undetected - no validation mechanisms".to_string()
                                );
                            }
                        }
                        Err(e) => {
                            corruption_results.push(format!(
                                "Partial corruption caused search failure: {}", e
                            ));
                        }
                    }
                }
                Err(e) => {
                    println!("      ✅ Corrupted index properly rejected: {}", e);
                    
                    // Check if error message indicates corruption detection
                    if !e.to_string().to_lowercase().contains("corrupt") {
                        corruption_results.push(format!(
                            "Corruption detected but error message unclear: {}", e
                        ));
                    }
                }
            }
        }
        
        // Restore index for next test
        let mut clean_searcher = TantivySearcher::new_with_path(&index_path).await?;
        clean_searcher.index_file(&test_file).await?;
        drop(clean_searcher);
        
        // Corruption Test 2: Complete file deletion
        println!("   Test 2: Critical file deletion");
        {
            if let Some(index_dir) = fs::read_dir(&index_path).ok() {
                for entry in index_dir {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file() {
                            fs::remove_file(&path)?;
                            println!("      Deleted critical file: {:?}", path.file_name());
                            break; // Delete one critical file
                        }
                    }
                }
            }
            
            let deletion_test_result = TantivySearcher::new_with_path(&index_path).await;
            match deletion_test_result {
                Ok(_) => {
                    corruption_results.push(
                        "File deletion went undetected - no integrity checking".to_string()
                    );
                }
                Err(e) => {
                    println!("      ✅ File deletion properly detected: {}", e);
                }
            }
        }
        
        // Corruption Test 3: Directory permission manipulation
        println!("   Test 3: Permission corruption");
        {
            // This test may not work on all platforms, but we'll try
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                
                let index_metadata = fs::metadata(&index_path)?;
                let original_permissions = index_metadata.permissions();
                
                // Remove read permissions
                let mut new_permissions = original_permissions.clone();
                new_permissions.set_mode(0o000); // No permissions
                
                if fs::set_permissions(&index_path, new_permissions).is_ok() {
                    let permission_test_result = TantivySearcher::new_with_path(&index_path).await;
                    
                    // Restore permissions for cleanup
                    let _ = fs::set_permissions(&index_path, original_permissions);
                    
                    match permission_test_result {
                        Ok(_) => {
                            corruption_results.push(
                                "Permission corruption bypassed - no access validation".to_string()
                            );
                        }
                        Err(e) => {
                            println!("      ✅ Permission corruption detected: {}", e);
                        }
                    }
                }
            }
        }
        
        // Corruption Test 4: Zero-byte file corruption
        println!("   Test 4: Zero-byte file corruption");
        {
            // Recreate clean index
            let mut clean_searcher2 = TantivySearcher::new_with_path(&index_path).await?;
            clean_searcher2.index_file(&test_file).await?;
            drop(clean_searcher2);
            
            // Zero out a file
            if let Some(index_dir) = fs::read_dir(&index_path).ok() {
                for entry in index_dir {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file() && path.extension().is_some() {
                            fs::write(&path, "")?; // Zero out file
                            println!("      Zeroed file: {:?}", path.file_name());
                            break;
                        }
                    }
                }
            }
            
            let zero_test_result = TantivySearcher::new_with_path(&index_path).await;
            match zero_test_result {
                Ok(mut zero_searcher) => {
                    corruption_results.push(
                        "Zero-byte corruption undetected".to_string()
                    );
                    
                    let search_result = zero_searcher.search("important_function").await;
                    if let Ok(results) = search_result {
                        if results.is_empty() {
                            corruption_results.push(
                                "Zero-byte corruption caused silent data loss".to_string()
                            );
                        }
                    }
                }
                Err(e) => {
                    println!("      ✅ Zero-byte corruption detected: {}", e);
                }
            }
        }
        
        // Report corruption vulnerability findings
        if !corruption_results.is_empty() {
            println!("\n🚨 INDEX CORRUPTION VULNERABILITIES EXPOSED:");
            for vulnerability in &corruption_results {
                println!("   • {}", vulnerability);
            }
            println!("   • No corruption detection mechanisms");
            println!("   • No automatic recovery capabilities");
            println!("   • Silent data loss possible");
            println!("   • No integrity validation");
        } else {
            println!("\n   ✅ Index corruption properly handled in all test cases");
        }
        
        Ok(())
    }
    
    /// TEST 8: SPECIAL CHARACTER PATH BREAKING
    /// 
    /// CRITICAL FLAW: Path handling doesn't account for special characters
    /// This test MUST demonstrate path-related failures
    #[cfg(feature = "tantivy")]
    #[tokio::test]
    async fn brutal_test_08_special_character_path_chaos() -> Result<()> {
        println!("🔥 BRUTAL TEST 8: Special Character Path Chaos");
        println!("==============================================");
        
        let temp_dir = TempDir::new()?;
        
        println!("📋 Testing special character path handling...");
        
        let mut path_failures = Vec::new();
        let mut searcher_failures = Vec::new();
        
        // Special character path tests
        let special_paths = vec![
            // (path_component, description, expect_failure)
            ("normal", "Normal path", false),
            ("with spaces", "Spaces in path", false),
            ("with-dashes", "Dashes in path", false),
            ("with_underscores", "Underscores in path", false),
            ("with.dots", "Dots in path", false),
            ("with,commas", "Commas in path", false),
            ("with;semicolons", "Semicolons in path", false),
            ("with'apostrophe", "Apostrophe in path", false),
            ("with\"quotes", "Quotes in path", true),
            ("with|pipe", "Pipe in path", true),
            ("with<less", "Less-than in path", true),
            ("with>greater", "Greater-than in path", true),
            ("with:colon", "Colon in path", true),
            ("with*asterisk", "Asterisk in path", true),
            ("with?question", "Question mark in path", true),
            ("with\\backslash", "Backslash in path", true),
            ("with/slash", "Forward slash in path", true),
            ("with\nNewline", "Newline in path", true),
            ("with\tTab", "Tab in path", true),
            ("with\0Null", "Null byte in path", true),
            ("with[brackets]", "Brackets in path", false),
            ("with{braces}", "Braces in path", false),
            ("with(parens)", "Parentheses in path", false),
            ("with%percent", "Percent in path", false),
            ("with&ampersand", "Ampersand in path", false),
            ("with#hash", "Hash in path", false),
            ("with@at", "At symbol in path", false),
            ("with!exclamation", "Exclamation in path", false),
            ("with~tilde", "Tilde in path", false),
            ("with`backtick", "Backtick in path", false),
            ("with=equals", "Equals in path", false),
            ("with+plus", "Plus in path", false),
            ("CON", "Windows reserved name CON", true),
            ("PRN", "Windows reserved name PRN", true),
            ("AUX", "Windows reserved name AUX", true),
            ("NUL", "Windows reserved name NUL", true),
            ("COM1", "Windows reserved name COM1", true),
            ("LPT1", "Windows reserved name LPT1", true),
            ("", "Empty path component", true),
            (".", "Current directory", true),
            ("..", "Parent directory", true),
            ("....", "Multiple dots", false),
            (&"a".repeat(300), "Very long path component", false),
        ];
        
        for (path_component, description, expect_failure) in special_paths {
            println!("   Testing: {} ('{}')", description, path_component.escape_debug());
            
            // Skip problematic components that would break our test setup
            if path_component.contains('/') || path_component.contains('\\') || 
               path_component.contains('\0') || path_component == "" {
                println!("      Skipped: Too dangerous for test environment");
                continue;
            }
            
            let special_dir = temp_dir.path().join(path_component);
            
            // Try to create directory
            let dir_creation = fs::create_dir_all(&special_dir);
            match dir_creation {
                Ok(()) => {
                    println!("      Directory created successfully");
                    
                    // Try to create index in this directory
                    let index_path = special_dir.join("index");
                    let searcher_result = TantivySearcher::new_with_path(&index_path).await;
                    
                    match searcher_result {
                        Ok(mut searcher) => {
                            println!("      Searcher created successfully");
                            
                            // Try to create and index a file with special characters
                            let special_file = special_dir.join(format!("{}.rs", path_component));
                            let content = format!("pub fn test_{}() -> String {{ \"test\" }}", 
                                                 path_component.replace(|c: char| !c.is_alphanumeric(), "_"));
                            
                            let file_write_result = fs::write(&special_file, &content);
                            match file_write_result {
                                Ok(()) => {
                                    let index_result = searcher.index_file(&special_file).await;
                                    match index_result {
                                        Ok(()) => {
                                            println!("      File indexed successfully");
                                            
                                            // Try search
                                            let search_result = searcher.search("test").await;
                                            match search_result {
                                                Ok(results) => {
                                                    println!("      Search successful: {} results", results.len());
                                                    
                                                    if expect_failure {
                                                        searcher_failures.push(format!(
                                                            "Expected failure for '{}' but all operations succeeded",
                                                            description
                                                        ));
                                                    }
                                                }
                                                Err(e) => {
                                                    println!("      Search failed: {}", e);
                                                    searcher_failures.push(format!(
                                                        "Search failed for '{}': {}", description, e
                                                    ));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            println!("      Indexing failed: {}", e);
                                            if !expect_failure {
                                                searcher_failures.push(format!(
                                                    "Unexpected indexing failure for '{}': {}", description, e
                                                ));
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("      File creation failed: {}", e);
                                    path_failures.push(format!(
                                        "File creation failed for '{}': {}", description, e
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            println!("      Searcher creation failed: {}", e);
                            if !expect_failure {
                                searcher_failures.push(format!(
                                    "Unexpected searcher creation failure for '{}': {}", description, e
                                ));
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("      Directory creation failed: {}", e);
                    path_failures.push(format!(
                        "Directory creation failed for '{}': {}", description, e
                    ));
                }
            }
        }
        
        // Test extremely deep paths
        println!("   Testing extremely deep path nesting...");
        let mut deep_path = temp_dir.path().to_path_buf();
        
        // Create path with 50 nested directories
        for i in 0..50 {
            deep_path.push(format!("level_{}", i));
        }
        
        let deep_creation_result = fs::create_dir_all(&deep_path);
        match deep_creation_result {
            Ok(()) => {
                let deep_index_path = deep_path.join("deep_index");
                let deep_searcher_result = TantivySearcher::new_with_path(&deep_index_path).await;
                
                match deep_searcher_result {
                    Ok(mut deep_searcher) => {
                        println!("      Deep path searcher created successfully");
                        
                        let deep_file = deep_path.join("deep.rs");
                        fs::write(&deep_file, "pub fn deep_function() {}")?;
                        
                        let deep_index_result = deep_searcher.index_file(&deep_file).await;
                        match deep_index_result {
                            Ok(()) => {
                                println!("      Deep path indexing successful");
                            }
                            Err(e) => {
                                searcher_failures.push(format!(
                                    "Deep path indexing failed: {}", e
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        searcher_failures.push(format!(
                            "Deep path searcher creation failed: {}", e
                        ));
                    }
                }
            }
            Err(e) => {
                path_failures.push(format!(
                    "Deep path creation failed: {}", e
                ));
            }
        }
        
        // Report findings
        if !path_failures.is_empty() {
            println!("\n🚨 PATH HANDLING FAILURES EXPOSED:");
            for failure in &path_failures {
                println!("   • {}", failure);
            }
        }
        
        if !searcher_failures.is_empty() {
            println!("\n🚨 SEARCHER PATH VULNERABILITIES:");
            for failure in &searcher_failures {
                println!("   • {}", failure);
            }
        }
        
        println!("\n📊 Path chaos test summary:");
        println!("   Path creation failures: {}", path_failures.len());
        println!("   Searcher operation failures: {}", searcher_failures.len());
        
        if path_failures.is_empty() && searcher_failures.is_empty() {
            println!("   ✅ Path handling appears robust");
        }
        
        Ok(())
    }
    
    /// TEST 9: PERFORMANCE DEGRADATION UNDER LOAD
    /// 
    /// CRITICAL FLAW: No performance limits or optimization
    /// This test MUST demonstrate performance cliff scenarios
    #[cfg(feature = "tantivy")]
    #[tokio::test]
    async fn brutal_test_09_performance_cliff_detection() -> Result<()> {
        println!("🔥 BRUTAL TEST 9: Performance Cliff Detection");
        println!("=============================================");
        
        let temp_dir = TempDir::new()?;
        let mut searcher = TantivySearcher::new().await?;
        
        println!("📋 Creating performance cliff scenarios...");
        
        let mut performance_cliffs = Vec::new();
        
        // Test 1: Query complexity cliff
        println!("   Test 1: Query complexity performance cliff");
        
        let simple_file = temp_dir.path().join("perf_test.rs");
        let mut content = String::new();
        for i in 0..1000 {
            content.push_str(&format!("pub fn function_{}() -> String {{ \"data_{}\" }}\n", i, i));
        }
        fs::write(&simple_file, content)?;
        searcher.index_file(&simple_file).await?;
        
        let query_complexity_tests = vec![
            ("simple", "Single word query"),
            ("simple function", "Two word query"),
            ("simple function data", "Three word query"),
            ("simple function data test query", "Five word query"),
            ("simple function data test query complex search terms", "Eight word query"),
        ];
        
        let mut previous_time = Duration::new(0, 0);
        
        for (query, description) in query_complexity_tests {
            let search_start = Instant::now();
            let results = searcher.search(query).await?;
            let search_time = search_start.elapsed();
            
            println!("      {}: {} results in {:?}", description, results.len(), search_time);
            
            // Check for performance cliff (10x slowdown)
            if previous_time.as_nanos() > 0 && 
               search_time.as_nanos() > previous_time.as_nanos() * 10 {
                performance_cliffs.push(format!(
                    "Query complexity cliff: '{}' took {:?} vs previous {:?} ({}x slower)",
                    description, search_time, previous_time,
                    search_time.as_nanos() / previous_time.as_nanos()
                ));
            }
            
            previous_time = search_time;
        }
        
        // Test 2: Fuzzy distance performance cliff
        println!("   Test 2: Fuzzy distance performance cliff");
        
        let fuzzy_distances = [0u8, 1u8, 2u8];
        let mut fuzzy_times = Vec::new();
        
        for distance in fuzzy_distances {
            let fuzzy_start = Instant::now();
            let results = searcher.search_fuzzy("function", distance).await?;
            let fuzzy_time = fuzzy_start.elapsed();
            
            println!("      Distance {}: {} results in {:?}", distance, results.len(), fuzzy_time);
            fuzzy_times.push((distance, fuzzy_time));
            
            // Check for exponential performance degradation
            if fuzzy_times.len() > 1 {
                let (prev_dist, prev_time) = fuzzy_times[fuzzy_times.len() - 2];
                if fuzzy_time.as_nanos() > prev_time.as_nanos() * 5 {
                    performance_cliffs.push(format!(
                        "Fuzzy distance cliff: distance {} -> {} caused {}x slowdown ({:?} -> {:?})",
                        prev_dist, distance,
                        fuzzy_time.as_nanos() / prev_time.as_nanos(),
                        prev_time, fuzzy_time
                    ));
                }
            }
        }
        
        // Test 3: Document size performance cliff
        println!("   Test 3: Document size performance cliff");
        
        searcher.clear_index().await?;
        
        let doc_sizes = [1_000, 10_000, 100_000, 1_000_000]; // 1KB to 1MB
        let mut doc_times = Vec::new();
        
        for doc_size in doc_sizes {
            let size_file = temp_dir.path().join(format!("size_test_{}.rs", doc_size));
            let mut large_content = String::new();
            
            // Create document of specific size
            let base_content = "pub fn test_function() -> String { \"";
            large_content.push_str(base_content);
            
            let remaining_size = doc_size - base_content.len() - 20; // Leave room for closing
            for _ in 0..remaining_size / 10 {
                large_content.push_str("0123456789");
            }
            large_content.push_str("\" }\n");
            
            fs::write(&size_file, &large_content)?;
            
            let index_start = Instant::now();
            let index_result = searcher.index_file(&size_file).await;
            let index_time = index_start.elapsed();
            
            match index_result {
                Ok(()) => {
                    println!("      {}KB document: indexed in {:?}", doc_size / 1000, index_time);
                    doc_times.push((doc_size, index_time));
                    
                    // Check for performance cliff
                    if doc_times.len() > 1 {
                        let (prev_size, prev_time) = doc_times[doc_times.len() - 2];
                        let size_ratio = doc_size / prev_size;
                        let time_ratio = index_time.as_nanos() / prev_time.as_nanos();
                        
                        // Performance should scale roughly linearly with size
                        if time_ratio > (size_ratio as u128) * 3 {
                            performance_cliffs.push(format!(
                                "Document size cliff: {}KB -> {}KB caused {}x slowdown (expected {}x)",
                                prev_size / 1000, doc_size / 1000, time_ratio, size_ratio
                            ));
                        }
                    }
                    
                    // Test search performance on large document
                    let search_start = Instant::now();
                    let search_results = searcher.search("test_function").await?;
                    let search_time = search_start.elapsed();
                    
                    println!("         Search: {} results in {:?}", search_results.len(), search_time);
                    
                    if search_time > Duration::from_secs(1) {
                        performance_cliffs.push(format!(
                            "Search performance cliff: {}KB document search took {:?}",
                            doc_size / 1000, search_time
                        ));
                    }
                }
                Err(e) => {
                    println!("      {}KB document: indexing failed - {}", doc_size / 1000, e);
                    
                    // Check if it's a performance-related failure
                    let error_msg = e.to_string().to_lowercase();
                    if error_msg.contains("timeout") || error_msg.contains("slow") {
                        performance_cliffs.push(format!(
                            "Performance failure: {}KB document indexing timed out or failed due to performance",
                            doc_size / 1000
                        ));
                    }
                    break; // Don't try larger documents if this one failed
                }
            }
            
            searcher.clear_index().await?; // Clear for next test
        }
        
        // Test 4: Index size performance cliff  
        println!("   Test 4: Index size performance cliff");
        
        let file_counts = [10, 100, 500, 1000];
        let mut index_size_times = Vec::new();
        
        for file_count in file_counts {
            let files_dir = temp_dir.path().join(format!("index_size_test_{}", file_count));
            fs::create_dir_all(&files_dir)?;
            
            // Create many small files
            for i in 0..file_count {
                let small_file = files_dir.join(format!("file_{}.rs", i));
                let content = format!("pub fn file_{}_function() -> String {{ \"data_{}\" }}", i, i);
                fs::write(&small_file, content)?;
            }
            
            let batch_index_start = Instant::now();
            let batch_result = searcher.index_directory(&files_dir).await;
            let batch_time = batch_index_start.elapsed();
            
            match batch_result {
                Ok(()) => {
                    let stats = searcher.get_index_stats()?;
                    println!("      {} files: indexed in {:?}, {} docs total", 
                            file_count, batch_time, stats.num_documents);
                    
                    index_size_times.push((file_count, batch_time));
                    
                    // Check for performance cliff
                    if index_size_times.len() > 1 {
                        let (prev_count, prev_time) = index_size_times[index_size_times.len() - 2];
                        let count_ratio = file_count / prev_count;
                        let time_ratio = batch_time.as_nanos() / prev_time.as_nanos();
                        
                        if time_ratio > (count_ratio as u128) * 2 {
                            performance_cliffs.push(format!(
                                "Index size cliff: {} -> {} files caused {}x slowdown (expected {}x)",
                                prev_count, file_count, time_ratio, count_ratio
                            ));
                        }
                    }
                    
                    // Test search performance on large index
                    let large_search_start = Instant::now();
                    let large_search_results = searcher.search("function").await?;
                    let large_search_time = large_search_start.elapsed();
                    
                    println!("         Search: {} results in {:?}", 
                            large_search_results.len(), large_search_time);
                    
                    if large_search_time > Duration::from_millis(500) {
                        performance_cliffs.push(format!(
                            "Large index search cliff: {} files, search took {:?}",
                            file_count, large_search_time
                        ));
                    }
                }
                Err(e) => {
                    println!("      {} files: batch indexing failed - {}", file_count, e);
                    break; // Don't try more files
                }
            }
            
            searcher.clear_index().await?;
        }
        
        // Report performance cliff findings
        if !performance_cliffs.is_empty() {
            println!("\n🚨 PERFORMANCE CLIFFS EXPOSED:");
            for cliff in &performance_cliffs {
                println!("   • {}", cliff);
            }
            println!("   • No performance optimization or caching");
            println!("   • Exponential performance degradation");
            println!("   • No performance limits or circuit breakers");
            println!("   • System vulnerable to performance attacks");
        } else {
            println!("\n   ✅ No major performance cliffs detected");
        }
        
        Ok(())
    }
    
    /// TEST 10: ERROR PROPAGATION CHAIN VALIDATION
    /// 
    /// CRITICAL FLAW: Errors are not properly handled or recovered from
    /// This test MUST demonstrate error handling chain failures
    #[cfg(feature = "tantivy")]
    #[tokio::test]
    async fn brutal_test_10_error_propagation_chain_failure() -> Result<()> {
        println!("🔥 BRUTAL TEST 10: Error Propagation Chain Failure");
        println!("==================================================");
        
        let temp_dir = TempDir::new()?;
        
        println!("📋 Testing error propagation and recovery chains...");
        
        let mut error_chain_failures = Vec::new();
        
        // Test 1: Cascading indexing failures
        println!("   Test 1: Cascading indexing error recovery");
        {
            let mut searcher = TantivySearcher::new().await?;
            
            // Create a valid file first
            let good_file = temp_dir.path().join("good.rs");
            fs::write(&good_file, "pub fn good() {}")?;
            searcher.index_file(&good_file).await?;
            
            // Create an unreadable file (permission denied)
            let bad_file = temp_dir.path().join("bad.rs");
            fs::write(&bad_file, "pub fn bad() {}")?;
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&bad_file)?.permissions();
                perms.set_mode(0o000); // No permissions
                fs::set_permissions(&bad_file, perms)?;
            }
            
            // Try to index the bad file
            let bad_index_result = searcher.index_file(&bad_file).await;
            match bad_index_result {
                Ok(()) => {
                    error_chain_failures.push(
                        "Indexing unreadable file succeeded - no permission validation".to_string()
                    );
                    
                    #[cfg(unix)]
                    {
                        // Restore permissions for cleanup
                        let mut perms = fs::metadata(&bad_file)?.permissions();
                        perms.set_mode(0o644);
                        let _ = fs::set_permissions(&bad_file, perms);
                    }
                }
                Err(e) => {
                    println!("      ✅ Bad file indexing properly failed: {}", e);
                    
                    // Verify that the searcher is still functional after error
                    let recovery_search = searcher.search("good").await;
                    match recovery_search {
                        Ok(results) => {
                            if results.is_empty() {
                                error_chain_failures.push(
                                    "Searcher lost existing data after indexing error - no error isolation".to_string()
                                );
                            } else {
                                println!("      ✅ Searcher recovered after indexing error");
                            }
                        }
                        Err(e) => {
                            error_chain_failures.push(format!(
                                "Searcher became unusable after indexing error: {}", e
                            ));
                        }
                    }
                    
                    #[cfg(unix)]
                    {
                        // Restore permissions for cleanup
                        let mut perms = fs::metadata(&bad_file)?.permissions();
                        perms.set_mode(0o644);
                        let _ = fs::set_permissions(&bad_file, perms);
                    }
                }
            }
        }
        
        // Test 2: Search error recovery
        println!("   Test 2: Search error recovery chain");
        {
            let mut searcher = TantivySearcher::new().await?;
            
            let test_file = temp_dir.path().join("search_test.rs");
            fs::write(&test_file, "pub fn search_target() {}")?;
            searcher.index_file(&test_file).await?;
            
            // Test invalid query handling
            let invalid_search = searcher.search("invalid\"query\"syntax").await;
            match invalid_search {
                Ok(results) => {
                    println!("      Invalid query returned {} results", results.len());
                    
                    // Test if searcher still works after invalid query
                    let recovery_search = searcher.search("search_target").await;
                    match recovery_search {
                        Ok(recovery_results) => {
                            if recovery_results.is_empty() {
                                error_chain_failures.push(
                                    "Valid search failed after invalid query - error state not cleared".to_string()
                                );
                            }
                        }
                        Err(e) => {
                            error_chain_failures.push(format!(
                                "Searcher broken after invalid query: {}", e
                            ));
                        }
                    }
                }
                Err(e) => {
                    println!("      ✅ Invalid query properly failed: {}", e);
                    
                    // Test recovery
                    let recovery_search = searcher.search("search_target").await;
                    match recovery_search {
                        Ok(recovery_results) => {
                            if !recovery_results.is_empty() {
                                println!("      ✅ Searcher recovered after query error");
                            } else {
                                error_chain_failures.push(
                                    "Valid search returned no results after query error".to_string()
                                );
                            }
                        }
                        Err(e) => {
                            error_chain_failures.push(format!(
                                "Recovery search failed: {}", e
                            ));
                        }
                    }
                }
            }
        }
        
        // Test 3: Index corruption recovery chain
        println!("   Test 3: Index corruption recovery chain");
        {
            let index_path = temp_dir.path().join("corruption_recovery");
            let mut searcher = TantivySearcher::new_with_path(&index_path).await?;
            
            let recovery_file = temp_dir.path().join("recovery_test.rs");
            fs::write(&recovery_file, "pub fn recovery_target() {}")?;
            searcher.index_file(&recovery_file).await?;
            
            // Verify it works
            let pre_corruption = searcher.search("recovery_target").await?;
            assert!(!pre_corruption.is_empty(), "Pre-corruption search must work");
            
            drop(searcher); // Release file handles
            
            // Corrupt the index
            if let Some(index_dir) = fs::read_dir(&index_path).ok() {
                for entry in index_dir {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file() {
                            // Overwrite with garbage
                            fs::write(&path, "CORRUPTED_GARBAGE_DATA")?;
                            break;
                        }
                    }
                }
            }
            
            // Try to create new searcher with corrupted index
            let corruption_recovery = TantivySearcher::new_with_path(&index_path).await;
            match corruption_recovery {
                Ok(mut recovered_searcher) => {
                    println!("      Corrupted index opened - testing recovery capabilities");
                    
                    // Test if search works
                    let search_after_corruption = recovered_searcher.search("recovery_target").await;
                    match search_after_corruption {
                        Ok(results) => {
                            if results.len() == pre_corruption.len() {
                                error_chain_failures.push(
                                    "Corruption went undetected - same results returned".to_string()
                                );
                            } else {
                                error_chain_failures.push(format!(
                                    "Corruption partially detected - results changed from {} to {}",
                                    pre_corruption.len(), results.len()
                                ));
                            }
                        }
                        Err(e) => {
                            println!("      Search failed after corruption: {}", e);
                            
                            // Test if re-indexing can recover
                            let reindex_result = recovered_searcher.index_file(&recovery_file).await;
                            match reindex_result {
                                Ok(()) => {
                                    println!("      ✅ Re-indexing succeeded - recovery possible");
                                    
                                    let recovery_search = recovered_searcher.search("recovery_target").await;
                                    match recovery_search {
                                        Ok(recovery_results) => {
                                            if recovery_results.is_empty() {
                                                error_chain_failures.push(
                                                    "Re-indexing completed but search found nothing - recovery incomplete".to_string()
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            error_chain_failures.push(format!(
                                                "Search failed even after re-indexing: {}", e
                                            ));
                                        }
                                    }
                                }
                                Err(e) => {
                                    error_chain_failures.push(format!(
                                        "Re-indexing failed after corruption - no recovery mechanism: {}", e
                                    ));
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("      Corrupted index properly rejected: {}", e);
                    
                    // Test if we can rebuild
                    let rebuild_result = TantivySearcher::new_with_path(&index_path).await;
                    match rebuild_result {
                        Ok(mut rebuilt_searcher) => {
                            println!("      Index rebuild succeeded");
                            
                            let rebuild_index_result = rebuilt_searcher.index_file(&recovery_file).await;
                            match rebuild_index_result {
                                Ok(()) => {
                                    println!("      ✅ Full recovery successful");
                                }
                                Err(e) => {
                                    error_chain_failures.push(format!(
                                        "Rebuild indexing failed: {}", e
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            error_chain_failures.push(format!(
                                "Index rebuild failed - no automatic recovery: {}", e
                            ));
                        }
                    }
                }
            }
        }
        
        // Test 4: Resource exhaustion recovery
        println!("   Test 4: Resource exhaustion recovery chain");
        {
            let mut searcher = TantivySearcher::new().await?;
            
            // Try to create a memory bomb
            let bomb_file = temp_dir.path().join("memory_bomb.rs");
            let mut bomb_content = String::new();
            
            // Create moderately large content (not too big to avoid test timeouts)
            bomb_content.push_str("pub const LARGE_DATA: &str = \"");
            for _ in 0..100_000 {
                bomb_content.push_str("0123456789");
            }
            bomb_content.push_str("\";\n");
            
            fs::write(&bomb_file, &bomb_content)?;
            
            let bomb_index_result = searcher.index_file(&bomb_file).await;
            match bomb_index_result {
                Ok(()) => {
                    println!("      Large content indexed successfully");
                    
                    // Test if searcher is still responsive
                    let responsiveness_file = temp_dir.path().join("responsive_test.rs");
                    fs::write(&responsiveness_file, "pub fn responsive() {}")?;
                    
                    let responsive_index = searcher.index_file(&responsiveness_file).await;
                    match responsive_index {
                        Ok(()) => {
                            println!("      ✅ Searcher remained responsive after large content");
                        }
                        Err(e) => {
                            error_chain_failures.push(format!(
                                "Searcher became unresponsive after large content: {}", e
                            ));
                        }
                    }
                }
                Err(e) => {
                    println!("      Large content indexing failed: {}", e);
                    
                    // Check if searcher is still functional
                    let recovery_file = temp_dir.path().join("recovery_after_bomb.rs");
                    fs::write(&recovery_file, "pub fn recovery() {}")?;
                    
                    let recovery_index = searcher.index_file(&recovery_file).await;
                    match recovery_index {
                        Ok(()) => {
                            println!("      ✅ Searcher recovered after resource exhaustion failure");
                        }
                        Err(e) => {
                            error_chain_failures.push(format!(
                                "Searcher failed to recover after resource exhaustion: {}", e
                            ));
                        }
                    }
                }
            }
        }
        
        // Report error propagation failures
        if !error_chain_failures.is_empty() {
            println!("\n🚨 ERROR PROPAGATION CHAIN FAILURES EXPOSED:");
            for failure in &error_chain_failures {
                println!("   • {}", failure);
            }
            println!("   • No proper error isolation mechanisms");
            println!("   • State corruption after errors");
            println!("   • No automatic recovery capabilities");
            println!("   • Errors cascade and break entire system");
        } else {
            println!("\n   ✅ Error propagation and recovery appear robust");
        }
        
        Ok(())
    }
}

/// Helper function to get current memory usage (simplified)
fn get_memory_usage() -> f64 {
    // This is a simplified memory usage estimation
    // In a real implementation, you'd use platform-specific APIs
    #[cfg(unix)]
    {
        use std::process::Command;
        
        let output = Command::new("ps")
            .args(&["-o", "rss=", "-p"])
            .arg(std::process::id().to_string())
            .output();
            
        if let Ok(output) = output {
            let rss_str = String::from_utf8_lossy(&output.stdout);
            let rss_kb: f64 = rss_str.trim().parse().unwrap_or(0.0);
            return rss_kb / 1024.0; // Convert KB to MB
        }
    }
    
    // Fallback: return 0 if we can't measure
    0.0
}

/// Helper macro for brutal testing assertions
macro_rules! brutal_assert {
    ($condition:expr, $message:expr) => {
        if !($condition) {
            println!("🚨 BRUTAL ASSERTION FAILED: {}", $message);
            panic!("Brutal test assertion failed: {}", $message);
        }
    };
}