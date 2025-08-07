use std::path::Path;
use std::fs;
use tempfile::TempDir;

use embed_search::search::tantivy_search::TantivySearcher;

/// Comprehensive test suite to verify that TantivySearcher actually works correctly
/// and returns meaningful, accurate search results - not just that it compiles.

#[cfg(feature = "tantivy")]
mod tantivy_search_verification {
    use super::*;

    /// Test data representing different file types with known, searchable content
    struct TestFile {
        path: &'static str,
        content: &'static str,
        expected_lines: Vec<(usize, &'static str)>, // line_number, content_snippet
    }

    /// Create comprehensive test files with varied, searchable content
    fn create_test_files() -> Vec<TestFile> {
        vec![
            TestFile {
                path: "math_utils.rs",
                content: r#"use std::f64::consts::PI;

/// Calculate the sum of two numbers
pub fn calculateSum(a: i32, b: i32) -> i32 {
    a + b
}

/// Calculate area of a circle
pub fn calculate_area_of_circle(radius: f64) -> f64 {
    PI * radius * radius
}

/// Fibonacci sequence generator
pub fn fibonacci(n: u32) -> u64 {
    if n <= 1 {
        n as u64
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

/// Error handling utility
pub fn safe_divide(numerator: f64, denominator: f64) -> Result<f64, &'static str> {
    if denominator == 0.0 {
        Err("Division by zero")
    } else {
        Ok(numerator / denominator)
    }
}
"#,
                expected_lines: vec![
                    (4, "pub fn calculateSum(a: i32, b: i32) -> i32 {"),
                    (9, "pub fn calculate_area_of_circle(radius: f64) -> f64 {"),
                    (14, "pub fn fibonacci(n: u32) -> u64 {"),
                    (22, "pub fn safe_divide(numerator: f64, denominator: f64) -> Result<f64, &'static str> {"),
                ],
            },
            TestFile {
                path: "auth_service.py",
                content: r#"import hashlib
import bcrypt

class AuthenticationService:
    """Service for user authentication and authorization."""
    
    def __init__(self, user_database):
        self.user_db = user_database
        self.session_cache = {}
    
    def authenticate_user(self, username, password):
        """Authenticate user credentials."""
        user = self.user_db.find_user(username)
        if user and self.verify_password(password, user.password_hash):
            return self.create_session(user)
        return None
    
    def verify_password(self, password, hash):
        """Verify password against stored hash."""
        return bcrypt.checkpw(password.encode('utf-8'), hash)
    
    def create_session(self, user):
        """Create authenticated user session."""
        session_id = hashlib.sha256(f"{user.id}-{user.username}".encode()).hexdigest()
        self.session_cache[session_id] = user
        return session_id
    
    def authorize_action(self, session_id, action):
        """Authorize user action based on permissions."""
        user = self.session_cache.get(session_id)
        return user and user.has_permission(action)
"#,
                expected_lines: vec![
                    (4, "class AuthenticationService:"),
                    (11, "def authenticate_user(self, username, password):"),
                    (18, "def verify_password(self, password, hash):"),
                    (22, "def create_session(self, user):"),
                    (28, "def authorize_action(self, session_id, action):"),
                ],
            },
            TestFile {
                path: "database.js",
                content: r#"const mysql = require('mysql2');

class DatabaseConnection {
    constructor(config) {
        this.config = config;
        this.pool = mysql.createPool({
            host: config.host,
            user: config.user,
            password: config.password,
            database: config.database,
            waitForConnections: true,
            connectionLimit: 10
        });
    }
    
    async executeQuery(query, params = []) {
        return new Promise((resolve, reject) => {
            this.pool.execute(query, params, (error, results) => {
                if (error) {
                    reject(error);
                } else {
                    resolve(results);
                }
            });
        });
    }
    
    async findUserById(userId) {
        const query = 'SELECT * FROM users WHERE id = ?';
        return this.executeQuery(query, [userId]);
    }
    
    async createUser(userData) {
        const query = 'INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)';
        return this.executeQuery(query, [userData.username, userData.email, userData.passwordHash]);
    }
    
    async updateUserEmail(userId, newEmail) {
        const query = 'UPDATE users SET email = ? WHERE id = ?';
        return this.executeQuery(query, [newEmail, userId]);
    }
}

module.exports = DatabaseConnection;
"#,
                expected_lines: vec![
                    (3, "class DatabaseConnection {"),
                    (15, "async executeQuery(query, params = []) {"),
                    (27, "async findUserById(userId) {"),
                    (32, "async createUser(userData) {"),
                    (37, "async updateUserEmail(userId, newEmail) {"),
                ],
            },
            TestFile {
                path: "config.json",
                content: r#"{
    "database": {
        "host": "localhost",
        "port": 5432,
        "name": "production_db",
        "username": "app_user",
        "password": "secure_password",
        "connection_pool_size": 20
    },
    "authentication": {
        "jwt_secret": "jwt-secret-key",
        "session_timeout": 3600,
        "bcrypt_rounds": 12
    },
    "logging": {
        "level": "info",
        "file": "/var/log/app.log",
        "max_size": "100MB"
    },
    "cache": {
        "redis_url": "redis://localhost:6379",
        "ttl": 300
    },
    "feature_flags": {
        "enable_new_ui": true,
        "enable_beta_features": false
    }
}
"#,
                expected_lines: vec![
                    (2, "\"database\": {"),
                    (10, "\"authentication\": {"),
                    (15, "\"logging\": {"),
                    (20, "\"cache\": {"),
                    (24, "\"feature_flags\": {"),
                ],
            },
            TestFile {
                path: "README.md",
                content: r#"# Project Documentation

## Overview

This is a comprehensive search system with multiple backends.

## Features

- **Authentication**: Secure user authentication with JWT tokens
- **Database**: MySQL database with connection pooling
- **Caching**: Redis-based caching for performance optimization
- **Search**: Full-text search with fuzzy matching capabilities
- **Mathematical Operations**: Utilities for mathematical calculations

## Installation

```bash
npm install
npm start
```

## Configuration

Edit the `config.json` file to configure:

- Database connection settings
- Authentication parameters
- Logging configuration
- Cache settings

## Usage Examples

### Authentication

```python
auth_service = AuthenticationService(user_db)
session = auth_service.authenticate_user("username", "password")
```

### Database Operations

```javascript
const db = new DatabaseConnection(config.database);
const user = await db.findUserById(123);
```

### Mathematical Functions

```rust
let result = calculateSum(5, 10);
let area = calculate_area_of_circle(3.14);
```

## API Endpoints

- `POST /api/auth/login` - User login
- `GET /api/users/:id` - Get user by ID
- `PUT /api/users/:id/email` - Update user email
- `POST /api/calculate/sum` - Calculate sum of numbers

## Troubleshooting

If you encounter issues:

1. Check database connectivity
2. Verify authentication configuration
3. Review log files for error messages
4. Ensure all dependencies are installed
"#,
                expected_lines: vec![
                    (1, "# Project Documentation"),
                    (9, "- **Authentication**: Secure user authentication with JWT tokens"),
                    (13, "- **Mathematical Operations**: Utilities for mathematical calculations"),
                    (27, "Edit the `config.json` file to configure:"),
                    (35, "auth_service = AuthenticationService(user_db)"),
                    (41, "const db = new DatabaseConnection(config.database);"),
                    (47, "let result = calculateSum(5, 10);"),
                    (52, "- `POST /api/auth/login` - User login"),
                ],
            },
        ]
    }

