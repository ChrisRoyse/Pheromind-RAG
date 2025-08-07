# Task 2.012: Test End-to-End BM25 Integration

**Time Estimate**: 10 minutes
**Priority**: HIGH
**Dependencies**: task_011
**File(s) to Modify**: `tests/bm25_integration_tests.rs`

## Objective
Run comprehensive end-to-end tests to verify the complete BM25 search pipeline works from UnifiedSearcher through to returning results.

## Success Criteria
- [ ] Basic search test passes
- [ ] Multi-word queries return results
- [ ] Results contain expected files
- [ ] Search performance is reasonable

## Instructions

### Step 1: Create comprehensive integration test
```rust
// Add this comprehensive test to tests/bm25_integration_tests.rs
#[tokio::test]
async fn test_bm25_end_to_end_integration() -> Result<()> {
    println!("🧪 INTEGRATION TEST: Starting comprehensive BM25 end-to-end test");
    
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join(".embed_db");
    
    // Create multiple test files with known content
    let files = vec![
        ("auth.rs", r#"
pub struct AuthService {
    database: DatabaseConnection,
}

impl AuthService {
    pub fn authenticate_user(&self, username: &str, password: &str) -> Result<User> {
        let user = self.database.get_user(username)?;
        if user.verify_password(password) {
            Ok(user)
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }
}
"#),
        ("database.rs", r#"
pub struct DatabaseConnection {
    connection_pool: ConnectionPool,
}

impl DatabaseConnection {
    pub fn get_user(&self, username: &str) -> Result<User> {
        let query = "SELECT * FROM users WHERE username = ?";
        self.connection_pool.execute_query(query, &[username])
    }
    
    pub fn create_connection() -> Result<DatabaseConnection> {
        let pool = ConnectionPool::new("database_url")?;
        Ok(DatabaseConnection { connection_pool: pool })
    }
}
"#),
        ("user.rs", r#"
pub struct User {
    id: u64,
    username: String,
    password_hash: String,
}

impl User {
    pub fn verify_password(&self, password: &str) -> bool {
        verify_hash(&self.password_hash, password)
    }
    
    pub fn new_user(username: String, password: String) -> Result<User> {
        let hash = hash_password(&password)?;
        Ok(User {
            id: generate_id(),
            username,
            password_hash: hash,
        })
    }
}
"#),
    ];
    
    // Write test files
    for (filename, content) in &files {
        fs::write(project_path.join(filename), content).await?;
    }
    
    println!("🧪 INTEGRATION TEST: Created {} test files", files.len());
    
    // Initialize searcher
    let searcher = UnifiedSearcher::new_with_backend(
        project_path.clone(),
        db_path,
        SearchBackend::Tantivy
    ).await?;
    
    // Index all files
    let mut indexed_files = 0;
    for entry in std::fs::read_dir(&project_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            println!("🧪 INTEGRATION TEST: Indexing {:?}", path.file_name());
            searcher.index_file(&path).await?;
            indexed_files += 1;
        }
    }
    
    println!("🧪 INTEGRATION TEST: Indexed {} files", indexed_files);
    
    // Verify index
    searcher.verify_bm25_index().await?;
    
    // Test suite: various query types
    let test_queries = vec![
        // Single terms
        ("authenticate", "Should find auth.rs"),
        ("database", "Should find database.rs and possibly others"),
        ("user", "Should find user.rs and possibly others"),
        ("password", "Should find auth.rs and user.rs"),
        
        // Multi-word queries  
        ("authenticate user", "Should find auth.rs primarily"),
        ("database connection", "Should find database.rs primarily"),
        ("verify password", "Should find both auth.rs and user.rs"),
        ("create connection", "Should find database.rs"),
        
        // Technical terms
        ("connection_pool", "Should find database.rs"),
        ("password_hash", "Should find user.rs"),
        ("username", "Should find multiple files"),
    ];
    
    let mut successful_queries = 0;
    let total_queries = test_queries.len();
    
    for (query, description) in test_queries {
        println!("\n🧪 INTEGRATION TEST: Testing query: '{}' ({})", query, description);
        
        match searcher.search(query).await {
            Ok(results) => {
                println!("✅ Query '{}' returned {} results", query, results.len());
                
                if !results.is_empty() {
                    successful_queries += 1;
                    
                    // Show top results
                    for (i, result) in results.iter().take(3).enumerate() {
                        println!("   Result {}: file={}, score={:?}, match_type={:?}", 
                                 i, result.file, result.score, result.match_type);
                    }
                } else {
                    println!("❌ Query '{}' returned no results", query);
                }
            }
            Err(e) => {
                println!("❌ Query '{}' failed with error: {}", query, e);
            }
        }
    }
    
    let success_rate = (successful_queries as f32 / total_queries as f32) * 100.0;
    println!("\n🧪 INTEGRATION TEST: Success rate: {:.1}% ({}/{})", 
             success_rate, successful_queries, total_queries);
    
    // Minimum acceptable success rate
    assert!(success_rate >= 70.0, 
            "BM25 integration should achieve at least 70% success rate, got {:.1}%", success_rate);
    
    println!("🧪 INTEGRATION TEST: ✅ End-to-end integration test passed!");
    
    Ok(())
}
```

