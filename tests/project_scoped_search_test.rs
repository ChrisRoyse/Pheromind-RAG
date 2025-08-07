#[cfg(feature = "tantivy")]
use embed_search::search::search_adapter::create_text_searcher_with_root;
use embed_search::config::SearchBackend;
use std::fs;
use tempfile::TempDir;

#[cfg(feature = "tantivy")]
#[tokio::test]
async fn test_project_scoped_search() {
    println!("🔍 Testing project-scoped search functionality");
    
    // Create two separate temporary directories representing different projects
    let project1_dir = TempDir::new().expect("Failed to create temp directory for project 1");
    let project2_dir = TempDir::new().expect("Failed to create temp directory for project 2");
    
    let project1_path = project1_dir.path().to_path_buf();
    let project2_path = project2_dir.path().to_path_buf();
    
    // Create test files in project 1
    let file1_p1 = project1_path.join("module.rs");
    fs::write(&file1_p1, r#"
pub fn authenticate_user() {
    println!("Authenticating user in project 1");
}

pub fn validate_credentials() {
    println!("Validating credentials in project 1");
}
"#).unwrap();
    
    // Create test files in project 2
    let file1_p2 = project2_path.join("module.rs");
    fs::write(&file1_p2, r#"
pub fn authenticate_user() {
    println!("Authenticating user in project 2");
}

pub fn handle_session() {
    println!("Handling session in project 2");
}
"#).unwrap();
    
    // Create searcher for project 1
    let mut searcher1 = create_text_searcher_with_root(&SearchBackend::Tantivy, project1_path.clone()).await
        .expect("Failed to create searcher for project 1");
    
    // Create searcher for project 2
    let mut searcher2 = create_text_searcher_with_root(&SearchBackend::Tantivy, project2_path.clone()).await
        .expect("Failed to create searcher for project 2");
    
    // Index files in their respective projects
    searcher1.index_file(&file1_p1).await.expect("Failed to index file in project 1");
    searcher2.index_file(&file1_p2).await.expect("Failed to index file in project 2");
    
    // Test that each searcher only finds results from its own project
    let results1 = searcher1.search("authenticate_user").await.expect("Search failed in project 1");
    let results2 = searcher2.search("authenticate_user").await.expect("Search failed in project 2");
    
    println!("Project 1 results: {} found", results1.len());
    println!("Project 2 results: {} found", results2.len());
    
    // Both should find the authenticate_user function
    assert!(!results1.is_empty(), "Project 1 should find authenticate_user");
    assert!(!results2.is_empty(), "Project 2 should find authenticate_user");
    
    // Verify that results only contain files from their respective projects
    for result in &results1 {
        assert!(result.file_path.contains("module.rs"), "Result should contain module.rs");
        let file_path = std::path::Path::new(&result.file_path);
        assert!(file_path.starts_with(&project1_path), "Result should be from project 1: {}", result.file_path);
    }
    
    for result in &results2 {
        assert!(result.file_path.contains("module.rs"), "Result should contain module.rs");
        let file_path = std::path::Path::new(&result.file_path);
        assert!(file_path.starts_with(&project2_path), "Result should be from project 2: {}", result.file_path);
    }
    
    // Test project-specific functions
    let validate_results = searcher1.search("validate_credentials").await.expect("Search failed");
    let session_results = searcher2.search("handle_session").await.expect("Search failed");
    
    assert!(!validate_results.is_empty(), "Project 1 should find validate_credentials");
    assert!(!session_results.is_empty(), "Project 2 should find handle_session");
    
    // Cross-project tests - should not find each other's unique functions
    let validate_in_p2 = searcher2.search("validate_credentials").await.expect("Search failed");
    let session_in_p1 = searcher1.search("handle_session").await.expect("Search failed");
    
    assert!(validate_in_p2.is_empty(), "Project 2 should NOT find validate_credentials (project 1 only)");
    assert!(session_in_p1.is_empty(), "Project 1 should NOT find handle_session (project 2 only)");
    
    println!("✅ Project-scoped search test passed!");
}

#[cfg(feature = "tantivy")]
#[tokio::test]
async fn test_cross_project_isolation() {
    println!("🔒 Testing cross-project isolation");
    
    let project_a_dir = TempDir::new().expect("Failed to create temp directory for project A");
    let project_b_dir = TempDir::new().expect("Failed to create temp directory for project B");
    
    let project_a_path = project_a_dir.path().to_path_buf();
    let project_b_path = project_b_dir.path().to_path_buf();
    
    // Create files with unique content in each project
    let file_a = project_a_path.join("service_a.rs");
    fs::write(&file_a, r#"
pub struct ServiceA {
    name: String,
}

impl ServiceA {
    pub fn unique_method_a(&self) -> String {
        format!("Service A: {}", self.name)
    }
}
"#).unwrap();
    
    let file_b = project_b_path.join("service_b.rs");
    fs::write(&file_b, r#"
pub struct ServiceB {
    id: u32,
}

impl ServiceB {
    pub fn unique_method_b(&self) -> u32 {
        self.id * 2
    }
}
"#).unwrap();
    
    // Create searchers for both projects
    let mut searcher_a = create_text_searcher_with_root(&SearchBackend::Tantivy, project_a_path.clone()).await
        .expect("Failed to create searcher A");
    let mut searcher_b = create_text_searcher_with_root(&SearchBackend::Tantivy, project_b_path.clone()).await
        .expect("Failed to create searcher B");
    
    // Index files
    searcher_a.index_file(&file_a).await.expect("Failed to index file A");
    searcher_b.index_file(&file_b).await.expect("Failed to index file B");
    
    // Test isolation - each searcher should only find its own content
    let a_finds_a = searcher_a.search("unique_method_a").await.expect("Search failed");
    let a_finds_b = searcher_a.search("unique_method_b").await.expect("Search failed");
    let b_finds_a = searcher_b.search("unique_method_a").await.expect("Search failed");
    let b_finds_b = searcher_b.search("unique_method_b").await.expect("Search failed");
    
    assert!(!a_finds_a.is_empty(), "Searcher A should find unique_method_a");
    assert!(a_finds_b.is_empty(), "Searcher A should NOT find unique_method_b");
    assert!(b_finds_a.is_empty(), "Searcher B should NOT find unique_method_a");
    assert!(!b_finds_b.is_empty(), "Searcher B should find unique_method_b");
    
    println!("✅ Cross-project isolation test passed!");
}

#[cfg(feature = "tantivy")]
#[tokio::test]
async fn test_project_boundary_enforcement() {
    println!("🚧 Testing project boundary enforcement");
    
    let main_project = TempDir::new().expect("Failed to create main project temp directory");
    let main_path = main_project.path().to_path_buf();
    
    // Create a nested structure
    let src_dir = main_path.join("src");
    let external_dir = TempDir::new().expect("Failed to create external temp directory");
    let external_path = external_dir.path().to_path_buf();
    
    fs::create_dir_all(&src_dir).unwrap();
    
    // Create file inside project
    let internal_file = src_dir.join("internal.rs");
    fs::write(&internal_file, r#"
pub fn internal_function() {
    println!("This is inside the project");
}
"#).unwrap();
    
    // Create file outside project
    let external_file = external_path.join("external.rs");
    fs::write(&external_file, r#"
pub fn external_function() {
    println!("This is outside the project");
}
"#).unwrap();
    
    // Create project-scoped searcher
    let mut searcher = create_text_searcher_with_root(&SearchBackend::Tantivy, main_path.clone()).await
        .expect("Failed to create project-scoped searcher");
    
    // Try to index both files
    searcher.index_file(&internal_file).await.expect("Failed to index internal file");
    searcher.index_file(&external_file).await.expect("Index external file should not fail, but should be ignored");
    
    // Search for both functions
    let internal_results = searcher.search("internal_function").await.expect("Search failed");
    let external_results = searcher.search("external_function").await.expect("Search failed");
    
    assert!(!internal_results.is_empty(), "Should find internal function");
    assert!(external_results.is_empty(), "Should NOT find external function (outside project scope)");
    
    // Verify the internal result is within project boundaries
    for result in &internal_results {
        let file_path = std::path::Path::new(&result.file_path);
        assert!(file_path.starts_with(&main_path), "Result should be within project boundaries: {}", result.file_path);
    }
    
    println!("✅ Project boundary enforcement test passed!");
}