    #[tokio::test]
    async fn test_tantivy_exact_search_comprehensive() {
        println!("🔍 Testing Tantivy Exact Search Functionality");
        
        // Create temporary directory and test files
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_files = create_test_files();
        
        // Write test files to disk
        for test_file in &test_files {
            let file_path = temp_dir.path().join(test_file.path);
            fs::write(&file_path, test_file.content).expect("Failed to write test file");
        }
        
        // Create and index with TantivySearcher
        let mut searcher = TantivySearcher::new().await.expect("Failed to create TantivySearcher");
        searcher.index_directory(temp_dir.path()).await.expect("Failed to index directory");
        
        // Test Case 1: Exact function name search
        println!("\n📋 Test Case 1: Exact function name search");
        let results = searcher.search("calculateSum").await.expect("Search failed");
        assert!(!results.is_empty(), "Should find matches for 'calculateSum'");
        
        let found_calculatesum = results.iter().any(|r| {
            r.content.contains("calculateSum") && r.file_path.contains("math_utils.rs")
        });
        assert!(found_calculatesum, "Should find calculateSum function in math_utils.rs");
        
        // Verify line number accuracy
        let calculatesum_match = results.iter().find(|r| r.content.contains("pub fn calculateSum")).unwrap();
        assert_eq!(calculatesum_match.line_number, 4, "calculateSum should be on line 4");
        println!("✅ Found calculateSum at line {}", calculatesum_match.line_number);
        
        // Test Case 2: Class name search in Python
        println!("\n📋 Test Case 2: Class name search");
        let results = searcher.search("AuthenticationService").await.expect("Search failed");
        assert!(!results.is_empty(), "Should find matches for 'AuthenticationService'");
        
        let found_auth_class = results.iter().any(|r| {
            r.content.contains("AuthenticationService") && r.file_path.contains("auth_service.py")
        });
        assert!(found_auth_class, "Should find AuthenticationService class");
        println!("✅ Found AuthenticationService class");
        
        // Test Case 3: JavaScript async function search
        println!("\n📋 Test Case 3: JavaScript async function search");
        let results = searcher.search("executeQuery").await.expect("Search failed");
        assert!(!results.is_empty(), "Should find matches for 'executeQuery'");
        
        let found_execute_query = results.iter().any(|r| {
            r.content.contains("executeQuery") && r.file_path.contains("database.js")
        });
        assert!(found_execute_query, "Should find executeQuery function in database.js");
        println!("✅ Found executeQuery function");
        
        // Test Case 4: JSON configuration search
        println!("\n📋 Test Case 4: JSON configuration search");
        let results = searcher.search("authentication").await.expect("Search failed");
        assert!(!results.is_empty(), "Should find matches for 'authentication'");
        
        let found_auth_config = results.iter().any(|r| {
            r.content.contains("authentication") && r.file_path.contains("config.json")
        });
        assert!(found_auth_config, "Should find authentication config in config.json");
        println!("✅ Found authentication configuration");
        
        // Test Case 5: Markdown documentation search
        println!("\n📋 Test Case 5: Markdown documentation search");
        let results = searcher.search("API Endpoints").await.expect("Search failed");
        assert!(!results.is_empty(), "Should find matches for 'API Endpoints'");
        
        let found_api_docs = results.iter().any(|r| {
            r.content.contains("API Endpoints") && r.file_path.contains("README.md")
        });
        assert!(found_api_docs, "Should find API Endpoints section in README.md");
        println!("✅ Found API Endpoints documentation");
        
        // Test Case 6: Cross-file search for common terms
        println!("\n📋 Test Case 6: Cross-file search verification");
        let results = searcher.search("password").await.expect("Search failed");
        assert!(!results.is_empty(), "Should find matches for 'password'");
        
        let files_with_password: std::collections::HashSet<_> = results.iter()
            .map(|r| {
                Path::new(&r.file_path).file_name().unwrap().to_str().unwrap()
            })
            .collect();
        
        // Should find password in multiple files
        assert!(files_with_password.len() >= 2, "Should find 'password' in multiple files");
        println!("✅ Found 'password' in {} files: {:?}", files_with_password.len(), files_with_password);
    }

