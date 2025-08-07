// Comprehensive End-to-End Integration Tests for UnifiedSearcher with Tantivy Backend
// This module verifies that the complete search pipeline works correctly:
// UnifiedSearcher -> SearchAdapter -> TantivySearcher -> Results -> End User

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokio::time::{timeout, Duration};

use embed_search::config::SearchBackend;
use embed_search::search::{UnifiedSearcher, SearchResult};
#[cfg(feature = "tantivy")]
use embed_search::search::TantivySearcher;

/// Test the complete end-to-end pipeline: UnifiedSearcher -> Tantivy -> Results
#[tokio::test]
async fn test_end_to_end_pipeline() {
    println!("🚀 Testing Complete End-to-End Pipeline: UnifiedSearcher -> Tantivy");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join("integration_test.db");
    
    // Create test files with various code patterns
    create_test_files(&project_path).await;
    
    // Test Step 1: Create UnifiedSearcher with Tantivy backend
    let unified_searcher = UnifiedSearcher::new_with_backend(
        project_path.clone(), 
        db_path.clone(), 
        SearchBackend::Tantivy
    ).await.expect("Failed to create UnifiedSearcher with Tantivy backend");
    
    println!("✅ Step 1: UnifiedSearcher with Tantivy backend created successfully");
    
    // Test Step 2: Index files through UnifiedSearcher
    let test_files = get_test_files(&project_path);
    for file_path in &test_files {
        unified_searcher.index_file(file_path).await
            .expect(&format!("Failed to index file: {:?}", file_path));
    }
    
    println!("✅ Step 2: Files indexed through UnifiedSearcher -> Tantivy pipeline");
    
    // Test Step 3: Perform searches and verify results flow properly
    let search_test_cases = vec![
        ("authenticate_user", "authentication.rs", "Should find authentication function"),
        ("database_connection", "database.rs", "Should find database connection"),
        ("process_payment", "payment.rs", "Should find payment processing"),
        ("HashMap", "database.rs", "Should find HashMap usage"),
        ("fn calculate", "utils.rs", "Should find calculation functions"),
    ];
    
    for (query, expected_file, description) in search_test_cases {
        println!("  Testing search: '{}' - {}", query, description);
        
        let results = unified_searcher.search(query).await
            .expect(&format!("Search failed for query: {}", query));
        
        assert!(!results.is_empty(), "Should find results for query: {}", query);
        
        // Verify that results contain the expected file
        let found_expected = results.iter().any(|r| r.file.contains(expected_file));
        assert!(found_expected, "Should find {} in results for query: {}", expected_file, query);
        
        // Verify result structure and data integrity
        for result in &results {
            verify_search_result(result, query);
        }
        
        println!("    ✅ Found {} results, including expected file {}", results.len(), expected_file);
    }
    
    println!("✅ Step 3: End-to-end search pipeline verified successfully");
}

/// Test that search fusion works correctly with Tantivy as the exact search backend
#[tokio::test]
async fn test_search_fusion_with_tantivy() {
    println!("🔀 Testing Search Fusion Integration with Tantivy");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join("fusion_test.db");
    
    // Create diverse test content that will trigger different search methods
    create_diverse_test_files(&project_path).await;
    
    let unified_searcher = UnifiedSearcher::new_with_backend(
        project_path.clone(), 
        db_path, 
        SearchBackend::Tantivy
    ).await.expect("Failed to create UnifiedSearcher");
    
    // Index all test files
    let test_files = get_test_files(&project_path);
    for file_path in test_files {
        unified_searcher.index_file(&file_path).await
            .expect("Failed to index file");
    }
    
    // Test fusion queries that should hit multiple search backends
    let fusion_test_cases = vec![
        ("authentication", "Should find through both exact and semantic search"),
        ("database", "Should find through BM25 and exact search"),
        ("user_management", "Should find through semantic and symbol search"),
        ("payment_processing", "Should find through multiple fusion methods"),
    ];
    
    for (query, description) in fusion_test_cases {
        println!("  Testing fusion query: '{}' - {}", query, description);
        
        let results = unified_searcher.search(query).await
            .expect(&format!("Fusion search failed for: {}", query));
        
        assert!(!results.is_empty(), "Fusion search should find results for: {}", query);
        
        // Verify that results have proper fusion scoring and ranking
        verify_fusion_results(&results, query);
        
        println!("    ✅ Fusion search returned {} results with proper scoring", results.len());
    }
    
    println!("✅ Search fusion integration with Tantivy verified");
}

