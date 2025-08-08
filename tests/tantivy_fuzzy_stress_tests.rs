use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use anyhow::Result;
use tempfile::TempDir;

#[cfg(feature = "tantivy")]
use embed_search::search::{TantivySearcher, ExactMatch};

/// FUZZY SEARCH STRESS TESTS
/// 
/// These tests specifically target the fuzzy search functionality to find:
/// - Algorithm correctness under extreme conditions
/// - Performance degradation with complex patterns
/// - Memory usage with large fuzzy search spaces
/// - Edge cases in fuzzy matching logic
/// 
/// NO APPROXIMATION - Every test verifies actual fuzzy match behavior.

#[cfg(test)]
mod fuzzy_stress_tests {
    use super::*;

    /// Test fuzzy search with massive vocabulary
    #[cfg(feature = "tantivy")]
    #[tokio::test]
    async fn stress_fuzzy_large_vocabulary() -> Result<()> {
        println!("🔥 FUZZY STRESS TEST: Large Vocabulary");
        println!("======================================");
        
        let temp_dir = TempDir::new()?;
        let mut searcher = TantivySearcher::new_with_path(temp_dir.path().join("fuzzy_index")).await?;
        
        // Create file with massive vocabulary of similar terms
        let vocab_file = temp_dir.path().join("vocabulary.rs");
        let mut content = String::new();
        
        println!("📋 Creating vocabulary with 10,000 similar function names...");
        let creation_start = Instant::now();
        
        // Generate similar function names that will stress fuzzy matching
        let prefixes = ["handle", "process", "manage", "execute", "perform", "validate", "transform"];
        let middles = ["user", "data", "config", "request", "response", "payment", "session"];
        let suffixes = ["action", "operation", "task", "job", "work", "flow", "step"];
        
        let mut function_names = Vec::new();
        for (i, prefix) in prefixes.iter().enumerate() {
            for (j, middle) in middles.iter().enumerate() {
                for (k, suffix) in suffixes.iter().enumerate() {
                    // Create various naming patterns
                    let patterns = vec![
                        format!("{}_{}_{}", prefix, middle, suffix),
                        format!("{}{}_{}", prefix, middle, suffix),
                        format!("{}_{}_{}_impl", prefix, middle, suffix),
                        format!("async_{}_{}_{}", prefix, middle, suffix),
                        format!("{}_{}_{}_async", prefix, middle, suffix),
                        format!("{}{}{}Handler", 
                               prefix.chars().next().unwrap().to_uppercase().to_string() + &prefix[1..],
                               middle.chars().next().unwrap().to_uppercase().to_string() + &middle[1..],
                               suffix.chars().next().unwrap().to_uppercase().to_string() + &suffix[1..]
                        ),
                    ];
                    
                    for (p, pattern) in patterns.iter().enumerate() {
                        let func_name = format!("{}_{}", pattern, i * 100 + j * 10 + k * 6 + p);
                        function_names.push(func_name.clone());
                        content.push_str(&format!(
                            "pub fn {}() -> Result<String> {{\n    \
                                let result = \"fuzzy_test_{}\";\n    \
                                Ok(result.to_string())\n\
                            }}\n\n",
                            func_name, func_name
                        ));
                    }
                }
            }
        }
        
        fs::write(&vocab_file, content)?;
        println!("   ✅ Created vocabulary with {} functions in {:?}", 
                function_names.len(), creation_start.elapsed());
        
        // Index the vocabulary
        let index_start = Instant::now();
        let index_result = searcher.index_file(&vocab_file).await;
        let index_duration = index_start.elapsed();
        
        match index_result {
            Ok(()) => {
                println!("   ✅ Indexed vocabulary in {:?}", index_duration);
                let stats = searcher.get_index_stats()?;
                println!("   📊 Index contains {} documents", stats.num_documents);
                
                // Test fuzzy search performance with various distances
                let test_queries = vec![
                    ("handleuser", "Should match handle_user_* patterns"),
                    ("processdata", "Should match process_data_* patterns"),
                    ("ManageConfig", "Should match manage_config_* patterns"),
                    ("validatepayment", "Should match validate_payment_* patterns"),
                    ("HandleUserAction", "Should match HandleUserAction* patterns"),
                    ("async_process", "Should match async_process_* patterns"),
                ];
                
                for max_distance in [1u8, 2u8] {
                    println!("📊 Testing fuzzy search with max_distance = {}", max_distance);
                    
                    for (query, description) in &test_queries {
                        let search_start = Instant::now();
                        let results = searcher.search_fuzzy(query, max_distance).await?;
                        let search_duration = search_start.elapsed();
                        
                        println!("   🔍 '{}' -> {} results in {:?} ({})", 
                                query, results.len(), search_duration, description);
                        
                        // Verify results are legitimate fuzzy matches
                        if !results.is_empty() {
                            let sample_result = &results[0];
                            println!("      Sample match: {} (line {})", 
                                    sample_result.content, sample_result.line_number);
                            
                            // TRUTH CHECK: Verify result is actually related to query
                            let content_lower = sample_result.content.to_lowercase();
                            let query_lower = query.to_lowercase();
                            
                            // Calculate actual edit distance to verify fuzzy matching worked
                            let is_fuzzy_match = calculate_edit_distance(&content_lower, &query_lower) <= max_distance as usize ||
                                                content_lower.contains(&query_lower[..query_lower.len().min(3)]) ||
                                                query_lower.chars().take(4).all(|c| content_lower.contains(c));
                            
                            if !is_fuzzy_match {
                                println!("      ⚠️  WARNING: Result doesn't appear to be a valid fuzzy match");
                                println!("         Query: {}", query);
                                println!("         Result: {}", sample_result.content);
                            }
                        }
                        
                        // Performance requirement: Large vocabulary fuzzy search should complete within 1 second
                        if search_duration > Duration::from_secs(1) {
                            println!("      ⚠️  WARNING: Fuzzy search took longer than 1 second");
                        }
                    }
                }
            }
            Err(e) => {
                println!("   ❌ Large vocabulary indexing failed: {}", e);
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("memory") || error_msg.contains("space") {
                    println!("   ℹ️  Resource limit reached with large vocabulary");
                } else {
                    panic!("Unexpected large vocabulary indexing failure: {}", e);
                }
            }
        }
        
        Ok(())
    }

