use embed_search::search::search_adapter::create_text_searcher_with_root;
use embed_search::config::SearchBackend;
use std::fs;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔍 Testing project-scoped search functionality");
    
    // Create two separate temporary directories representing different projects
    let project1_dir = TempDir::new().expect("Failed to create temp directory for project 1");
    let project2_dir = TempDir::new().expect("Failed to create temp directory for project 2");
    
    let project1_path = project1_dir.path().to_path_buf();
    let project2_path = project2_dir.path().to_path_buf();
    
    println!("Project 1 path: {:?}", project1_path);
    println!("Project 2 path: {:?}", project2_path);
    
    // Create test files in project 1
    let file1_p1 = project1_path.join("module.rs");
    fs::write(&file1_p1, r#"
pub fn authenticate_user() {
    println!("Authenticating user in project 1");
}

pub fn validate_credentials() {
    println!("Validating credentials in project 1");
}
"#)?;
    
    // Create test files in project 2
    let file1_p2 = project2_path.join("module.rs");
    fs::write(&file1_p2, r#"
pub fn authenticate_user() {
    println!("Authenticating user in project 2");
}

pub fn handle_session() {
    println!("Handling session in project 2");
}
"#)?;
    
    // Create searcher for project 1
    let mut searcher1 = create_text_searcher_with_root(&SearchBackend::Tantivy, project1_path.clone()).await
        .expect("Failed to create searcher for project 1");
    
    // Create searcher for project 2
    let mut searcher2 = create_text_searcher_with_root(&SearchBackend::Tantivy, project2_path.clone()).await
        .expect("Failed to create searcher for project 2");
    
    println!("✅ Created searchers for both projects");
    
    // Index files in their respective projects
    searcher1.index_file(&file1_p1).await.expect("Failed to index file in project 1");
    searcher2.index_file(&file1_p2).await.expect("Failed to index file in project 2");
    
    println!("✅ Indexed files in both projects");
    
    // Test that each searcher only finds results from its own project
    let results1 = searcher1.search("authenticate_user").await.expect("Search failed in project 1");
    let results2 = searcher2.search("authenticate_user").await.expect("Search failed in project 2");
    
    println!("Project 1 results: {} found", results1.len());
    for result in &results1 {
        println!("  - Found in: {}", result.file_path);
    }
    
    println!("Project 2 results: {} found", results2.len());
    for result in &results2 {
        println!("  - Found in: {}", result.file_path);
    }
    
    // Verify that results only contain files from their respective projects
    let all_results_in_project1 = results1.iter().all(|result| {
        let file_path = std::path::Path::new(&result.file_path);
        file_path.starts_with(&project1_path)
    });
    
    let all_results_in_project2 = results2.iter().all(|result| {
        let file_path = std::path::Path::new(&result.file_path);
        file_path.starts_with(&project2_path)
    });
    
    println!("\n📊 Verification results:");
    println!("All Project 1 results are in Project 1 directory: {}", all_results_in_project1);
    println!("All Project 2 results are in Project 2 directory: {}", all_results_in_project2);
    
    // Test project-specific functions
    let validate_results = searcher1.search("validate_credentials").await.expect("Search failed");
    let session_results = searcher2.search("handle_session").await.expect("Search failed");
    
    println!("\nProject-specific function tests:");
    println!("Project 1 'validate_credentials' results: {}", validate_results.len());
    println!("Project 2 'handle_session' results: {}", session_results.len());
    
    // Cross-project tests - should not find each other's unique functions
    let validate_in_p2 = searcher2.search("validate_credentials").await.expect("Search failed");
    let session_in_p1 = searcher1.search("handle_session").await.expect("Search failed");
    
    println!("\nCross-project isolation tests:");
    println!("Project 2 searching for 'validate_credentials' (should be 0): {}", validate_in_p2.len());
    println!("Project 1 searching for 'handle_session' (should be 0): {}", session_in_p1.len());
    
    // Final verification
    if all_results_in_project1 && 
       all_results_in_project2 && 
       !results1.is_empty() && 
       !results2.is_empty() &&
       !validate_results.is_empty() &&
       !session_results.is_empty() &&
       validate_in_p2.is_empty() &&
       session_in_p1.is_empty() {
        println!("\n✅ ALL TESTS PASSED! Project-scoped search is working correctly.");
        println!("   - Each project finds its own files");
        println!("   - Cross-project isolation is working");
        println!("   - Project-specific content is properly isolated");
    } else {
        println!("\n❌ SOME TESTS FAILED! Project-scoped search has issues.");
        std::process::exit(1);
    }
    
    Ok(())
}