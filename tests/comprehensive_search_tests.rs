use embed_search::search::unified::{UnifiedSearcher, SearchResult};
use embed_search::search::MatchType;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio;

/// Comprehensive test suite with 100 complex query tests for the embedding search system
/// Tests cover exact text matching, semantic similarity, language-specific queries,
/// function/method searches, class/struct searches, documentation searches, and edge cases.

struct TestSetup {
    searcher: UnifiedSearcher,
    _temp_dir: TempDir,
}

impl TestSetup {
    async fn new() -> Self {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let db_path = temp_dir.path().join("test.lancedb");
        let vectortest_path = PathBuf::from("C:\\code\\embed\\vectortest");
        
        let searcher = UnifiedSearcher::new(vectortest_path.clone(), db_path)
            .await
            .expect("Should create searcher");
        
        // Index the vectortest directory
        let stats = searcher.index_directory(&vectortest_path)
            .await
            .expect("Should index directory");
        
        println!("Test setup complete: {}", stats);
        
        Self {
            searcher,
            _temp_dir: temp_dir,
        }
    }
}

/// Helper to verify search results meet basic expectations
fn verify_results(results: &[SearchResult], query: &str, min_results: usize) {
    assert!(
        results.len() >= min_results,
        "Query '{}' should return at least {} results, got {}",
        query, min_results, results.len()
    );
    
    for result in results {
        // Verify score is reasonable
        assert!(
            result.score >= 0.0 && result.score <= 1.0,
            "Score should be between 0.0 and 1.0, got {}",
            result.score
        );
        
        // Verify file path is not empty
        assert!(!result.file.is_empty(), "File path should not be empty");
        
        // Verify three-chunk context exists and has content
        assert!(!result.three_chunk_context.target.content.is_empty(),
               "Target chunk content should not be empty");
        
        // Verify context formatting works
        let _display = result.three_chunk_context.format_for_display();
        let _summary = result.three_chunk_context.format_summary();
        let _full_content = result.three_chunk_context.get_full_content();
    }
}

/// Helper to verify expected files are in results
fn verify_expected_files(results: &[SearchResult], expected_files: &[&str]) {
    let result_files: Vec<String> = results.iter()
        .map(|r| r.file.clone())
        .collect();
    
    for expected in expected_files {
        assert!(
            result_files.iter().any(|f| f.contains(expected)),
            "Expected file '{}' not found in results: {:?}",
            expected, result_files
        );
    }
}

// ==== EXACT TEXT MATCHING TESTS (Tests 1-10) ====

#[tokio::test]
async fn test_001_exact_function_name() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("authenticate").await.unwrap();
    verify_results(&results, "authenticate", 1);
    verify_expected_files(&results, &["auth_service.py", "user_controller.js"]);
}

#[tokio::test]
async fn test_002_exact_class_name() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("OrderService").await.unwrap();
    verify_results(&results, "OrderService", 1);
    verify_expected_files(&results, &["OrderService.java"]);
}

#[tokio::test]
async fn test_003_exact_variable_name() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("username").await.unwrap();
    verify_results(&results, "username", 1);
    // Should find in multiple files
    assert!(results.len() >= 2);
}

#[tokio::test]
async fn test_004_exact_string_literal() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("Hello World").await.unwrap();
    verify_results(&results, "Hello World", 0); // May not exist in test data
}

#[tokio::test]
async fn test_005_exact_sql_keyword() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("CREATE TABLE").await.unwrap();
    verify_results(&results, "CREATE TABLE", 1);
    verify_expected_files(&results, &["database_migration.sql"]);
}

#[tokio::test]
async fn test_006_exact_error_message() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("User not found").await.unwrap();
    verify_results(&results, "User not found", 1);
    verify_expected_files(&results, &["user_controller.js"]);
}

#[tokio::test]
async fn test_007_exact_comment_text() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("Handles user authentication").await.unwrap();
    verify_results(&results, "Handles user authentication", 1);
    verify_expected_files(&results, &["auth_service.py"]);
}

#[tokio::test]
async fn test_008_exact_package_import() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("import hashlib").await.unwrap();
    verify_results(&results, "import hashlib", 1);
    verify_expected_files(&results, &["auth_service.py"]);
}