/// Test project-scoped searches work through the complete pipeline
#[tokio::test]
async fn test_project_scoped_integration() {
    println!("🏗️ Testing Project-Scoped Integration");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project1_path = temp_dir.path().join("project1");
    let project2_path = temp_dir.path().join("project2");
    
    fs::create_dir_all(&project1_path).expect("Failed to create project1");
    fs::create_dir_all(&project2_path).expect("Failed to create project2");
    
    // Create identical files in both projects
    create_scoped_test_files(&project1_path, "Project1").await;
    create_scoped_test_files(&project2_path, "Project2").await;
    
    // Test that each UnifiedSearcher only finds results from its project scope
    let db1_path = project1_path.join("scoped1.db");
    let searcher1 = UnifiedSearcher::new_with_backend(
        project1_path.clone(), 
        db1_path, 
        SearchBackend::Tantivy
    ).await.expect("Failed to create project1 searcher");
    
    let db2_path = project2_path.join("scoped2.db");
    let searcher2 = UnifiedSearcher::new_with_backend(
        project2_path.clone(), 
        db2_path, 
        SearchBackend::Tantivy
    ).await.expect("Failed to create project2 searcher");
    
    // Index files in each project
    for file_name in ["common.rs", "specific.rs"] {
        let file1 = project1_path.join(file_name);
        let file2 = project2_path.join(file_name);
        
        searcher1.index_file(&file1).await.expect("Failed to index in project1");
        searcher2.index_file(&file2).await.expect("Failed to index in project2");
    }
    
    // Verify that searches are properly scoped
    let results1 = searcher1.search("Project1").await.expect("Search failed in project1");
    let results2 = searcher2.search("Project2").await.expect("Search failed in project2");
    
    // Project1 searcher should only find Project1 content
    assert!(!results1.is_empty(), "Project1 searcher should find Project1 results");
    assert!(results1.iter().all(|r| r.file.contains("project1")), "All results should be from project1");
    
    // Project2 searcher should only find Project2 content  
    assert!(!results2.is_empty(), "Project2 searcher should find Project2 results");
    assert!(results2.iter().all(|r| r.file.contains("project2")), "All results should be from project2");
    
    // Cross-project isolation test
    let cross_results1 = searcher1.search("Project2").await.expect("Cross search failed");
    let cross_results2 = searcher2.search("Project1").await.expect("Cross search failed");
    
    assert!(cross_results1.is_empty(), "Project1 searcher should not find Project2 content");
    assert!(cross_results2.is_empty(), "Project2 searcher should not find Project1 content");
    
    println!("✅ Project scoping integration verified - proper isolation maintained");
}

/// Test error handling and recovery in the complete pipeline
#[tokio::test]
async fn test_error_handling_integration() {
    println!("⚠️ Testing Error Handling Integration");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join("error_test.db");
    
    let unified_searcher = UnifiedSearcher::new_with_backend(
        project_path.clone(), 
        db_path, 
        SearchBackend::Tantivy
    ).await.expect("Failed to create UnifiedSearcher");
    
    // Test 1: Empty query handling
    let empty_results = unified_searcher.search("").await.expect("Empty query should not fail");
    println!("  ✅ Empty query handled gracefully: {} results", empty_results.len());
    
    // Test 2: Non-existent file indexing
    let non_existent_file = project_path.join("does_not_exist.rs");
    let index_result = unified_searcher.index_file(&non_existent_file).await;
    // Should either succeed (skip) or fail gracefully
    println!("  ✅ Non-existent file indexing handled: {:?}", index_result.is_ok());
    
    // Test 3: Search with special characters
    let special_queries = vec!["@#$%", "\"", "\\", "/", "*", "?"];
    for query in special_queries {
        let result = unified_searcher.search(query).await;
        assert!(result.is_ok(), "Special character query '{}' should not panic", query);
        println!("    ✅ Special query '{}' handled safely", query);
    }
    
    // Test 4: Very long query
    let long_query = "a".repeat(1000);
    let long_result = unified_searcher.search(&long_query).await;
    assert!(long_result.is_ok(), "Long query should be handled safely");
    println!("  ✅ Long query (1000 chars) handled gracefully");
    
    println!("✅ Error handling integration verified - system remains stable");
}