    #[tokio::test] 
    async fn test_tantivy_fuzzy_search_comprehensive() {
        println!("🔍 Testing Tantivy Fuzzy Search Functionality");
        
        // Create temporary directory and test files
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_files = create_test_files();
        
        // Write test files to disk
        for test_file in &test_files {
            let file_path = temp_dir.path().join(test_file.path);
            fs::write(&file_path, test_file.content).expect("Failed to write test file");
        }
        
        // Create and index with TantivySearcher
        let mut searcher = TantivySearcher::new().await.expect("Failed to create TantivySearcher");
        searcher.index_directory(temp_dir.path()).await.expect("Failed to index directory");
        
        // Test Case 1: Single character typo
        println!("\n📋 Fuzzy Test Case 1: Single character typo");
        let results = searcher.search_fuzzy("calculatSum", 1).await.expect("Fuzzy search failed");
        assert!(!results.is_empty(), "Should find fuzzy matches for 'calculatSum'");
        
        let found_calculatesum = results.iter().any(|r| {
            r.content.contains("calculateSum") || r.line_content.contains("calculateSum")
        });
        assert!(found_calculatesum, "Should find calculateSum with fuzzy search");
        println!("✅ Found calculateSum with single character typo");
        
        // Test Case 2: Transposition error
        println!("\n📋 Fuzzy Test Case 2: Transposition error");
        let results = searcher.search_fuzzy("Authenticaiton", 2).await.expect("Fuzzy search failed");
        assert!(!results.is_empty(), "Should find fuzzy matches for 'Authenticaiton'");
        
        let found_authentication = results.iter().any(|r| {
            r.content.contains("Authentication") || r.line_content.contains("Authentication")
        });
        assert!(found_authentication, "Should find Authentication with transposition");
        println!("✅ Found Authentication with transposition error");
        
        // Test Case 3: Missing character
        println!("\n📋 Fuzzy Test Case 3: Missing character");
        let results = searcher.search_fuzzy("executeQury", 1).await.expect("Fuzzy search failed");
        assert!(!results.is_empty(), "Should find fuzzy matches for 'executeQury'");
        
        let found_execute_query = results.iter().any(|r| {
            r.content.contains("executeQuery") || r.line_content.contains("executeQuery")
        });
        assert!(found_execute_query, "Should find executeQuery with missing character");
        println!("✅ Found executeQuery with missing character");
        
        // Test Case 4: Extra character
        println!("\n📋 Fuzzy Test Case 4: Extra character");
        let results = searcher.search_fuzzy("fibonaccii", 1).await.expect("Fuzzy search failed");
        assert!(!results.is_empty(), "Should find fuzzy matches for 'fibonaccii'");
        
        let found_fibonacci = results.iter().any(|r| {
            r.content.contains("fibonacci") || r.line_content.contains("fibonacci")
        });
        assert!(found_fibonacci, "Should find fibonacci with extra character");
        println!("✅ Found fibonacci with extra character");
        
        // Test Case 5: Multiple errors within edit distance
        println!("\n📋 Fuzzy Test Case 5: Multiple errors");
        let results = searcher.search_fuzzy("databse", 2).await.expect("Fuzzy search failed"); 
        assert!(!results.is_empty(), "Should find fuzzy matches for 'databse'");
        
        let found_database = results.iter().any(|r| {
            r.content.contains("database") || r.line_content.contains("database")
        });
        assert!(found_database, "Should find database with multiple errors");
        println!("✅ Found database with multiple character errors");
        
        // Test Case 6: Should NOT find matches beyond edit distance
        println!("\n📋 Fuzzy Test Case 6: Beyond edit distance");
        let results = searcher.search_fuzzy("xyz123notfound", 2).await.expect("Fuzzy search failed");
        assert!(results.is_empty(), "Should NOT find matches for completely different terms");
        println!("✅ Correctly rejected term beyond edit distance");
    }