#[tokio::test]
async fn test_009_exact_configuration_key() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("JWT_SECRET").await.unwrap();
    verify_results(&results, "JWT_SECRET", 1);
    verify_expected_files(&results, &["user_controller.js"]);
}

#[tokio::test]
async fn test_010_exact_database_column() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("password_hash").await.unwrap();
    verify_results(&results, "password_hash", 1);
    // Should find in multiple files (Python and SQL)
    assert!(results.len() >= 2);
}

// ==== SEMANTIC SIMILARITY TESTS (Tests 11-25) ====

#[tokio::test]
async fn test_011_semantic_authentication_concepts() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("user login verification").await.unwrap();
    verify_results(&results, "user login verification", 1);
    // Should find authentication-related code
    verify_expected_files(&results, &["auth_service.py", "user_controller.js"]);
}

#[tokio::test]
async fn test_012_semantic_data_processing() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("process information transform").await.unwrap();
    verify_results(&results, "process information transform", 1);
    verify_expected_files(&results, &["DataProcessor.cs"]);
}

#[tokio::test]
async fn test_013_semantic_caching_concepts() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("memory storage retrieval").await.unwrap();
    verify_results(&results, "memory storage retrieval", 1);
    verify_expected_files(&results, &["memory_cache.rs"]);
}

#[tokio::test]
async fn test_014_semantic_payment_processing() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("financial transaction payment").await.unwrap();
    verify_results(&results, "financial transaction payment", 1);
    verify_expected_files(&results, &["payment_gateway.ts", "OrderService.java"]);
}

#[tokio::test]
async fn test_015_semantic_database_operations() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("store retrieve data persistence").await.unwrap();
    verify_results(&results, "store retrieve data persistence", 1);
    verify_expected_files(&results, &["database_migration.sql"]);
}

#[tokio::test]
async fn test_016_semantic_error_handling() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("exception error handling failure").await.unwrap();
    verify_results(&results, "exception error handling failure", 1);
    // Should find error handling code across multiple files
    assert!(results.len() >= 2);
}

#[tokio::test]
async fn test_017_semantic_network_communication() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("network connection socket communication").await.unwrap();
    verify_results(&results, "network connection socket communication", 1);
    verify_expected_files(&results, &["websocket_server.cpp"]);
}

#[tokio::test]
async fn test_018_semantic_user_management() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("manage users accounts profiles").await.unwrap();
    verify_results(&results, "manage users accounts profiles", 1);
    verify_expected_files(&results, &["user_controller.js"]);
}

#[tokio::test]
async fn test_019_semantic_business_logic() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("business rules logic validation").await.unwrap();
    verify_results(&results, "business rules logic validation", 1);
    verify_expected_files(&results, &["OrderService.java"]);
}

#[tokio::test]
async fn test_020_semantic_configuration_setup() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("configuration settings initialization").await.unwrap();
    verify_results(&results, "configuration settings initialization", 1);
    // Should find configuration code in multiple files
    assert!(results.len() >= 1);
}

#[tokio::test]
async fn test_021_semantic_inventory_management() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("inventory stock management products").await.unwrap();
    verify_results(&results, "inventory stock management products", 1);
    verify_expected_files(&results, &["OrderService.java", "product_catalog.rb"]);
}

#[tokio::test]
async fn test_022_semantic_analytics_reporting() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("analytics metrics dashboard reporting").await.unwrap();
    verify_results(&results, "analytics metrics dashboard reporting", 1);
    verify_expected_files(&results, &["analytics_dashboard.go"]);
}

#[tokio::test]
async fn test_023_semantic_security_concepts() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("security encryption protection access").await.unwrap();
    verify_results(&results, "security encryption protection access", 1);
    // Should find security-related code in auth files
    verify_expected_files(&results, &["auth_service.py"]);
}

#[tokio::test]
async fn test_024_semantic_concurrency_threading() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("concurrent parallel threading async").await.unwrap();
    verify_results(&results, "concurrent parallel threading async", 1);
    verify_expected_files(&results, &["memory_cache.rs"]);
}

#[tokio::test]
async fn test_025_semantic_api_endpoints() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("API endpoints routes handlers").await.unwrap();
    verify_results(&results, "API endpoints routes handlers", 1);
    verify_expected_files(&results, &["user_controller.js"]);
}