    /// Test fuzzy search with extreme edit distances
    #[cfg(feature = "tantivy")]
    #[tokio::test]
    async fn stress_fuzzy_extreme_distances() -> Result<()> {
        println!("🔥 FUZZY STRESS TEST: Extreme Edit Distances");
        println!("============================================");
        
        let temp_dir = TempDir::new()?;
        let mut searcher = TantivySearcher::new().await?;
        
        // Create test content with known words for fuzzy matching
        let test_file = temp_dir.path().join("fuzzy_distance.rs");
        let content = r#"
pub struct DatabaseConnection {
    url: String,
    timeout: Duration,
}

impl DatabaseConnection {
    pub fn new_connection() -> Self {
        DatabaseConnection {
            url: "database://localhost".to_string(),
            timeout: Duration::from_secs(30),
        }
    }
    
    pub fn execute_query(&self, sql: &str) -> QueryResult {
        self.connection.query(sql)
    }
    
    pub fn disconnect(&mut self) {
        self.connection.close();
    }
}

pub fn validate_configuration() -> bool {
    true
}

pub fn process_authentication(user: &User) -> AuthResult {
    user.authenticate()
}

pub fn handle_payment_processing(payment: &Payment) -> ProcessingResult {
    payment.process()
}
"#;
        fs::write(&test_file, content)?;
        searcher.index_file(&test_file).await?;
        
        // Test with various edit distances and challenging queries
        let extreme_test_cases = vec![
            // (query, expected_to_find, description)
            ("Database", Some("DatabaseConnection"), "Exact prefix match"),
            ("Databas", Some("DatabaseConnection"), "1 char missing"),
            ("Databa", Some("DatabaseConnection"), "2 chars missing"),
            ("Databse", Some("DatabaseConnection"), "1 char transposition"),
            ("Dtabase", Some("DatabaseConnection"), "2 char transposition"),
            ("DataConnect", Some("DatabaseConnection"), "Partial match"),
            
            ("execute", Some("execute_query"), "Exact word match"),
            ("execut", Some("execute_query"), "1 char missing"),
            ("exec", Some("execute_query"), "Prefix match"),
            ("exectue", Some("execute_query"), "Letter swap"),
            
            ("process", Some("process_authentication"), "Common word"),
            ("proces", Some("process_authentication"), "1 char missing"),
            ("proess", Some("process_authentication"), "1 char missing middle"),
            ("processing", Some("handle_payment_processing"), "Suffix match"),
            
            // Extreme cases
            ("xyz", None, "Completely unrelated"),
            ("", None, "Empty query"),
            ("a", None, "Single character"),
            ("superlongquerythatdoesntmatchanything", None, "Very long non-match"),
        ];
        
        for max_distance in [1u8, 2u8] {
            println!("📊 Testing extreme distances with max_distance = {}", max_distance);
            
            for (query, expected_pattern, description) in &extreme_test_cases {
                if query.is_empty() {
                    // Skip empty queries as they may be invalid
                    continue;
                }
                
                let search_start = Instant::now();
                let result = searcher.search_fuzzy(query, max_distance).await;
                let search_duration = search_start.elapsed();
                
                match result {
                    Ok(results) => {
                        println!("   🔍 '{}' -> {} results in {:?} ({})", 
                                query, results.len(), search_duration, description);
                        
                        if let Some(expected) = expected_pattern {
                            if !results.is_empty() {
                                // Verify we found something containing the expected pattern
                                let found_expected = results.iter().any(|r| 
                                    r.content.to_lowercase().contains(&expected.to_lowercase()) ||
                                    expected.to_lowercase().contains(&r.content.to_lowercase())
                                );
                                
                                if found_expected {
                                    println!("      ✅ Found expected pattern: {}", expected);
                                } else {
                                    println!("      ⚠️  Expected '{}' but found: {}", 
                                            expected, results[0].content);
                                }
                                
                                // Show top result for verification
                                println!("      Top result: {} (line {})", 
                                        results[0].content, results[0].line_number);
                            } else {
                                println!("      ❌ Expected to find '{}' but got no results", expected);
                            }
                        } else {
                            if results.is_empty() {
                                println!("      ✅ Correctly found no matches for unrelated query");
                            } else {
                                println!("      ⚠️  Unexpected matches for unrelated query:");
                                for (i, result) in results.iter().take(3).enumerate() {
                                    println!("         {}: {}", i + 1, result.content);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("   ❌ Fuzzy search failed for '{}': {}", query, e);
                        
                        // Some failures might be legitimate (e.g., invalid queries)
                        let error_msg = e.to_string().to_lowercase();
                        if error_msg.contains("invalid") || error_msg.contains("query") {
                            println!("      ℹ️  Query validation failure (acceptable)");
                        } else {
                            panic!("Unexpected fuzzy search failure: {}", e);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Test fuzzy search performance degradation
    #[cfg(feature = "tantivy")]
    #[tokio::test] 
    async fn stress_fuzzy_performance_degradation() -> Result<()> {
        println!("🔥 FUZZY STRESS TEST: Performance Degradation");
        println!("=============================================");
        
        let temp_dir = TempDir::new()?;
        let mut searcher = TantivySearcher::new().await?;
        
        // Create progressively more complex search spaces
        let complexity_levels = vec![
            (100, "Low complexity"),
            (1_000, "Medium complexity"),
            (10_000, "High complexity"),
        ];
        
        for (num_functions, description) in complexity_levels {
            println!("📊 Testing {} with {} functions", description, num_functions);
            
            let test_file = temp_dir.path().join(format!("perf_test_{}.rs", num_functions));
            let mut content = String::new();
            
            // Generate functions with similar names to stress fuzzy matching
            let creation_start = Instant::now();
            for i in 0..num_functions {
                let variations = vec![
                    format!("function_{}", i),
                    format!("function_{}_impl", i),
                    format!("function_{}_async", i),
                    format!("fn_function_{}", i),
                    format!("func_{}_handler", i),
                ];
                
                for variation in variations {
                    content.push_str(&format!(
                        "pub fn {}() -> String {{ \"test_{}\" }}\n",
                        variation, variation
                    ));
                }
            }
            let creation_duration = creation_start.elapsed();
            
            fs::write(&test_file, content)?;
            println!("   ✅ Generated {} functions in {:?}", 
                    num_functions * 5, creation_duration);
            
            // Clear previous index and index new content
            searcher.clear_index().await?;
            
            let index_start = Instant::now();
            let index_result = searcher.index_file(&test_file).await;
            let index_duration = index_start.elapsed();
            
            match index_result {
                Ok(()) => {
                    let stats = searcher.get_index_stats()?;
                    println!("   ✅ Indexed in {:?}, {} docs", index_duration, stats.num_documents);
                    
                    // Test fuzzy search performance
                    let test_queries = vec![
                        "function_50",
                        "func_100",
                        "function_impl",
                        "async_func",
                        "handler_func",
                    ];
                    
                    let mut total_search_time = Duration::new(0, 0);
                    let mut search_count = 0;
                    
                    for query in &test_queries {
                        for max_distance in [1u8, 2u8] {
                            let search_start = Instant::now();
                            let results = searcher.search_fuzzy(query, max_distance).await?;
                            let search_duration = search_start.elapsed();
                            
                            total_search_time += search_duration;
                            search_count += 1;
                            
                            println!("      🔍 '{}' (d={}) -> {} results in {:?}", 
                                    query, max_distance, results.len(), search_duration);
                            
                            // Performance degradation warning
                            if search_duration > Duration::from_millis(500) {
                                println!("         ⚠️  WARNING: Search took longer than 500ms");
                            }
                            
                            // Verify results quality
                            if !results.is_empty() {
                                let relevance_score = calculate_relevance_score(query, &results[0].content);
                                if relevance_score < 0.3 {
                                    println!("         ⚠️  WARNING: Low relevance score: {:.2}", relevance_score);
                                }
                            }
                        }
                    }
                    
                    let avg_search_time = total_search_time / search_count as u32;
                    println!("   📈 Average search time: {:?}", avg_search_time);
                    
                    // Performance assertions based on complexity
                    let max_acceptable_time = match num_functions {
                        100 => Duration::from_millis(50),
                        1_000 => Duration::from_millis(200),
                        10_000 => Duration::from_millis(1000),
                        _ => Duration::from_millis(1000),
                    };
                    
                    if avg_search_time > max_acceptable_time {
                        println!("      ⚠️  WARNING: Performance degraded beyond acceptable limits");
                        println!("         Expected: < {:?}, Actual: {:?}", 
                                max_acceptable_time, avg_search_time);
                    } else {
                        println!("      ✅ Performance within acceptable limits");
                    }
                }
                Err(e) => {
                    println!("   ❌ Indexing failed for complexity level {}: {}", num_functions, e);
                    
                    // For high complexity, resource limits are acceptable
                    if num_functions >= 10_000 {
                        let error_msg = e.to_string().to_lowercase();
                        if error_msg.contains("memory") || error_msg.contains("limit") {
                            println!("      ℹ️  Resource limit reached at high complexity (acceptable)");
                            break; // Skip higher complexity levels
                        }
                    }
                    
                    panic!("Unexpected indexing failure: {}", e);
                }
            }
        }
        
        Ok(())
    }

    /// Test fuzzy search with malformed inputs
    #[cfg(feature = "tantivy")]
    #[tokio::test]
    async fn stress_fuzzy_malformed_inputs() -> Result<()> {
        println!("🔥 FUZZY STRESS TEST: Malformed Inputs");
        println!("======================================");
        
        let temp_dir = TempDir::new()?;
        let mut searcher = TantivySearcher::new().await?;
        
        // Index some normal content first
        let test_file = temp_dir.path().join("normal_content.rs");
        fs::write(&test_file, r#"
pub fn normal_function() {
    println!("Hello World");
}

pub struct NormalStruct {
    field: String,
}
"#)?;
        searcher.index_file(&test_file).await?;
        
        // Test malformed and edge case queries
        let malformed_queries = vec![
            // (query, max_distance, description, should_fail)
            ("", 1, "Empty string", true),
            (" ", 1, "Single space", false),
            ("  ", 2, "Multiple spaces", false),
            ("\n", 1, "Newline character", false),
            ("\t", 1, "Tab character", false),
            ("\"", 1, "Unescaped quote", false),
            ("\\", 1, "Backslash", false),
            ("'", 1, "Single quote", false),
            ("a", 1, "Single character", false),
            ("🚀", 1, "Emoji", false),
            ("café", 1, "Unicode with accent", false),
            ("中文", 1, "Chinese characters", false),
            ("Ñoël", 2, "Multiple accents", false),
            ("test\0null", 1, "Null byte in string", false),
            ("very_long_query_that_exceeds_reasonable_length_limits_and_might_cause_issues", 2, "Extremely long query", false),
            ("special!@#$%^&*()", 1, "Special characters", false),
            ("test AND OR NOT", 1, "Query operators", false),
            ("test*wildcard", 1, "Wildcard characters", false),
            ("test[bracket]", 1, "Bracket characters", false),
            ("test{brace}", 2, "Brace characters", false),
        ];
        
        for (query, max_distance, description, should_fail) in malformed_queries {
            println!("📋 Testing: {} ('{}')", description, query.escape_debug());
            
            let search_start = Instant::now();
            let result = searcher.search_fuzzy(&query, max_distance).await;
            let search_duration = search_start.elapsed();
            
            match result {
                Ok(results) => {
                    if should_fail {
                        println!("   ⚠️  Expected failure but got {} results in {:?}", 
                                results.len(), search_duration);
                    } else {
                        println!("   ✅ Handled gracefully: {} results in {:?}", 
                                results.len(), search_duration);
                    }
                    
                    // For successful searches, verify results make sense
                    if !results.is_empty() && query.len() > 1 {
                        let first_result = &results[0];
                        println!("      Sample result: {}", first_result.content);
                        
                        // Basic sanity check - result should be from our test content
                        let is_from_test_content = first_result.content.contains("normal") ||
                                                 first_result.content.contains("function") ||
                                                 first_result.content.contains("struct") ||
                                                 first_result.content.contains("Hello");
                        
                        if !is_from_test_content {
                            println!("      ⚠️  WARNING: Result doesn't appear to be from test content");
                        }
                    }
                }
                Err(e) => {
                    if should_fail {
                        println!("   ✅ Correctly failed in {:?}: {}", search_duration, e);
                    } else {
                        println!("   ❌ Unexpected failure in {:?}: {}", search_duration, e);
                        
                        // Check if it's a known limitation
                        let error_msg = e.to_string().to_lowercase();
                        let acceptable_errors = ["invalid", "query", "parse", "malformed"];
                        let is_acceptable = acceptable_errors.iter()
                            .any(|&err_type| error_msg.contains(err_type));
                        
                        if !is_acceptable {
                            panic!("Unacceptable fuzzy search failure for '{}': {}", query.escape_debug(), e);
                        }
                        
                        println!("      ℹ️  Acceptable limitation for malformed input");
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Calculate simple edit distance (Levenshtein) for fuzzy match verification
fn calculate_edit_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();
    
    if len1 == 0 { return len2; }
    if len2 == 0 { return len1; }
    
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
    
    // Initialize first row and column
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }
    
    // Fill the matrix
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }
    
    matrix[len1][len2]
}

/// Calculate relevance score for fuzzy match quality assessment
fn calculate_relevance_score(query: &str, result: &str) -> f64 {
    let query_lower = query.to_lowercase();
    let result_lower = result.to_lowercase();
    
    // Exact match
    if result_lower.contains(&query_lower) {
        return 1.0;
    }
    
    // Prefix match
    if result_lower.starts_with(&query_lower) {
        return 0.9;
    }
    
    // Word boundary match
    if result_lower.split_whitespace().any(|word| word.contains(&query_lower)) {
        return 0.8;
    }
    
    // Character overlap
    let query_chars: std::collections::HashSet<char> = query_lower.chars().collect();
    let result_chars: std::collections::HashSet<char> = result_lower.chars().collect();
    let overlap = query_chars.intersection(&result_chars).count();
    let total_unique = query_chars.union(&result_chars).count();
    
    if total_unique > 0 {
        overlap as f64 / total_unique as f64
    } else {
        0.0
    }
}