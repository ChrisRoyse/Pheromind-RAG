#[cfg(feature = "tantivy")]
use std::fs;
#[cfg(feature = "tantivy")]
use anyhow::Result;
#[cfg(feature = "tantivy")]
use embed_search::search::tantivy_search::TantivySearcher;

#[cfg(feature = "tantivy")]
#[tokio::main]
async fn main() -> Result<()> {
    println!("🔥 Testing Tantivy Persistent Storage Implementation");
    
    let test_dir = std::env::temp_dir().join("tantivy_persistence_test");
    let index_path = test_dir.join("test_index");
    
    // Clean up from previous runs
    if test_dir.exists() {
        fs::remove_dir_all(&test_dir)?;
    }
    
    fs::create_dir_all(&test_dir)?;
    
    println!("\n📁 Test directory: {:?}", test_dir);
    println!("📊 Index path: {:?}", index_path);
    
    // Create test file with content to index
    let test_file = test_dir.join("sample_code.rs");
    let test_content = r#"
/// Authentication module for secure user access
pub mod authentication {
    use std::collections::HashMap;
    
    pub struct AuthenticationService {
        users: HashMap<String, String>,
    }
    
    impl AuthenticationService {
        pub fn new() -> Self {
            Self {
                users: HashMap::new(),
            }
        }
        
        /// Authenticate a user with username and password
        pub fn authenticate_user(&self, username: &str, password: &str) -> bool {
            if let Some(stored_password) = self.users.get(username) {
                stored_password == password
            } else {
                false
            }
        }
        
        /// Configure the authentication system
        pub fn configure_auth_settings(&mut self, timeout_seconds: u32) {
            println!("Configuring authentication timeout: {} seconds", timeout_seconds);
        }
        
        /// Add a new user to the system
        pub fn add_user(&mut self, username: String, password: String) {
            self.users.insert(username, password);
        }
    }
}

/// Database connection module
pub mod database {
    /// Connect to database with retry logic
    pub async fn connect_database() -> Result<DatabaseConnection, std::io::Error> {
        println!("Connecting to database...");
        // Create actual connection - no simulation
        Ok(DatabaseConnection::new())
    }
    
    pub struct DatabaseConnection {
        connected: bool,
    }
    
    impl DatabaseConnection {
        pub fn new() -> Self {
            Self { connected: true }
        }
    }
}
"#;
    
    fs::write(&test_file, test_content)?;
    println!("✅ Created test file: {:?}", test_file);
    
    // PHASE 1: Create searcher and index the file
    println!("\n🏗️  PHASE 1: Creating persistent searcher and indexing...");
    {
        let mut searcher = TantivySearcher::new_with_path(&index_path).await?;
        println!("✅ Created persistent TantivySearcher");
        
        println!("Is persistent: {}", searcher.is_persistent());
        println!("Index path: {:?}", searcher.index_path());
        
        // Index the test file
        println!("Indexing file...");
        searcher.index_file(&test_file).await?;
        println!("✅ File indexed successfully");
        
        // Test basic search
        let results = searcher.search("authenticate").await?;
        println!("Found {} results for 'authenticate'", results.len());
        
        if !results.is_empty() {
            println!("  Sample result: {} at line {}", results[0].file_path, results[0].line_number);
        }
        
        // Test fuzzy search
        let fuzzy_results = searcher.search_fuzzy("authenticat", 2).await?;
        println!("Found {} fuzzy results for 'authenticat' (missing 'e')", fuzzy_results.len());
        
        // Get index stats
        let stats = searcher.get_index_stats()?;
        println!("Index stats: {}", stats);
        
        println!("✅ Phase 1 completed - searcher will be dropped now (simulating restart)");
    }
    
    // Verify index files were created
    println!("\n💾 Verifying persistent storage...");
    assert!(index_path.exists(), "Index directory should exist on disk");
    assert!(index_path.is_dir(), "Index path should be a directory");
    
    let mut index_files = Vec::new();
    for entry_result in fs::read_dir(&index_path)? {
        match entry_result {
            Ok(entry) => index_files.push(entry),
            Err(e) => {
                eprintln!("Failed to read directory entry: {}", e);
                return Err(e.into());
            }
        }
    }
    println!("Found {} index files on disk", index_files.len());
    for file in &index_files {
        let metadata = file.metadata()
            .map_err(|e| anyhow::anyhow!("Failed to read metadata for file {:?}: {}", file.file_name(), e))?;
        println!("  {:?} ({} bytes)", file.file_name(), metadata.len());
    }
    
    // PHASE 2: Create new searcher instance - should load existing index
    println!("\n🔄 PHASE 2: Loading persisted index (simulating restart)...");
    {
        let searcher = TantivySearcher::new_with_path(&index_path).await?;
        println!("✅ Created new searcher instance from persisted index");
        
        // Verify we can search without re-indexing
        let results = searcher.search("authenticate").await?;
        println!("Found {} results for 'authenticate' (from persisted index)", results.len());
        assert!(!results.is_empty(), "Should find 'authenticate' in persisted index");
        
        let configure_results = searcher.search("configure").await?;
        println!("Found {} results for 'configure'", configure_results.len());
        assert!(!configure_results.is_empty(), "Should find 'configure' in persisted index");
        
        let database_results = searcher.search("database").await?;
        println!("Found {} results for 'database'", database_results.len());
        
        // Test fuzzy search with persisted data
        let fuzzy_auth = searcher.search_fuzzy("authenticaton", 2).await?;
        println!("Fuzzy search 'authenticaton' -> found {} results", fuzzy_auth.len());
        
        let fuzzy_config = searcher.search_fuzzy("configurr", 2).await?;
        println!("Fuzzy search 'configurr' -> found {} results", fuzzy_config.len());
        
        let fuzzy_databse = searcher.search_fuzzy("databse", 2).await?;
        println!("Fuzzy search 'databse' -> found {} results", fuzzy_databse.len());
        
        // Final stats
        let final_stats = searcher.get_index_stats()?;
        println!("Final index stats: {}", final_stats);
        
        println!("✅ Phase 2 completed - persistent storage verified");
    }
    
    // PHASE 3: Test incremental indexing
    println!("\n➕ PHASE 3: Testing incremental indexing...");
    {
        // Create a second test file
        let test_file2 = test_dir.join("additional_code.rs");
        let additional_content = r#"
/// Logging and monitoring utilities
pub mod logging {
    /// Initialize the logging system
    pub fn initialize_logger() {
        println!("Logger initialized");
    }
    
    /// Log an authentication event
    pub fn log_authentication_event(username: &str, success: bool) {
        if success {
            println!("User {} authenticated successfully", username);
        } else {
            println!("Authentication failed for user {}", username);
        }
    }
}

/// Configuration management
pub mod configuration {
    /// Load configuration explicitly - no defaults allowed
    pub fn load_configuration() -> Config {
        Config {
            timeout: 30,  // Explicit configuration
            debug: false, // Explicit configuration
        }
    }
    
    pub struct Config {
        pub timeout: u32,
        pub debug: bool,
    }
    
    // Config must be explicitly created - no Default fallback allowed
}
"#;
        
        fs::write(&test_file2, additional_content)?;
        println!("✅ Created additional test file: {:?}", test_file2);
        
        // Open existing index and add new file
        let mut searcher = TantivySearcher::new_with_path(&index_path).await?;
        println!("✅ Reopened existing index");
        
        // Verify old content is still there
        let old_results = searcher.search("authenticate").await?;
        println!("Old content check - 'authenticate': {} results", old_results.len());
        
        // Index new file
        searcher.index_file(&test_file2).await?;
        println!("✅ Added new file to existing index");
        
        // Test searches for both old and new content
        let auth_results = searcher.search("authenticate").await?;
        println!("After incremental indexing - 'authenticate': {} results", auth_results.len());
        
        let logger_results = searcher.search("logger").await?;
        println!("New content - 'logger': {} results", logger_results.len());
        assert!(!logger_results.is_empty(), "Should find 'logger' from new file");
        
        let config_results = searcher.search("configuration").await?;
        println!("New content - 'configuration': {} results", config_results.len());
        assert!(!config_results.is_empty(), "Should find 'configuration' from new file");
        
        let final_stats = searcher.get_index_stats()?;
        println!("After incremental indexing: {}", final_stats);
        
        println!("✅ Phase 3 completed - incremental indexing verified");
    }
    
    // PHASE 4: Verify persistence after incremental indexing
    println!("\n🔍 PHASE 4: Final persistence verification...");
    {
        let searcher = TantivySearcher::new_with_path(&index_path).await?;
        
        // Should find content from both files
        let auth_results = searcher.search("authenticate").await?;
        let logger_results = searcher.search("logger").await?;
        let config_results = searcher.search("configuration").await?;
        
        println!("Final verification:");
        println!("  'authenticate' (original file): {} results", auth_results.len());
        println!("  'logger' (new file): {} results", logger_results.len());
        println!("  'configuration' (new file): {} results", config_results.len());
        
        assert!(!auth_results.is_empty(), "Should still find original content");
        assert!(!logger_results.is_empty(), "Should find new content");
        assert!(!config_results.is_empty(), "Should find new content");
        
        println!("✅ All content persisted correctly across restarts");
    }
    
    println!("\n🎉 SUCCESS: Tantivy persistent storage implementation is FULLY OPERATIONAL!");
    println!("🚀 Ready for production deployment!");
    
    // Clean up
    fs::remove_dir_all(&test_dir)?;
    println!("🧹 Cleaned up test directory");
    
    Ok(())
}

#[cfg(not(feature = "tantivy"))]
fn main() {
    println!("❌ test_persistence requires 'tantivy' feature to be enabled");
    std::process::exit(1);
}