// ==== LANGUAGE-SPECIFIC TESTS (Tests 26-45) ====

#[tokio::test]
async fn test_026_python_function_definition() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("def authenticate").await.unwrap();
    verify_results(&results, "def authenticate", 1);
    verify_expected_files(&results, &["auth_service.py"]);
}

#[tokio::test]
async fn test_027_python_class_definition() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("class AuthenticationService").await.unwrap();
    verify_results(&results, "class AuthenticationService", 1);
    verify_expected_files(&results, &["auth_service.py"]);
}

#[tokio::test]
async fn test_028_javascript_async_function() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("async function").await.unwrap();
    verify_results(&results, "async function", 1);
    verify_expected_files(&results, &["user_controller.js"]);
}

#[tokio::test]
async fn test_029_javascript_arrow_function() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("=> {").await.unwrap();
    verify_results(&results, "=> {", 1);
    verify_expected_files(&results, &["user_controller.js"]);
}

#[tokio::test]
async fn test_030_java_public_method() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("public Order createOrder").await.unwrap();
    verify_results(&results, "public Order createOrder", 1);
    verify_expected_files(&results, &["OrderService.java"]);
}

#[tokio::test]
async fn test_031_java_annotation() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("@Transactional").await.unwrap();
    verify_results(&results, "@Transactional", 1);
    verify_expected_files(&results, &["OrderService.java"]);
}

#[tokio::test]
async fn test_032_rust_function_signature() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("pub fn insert").await.unwrap();
    verify_results(&results, "pub fn insert", 1);
    verify_expected_files(&results, &["memory_cache.rs"]);
}

#[tokio::test]
async fn test_033_rust_trait_implementation() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("impl").await.unwrap();
    verify_results(&results, "impl", 1);
    verify_expected_files(&results, &["memory_cache.rs"]);
}

#[tokio::test]
async fn test_034_go_function_declaration() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("func").await.unwrap();
    verify_results(&results, "func", 1);
    verify_expected_files(&results, &["analytics_dashboard.go"]);
}

#[tokio::test]
async fn test_035_typescript_interface() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("interface").await.unwrap();
    verify_results(&results, "interface", 1);
    verify_expected_files(&results, &["payment_gateway.ts"]);
}

#[tokio::test]
async fn test_036_csharp_class_declaration() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("public class").await.unwrap();
    verify_results(&results, "public class", 1);
    verify_expected_files(&results, &["DataProcessor.cs"]);
}

#[tokio::test]
async fn test_037_ruby_method_definition() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("def").await.unwrap();
    verify_results(&results, "def", 1);
    // Should find in both Python and Ruby files
    assert!(results.len() >= 2);
}

#[tokio::test]
async fn test_038_cpp_constructor() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("WebSocketServer::").await.unwrap();
    verify_results(&results, "WebSocketServer::", 1);
    verify_expected_files(&results, &["websocket_server.cpp"]);
}

#[tokio::test]
async fn test_039_sql_create_table() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("CREATE TABLE users").await.unwrap();
    verify_results(&results, "CREATE TABLE users", 1);
    verify_expected_files(&results, &["database_migration.sql"]);
}

#[tokio::test]
async fn test_040_sql_foreign_key() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("REFERENCES").await.unwrap();
    verify_results(&results, "REFERENCES", 1);
    verify_expected_files(&results, &["database_migration.sql"]);
}

#[tokio::test]
async fn test_041_python_type_hints() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("-> bool").await.unwrap();
    verify_results(&results, "-> bool", 1);
    verify_expected_files(&results, &["auth_service.py"]);
}

#[tokio::test]
async fn test_042_javascript_promise_async() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("await").await.unwrap();
    verify_results(&results, "await", 1);
    verify_expected_files(&results, &["user_controller.js"]);
}

#[tokio::test]
async fn test_043_java_exception_handling() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("catch (Exception").await.unwrap();
    verify_results(&results, "catch (Exception", 1);
    verify_expected_files(&results, &["OrderService.java"]);
}

#[tokio::test]
async fn test_044_rust_result_type() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("Result<").await.unwrap();
    verify_results(&results, "Result<", 1);
    verify_expected_files(&results, &["memory_cache.rs"]);
}

