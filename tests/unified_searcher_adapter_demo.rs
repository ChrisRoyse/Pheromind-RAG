use std::path::Path;
use tempfile::TempDir;
use std::fs;

use embed_search::search::{TextSearcher, create_text_searcher};
use embed_search::config::{Config, SearchBackend};

/// Demonstration of how UnifiedSearcher could be enhanced to use the TextSearcher trait
/// This shows the concept without modifying the actual UnifiedSearcher yet
#[tokio::test]
async fn test_unified_searcher_adapter_concept() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("demo.rs");
    
    let content = r#"
fn authenticate_user(username: &str, password: &str) -> bool {
    // User authentication logic
    verify_credentials(username, password)
}

fn process_authentication_request(req: AuthRequest) -> AuthResponse {
    // Process authentication request
    if authenticate_user(&req.username, &req.password) {
        AuthResponse::success()
    } else {
        AuthResponse::failure()
    }
}
"#;
    
    fs::write(&test_file, content).unwrap();
    
    // Demonstrate switching between different backends
    let backends = vec![
        SearchBackend::Ripgrep,
        SearchBackend::Tantivy,
        SearchBackend::Auto,
    ];
    
    for backend in backends {
        println!("Testing with backend: {:?}", backend);
        
        // Create searcher based on config
        let mut searcher = create_text_searcher(&backend).await.unwrap();
        
        // Index the file
        searcher.index_file(&test_file).await.unwrap();
        
        // Search for terms
        let matches = searcher.search("authenticate").await.unwrap();
        
        // Verify we get results
        assert!(!matches.is_empty(), "Should find matches with {:?} backend", backend);
        
        // Verify the structure is consistent across backends
        for match_result in &matches {
            assert!(!match_result.file_path.is_empty(), "File path should not be empty");
            assert!(match_result.line_number > 0, "Line number should be positive");
            assert!(!match_result.content.is_empty(), "Content should not be empty");
            assert!(!match_result.line_content.is_empty(), "Line content should not be empty");
            assert!(
                match_result.content.contains("authenticate") || 
                match_result.line_content.contains("authenticate"),
                "Match should contain 'authenticate'"
            );
        }
        
        println!("✅ Backend {:?} found {} matches", backend, matches.len());
        
        // Clear the index
        searcher.clear_index().await.unwrap();
    }
}

/// Example of how a simplified UnifiedSearcher might look with the adapter
struct SimplifiedUnifiedSearcher {
    text_searcher: Box<dyn TextSearcher>,
}

impl SimplifiedUnifiedSearcher {
    pub async fn new(backend: SearchBackend) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let text_searcher = create_text_searcher(&backend).await?;
        Ok(Self { text_searcher })
    }
    
    pub async fn search(&mut self, query: &str) -> Result<Vec<embed_search::search::ExactMatch>, Box<dyn std::error::Error + Send + Sync>> {
        // This could be the simplified exact search part of UnifiedSearcher
        self.text_searcher.search(query).await.map_err(|e| e.into())
    }
    
    pub async fn index_file(&mut self, file_path: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.text_searcher.index_file(file_path).await.map_err(|e| e.into())
    }
    
    pub async fn clear_index(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.text_searcher.clear_index().await.map_err(|e| e.into())
    }
}

#[tokio::test]
async fn test_simplified_unified_searcher() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("unified_test.rs");
    
    let content = r#"
fn main() {
    println!("Starting authentication service");
    let auth_result = authenticate_user("admin", "password");
    println!("Authentication result: {}", auth_result);
}
"#;
    
    fs::write(&test_file, content).unwrap();
    
    // Test with Ripgrep backend
    let mut unified_searcher = SimplifiedUnifiedSearcher::new(SearchBackend::Ripgrep).await.unwrap();
    
    // Index and search
    unified_searcher.index_file(&test_file).await.unwrap();
    let matches = unified_searcher.search("authentication").await.unwrap();
    
    assert!(!matches.is_empty(), "Should find authentication matches");
    println!("Found {} matches with unified searcher", matches.len());
    
    // Test backend switching by creating a new searcher
    let mut unified_searcher_tantivy = SimplifiedUnifiedSearcher::new(SearchBackend::Tantivy).await.unwrap();
    unified_searcher_tantivy.index_file(&test_file).await.unwrap();
    let tantivy_matches = unified_searcher_tantivy.search("authentication").await.unwrap();
    
    assert!(!tantivy_matches.is_empty(), "Should find authentication matches with Tantivy");
    println!("Found {} matches with Tantivy backend", tantivy_matches.len());
}