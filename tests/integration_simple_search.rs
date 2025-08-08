// Integration tests for SimpleSearcher
// Verifies actual working functionality

use embed_search::search::{SearchConfig, SimpleSearcher};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_simple_search_basic_functionality() {
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    // Create test files with known content
    std::fs::write(
        project_path.join("important.txt"),
        "This is a very important document about authentication and security."
    ).unwrap();
    
    std::fs::write(
        project_path.join("readme.md"),
        "# Project README\nThis project implements user authentication."
    ).unwrap();
    
    std::fs::write(
        project_path.join("config.toml"),
        "[security]\nenable_authentication = true\npassword_min_length = 8"
    ).unwrap();
    
    // Create searcher with minimal config
    let config = SearchConfig::minimal();
    let mut searcher = SimpleSearcher::new(config).await.unwrap();
    
    // Index the project
    searcher.index_project(&project_path.to_path_buf()).await.unwrap();
    
    // Test search for "authentication"
    let results = searcher.search("authentication").await.unwrap();
    
    // Verify results
    assert!(results.len() > 0, "Should find results for 'authentication'");
    
    // Check that relevant files are found
    let file_paths: Vec<String> = results.iter().map(|r| r.path.clone()).collect();
    assert!(
        file_paths.iter().any(|p| p.contains("important.txt")),
        "Should find important.txt"
    );
    assert!(
        file_paths.iter().any(|p| p.contains("readme.md")),
        "Should find readme.md"
    );
}

#[tokio::test]
async fn test_empty_search_returns_empty() {
    let config = SearchConfig::minimal();
    let searcher = SimpleSearcher::new(config).await.unwrap();
    
    // Search without indexing should return empty results
    let results = searcher.search("anything").await.unwrap();
    assert_eq!(results.len(), 0, "Empty index should return no results");
}

#[tokio::test]
async fn test_search_scoring_order() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    // Create files with different relevance
    std::fs::write(
        project_path.join("exact_match.txt"),
        "authentication authentication authentication"
    ).unwrap();
    
    std::fs::write(
        project_path.join("partial_match.txt"),
        "This mentions authentication once"
    ).unwrap();
    
    std::fs::write(
        project_path.join("no_match.txt"),
        "This file is about something completely different"
    ).unwrap();
    
    let config = SearchConfig::minimal();
    let mut searcher = SimpleSearcher::new(config).await.unwrap();
    searcher.index_project(&project_path.to_path_buf()).await.unwrap();
    
    let results = searcher.search("authentication").await.unwrap();
    
    // Verify scoring order
    assert!(results.len() >= 2, "Should find at least 2 matches");
    assert!(
        results[0].path.contains("exact_match"),
        "File with most occurrences should rank first"
    );
    assert!(
        results[0].score > results[1].score,
        "First result should have higher score"
    );
}

#[tokio::test]
async fn test_multi_term_search() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    std::fs::write(
        project_path.join("both_terms.txt"),
        "User authentication is important for security"
    ).unwrap();
    
    std::fs::write(
        project_path.join("one_term.txt"),
        "This is about user management"
    ).unwrap();
    
    std::fs::write(
        project_path.join("other_term.txt"),
        "Authentication mechanisms vary"
    ).unwrap();
    
    let config = SearchConfig::minimal();
    let mut searcher = SimpleSearcher::new(config).await.unwrap();
    searcher.index_project(&project_path.to_path_buf()).await.unwrap();
    
    let results = searcher.search("user authentication").await.unwrap();
    
    assert!(results.len() >= 1, "Should find results");
    // File with both terms should score higher
    assert!(
        results[0].path.contains("both_terms"),
        "File with both terms should rank highest"
    );
}

#[tokio::test]
async fn test_case_insensitive_search() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();
    
    std::fs::write(
        project_path.join("mixed_case.txt"),
        "Authentication AUTHENTICATION authentication"
    ).unwrap();
    
    let config = SearchConfig::minimal();
    let mut searcher = SimpleSearcher::new(config).await.unwrap();
    searcher.index_project(&project_path.to_path_buf()).await.unwrap();
    
    // Test different case variations
    for query in &["authentication", "AUTHENTICATION", "Authentication"] {
        let results = searcher.search(query).await.unwrap();
        assert!(
            results.len() > 0,
            "Should find results regardless of case: {}",
            query
        );
    }
}

#[tokio::test]
async fn test_graceful_degradation() {
    // Create config with unavailable features
    let config = SearchConfig {
        enable_bm25: true,
        enable_tantivy: true, // Will fail to init if not compiled
        enable_ml: true, // Will fail due to compilation issues
        enable_tree_sitter: true, // May not be available
        index_path: PathBuf::from(".test_index"),
    };
    
    // Should still create with at least BM25
    let searcher = SimpleSearcher::new(config);
    assert!(searcher.is_ok(), "Should create with fallback to BM25");
    
    let searcher = searcher.unwrap();
    assert!(
        searcher.available_engines().contains("bm25"),
        "BM25 should always be available"
    );
}