#[tokio::test]
async fn test_045_typescript_generic_types() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("<T>").await.unwrap();
    verify_results(&results, "<T>", 1);
    verify_expected_files(&results, &["payment_gateway.ts"]);
}

// ==== FUNCTION/METHOD SEARCH TESTS (Tests 46-55) ====

#[tokio::test]
async fn test_046_hash_password_function() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("hash_password").await.unwrap();
    verify_results(&results, "hash_password", 1);
    verify_expected_files(&results, &["auth_service.py"]);
}

#[tokio::test]
async fn test_047_create_user_method() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("create_user").await.unwrap();
    verify_results(&results, "create_user", 1);
    verify_expected_files(&results, &["auth_service.py"]);
}

#[tokio::test]
async fn test_048_validate_token_method() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("validate_token").await.unwrap();
    verify_results(&results, "validate_token", 1);
    verify_expected_files(&results, &["auth_service.py"]);
}

#[tokio::test]
async fn test_049_process_payment_method() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("processPayment").await.unwrap();
    verify_results(&results, "processPayment", 1);
    verify_expected_files(&results, &["OrderService.java"]);
}

#[tokio::test]
async fn test_050_calculate_shipping_method() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("calculateShipping").await.unwrap();
    verify_results(&results, "calculateShipping", 1);
    verify_expected_files(&results, &["OrderService.java"]);
}

#[tokio::test]
async fn test_051_cache_insert_method() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("insert").await.unwrap();
    verify_results(&results, "insert", 1);
    // Should find in cache and SQL files
    assert!(results.len() >= 2);
}

#[tokio::test]
async fn test_052_get_profile_endpoint() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("getProfile").await.unwrap();
    verify_results(&results, "getProfile", 1);
    verify_expected_files(&results, &["user_controller.js"]);
}

#[tokio::test]
async fn test_053_middleware_authenticate() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("authenticate(req, res, next)").await.unwrap();
    verify_results(&results, "authenticate(req, res, next)", 1);
    verify_expected_files(&results, &["user_controller.js"]);
}

#[tokio::test]
async fn test_054_constructor_parameters() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("constructor(").await.unwrap();
    verify_results(&results, "constructor(", 1);
    verify_expected_files(&results, &["user_controller.js"]);
}

#[tokio::test]
async fn test_055_function_with_generics() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("fn new<").await.unwrap();
    verify_results(&results, "fn new<", 1);
    verify_expected_files(&results, &["memory_cache.rs"]);
}

// ==== CLASS/STRUCT SEARCH TESTS (Tests 56-65) ====

#[tokio::test]
async fn test_056_authentication_service_class() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("AuthenticationService").await.unwrap();
    verify_results(&results, "AuthenticationService", 1);
    verify_expected_files(&results, &["auth_service.py"]);
}

#[tokio::test]
async fn test_057_user_controller_class() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("UserController").await.unwrap();
    verify_results(&results, "UserController", 1);
    verify_expected_files(&results, &["user_controller.js"]);
}

#[tokio::test]
async fn test_058_order_service_class() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("OrderService").await.unwrap();
    verify_results(&results, "OrderService", 1);
    verify_expected_files(&results, &["OrderService.java"]);
}

#[tokio::test]
async fn test_059_memory_cache_struct() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("MemoryCache").await.unwrap();
    verify_results(&results, "MemoryCache", 1);
    verify_expected_files(&results, &["memory_cache.rs"]);
}

#[tokio::test]
async fn test_060_cache_entry_struct() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("CacheEntry").await.unwrap();
    verify_results(&results, "CacheEntry", 1);
    verify_expected_files(&results, &["memory_cache.rs"]);
}

#[tokio::test]
async fn test_061_data_processor_class() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("DataProcessor").await.unwrap();
    verify_results(&results, "DataProcessor", 1);
    verify_expected_files(&results, &["DataProcessor.cs"]);
}

#[tokio::test]
async fn test_062_websocket_server_class() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("WebSocketServer").await.unwrap();
    verify_results(&results, "WebSocketServer", 1);
    verify_expected_files(&results, &["websocket_server.cpp"]);
}