    #[tokio::test]
    async fn test_tantivy_search_accuracy_and_ranking() {
        println!("🎯 Testing Tantivy Search Accuracy and Result Ranking");
        
        // Create temporary directory and test files
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_files = create_test_files();
        
        // Write test files to disk
        for test_file in &test_files {
            let file_path = temp_dir.path().join(test_file.path);
            fs::write(&file_path, test_file.content).expect("Failed to write test file");
        }
        
        // Create and index with TantivySearcher
        let mut searcher = TantivySearcher::new().await.expect("Failed to create TantivySearcher");
        searcher.index_directory(temp_dir.path()).await.expect("Failed to index directory");
        
        // Test Case 1: Verify all expected content is indexed and searchable
        println!("\n📋 Test Case 1: Content indexing verification");
        let test_cases = vec![
            ("calculateSum", "math_utils.rs", 4),
            ("AuthenticationService", "auth_service.py", 4),
            ("executeQuery", "database.js", 15),
            ("authentication", "config.json", 10),
            ("API Endpoints", "README.md", 52),
        ];
        
        for (query, expected_file, expected_line) in test_cases {
            let results = searcher.search(query).await.expect("Search failed");
            assert!(!results.is_empty(), "Should find matches for '{}'", query);
            
            let found_in_expected_file = results.iter().any(|r| {
                r.file_path.contains(expected_file) && r.line_number == expected_line
            });
            assert!(found_in_expected_file, "Should find '{}' in {} at line {}", query, expected_file, expected_line);
            println!("✅ Found '{}' in {} at line {}", query, expected_file, expected_line);
        }
        
        // Test Case 2: Verify result structure correctness
        println!("\n📋 Test Case 2: Result structure verification");
        let results = searcher.search("function").await.expect("Search failed");
        
        for result in &results {
            // Verify all required fields are present and valid
            assert!(!result.file_path.is_empty(), "File path should not be empty");
            assert!(result.line_number > 0, "Line number should be positive");
            assert!(!result.content.is_empty(), "Content should not be empty");
            assert!(!result.line_content.is_empty(), "Line content should not be empty");
            
            // Verify file path is actually pointing to our test files
            let file_name = Path::new(&result.file_path).file_name().unwrap().to_str().unwrap();
            assert!(test_files.iter().any(|tf| tf.path == file_name), "File should be one of our test files: {}", file_name);
        }
        println!("✅ All {} results have valid structure", results.len());
        
        // Test Case 3: Verify no false positives
        println!("\n📋 Test Case 3: False positive detection");
        let results = searcher.search("nonexistenttermneverused").await.expect("Search failed");
        assert!(results.is_empty(), "Should not find matches for non-existent terms");
        println!("✅ No false positives for non-existent terms");
        
        // Test Case 4: Search term highlighting/context verification
        println!("\n📋 Test Case 4: Search context verification");
        let results = searcher.search("authenticate_user").await.expect("Search failed");
        assert!(!results.is_empty(), "Should find matches for 'authenticate_user'");
        
        let auth_user_result = results.iter().find(|r| r.content.contains("authenticate_user")).unwrap();
        assert!(auth_user_result.file_path.contains("auth_service.py"), "Should find in Python file");
        assert_eq!(auth_user_result.line_number, 11, "Should be on line 11");
        assert!(auth_user_result.line_content.contains("def authenticate_user"), "Line content should contain function definition");
        println!("✅ Found authenticate_user with correct context");
        
        // Test Case 5: Multi-term search
        println!("\n📋 Test Case 5: Multi-term search");
        let results = searcher.search("mysql createPool").await.expect("Search failed");
        assert!(!results.is_empty(), "Should find matches for multi-term search");
        
        let found_mysql_pool = results.iter().any(|r| {
            (r.content.contains("mysql") || r.content.contains("createPool")) &&
            r.file_path.contains("database.js")
        });
        assert!(found_mysql_pool, "Should find MySQL connection pool code");
        println!("✅ Multi-term search working correctly");
    }