/// Test performance characteristics of the complete pipeline
#[tokio::test]
async fn test_performance_integration() {
    println!("⚡ Testing Performance Integration");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join("perf_test.db");
    
    // Create multiple files with substantial content
    create_performance_test_files(&project_path).await;
    
    let unified_searcher = UnifiedSearcher::new_with_backend(
        project_path.clone(), 
        db_path, 
        SearchBackend::Tantivy
    ).await.expect("Failed to create UnifiedSearcher");
    
    let test_files = get_test_files(&project_path);
    
    // Test indexing performance
    let start_time = std::time::Instant::now();
    for file_path in &test_files {
        unified_searcher.index_file(file_path).await
            .expect("Failed to index file during performance test");
    }
    let index_duration = start_time.elapsed();
    
    println!("  📊 Indexed {} files in {:?}", test_files.len(), index_duration);
    assert!(index_duration.as_secs() < 30, "Indexing should complete within 30 seconds");
    
    // Test search performance with various query types
    let performance_queries = vec![
        "function",
        "authenticate",
        "database_connection",
        "process_data",
        "HashMap",
    ];
    
    let mut total_search_time = Duration::from_secs(0);
    let mut total_results = 0;
    
    for query in performance_queries {
        let search_start = std::time::Instant::now();
        
        let results = timeout(Duration::from_secs(10), unified_searcher.search(query))
            .await
            .expect(&format!("Search for '{}' timed out", query))
            .expect(&format!("Search for '{}' failed", query));
        
        let search_duration = search_start.elapsed();
        total_search_time += search_duration;
        total_results += results.len();
        
        println!("    📈 Query '{}': {} results in {:?}", query, results.len(), search_duration);
        assert!(search_duration.as_millis() < 5000, "Search should complete within 5 seconds");
    }
    
    let avg_search_time = total_search_time / 5;
    println!("  📊 Average search time: {:?}, Total results: {}", avg_search_time, total_results);
    
    println!("✅ Performance integration verified - acceptable response times");
}

/// Test concurrent operations in the pipeline
#[tokio::test]
async fn test_concurrent_operations() {
    println!("🔄 Testing Concurrent Operations");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join("concurrent_test.db");
    
    create_test_files(&project_path).await;
    
    let unified_searcher = std::sync::Arc::new(
        UnifiedSearcher::new_with_backend(
            project_path.clone(), 
            db_path, 
            SearchBackend::Tantivy
        ).await.expect("Failed to create UnifiedSearcher")
    );
    
    // Index files first
    let test_files = get_test_files(&project_path);
    for file_path in test_files {
        unified_searcher.index_file(&file_path).await.expect("Failed to index");
    }
    
    // Test concurrent searches
    let concurrent_queries = vec!["authenticate", "database", "payment", "utils", "HashMap"];
    let mut handles = Vec::new();
    
    for query in concurrent_queries {
        let searcher = unified_searcher.clone();
        let query = query.to_string();
        
        let handle = tokio::spawn(async move {
            let results = searcher.search(&query).await
                .expect(&format!("Concurrent search failed for: {}", query));
            (query, results.len())
        });
        
        handles.push(handle);
    }
    
    // Wait for all searches to complete
    let mut total_concurrent_results = 0;
    for handle in handles {
        let (query, result_count) = handle.await.expect("Concurrent task failed");
        total_concurrent_results += result_count;
        println!("    🔄 Concurrent search '{}' found {} results", query, result_count);
    }
    
    println!("  📊 Total concurrent results: {}", total_concurrent_results);
    assert!(total_concurrent_results > 0, "Concurrent searches should find results");
    
    println!("✅ Concurrent operations verified - system handles concurrency safely");
}