### Step 2: Test the original failing test case
```rust
// Update the original failing test with better debugging
#[tokio::test]
async fn test_bm25_original_case() -> Result<()> {
    println!("🧪 ORIGINAL TEST: Testing the original failing BM25 case");
    
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join(".embed_db");
    
    // Create the original test codebase
    create_test_codebase(&project_path).await?;
    
    // Initialize searcher with BM25
    let searcher = UnifiedSearcher::new_with_backend(
        project_path.clone(),
        db_path,
        SearchBackend::Tantivy
    ).await?;
    
    // Index all files
    for entry in std::fs::read_dir(&project_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            searcher.index_file(&path).await?;
        }
    }
    
    // The original failing assertion
    let results = searcher.search("database connection").await?;
    
    println!("🧪 ORIGINAL TEST: 'database connection' query returned {} results", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("   Result {}: {}", i, result.file);
    }
    
    // This should now pass
    assert!(!results.is_empty(), "Should find results for 'database connection'");
    
    println!("🧪 ORIGINAL TEST: ✅ Original test case now passes!");
    Ok(())
}
```

### Step 3: Performance and stress test
```rust
// Add performance test
#[tokio::test]
async fn test_bm25_performance() -> Result<()> {
    println!("🧪 PERFORMANCE TEST: Testing BM25 search performance");
    
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join(".embed_db");
    
    // Create multiple files for performance testing
    for i in 0..10 {
        let content = format!(r#"
pub struct Service{} {{
    database: Database,
    cache: Cache,
    logger: Logger,
}}

impl Service{} {{
    pub fn process_data(&self, data: &str) -> Result<String> {{
        let processed = self.database.query("SELECT * FROM data WHERE value = ?", &[data])?;
        self.logger.info("Processing completed");
        self.cache.store("key_{}", processed.clone());
        Ok(processed)
    }}
    
    pub fn authenticate(&self, user: &str, password: &str) -> bool {{
        self.database.verify_credentials(user, password)
    }}
}}
"#, i, i, i);
        
        fs::write(project_path.join(format!("service_{}.rs", i)), content).await?;
    }
    
    let searcher = UnifiedSearcher::new_with_backend(
        project_path.clone(),
        db_path,
        SearchBackend::Tantivy
    ).await?;
    
    // Index files and measure time
    let start = std::time::Instant::now();
    for i in 0..10 {
        searcher.index_file(&project_path.join(format!("service_{}.rs", i))).await?;
    }
    let indexing_time = start.elapsed();
    
    println!("🧪 PERFORMANCE TEST: Indexed 10 files in {:?}", indexing_time);
    
    // Search performance
    let queries = vec!["database", "process data", "authenticate user", "cache store"];
    
    let start = std::time::Instant::now();
    for query in queries {
        let results = searcher.search(query).await?;
        println!("🧪 PERFORMANCE TEST: Query '{}' returned {} results", query, results.len());
    }
    let search_time = start.elapsed();
    
    println!("🧪 PERFORMANCE TEST: Completed 4 searches in {:?}", search_time);
    
    // Performance assertions
    assert!(indexing_time.as_millis() < 5000, "Indexing should be fast");
    assert!(search_time.as_millis() < 1000, "Searching should be fast");
    
    Ok(())
}
```

### Step 4: Run all integration tests
```bash
cd C:\code\embed
cargo test test_bm25_end_to_end_integration -- --nocapture
cargo test test_bm25_original_case -- --nocapture
cargo test test_bm25_performance -- --nocapture
```

## Terminal Commands
```bash
cd C:\code\embed
cargo test test_bm25_end_to_end_integration -- --nocapture
cargo test test_bm25_original_case -- --nocapture
cargo test test_bm25_performance -- --nocapture
```

## Expected Results
- End-to-end test achieves >70% success rate
- Original test case passes (returns non-empty results)
- Performance is reasonable (<5s indexing, <1s searching)

## Success Criteria
- All integration tests pass
- No panic or error conditions
- Performance meets basic expectations
- Debug output shows healthy operation

## Next Task
task_013 - Remove debug logging and clean up code