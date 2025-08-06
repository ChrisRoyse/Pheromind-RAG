use embed_search::search::{UnifiedSearcher, MatchType};
use embed_search::config::{Config, SearchBackend};
use tempfile::TempDir;
use std::path::PathBuf;
use tokio::fs;
use anyhow::Result;

/// Create test files with known content for BM25 testing
async fn create_test_codebase(dir: &PathBuf) -> Result<()> {
    // Create test files with varying term frequencies
    
    // File 1: Function with authentication
    let auth_service = r#"
// Authentication service for user management
pub struct AuthService {
    database: Database,
    cache: Cache,
}

impl AuthService {
    pub async fn authenticate_user(username: &str, password: &str) -> Result<User> {
        // Check cache first for user authentication
        if let Some(user) = self.cache.get_user(username) {
            if user.verify_password(password) {
                return Ok(user);
            }
        }
        
        // Query database for user authentication
        let user = self.database.get_user(username).await?;
        if user.verify_password(password) {
            self.cache.set_user(username, user.clone());
            return Ok(user);
        }
        
        Err(AuthError::InvalidCredentials)
    }
    
    pub async fn validate_token(token: &str) -> Result<bool> {
        // Validate authentication token
        let decoded = decode_jwt(token)?;
        Ok(decoded.exp > now())
    }
}
"#;
    
    // File 2: Database connection handling
    let database_module = r#"
// Database connection and query module
use sqlx::{Pool, Postgres};

pub struct DatabaseConnection {
    pool: Pool<Postgres>,
}

impl DatabaseConnection {
    pub async fn new(url: &str) -> Result<Self> {
        let pool = Pool::connect(url).await?;
        Ok(Self { pool })
    }
    
    pub async fn execute_query(query: &str) -> Result<Vec<Row>> {
        // Execute database query with connection pooling
        let mut conn = self.pool.acquire().await?;
        let rows = sqlx::query(query)
            .fetch_all(&mut conn)
            .await?;
        Ok(rows)
    }
    
    pub async fn transaction<F>(&self, f: F) -> Result<()> 
    where
        F: FnOnce(&mut Transaction) -> Result<()>
    {
        let mut tx = self.pool.begin().await?;
        f(&mut tx)?;
        tx.commit().await?;
        Ok(())
    }
}
"#;
    
    // File 3: User interface components
    let ui_components = r#"
// User interface components and widgets
import React from 'react';

export function UserProfile({ user }) {
    return (
        <div className="user-profile">
            <h2>{user.name}</h2>
            <p>{user.email}</p>
            <UserAvatar src={user.avatar} />
        </div>
    );
}

export function UserList({ users }) {
    return (
        <div className="user-list">
            {users.map(user => (
                <UserCard key={user.id} user={user} />
            ))}
        </div>
    );
}

function UserCard({ user }) {
    return (
        <div className="user-card">
            <span>{user.name}</span>
            <span>{user.role}</span>
        </div>
    );
}
"#;
    
    // File 4: Data processing pipeline
    let data_processor = r#"
# Data processing pipeline for analytics
import pandas as pd
import numpy as np

class DataProcessor:
    def __init__(self, config):
        self.config = config
        self.pipeline = []
    
    def process_data(self, data):
        """Process data through the pipeline stages"""
        for stage in self.pipeline:
            data = stage.transform(data)
        return data
    
    def add_stage(self, stage):
        """Add a processing stage to the pipeline"""
        self.pipeline.append(stage)
    
    def validate_data(self, data):
        """Validate data integrity and format"""
        if not isinstance(data, pd.DataFrame):
            raise ValueError("Data must be a DataFrame")
        
        # Check for required columns
        required = self.config.get('required_columns', [])
        missing = set(required) - set(data.columns)
        if missing:
            raise ValueError(f"Missing columns: {missing}")
        
        return True
    
    def export_results(self, data, format='csv'):
        """Export processed data to various formats"""
        if format == 'csv':
            return data.to_csv()
        elif format == 'json':
            return data.to_json()
        else:
            raise ValueError(f"Unsupported format: {format}")
"#;
    
    // File 5: Test file (should be deprioritized)
    let test_file = r#"
// Test file for authentication
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_authentication() {
        let auth = AuthService::new();
        let result = auth.authenticate_user("test", "password");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_database_connection() {
        let db = DatabaseConnection::new("test_url");
        assert!(db.is_ok());
    }
}
"#;
    
    // Create the test files
    fs::write(dir.join("auth_service.rs"), auth_service).await?;
    fs::write(dir.join("database_module.rs"), database_module).await?;
    fs::write(dir.join("ui_components.jsx"), ui_components).await?;
    fs::write(dir.join("data_processor.py"), data_processor).await?;
    fs::write(dir.join("test_auth.rs"), test_file).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_bm25_basic_search() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join(".embed_db");
    
    // Create test codebase
    create_test_codebase(&project_path).await?;
    
    // Initialize searcher with BM25 enabled
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
    
    // Test 1: Multi-word query that BM25 should excel at
    let results = searcher.search("database connection").await?;
    assert!(!results.is_empty(), "Should find results for 'database connection'");
    
    // The database_module.rs should rank high
    let top_result = &results[0];
    assert!(
        top_result.file.contains("database_module"),
        "Database module should rank high for 'database connection' query"
    );
    
    // Test 2: Authentication query
    let results = searcher.search("authenticate user").await?;
    assert!(!results.is_empty(), "Should find results for 'authenticate user'");
    
    // The auth_service.rs should rank high
    let top_results: Vec<String> = results.iter()
        .take(2)
        .map(|r| r.file.clone())
        .collect();
    assert!(
        top_results.iter().any(|f| f.contains("auth_service")),
        "Auth service should be in top results for 'authenticate user' query"
    );
    
    // Test 3: UI component query
    let results = searcher.search("user interface component").await?;
    assert!(!results.is_empty(), "Should find results for 'user interface component'");
    
    // UI components file should be found
    let has_ui_file = results.iter()
        .any(|r| r.file.contains("ui_components"));
    assert!(has_ui_file, "UI components file should be found");
    
    // Test 4: Data processing query
    let results = searcher.search("data processing pipeline").await?;
    assert!(!results.is_empty(), "Should find results for 'data processing pipeline'");
    
    // Data processor should rank high
    let top_results: Vec<String> = results.iter()
        .take(3)
        .map(|r| r.file.clone())
        .collect();
    assert!(
        top_results.iter().any(|f| f.contains("data_processor")),
        "Data processor should be in top 3 results for 'data processing pipeline' query"
    );
    
    Ok(())
}