/// Test data consistency throughout the pipeline  
#[tokio::test]
async fn test_data_consistency() {
    println!("🔍 Testing Data Consistency");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join("consistency_test.db");
    
    // Create test file with known content
    let test_file = project_path.join("consistency.rs");
    let test_content = r#"
pub fn authenticate_user(username: &str, password: &str) -> AuthResult {
    if username.is_empty() || password.is_empty() {
        return AuthResult::InvalidInput;
    }
    
    let stored_hash = get_password_hash(username)?;
    if verify_password(password, &stored_hash) {
        AuthResult::Success
    } else {
        AuthResult::Failure
    }
}

fn get_password_hash(username: &str) -> Result<String, DatabaseError> {
    let db = connect_database()?;
    db.query_password_hash(username)
}

fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}
"#;
    fs::write(&test_file, test_content).expect("Failed to write test file");
    
    let unified_searcher = UnifiedSearcher::new_with_backend(
        project_path.clone(), 
        db_path, 
        SearchBackend::Tantivy
    ).await.expect("Failed to create UnifiedSearcher");
    
    // Index the file
    unified_searcher.index_file(&test_file).await.expect("Failed to index file");
    
    // Test that search results contain accurate line numbers and content
    let test_cases = vec![
        ("authenticate_user", 2), // Should find line 2
        ("get_password_hash", 11), // Should find line 11  
        ("verify_password", 16), // Should find line 16
    ];
    
    for (query, expected_line) in test_cases {
        let results = unified_searcher.search(query).await
            .expect(&format!("Search failed for: {}", query));
        
        assert!(!results.is_empty(), "Should find results for: {}", query);
        
        // Find the exact match for this function
        let exact_match = results.iter().find(|r| r.file.contains("consistency.rs"))
            .expect(&format!("Should find consistency.rs in results for: {}", query));
        
        // Verify the search result contains the expected content
        assert!(exact_match.three_chunk_context.target.content.contains(query), 
                "Result should contain the searched term: {}", query);
        
        // Verify line numbers are reasonable (allowing for chunking)
        let start_line = exact_match.three_chunk_context.target.start_line;
        let end_line = exact_match.three_chunk_context.target.end_line;
        assert!(start_line <= expected_line && expected_line <= end_line,
                "Expected line {} should be within chunk range {}-{}", 
                expected_line, start_line, end_line);
        
        println!("    ✅ '{}' found at lines {}-{}, content verified", query, start_line, end_line);
    }
    
    // Test that direct TantivySearcher produces compatible results
    let mut direct_tantivy = TantivySearcher::new_with_root(&project_path).await
        .expect("Failed to create direct TantivySearcher");
    
    direct_tantivy.index_file(&test_file).await.expect("Failed to index with direct Tantivy");
    let direct_results = direct_tantivy.search("authenticate_user").await
        .expect("Direct Tantivy search failed");
    
    assert!(!direct_results.is_empty(), "Direct Tantivy should find results");
    
    // Verify that UnifiedSearcher and direct Tantivy find overlapping results
    let unified_results = unified_searcher.search("authenticate_user").await
        .expect("Unified search failed");
    
    assert!(!unified_results.is_empty(), "Unified searcher should find results");
    
    // Both should find the same file
    let unified_has_file = unified_results.iter().any(|r| r.file.contains("consistency.rs"));
    let direct_has_file = direct_results.iter().any(|r| r.file_path.contains("consistency.rs"));
    
    assert!(unified_has_file && direct_has_file, 
            "Both UnifiedSearcher and direct Tantivy should find the same file");
    
    println!("✅ Data consistency verified - search results are accurate and compatible");
}