    #[tokio::test]
    async fn test_tantivy_performance_and_edge_cases() {
        println!("⚡ Testing Tantivy Performance and Edge Cases");
        
        // Create temporary directory and test files
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_files = create_test_files();
        
        // Write test files to disk
        for test_file in &test_files {
            let file_path = temp_dir.path().join(test_file.path);
            fs::write(&file_path, test_file.content).expect("Failed to write test file");
        }
        
        // Create and index with TantivySearcher
        let mut searcher = TantivySearcher::new().await.expect("Failed to create TantivySearcher");
        
        // Performance test: Measure indexing time
        let index_start = std::time::Instant::now();
        searcher.index_directory(temp_dir.path()).await.expect("Failed to index directory");
        let index_time = index_start.elapsed();
        println!("✅ Indexing completed in {:?}", index_time);
        assert!(index_time.as_secs() < 5, "Indexing should complete within 5 seconds");
        
        // Performance test: Search latency
        println!("\n📋 Performance Test: Search latency");
        let search_terms = vec![
            "calculateSum", "authenticate", "database", "configuration", "documentation"
        ];
        
        let mut search_times = Vec::new();
        for term in &search_terms {
            let search_start = std::time::Instant::now();
            let _results = searcher.search(term).await.expect("Search failed");
            let search_time = search_start.elapsed();
            search_times.push(search_time.as_millis());
            
            assert!(search_time.as_millis() < 100, "Search should complete within 100ms");
        }
        
        let avg_search_time = search_times.iter().sum::<u128>() / search_times.len() as u128;
        println!("✅ Average search time: {}ms", avg_search_time);
        
        // Edge Case 1: Empty query
        println!("\n📋 Edge Case 1: Empty query");
        let results = searcher.search("").await.expect("Empty search should not fail");
        // Empty search should return no results (not crash)
        println!("✅ Empty query handled gracefully ({} results)", results.len());
        
        // Edge Case 2: Very long query
        println!("\n📋 Edge Case 2: Very long query");
        let long_query = "a ".repeat(1000);
        let results = searcher.search(&long_query).await.expect("Long query should not fail");
        println!("✅ Very long query handled gracefully ({} results)", results.len());
        
        // Edge Case 3: Special characters and symbols
        println!("\n📋 Edge Case 3: Special characters");
        let special_queries = vec![
            "const mysql = require('mysql2');",
            "pub fn calculateSum(a: i32, b: i32) -> i32 {",
            "def authenticate_user(self, username, password):",
            "SELECT * FROM users WHERE id = ?",
            "npm install && npm start",
        ];
        
        for query in special_queries {
            let results = searcher.search(query).await.expect("Special character search should not fail");
            println!("  - Query with special chars: {} results", results.len());
        }
        println!("✅ Special character queries handled");
        
        // Edge Case 4: Unicode and international characters
        println!("\n📋 Edge Case 4: Unicode characters");
        let unicode_results = searcher.search("café résumé naïve").await.expect("Unicode search should not fail");
        println!("✅ Unicode query handled gracefully ({} results)", unicode_results.len());
        
        // Edge Case 5: Case sensitivity verification
        println!("\n📋 Edge Case 5: Case sensitivity");
        let lower_results = searcher.search("authenticateuser").await.expect("Lowercase search failed");
        let upper_results = searcher.search("AUTHENTICATEUSER").await.expect("Uppercase search failed");
        let mixed_results = searcher.search("AuthenticateUser").await.expect("Mixed case search failed");
        
        // Tantivy should handle case variations
        println!("✅ Case variations: lower={}, upper={}, mixed={}", 
                 lower_results.len(), upper_results.len(), mixed_results.len());
        
        // Edge Case 6: Index statistics and health
        println!("\n📋 Edge Case 6: Index statistics");
        let stats = searcher.get_index_stats().expect("Failed to get index stats");
        println!("✅ Index stats: {}", stats);
        assert!(stats.num_documents > 0, "Index should contain documents");
        assert!(!stats.is_persistent || stats.index_size_bytes > 0, "Persistent index should have size");
    }