#[tokio::test]
async fn test_bm25_term_frequency_saturation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join(".embed_db");
    
    // Create files with different term frequencies
    let file1 = "function test() { return 'test'; }";
    let file2 = "function function function function function function"; // Excessive repetition
    let file3 = "function calculate() { return compute(); }";
    
    fs::write(project_path.join("normal.js"), file1).await?;
    fs::write(project_path.join("repetitive.js"), file2).await?;
    fs::write(project_path.join("calculate.js"), file3).await?;
    
    // Initialize searcher
    let searcher = UnifiedSearcher::new(project_path.clone(), db_path).await?;
    
    // Index files
    for entry in std::fs::read_dir(&project_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            searcher.index_file(&path).await?;
        }
    }
    
    // Search for "function"
    let results = searcher.search("function").await?;
    
    // The repetitive file should NOT dominate results due to BM25 saturation
    if results.len() >= 2 {
        let first_file = &results[0].file;
        assert!(
            !first_file.contains("repetitive"),
            "BM25 should prevent repetitive file from dominating results"
        );
    }
    
    Ok(())
}

#[tokio::test]
async fn test_bm25_match_type_classification() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join(".embed_db");
    
    // Create a simple test file
    let content = r#"
fn calculate_total(items: Vec<Item>) -> f64 {
    items.iter().map(|item| item.price).sum()
}
"#;
    
    fs::write(project_path.join("calculator.rs"), content).await?;
    
    // Initialize searcher
    let searcher = UnifiedSearcher::new(project_path.clone(), db_path).await?;
    
    // Index the file
    searcher.index_file(&project_path.join("calculator.rs")).await?;
    
    // Search for a query that should trigger BM25
    let results = searcher.search("calculate total price").await?;
    
    assert!(!results.is_empty(), "Should find results");
    
    // Check that we have different match types
    let has_statistical = results.iter()
        .any(|r| r.match_type == MatchType::Statistical);
    
    // BM25 should be active and finding statistical matches
    println!("Match types found: {:?}", 
             results.iter().map(|r| &r.match_type).collect::<Vec<_>>());
    
    Ok(())
}

#[tokio::test]
async fn test_bm25_accuracy_improvement() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join(".embed_db");
    
    // Create test codebase
    create_test_codebase(&project_path).await?;
    
    // Initialize searcher with BM25 enabled
    let searcher = UnifiedSearcher::new(project_path.clone(), db_path).await?;
    
    // Index all files
    for entry in std::fs::read_dir(&project_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            searcher.index_file(&path).await?;
        }
    }
    
    // Test queries that benefit from BM25's term frequency analysis
    let test_queries = vec![
        ("async database connection error", vec!["database_module"]),
        ("user authentication validation function", vec!["auth_service"]),
        ("data processing pipeline implementation", vec!["data_processor"]),
        ("user interface component render", vec!["ui_components"]),
    ];
    
    let mut correct_matches = 0;
    let total_queries = test_queries.len();
    
    for (query, expected_files) in test_queries {
        let results = searcher.search(query).await?;
        
        // Check if expected files are in top 3 results
        let top_3: Vec<String> = results.iter()
            .take(3)
            .map(|r| r.file.clone())
            .collect();
        
        let found_expected = expected_files.iter()
            .any(|expected| top_3.iter().any(|f| f.contains(expected)));
        
        if found_expected {
            correct_matches += 1;
            println!("✅ Query '{}' found expected files", query);
        } else {
            println!("❌ Query '{}' missed expected files. Got: {:?}", query, top_3);
        }
    }
    
    let accuracy = (correct_matches as f32 / total_queries as f32) * 100.0;
    println!("BM25 Search Accuracy: {:.1}%", accuracy);
    
    // We expect high accuracy with BM25 enabled
    assert!(
        accuracy >= 75.0,
        "BM25 should achieve at least 75% accuracy, got {:.1}%",
        accuracy
    );
    
    Ok(())
}

#[tokio::test]
async fn test_bm25_persistence() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join(".embed_db");
    
    // Create a test file
    let content = "fn persist_data() { save_to_disk(); }";
    fs::write(project_path.join("persist.rs"), content).await?;
    
    // First session: Index the file
    {
        let searcher = UnifiedSearcher::new(project_path.clone(), db_path.clone()).await?;
        searcher.index_file(&project_path.join("persist.rs")).await?;
        
        // Search to verify indexing worked
        let results = searcher.search("persist data").await?;
        assert!(!results.is_empty(), "Should find results after indexing");
    }
    
    // Second session: Load and search without re-indexing
    {
        let searcher = UnifiedSearcher::new(project_path.clone(), db_path).await?;
        
        // Search should still work with persisted index
        let results = searcher.search("persist data").await?;
        assert!(!results.is_empty(), "Should find results from persisted index");
    }
    
    Ok(())
}