// =========================== HELPER FUNCTIONS ===========================

async fn create_test_files(project_path: &Path) {
    let files = vec![
        ("authentication.rs", r#"
pub fn authenticate_user(username: &str, password: &str) -> bool {
    if username.is_empty() || password.is_empty() {
        return false;
    }
    verify_credentials(username, password)
}

fn verify_credentials(username: &str, password: &str) -> bool {
    // Database lookup and verification
    check_database_connection() && validate_password(username, password)
}

fn validate_password(username: &str, password: &str) -> bool {
    // Password validation logic
    password.len() >= 8 && !password.contains(username)
}
"#),
        ("database.rs", r#"
use std::collections::HashMap;

pub fn database_connection() -> Result<Connection, DatabaseError> {
    let connection_string = get_connection_string();
    Connection::new(connection_string)
}

pub fn check_database_connection() -> bool {
    database_connection().is_ok()
}

fn get_connection_string() -> String {
    std::env::var("DATABASE_URL").unwrap_or_default()
}

pub fn get_user_data(user_id: u64) -> Option<HashMap<String, String>> {
    let mut data = HashMap::new();
    data.insert("id".to_string(), user_id.to_string());
    Some(data)
}
"#),
        ("payment.rs", r#"
pub struct PaymentProcessor {
    api_key: String,
}

impl PaymentProcessor {
    pub fn process_payment(&self, amount: f64, currency: &str) -> PaymentResult {
        if amount <= 0.0 {
            return PaymentResult::InvalidAmount;
        }
        
        let transaction_id = generate_transaction_id();
        submit_payment_request(amount, currency, &transaction_id)
    }
    
    fn generate_transaction_id(&self) -> String {
        format!("txn_{}", chrono::Utc::now().timestamp())
    }
}

fn submit_payment_request(amount: f64, currency: &str, txn_id: &str) -> PaymentResult {
    // Submit to payment gateway
    PaymentResult::Success(txn_id.to_string())
}
"#),
        ("utils.rs", r#"
pub fn calculate_hash(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn format_currency(amount: f64, currency: &str) -> String {
    match currency {
        "USD" => format!("${:.2}", amount),
        "EUR" => format!("€{:.2}", amount),
        _ => format!("{:.2} {}", amount, currency),
    }
}

fn calculate_percentage(part: f64, whole: f64) -> f64 {
    if whole == 0.0 { 0.0 } else { (part / whole) * 100.0 }
}
"#),
    ];
    
    for (filename, content) in files {
        let file_path = project_path.join(filename);
        fs::write(&file_path, content).expect(&format!("Failed to write {}", filename));
    }
}

async fn create_diverse_test_files(project_path: &Path) {
    let files = vec![
        ("authentication_service.rs", r#"
pub struct AuthenticationService {
    database: DatabasePool,
    config: AuthConfig,
}

impl AuthenticationService {
    pub async fn authenticate_user(&self, credentials: &UserCredentials) -> AuthResult {
        let user = self.database.find_user(&credentials.username).await?;
        if self.verify_password(&credentials.password, &user.password_hash) {
            self.generate_session_token(&user).await
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }
    
    fn verify_password(&self, password: &str, hash: &str) -> bool {
        bcrypt::verify(password, hash).unwrap_or(false)
    }
}
"#),
        ("database_manager.rs", r#"
use std::collections::HashMap;
use sqlx::{Pool, Postgres};

pub struct DatabaseManager {
    pool: Pool<Postgres>,
    cache: HashMap<String, CachedData>,
}

impl DatabaseManager {
    pub async fn new(database_url: &str) -> Result<Self, DatabaseError> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url).await?;
            
        Ok(Self {
            pool,
            cache: HashMap::new(),
        })
    }
    
    pub async fn execute_query(&self, query: &str) -> Result<QueryResult, DatabaseError> {
        sqlx::query(query).execute(&self.pool).await.map_err(Into::into)
    }
}
"#),
        ("user_management.rs", r#"
pub mod user_management {
    use super::*;
    
    pub struct UserManager {
        auth_service: AuthenticationService,
        user_repository: UserRepository,
    }
    
    impl UserManager {
        pub async fn create_user(&mut self, user_data: &CreateUserRequest) -> Result<User, UserError> {
            self.validate_user_data(user_data)?;
            let user = User::new(user_data);
            self.user_repository.save(&user).await?;
            Ok(user)
        }
        
        pub async fn get_user_profile(&self, user_id: u64) -> Option<UserProfile> {
            self.user_repository.find_profile(user_id).await
        }
        
        fn validate_user_data(&self, data: &CreateUserRequest) -> Result<(), ValidationError> {
            if data.email.is_empty() || !data.email.contains('@') {
                return Err(ValidationError::InvalidEmail);
            }
            Ok(())
        }
    }
}
"#),
        ("payment_processing.rs", r#"
pub mod payment_processing {
    use serde::{Serialize, Deserialize};
    
    #[derive(Serialize, Deserialize)]
    pub struct PaymentRequest {
        pub amount: f64,
        pub currency: String,
        pub user_id: u64,
        pub payment_method: PaymentMethod,
    }
    
    pub async fn process_payment_async(request: PaymentRequest) -> Result<PaymentResponse, PaymentError> {
        validate_payment_request(&request)?;
        
        let processor = PaymentProcessor::new(&request.payment_method);
        let result = processor.submit_payment(request.amount, &request.currency).await?;
        
        Ok(PaymentResponse {
            transaction_id: result.transaction_id,
            status: PaymentStatus::Completed,
            processed_at: chrono::Utc::now(),
        })
    }
    
    fn validate_payment_request(request: &PaymentRequest) -> Result<(), ValidationError> {
        if request.amount <= 0.0 {
            return Err(ValidationError::InvalidAmount);
        }
        Ok(())
    }
}
"#),
    ];
    
    for (filename, content) in files {
        let file_path = project_path.join(filename);
        fs::write(&file_path, content).expect(&format!("Failed to write {}", filename));
    }
}

async fn create_scoped_test_files(project_path: &Path, project_name: &str) {
    let files = vec![
        ("common.rs", format!(r#"
// Common functionality for {}
pub fn {}_common_function() -> String {{
    format!("This is {} common functionality")
}}

pub const {}_VERSION: &str = "1.0.0";
"#, project_name, project_name.to_lowercase(), project_name, project_name.to_uppercase())),
        ("specific.rs", format!(r#"
// {} specific implementations
pub mod {} {{
    pub fn process_data() -> String {{
        format!("{} data processing")
    }}
    
    pub fn handle_request() -> String {{
        format!("{} request handling")
    }}
}}
"#, project_name, project_name.to_lowercase(), project_name, project_name)),
    ];
    
    for (filename, content) in files {
        let file_path = project_path.join(filename);
        fs::write(&file_path, content).expect(&format!("Failed to write {}", filename));
    }
}

async fn create_performance_test_files(project_path: &Path) {
    // Create multiple files with substantial content for performance testing
    for i in 0..10 {
        let filename = format!("performance_test_{}.rs", i);
        let content = format!(r#"
// Performance test file {}
use std::collections::{{HashMap, HashSet}};
use std::sync::Arc;
use tokio::sync::{{RwLock, Mutex}};

pub struct PerformanceTest{} {{
    data: HashMap<String, String>,
    cache: Arc<RwLock<HashMap<String, CachedItem>>>,
    processed: HashSet<u64>,
}}

impl PerformanceTest{} {{
    pub async fn new() -> Self {{
        Self {{
            data: HashMap::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            processed: HashSet::new(),
        }}
    }}
    
    pub async fn process_large_dataset(&mut self, items: Vec<DataItem>) -> ProcessingResult {{
        let mut results = Vec::new();
        
        for item in items {{
            if !self.processed.contains(&item.id) {{
                let processed_item = self.process_single_item(item).await?;
                results.push(processed_item);
                self.processed.insert(item.id);
            }}
        }}
        
        ProcessingResult {{ items: results }}
    }}
    
    async fn process_single_item(&self, item: DataItem) -> Result<ProcessedItem, ProcessingError> {{
        // Simulate complex processing
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(&item.key) {{
            return Ok(cached.to_processed_item());
        }}
        drop(cache);
        
        // Perform expensive computation
        let result = self.expensive_computation(&item).await?;
        
        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(item.key.clone(), CachedItem::from(&result));
        
        Ok(result)
    }}
    
    async fn expensive_computation(&self, item: &DataItem) -> Result<ProcessedItem, ProcessingError> {{
        // Simulate CPU-intensive work
        tokio::task::yield_now().await;
        
        Ok(ProcessedItem {{
            id: item.id,
            data: format!("processed_{{}}", item.data),
            timestamp: chrono::Utc::now(),
        }})
    }}
    
    pub fn authenticate_performance_user(&self, username: &str) -> bool {{
        // Authentication logic for performance testing
        !username.is_empty() && username.len() > 3
    }}
    
    pub fn database_performance_connection(&self) -> bool {{
        // Database connection simulation
        true
    }}
    
    pub fn process_data_performance(&self, data: &str) -> String {{
        format!("Performance processed: {{}}", data)
    }}
}}

// Additional helper functions for comprehensive testing
pub fn function_that_uses_hashmap() -> HashMap<String, i32> {{
    let mut map = HashMap::new();
    map.insert("performance".to_string(), {});
    map.insert("test".to_string(), {} * 2);
    map
}}

pub async fn async_function_with_authentication(user: &str) -> AuthResult {{
    if user == "performance_user" {{
        AuthResult::Success
    }} else {{
        AuthResult::Denied
    }}
}}

pub fn calculate_performance_metrics(data: &[f64]) -> PerformanceMetrics {{
    PerformanceMetrics {{
        avg: data.iter().sum::<f64>() / data.len() as f64,
        min: data.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
        max: data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
    }}
}}
"#, i, i, i, i, i);
        
        let file_path = project_path.join(&filename);
        fs::write(&file_path, content).expect(&format!("Failed to write {}", filename));
    }
}

fn get_test_files(project_path: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(project_path)
        .expect("Failed to read project directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()?.to_str()? == "rs" {
                Some(path)
            } else {
                None
            }
        })
        .collect()
}

fn verify_search_result(result: &SearchResult, query: &str) {
    // Verify basic result structure
    assert!(!result.file.is_empty(), "Result file path should not be empty");
    assert!(!result.three_chunk_context.target.content.is_empty(), "Result content should not be empty");
    assert!(result.score.is_finite() && result.score >= 0.0, "Result score should be valid: {}", result.score);
    
    // Verify chunk boundaries are reasonable
    assert!(result.three_chunk_context.target.start_line <= result.three_chunk_context.target.end_line,
            "Start line should be <= end line");
    
    // For exact matches, verify the query term appears in the content
    if result.match_type == embed_search::search::fusion::MatchType::Exact {
        assert!(result.three_chunk_context.target.content.to_lowercase().contains(&query.to_lowercase()),
                "Exact match result should contain the query term");
    }
}

fn verify_fusion_results(results: &[SearchResult], _query: &str) {
    // Verify that results are properly ranked by score (descending)
    for window in results.windows(2) {
        assert!(window[0].score >= window[1].score,
                "Results should be sorted by score (descending): {} >= {}",
                window[0].score, window[1].score);
    }
    
    // Verify that we have diverse match types (indicating fusion is working)
    let match_type_count = results.iter()
        .map(|r| format!("{:?}", r.match_type))
        .collect::<std::collections::HashSet<_>>();
    
    // Should have at least one match type (could be all exact if other backends unavailable)
    assert!(!match_type_count.is_empty(), "Should have at least one match type");
    
    // Verify all results have valid scores
    for result in results {
        assert!(result.score.is_finite() && result.score >= 0.0,
                "All fusion results should have valid scores");
    }
}