    #[tokio::test]
    async fn test_tantivy_file_type_coverage() {
        println!("📁 Testing Tantivy File Type Coverage");
        
        // Create temporary directory 
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Test different file types that should be indexed
        let file_types = vec![
            ("test.rs", "fn main() { println!(\"Rust code\"); }"),
            ("test.py", "def main(): print(\"Python code\")"),
            ("test.js", "function main() { console.log(\"JavaScript code\"); }"),
            ("test.ts", "function main(): void { console.log(\"TypeScript code\"); }"),
            ("test.go", "func main() { fmt.Println(\"Go code\") }"),
            ("test.java", "public class Test { public static void main(String[] args) {} }"),
            ("test.cpp", "#include <iostream>\nint main() { std::cout << \"C++ code\"; }"),
            ("test.c", "#include <stdio.h>\nint main() { printf(\"C code\"); }"),
            ("test.rb", "def main; puts \"Ruby code\"; end"),
            ("test.php", "<?php function main() { echo \"PHP code\"; } ?>"),
            ("test.md", "# Markdown\n\nThis is **markdown** content."),
            ("test.json", "{\"language\": \"JSON\", \"type\": \"configuration\"}"),
            ("test.yaml", "language: YAML\ntype: configuration"),
            ("test.toml", "[section]\nlanguage = \"TOML\""),
            ("test.sql", "SELECT * FROM users WHERE active = 1;"),
        ];
        
        // Write all test files
        for (filename, content) in &file_types {
            let file_path = temp_dir.path().join(filename);
            fs::write(&file_path, content).expect("Failed to write test file");
        }
        
        // Index and search
        let mut searcher = TantivySearcher::new().await.expect("Failed to create TantivySearcher");
        searcher.index_directory(temp_dir.path()).await.expect("Failed to index directory");
        
        // Verify each file type was indexed and is searchable
        let mut indexed_types = Vec::new();
        
        for (filename, _content) in &file_types {
            // Extract a unique term from each file's content
            let search_term = if filename.contains(".rs") { "println!" }
            else if filename.contains(".py") { "print" }
            else if filename.contains(".js") { "console.log" }
            else if filename.contains(".ts") { "void" }
            else if filename.contains(".go") { "fmt.Println" }
            else if filename.contains(".java") { "String[]" }
            else if filename.contains(".cpp") { "iostream" }
            else if filename.contains(".c") { "stdio.h" }
            else if filename.contains(".rb") { "puts" }
            else if filename.contains(".php") { "echo" }
            else if filename.contains(".md") { "Markdown" }
            else if filename.contains(".json") { "JSON" }
            else if filename.contains(".yaml") { "YAML" }
            else if filename.contains(".toml") { "TOML" }
            else if filename.contains(".sql") { "SELECT" }
            else { "code" };
            
            let results = searcher.search(search_term).await.expect("Search failed");
            let found_in_file = results.iter().any(|r| r.file_path.contains(filename));
            
            if found_in_file {
                indexed_types.push(*filename);
                println!("✅ Found '{}' in {}", search_term, filename);
            } else {
                println!("❌ Could not find '{}' in {}", search_term, filename);
            }
        }
        
        println!("\n📊 File Type Coverage Summary:");
        println!("  - Indexed file types: {}/{}", indexed_types.len(), file_types.len());
        println!("  - Coverage: {:.1}%", (indexed_types.len() as f64 / file_types.len() as f64) * 100.0);
        
        // We expect most common code file types to be indexed
        assert!(indexed_types.len() >= 10, "Should index at least 10 different file types");
        
        // Verify critical file types are covered
        let critical_types = ["test.rs", "test.py", "test.js", "test.md", "test.json"];
        for critical_type in critical_types {
            assert!(indexed_types.contains(&critical_type), "Critical file type {} should be indexed", critical_type);
        }
        
        println!("✅ All critical file types are properly indexed");
    }
}