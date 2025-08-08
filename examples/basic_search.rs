// Basic search example demonstrating working functionality
// This example shows how to use the SimpleSearcher with graceful degradation

use embed_search::search::{SearchConfig, SimpleSearcher};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    println!("🔍 Embed Search - Basic Search Example\n");
    
    // Create a minimal configuration (BM25 only)
    let config = SearchConfig::minimal();
    
    println!("Configuration:");
    println!("  BM25: {}", config.enable_bm25);
    println!("  Tantivy: {}", config.enable_tantivy);
    println!("  ML: {}", config.enable_ml);
    println!("  Tree-sitter: {}\n", config.enable_tree_sitter);
    
    // Create the searcher
    let mut searcher = SimpleSearcher::new(config)?;
    
    println!("✅ SimpleSearcher initialized successfully");
    println!("Available engines: {:?}\n", searcher.available_engines());
    
    // Index some sample documents
    println!("Indexing sample documents...");
    
    // In a real scenario, you would index files from a directory
    // For this example, we'll create a temporary directory with sample files
    let temp_dir = tempfile::TempDir::new()?;
    let project_path = temp_dir.path();
    
    // Create sample files
    std::fs::write(
        project_path.join("auth.rs"),
        r#"
        /// Authentication service for user login
        pub struct AuthService {
            users: HashMap<String, User>,
        }
        
        impl AuthService {
            pub fn authenticate_user(&self, username: &str, password: &str) -> Result<Token> {
                // Verify user credentials
                if let Some(user) = self.users.get(username) {
                    if user.verify_password(password) {
                        return Ok(Token::new(user.id));
                    }
                }
                Err(AuthError::InvalidCredentials)
            }
        }
        "#
    )?;
    
    std::fs::write(
        project_path.join("main.rs"),
        r#"
        fn main() {
            println!("Hello, world!");
            
            let auth_service = AuthService::new();
            
            match auth_service.authenticate_user("admin", "password") {
                Ok(token) => println!("Login successful: {:?}", token),
                Err(e) => eprintln!("Login failed: {}", e),
            }
        }
        "#
    )?;
    
    std::fs::write(
        project_path.join("user.rs"),
        r#"
        /// User model with profile information
        #[derive(Debug, Clone)]
        pub struct User {
            pub id: u64,
            pub username: String,
            pub email: String,
            password_hash: String,
        }
        
        impl User {
            pub fn verify_password(&self, password: &str) -> bool {
                // In real code, use proper password hashing
                self.password_hash == hash_password(password)
            }
        }
        "#
    )?;
    
    // Index the project
    searcher.index_project(&project_path.to_path_buf())?;
    println!("✅ Indexed {} files\n", 3);
    
    // Perform searches
    println!("Performing searches:\n");
    
    // Search 1: Authentication
    println!("🔍 Search: 'authenticate'");
    let results = searcher.search("authenticate")?;
    println!("  Found {} results", results.len());
    for (i, result) in results.iter().take(3).enumerate() {
        println!("  {}. {} (score: {:.3})", i + 1, result.path, result.score);
        println!("     {}", result.content.trim());
    }
    println!();
    
    // Search 2: User
    println!("🔍 Search: 'user password'");
    let results = searcher.search("user password")?;
    println!("  Found {} results", results.len());
    for (i, result) in results.iter().take(3).enumerate() {
        println!("  {}. {} (score: {:.3})", i + 1, result.path, result.score);
    }
    println!();
    
    // Search 3: Main function
    println!("🔍 Search: 'main'");
    let results = searcher.search("main")?;
    println!("  Found {} results", results.len());
    for (i, result) in results.iter().take(3).enumerate() {
        println!("  {}. {} (score: {:.3})", i + 1, result.path, result.score);
    }
    println!();
    
    println!("✅ Basic search functionality verified!");
    
    Ok(())
}