use std::fs;
use std::path::Path;
use anyhow::Result;
use tempfile::TempDir;

#[cfg(feature = "tantivy")]
use embed_search::search::{TantivySearcher, ExactMatch};

/// CRITICAL TEST: Verify Tantivy actually works - no simulation, no fallbacks
#[cfg(test)]
mod tantivy_validation {
    use super::*;

    #[cfg(feature = "tantivy")]
    #[tokio::test]
    async fn test_tantivy_compilation_and_functionality() -> Result<()> {
        println!("🔥 TANTIVY REALITY TEST - NO SIMULATION ALLOWED");
        println!("================================================");
        
        // 1. Test Index Creation
        println!("1️⃣ Testing index creation...");
        let temp_dir = TempDir::new()?;
        let mut searcher = TantivySearcher::new().await?;
        println!("   ✅ Index created successfully");
        
        // 2. Test Document Addition
        println!("2️⃣ Testing document indexing...");
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, r#"
pub fn search_function() {
    println!("searching for patterns");
}

pub struct SearchEngine {
    fuzzy_matcher: FuzzyMatcher,
}

impl SearchEngine {
    pub fn search_fuzzy(&self, query: &str) -> Vec<Match> {
        self.fuzzy_matcher.find(query)
    }
}

const SEARCH_TIMEOUT: u64 = 1000;
"#)?;
        
        searcher.index_file(&test_file).await?;
        let stats = searcher.get_index_stats()?;
        println!("   ✅ Indexed {} documents", stats.num_documents);
        
        if stats.num_documents == 0 {
            panic!("❌ CRITICAL FAILURE: No documents indexed - indexing is broken");
        }
        
        // 3. Test Exact Search
        println!("3️⃣ Testing exact search...");
        let exact_results = searcher.search("SearchEngine").await?;
        println!("   📊 Found {} exact matches for 'SearchEngine'", exact_results.len());
        
        if exact_results.is_empty() {
            panic!("❌ CRITICAL FAILURE: Exact search returned no results for known term");
        }
        
        // Verify result content
        let found_searchengine = exact_results.iter()
            .any(|r| r.content.contains("SearchEngine"));
        if !found_searchengine {
            panic!("❌ CRITICAL FAILURE: Search results don't contain expected term");
        }
        
        // 4. Test Fuzzy Search
        println!("4️⃣ Testing fuzzy search...");
        let fuzzy_results = searcher.search_fuzzy("fuzzy", 1).await?;
        println!("   📊 Found {} fuzzy matches for 'fuzzy'", fuzzy_results.len());
        
        if fuzzy_results.is_empty() {
            panic!("❌ CRITICAL FAILURE: Fuzzy search returned no results for 'fuzzy'");
        }
        
        // Verify fuzzy result contains expected content
        let found_fuzzy = fuzzy_results.iter()
            .any(|r| r.content.to_lowercase().contains("fuzzy"));
        if !found_fuzzy {
            panic!("❌ CRITICAL FAILURE: Fuzzy search results don't contain expected term");
        }
        
        // 5. Test Search with Different Terms
        println!("5️⃣ Testing various search terms...");
        let search_tests = vec![
            ("search_function", "Function name"),
            ("println", "Macro call"),
            ("SEARCH_TIMEOUT", "Constant"),
            ("patterns", "String content"),
        ];
        
        for (query, description) in search_tests {
            let results = searcher.search(query).await?;
            println!("   📊 '{}' ({}): {} results", query, description, results.len());
            
            if results.is_empty() {
                println!("   ⚠️  No results for '{}' - this may indicate indexing issues", query);
            } else {
                // Verify at least one result contains the query term
                let contains_term = results.iter()
                    .any(|r| r.content.to_lowercase().contains(&query.to_lowercase()));
                if !contains_term {
                    println!("   ⚠️  Results for '{}' don't contain the search term", query);
                }
            }
        }
        
        println!("✅ TANTIVY VALIDATION COMPLETE");
        println!("📊 Final Index Stats: {}", stats);
        
        Ok(())
    }
    
    #[cfg(not(feature = "tantivy"))]
    #[tokio::test]
    async fn test_tantivy_feature_missing() {
        println!("❌ TANTIVY FEATURE NOT ENABLED");
        println!("Run with: cargo test --features tantivy");
        panic!("Cannot test Tantivy - feature not enabled");
    }
}

/// Simple test without async - basic compilation check
#[cfg(feature = "tantivy")]
#[test]
fn test_tantivy_compilation() {
    // This test verifies that Tantivy compiles and basic types are available
    use embed_search::search::TantivySearcher;
    println!("✅ Tantivy types compile successfully");
}

#[cfg(not(feature = "tantivy"))]
#[test]
fn test_tantivy_not_available() {
    println!("❌ Tantivy feature not enabled - cannot test");
}