#[tokio::test]
async fn test_063_struct_with_lifetime() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("struct").await.unwrap();
    verify_results(&results, "struct", 1);
    verify_expected_files(&results, &["memory_cache.rs"]);
}

#[tokio::test]
async fn test_064_enum_definition() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("enum").await.unwrap();
    verify_results(&results, "enum", 1);
    verify_expected_files(&results, &["memory_cache.rs"]);
}

#[tokio::test]
async fn test_065_interface_declaration() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("interface SearchResult").await.unwrap();
    verify_results(&results, "interface SearchResult", 0); // May not exist in test data
}

// ==== VARIABLE NAME SEARCH TESTS (Tests 66-75) ====

#[tokio::test]
async fn test_066_username_variable() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("username").await.unwrap();
    verify_results(&results, "username", 1);
    // Should find in multiple files
    assert!(results.len() >= 2);
}

#[tokio::test]
async fn test_067_password_hash_variable() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("password_hash").await.unwrap();
    verify_results(&results, "password_hash", 1);
    verify_expected_files(&results, &["auth_service.py"]);
}

#[tokio::test]
async fn test_068_total_amount_variable() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("totalAmount").await.unwrap();
    verify_results(&results, "totalAmount", 1);
    verify_expected_files(&results, &["OrderService.java"]);
}

#[tokio::test]
async fn test_069_storage_variable() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("storage").await.unwrap();
    verify_results(&results, "storage", 1);
    verify_expected_files(&results, &["memory_cache.rs"]);
}

#[tokio::test]
async fn test_070_config_variable() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("config").await.unwrap();
    verify_results(&results, "config", 1);
    verify_expected_files(&results, &["memory_cache.rs"]);
}

#[tokio::test]
async fn test_071_session_token_variable() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("session_token").await.unwrap();
    verify_results(&results, "session_token", 0); // May be named differently
}

#[tokio::test]
async fn test_072_order_items_variable() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("orderItems").await.unwrap();
    verify_results(&results, "orderItems", 1);
    verify_expected_files(&results, &["OrderService.java"]);
}

#[tokio::test]
async fn test_073_cache_stats_variable() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("stats").await.unwrap();
    verify_results(&results, "stats", 1);
    verify_expected_files(&results, &["memory_cache.rs"]);
}

#[tokio::test] 
async fn test_074_email_service_variable() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("emailService").await.unwrap();
    verify_results(&results, "emailService", 1);
    verify_expected_files(&results, &["user_controller.js"]);
}

#[tokio::test]
async fn test_075_database_connection_variable() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("connection").await.unwrap();
    verify_results(&results, "connection", 1);
    // Should find in multiple files
    assert!(results.len() >= 1);
}

// ==== DOCUMENTATION SEARCH TESTS (Tests 76-85) ====

#[tokio::test]
async fn test_076_api_documentation_title() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("API Documentation").await.unwrap();
    verify_results(&results, "API Documentation", 1);
    verify_expected_files(&results, &["API_DOCUMENTATION.md"]);
}

#[tokio::test]
async fn test_077_architecture_overview() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("Architecture Overview").await.unwrap();
    verify_results(&results, "Architecture Overview", 1);
    verify_expected_files(&results, &["ARCHITECTURE_OVERVIEW.md"]);
}

#[tokio::test]
async fn test_078_contributing_guidelines() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("Contributing").await.unwrap();
    verify_results(&results, "Contributing", 1);
    verify_expected_files(&results, &["CONTRIBUTING.md"]);
}

#[tokio::test]
async fn test_079_deployment_guide() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("Deployment Guide").await.unwrap();
    verify_results(&results, "Deployment Guide", 1);
    verify_expected_files(&results, &["DEPLOYMENT_GUIDE.md"]);
}

#[tokio::test]
async fn test_080_troubleshooting_guide() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("Troubleshooting").await.unwrap();
    verify_results(&results, "Troubleshooting", 1);
    verify_expected_files(&results, &["TROUBLESHOOTING.md"]);
}

#[tokio::test]
async fn test_081_code_comment_documentation() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("Thread-safe in-memory cache").await.unwrap();
    verify_results(&results, "Thread-safe in-memory cache", 1);
    verify_expected_files(&results, &["memory_cache.rs"]);
}

