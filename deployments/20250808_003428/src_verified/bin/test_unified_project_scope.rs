use embed_search::search::unified::UnifiedSearcher;
use embed_search::config::{SearchBackend, Config};
use std::fs;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔍 Testing UnifiedSearcher with project scoping");
    
    // Initialize config
    Config::init().expect("Failed to initialize config");
    
    // Create two separate temporary directories
    let project1_dir = TempDir::new().expect("Failed to create temp directory for project 1");
    let project2_dir = TempDir::new().expect("Failed to create temp directory for project 2");
    
    let project1_path = project1_dir.path().to_path_buf();
    let project2_path = project2_dir.path().to_path_buf();
    let db1_path = project1_path.join("search.db");
    let db2_path = project2_path.join("search.db");
    
    println!("Project 1 path: {:?}", project1_path);
    println!("Project 2 path: {:?}", project2_path);
    
    // Create test files in project 1
    let file1_p1 = project1_path.join("authentication.rs");
    fs::write(&file1_p1, r#"
pub struct AuthService {
    pub name: String,
}

impl AuthService {
    pub fn authenticate_user(&self, username: &str) -> bool {
        println!("Authenticating {} in project 1", username);
        username.len() > 0
    }
    
    pub fn validate_session(&self, token: &str) -> bool {
        println!("Validating session in project 1");
        token.len() > 10
    }
}
"#)?;
    
    // Create test files in project 2
    let file1_p2 = project2_path.join("authentication.rs");
    fs::write(&file1_p2, r#"
pub struct AuthManager {
    pub config: String,
}

impl AuthManager {
    pub fn authenticate_user(&self, user_id: u32) -> bool {
        println!("Authenticating user {} in project 2", user_id);
        user_id > 0
    }
    
    pub fn refresh_token(&self, old_token: &str) -> String {
        println!("Refreshing token in project 2");
        format!("new-{}", old_token)
    }
}
"#)?;
    
    // Create UnifiedSearchers with specific project roots
    let searcher1 = UnifiedSearcher::new_with_backend(project1_path.clone(), db1_path, SearchBackend::Tantivy).await
        .expect("Failed to create UnifiedSearcher for project 1");
    
    let searcher2 = UnifiedSearcher::new_with_backend(project2_path.clone(), db2_path, SearchBackend::Tantivy).await
        .expect("Failed to create UnifiedSearcher for project 2");
    
    println!("✅ Created UnifiedSearchers for both projects");
    
    // Index files in their respective projects
    searcher1.index_file(&file1_p1).await.expect("Failed to index file in project 1");
    searcher2.index_file(&file1_p2).await.expect("Failed to index file in project 2");
    
    println!("✅ Indexed files in both projects");
    
    // Test searches in each project
    let results1 = searcher1.search("authenticate_user").await.expect("Search failed in project 1");
    let results2 = searcher2.search("authenticate_user").await.expect("Search failed in project 2");
    
    println!("Project 1 UnifiedSearcher results: {} found", results1.len());
    for result in &results1 {
        println!("  - Found in: {}", result.file);
    }
    
    println!("Project 2 UnifiedSearcher results: {} found", results2.len());
    for result in &results2 {
        println!("  - Found in: {}", result.file);
    }
    
    // Verify that results only contain files from their respective projects
    let all_results_in_project1 = results1.iter().all(|result| {
        let file_path = std::path::Path::new(&result.file);
        file_path.starts_with(&project1_path)
    });
    
    let all_results_in_project2 = results2.iter().all(|result| {
        let file_path = std::path::Path::new(&result.file);
        file_path.starts_with(&project2_path)
    });
    
    println!("\n📊 Verification results:");
    println!("All Project 1 UnifiedSearcher results are in Project 1 directory: {}", all_results_in_project1);
    println!("All Project 2 UnifiedSearcher results are in Project 2 directory: {}", all_results_in_project2);
    
    // Test project-specific functions
    let validate_results = searcher1.search("validate_session").await.expect("Search failed");
    let refresh_results = searcher2.search("refresh_token").await.expect("Search failed");
    
    println!("\nProject-specific function tests:");
    println!("Project 1 'validate_session' results: {}", validate_results.len());
    println!("Project 2 'refresh_token' results: {}", refresh_results.len());
    
    // Cross-project tests - should not find each other's unique functions
    let validate_in_p2 = searcher2.search("validate_session").await.expect("Search failed");
    let refresh_in_p1 = searcher1.search("refresh_token").await.expect("Search failed");
    
    println!("\nCross-project isolation tests:");
    println!("Project 2 searching for 'validate_session' (should be 0): {}", validate_in_p2.len());
    println!("Project 1 searching for 'refresh_token' (should be 0): {}", refresh_in_p1.len());
    
    // Final verification
    if all_results_in_project1 && 
       all_results_in_project2 && 
       !results1.is_empty() && 
       !results2.is_empty() &&
       !validate_results.is_empty() &&
       !refresh_results.is_empty() &&
       validate_in_p2.is_empty() &&
       refresh_in_p1.is_empty() {
        println!("\n✅ ALL UNIFIED SEARCHER TESTS PASSED! Project-scoped search is working correctly.");
        println!("   - Each project's UnifiedSearcher finds its own files");
        println!("   - Cross-project isolation is working at the UnifiedSearcher level");
        println!("   - Project-specific content is properly isolated");
    } else {
        println!("\n❌ SOME UNIFIED SEARCHER TESTS FAILED! Project-scoped search has issues.");
        std::process::exit(1);
    }
    
    Ok(())
}