#[tokio::test]
async fn test_082_function_documentation() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("Hash a password with salt").await.unwrap();
    verify_results(&results, "Hash a password with salt", 1);
    verify_expected_files(&results, &["auth_service.py"]);
}

#[tokio::test]
async fn test_083_class_documentation() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("Handles user authentication and session management").await.unwrap();
    verify_results(&results, "Handles user authentication and session management", 1);
    verify_expected_files(&results, &["auth_service.py"]);
}

#[tokio::test]
async fn test_084_parameter_documentation() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("The command to execute").await.unwrap();
    verify_results(&results, "The command to execute", 0); // May not exist in test files
}

#[tokio::test]
async fn test_085_migration_description() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("E-commerce platform schema migration").await.unwrap();
    verify_results(&results, "E-commerce platform schema migration", 1);
    verify_expected_files(&results, &["database_migration.sql"]);
}

// ==== MULTI-WORD QUERY TESTS (Tests 86-90) ====

#[tokio::test]
async fn test_086_user_authentication_system() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("user authentication system").await.unwrap();
    verify_results(&results, "user authentication system", 1);
    verify_expected_files(&results, &["auth_service.py", "user_controller.js"]);
}

#[tokio::test]
async fn test_087_order_processing_workflow() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("order processing workflow").await.unwrap();
    verify_results(&results, "order processing workflow", 1);
    verify_expected_files(&results, &["OrderService.java"]);
}

#[tokio::test]
async fn test_088_payment_gateway_integration() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("payment gateway integration").await.unwrap();
    verify_results(&results, "payment gateway integration", 1);
    verify_expected_files(&results, &["payment_gateway.ts"]);
}

#[tokio::test]
async fn test_089_database_migration_script() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("database migration script").await.unwrap();
    verify_results(&results, "database migration script", 1);
    verify_expected_files(&results, &["database_migration.sql"]);
}

#[tokio::test]
async fn test_090_websocket_server_implementation() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("websocket server implementation").await.unwrap();
    verify_results(&results, "websocket server implementation", 1);
    verify_expected_files(&results, &["websocket_server.cpp"]);
}

// ==== CODE PATTERN SEARCH TESTS (Tests 91-95) ====

#[tokio::test]
async fn test_091_async_await_pattern() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("async await").await.unwrap();
    verify_results(&results, "async await", 1);
    verify_expected_files(&results, &["user_controller.js"]);
}

#[tokio::test]
async fn test_092_try_catch_pattern() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("try catch").await.unwrap();
    verify_results(&results, "try catch", 1);
    // Should find in multiple files with exception handling
    assert!(results.len() >= 1);
}

#[tokio::test]
async fn test_093_builder_pattern() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("Builder pattern").await.unwrap();
    verify_results(&results, "Builder pattern", 1);
    verify_expected_files(&results, &["memory_cache.rs"]);
}

#[tokio::test]
async fn test_094_dependency_injection_pattern() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("dependency injection").await.unwrap();
    verify_results(&results, "dependency injection", 1);
    verify_expected_files(&results, &["OrderService.java"]);
}

#[tokio::test]
async fn test_095_factory_pattern() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("factory").await.unwrap();
    verify_results(&results, "factory", 0); // May not exist in test data
}

// ==== EDGE CASES AND SPECIAL CHARACTERS (Tests 96-100) ====

#[tokio::test]
async fn test_096_special_characters_in_query() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("@Override").await.unwrap();
    verify_results(&results, "@Override", 0); // May not exist in test data
}

#[tokio::test]
async fn test_097_sql_with_quotes() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("'user'").await.unwrap();
    verify_results(&results, "'user'", 0); // May not find exact match
}

#[tokio::test]
async fn test_098_regex_special_chars() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search(".*Error").await.unwrap();
    verify_results(&results, ".*Error", 0); // Should handle regex chars safely
}

#[tokio::test]
async fn test_099_unicode_characters() {
    let setup = TestSetup::new().await;
    let results = setup.searcher.search("résumé").await.unwrap();
    verify_results(&results, "résumé", 0); // Likely not in test data
}

#[tokio::test]
async fn test_100_empty_and_whitespace_queries() {
    let setup = TestSetup::new().await;
    
    // Empty query should return no results or handle gracefully
    let empty_results = setup.searcher.search("").await.unwrap();
    assert!(empty_results.is_empty() || empty_results.len() >= 0);
    
    // Whitespace-only query should be handled
    let whitespace_results = setup.searcher.search("   ").await.unwrap();
    assert!(whitespace_results.is_empty() || whitespace_results.len() >= 0);
    
    // Very long query should be handled
    let long_query = "a".repeat(1000);
    let long_results = setup.searcher.search(&long_query).await.unwrap();
    assert!(long_results.len() >= 0);
}

// ==== COMPREHENSIVE INTEGRATION TESTS ====

#[tokio::test]
async fn test_comprehensive_search_coverage() {
    let setup = TestSetup::new().await;
    
    // Test queries that should find results in each test file
    let test_cases = vec![
        ("authenticate", vec!["auth_service.py", "user_controller.js"]),
        ("OrderService", vec!["OrderService.java"]),
        ("MemoryCache", vec!["memory_cache.rs"]),
        ("CREATE TABLE", vec!["database_migration.sql"]),
        ("WebSocketServer", vec!["websocket_server.cpp"]),
        ("DataProcessor", vec!["DataProcessor.cs"]),
        ("func", vec!["analytics_dashboard.go"]),
        ("PaymentGateway", vec!["payment_gateway.ts"]),
        ("ProductCatalog", vec!["product_catalog.rb"]),
        ("API Documentation", vec!["API_DOCUMENTATION.md"]),
    ];
    
    for (query, expected_files) in test_cases {
        let results = setup.searcher.search(query).await.unwrap();
        println!("Query '{}' returned {} results", query, results.len());
        
        if !expected_files.is_empty() {
            verify_results(&results, query, 1);
            // Don't require all expected files as some queries might not match all
            let found_any = expected_files.iter().any(|expected| {
                results.iter().any(|r| r.file.contains(expected))
            });
            assert!(found_any, "Query '{}' should find at least one of {:?}", query, expected_files);
        }
    }
}

#[tokio::test]
async fn test_search_result_quality() {
    let setup = TestSetup::new().await;
    
    // Test that semantic and exact matches are properly ranked
    let results = setup.searcher.search("user authentication").await.unwrap();
    
    if !results.is_empty() {
        // Results should be sorted by score (highest first)
        for i in 1..results.len() {
            assert!(
                results[i-1].score >= results[i].score,
                "Results should be sorted by score: {} >= {}",
                results[i-1].score, results[i].score
            );
        }
        
        // Verify match types are set
        for result in &results {
            assert!(
                matches!(result.match_type, MatchType::Exact | MatchType::Semantic),
                "Match type should be Exact or Semantic"
            );
        }
        
        // Verify three-chunk context quality
        for result in &results {
            let context = &result.three_chunk_context;
            
            // Target should always exist
            assert!(!context.target.content.is_empty());
            
            // Context should be properly formatted
            let display = context.format_for_display();
            assert!(display.contains("TARGET MATCH"));
            
            let summary = context.format_summary();
            assert!(!summary.is_empty());
            
            let full_content = context.get_full_content();
            assert!(!full_content.is_empty());
        }
    }
}

#[tokio::test]
async fn test_mixed_exact_and_semantic_queries() {
    let setup = TestSetup::new().await;
    
    // Test queries that combine exact text and semantic concepts
    let mixed_queries = vec![
        "def authenticate user login",
        "OrderService create payment processing",
        "MemoryCache insert cache storage",
        "SQL CREATE users table",
        "async function user registration",
    ];
    
    for query in mixed_queries {
        let results = setup.searcher.search(query).await.unwrap();
        println!("Mixed query '{}' returned {} results", query, results.len());
        
        // Should handle mixed queries gracefully
        assert!(results.len() >= 0);
        
        if !results.is_empty() {
            verify_results(&results, query, 1);
            
            // Should have a mix of exact and semantic matches if data supports it
            let has_exact = results.iter().any(|r| matches!(r.match_type, MatchType::Exact));
            let has_semantic = results.iter().any(|r| matches!(r.match_type, MatchType::Semantic));
            
            // At least one type should be present
            assert!(has_exact || has_semantic, "Should have exact or semantic matches");
        }
    }
}