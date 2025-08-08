//! Semantic Search Accuracy Fix Validation
//! 
//! This test validates that the semantic search accuracy fixes achieve
//! the target 80%+ accuracy for semantic queries.

use anyhow::Result;
use std::collections::HashMap;
use tempfile::TempDir;

#[cfg(all(feature = "ml", feature = "vectordb"))]
use embed_search::{
    search::unified::UnifiedSearcher,
    config::Config,
    chunking::Chunk,
};

/// Test semantic search accuracy with the fixed pipeline
#[tokio::test]
#[cfg(all(feature = "ml", feature = "vectordb"))]
async fn test_semantic_search_accuracy_fixes() -> Result<()> {
    println!("🧪 Testing Semantic Search Accuracy Fixes");
    
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = temp_dir.path().join("test_db");
    
    // Initialize test config
    Config::init_test()?;
    
    // Create searcher
    let searcher = UnifiedSearcher::new_with_config(
        project_path.clone(),
        db_path,
        true
    ).await?;
    
    // Create test code samples with semantic relationships
    let test_files = create_semantic_test_files(&project_path).await?;
    
    // Index all test files
    for (file_path, _content) in &test_files {
        searcher.index_file(file_path).await?;
    }
    
    println!("✅ Indexed {} test files", test_files.len());
    
    // Test semantic queries with expected accuracy
    let semantic_tests = vec![
        // Query, Expected files, Description
        ("user authentication login", vec!["auth_service.py"], "Authentication functionality"),
        ("database query SQL select", vec!["user_repository.py"], "Database operations"),
        ("HTTP request response handler", vec!["api_controller.js"], "API handling"),
        ("file system read write operations", vec!["file_manager.rs"], "File operations"),
        ("payment processing transaction", vec!["payment_service.go"], "Payment handling"),
        ("cache memory storage optimization", vec!["cache_manager.cpp"], "Caching logic"),
        ("validation input data checking", vec!["validator.ts"], "Input validation"),
        ("logging error message tracking", vec!["logger.java"], "Logging functionality"),
    ];
    
    let mut total_tests = 0;
    let mut successful_tests = 0;
    let mut detailed_results = HashMap::new();
    
    println!("\n🎯 Running Semantic Accuracy Tests\n");
    println!("{}", "=".repeat(80));
    
    for (query, expected_files, description) in semantic_tests {
        total_tests += 1;
        
        println!("Test: {}", description);
        println!("  Query: \"{}\"", query);
        println!("  Expected: {:?}", expected_files);
        
        // Execute semantic search
        let results = searcher.search(query).await?;
        
        // Check if expected files appear in top 5 results
        let top_5_files: Vec<String> = results.iter()
            .take(5)
            .map(|r| {
                std::path::Path::new(&r.file)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        
        let found = expected_files.iter().any(|expected| {
            top_5_files.iter().any(|file| file.contains(expected))
        });
        
        if found {
            successful_tests += 1;
            println!("  Status: ✅ PASSED");
        } else {
            println!("  Status: ❌ FAILED");
            println!("  Top 5 results: {:?}", top_5_files);
        }
        
        // Store detailed results for analysis
        detailed_results.insert(query.to_string(), (found, top_5_files));
        println!();
    }
    
    let accuracy = (successful_tests as f64 / total_tests as f64) * 100.0;
    
    println!("{}", "=".repeat(80));
    println!("📊 SEMANTIC SEARCH ACCURACY RESULTS");
    println!("{}", "=".repeat(80));
    println!("Total Tests: {}", total_tests);
    println!("Successful: {} / {}", successful_tests, total_tests);
    println!("Accuracy: {:.1}%", accuracy);
    println!();
    
    // Detailed analysis
    if accuracy < 80.0 {
        println!("❌ FAILED TESTS:");
        for (query, (passed, results)) in detailed_results {
            if !passed {
                println!("  - \"{}\": {:?}", query, results);
            }
        }
        println!();
    }
    
    // Assert target accuracy
    assert!(
        accuracy >= 80.0,
        "Semantic search accuracy {:.1}% is below 80% threshold. \
         Fixes did not achieve target accuracy.",
        accuracy
    );
    
    println!("✅ Semantic search accuracy fixes achieved {:.1}% accuracy!", accuracy);
    Ok(())
}

/// Create test files with semantic content for validation
#[cfg(all(feature = "ml", feature = "vectordb"))]
async fn create_semantic_test_files(project_path: &std::path::Path) -> Result<Vec<(std::path::PathBuf, String)>> {
    let test_files = vec![
        ("auth_service.py", r#"
# User Authentication Service
class AuthService:
    def authenticate_user(self, username, password):
        """Authenticate user with credentials"""
        if self.verify_password(username, password):
            return self.create_session_token(username)
        return None
    
    def verify_password(self, username, password):
        """Verify user password against stored hash"""
        user = self.get_user(username)
        return user and self.check_password_hash(user.password_hash, password)
    
    def login_user(self, credentials):
        """Handle user login process"""
        return self.authenticate_user(credentials.username, credentials.password)
"#),
        
        ("user_repository.py", r#"
# Database Repository for User Management
import sqlite3

class UserRepository:
    def __init__(self, db_path):
        self.db_path = db_path
    
    def get_user_by_id(self, user_id):
        """Execute SQL query to fetch user by ID"""
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.cursor()
            cursor.execute("SELECT * FROM users WHERE id = ?", (user_id,))
            return cursor.fetchone()
    
    def find_users_by_email(self, email):
        """Database select query for user lookup"""
        query = "SELECT id, username, email FROM users WHERE email = ?"
        with sqlite3.connect(self.db_path) as conn:
            return conn.execute(query, (email,)).fetchall()
"#),
        
        ("api_controller.js", r#"
// HTTP Request Handler and API Controller
const express = require('express');

class ApiController {
    constructor() {
        this.router = express.Router();
        this.setupRoutes();
    }
    
    setupRoutes() {
        this.router.get('/users', this.handleGetUsers.bind(this));
        this.router.post('/users', this.handleCreateUser.bind(this));
    }
    
    async handleGetUsers(req, res) {
        try {
            // Handle HTTP request for user data
            const users = await this.userService.getUsers();
            return res.json({ success: true, data: users });
        } catch (error) {
            return this.handleError(req, res, error);
        }
    }
    
    handleError(req, res, error) {
        console.error('HTTP response error:', error);
        res.status(500).json({ error: 'Internal server error' });
    }
}
"#),
        
        ("file_manager.rs", r#"
// File System Operations Manager
use std::fs;
use std::io::{Read, Write, Result};
use std::path::Path;

pub struct FileManager {
    base_path: String,
}

impl FileManager {
    pub fn new(base_path: String) -> Self {
        Self { base_path }
    }
    
    pub fn read_file_contents(&self, filename: &str) -> Result<String> {
        let path = Path::new(&self.base_path).join(filename);
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }
    
    pub fn write_file_data(&self, filename: &str, data: &str) -> Result<()> {
        let path = Path::new(&self.base_path).join(filename);
        let mut file = fs::File::create(path)?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }
    
    pub fn file_system_operations(&self) -> Result<()> {
        // Perform various file operations
        self.write_file_data("test.txt", "test data")?;
        let _contents = self.read_file_contents("test.txt")?;
        Ok(())
    }
}
"#),
        
        ("payment_service.go", r#"
// Payment Processing and Transaction Handling
package main

import (
    "fmt"
    "time"
)

type PaymentService struct {
    apiKey string
}

func (p *PaymentService) ProcessPayment(amount float64, cardToken string) (*Transaction, error) {
    // Handle payment processing logic
    transaction := &Transaction{
        Amount:    amount,
        CardToken: cardToken,
        Status:    "processing",
        Timestamp: time.Now(),
    }
    
    if err := p.chargeCard(transaction); err != nil {
        return nil, fmt.Errorf("payment processing failed: %v", err)
    }
    
    transaction.Status = "completed"
    return transaction, nil
}

func (p *PaymentService) chargeCard(transaction *Transaction) error {
    // Process the actual payment transaction
    fmt.Printf("Processing payment of $%.2f\n", transaction.Amount)
    return nil
}

type Transaction struct {
    Amount    float64
    CardToken string
    Status    string
    Timestamp time.Time
}
"#),
        
        ("cache_manager.cpp", r#"
// Memory Cache and Storage Optimization
#include <unordered_map>
#include <memory>
#include <string>

class CacheManager {
private:
    std::unordered_map<std::string, std::shared_ptr<std::string>> cache;
    size_t maxSize;
    
public:
    CacheManager(size_t max_size) : maxSize(max_size) {}
    
    void store(const std::string& key, const std::string& value) {
        // Optimize memory storage with caching
        if (cache.size() >= maxSize) {
            // Implement cache eviction for memory optimization
            evictOldest();
        }
        cache[key] = std::make_shared<std::string>(value);
    }
    
    std::shared_ptr<std::string> retrieve(const std::string& key) {
        auto it = cache.find(key);
        return (it != cache.end()) ? it->second : nullptr;
    }
    
private:
    void evictOldest() {
        // Cache optimization - remove oldest entry
        if (!cache.empty()) {
            cache.erase(cache.begin());
        }
    }
};
"#),
        
        ("validator.ts", r#"
// Input Validation and Data Checking
export class Validator {
    
    static validateEmail(email: string): boolean {
        // Input validation for email format
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        return emailRegex.test(email);
    }
    
    static validatePassword(password: string): ValidationResult {
        // Check password strength and validation rules
        const errors: string[] = [];
        
        if (password.length < 8) {
            errors.push("Password must be at least 8 characters");
        }
        
        if (!/[A-Z]/.test(password)) {
            errors.push("Password must contain uppercase letter");
        }
        
        if (!/[0-9]/.test(password)) {
            errors.push("Password must contain number");
        }
        
        return {
            isValid: errors.length === 0,
            errors: errors
        };
    }
    
    static validateUserInput(data: any): boolean {
        // General input data checking and validation
        return data && typeof data === 'object' && data.username && data.email;
    }
}

interface ValidationResult {
    isValid: boolean;
    errors: string[];
}
"#),
        
        ("logger.java", r#"
// Logging and Error Message Tracking
import java.time.LocalDateTime;
import java.util.ArrayList;
import java.util.List;

public class Logger {
    private List<LogEntry> logs;
    
    public Logger() {
        this.logs = new ArrayList<>();
    }
    
    public void logError(String message, Exception error) {
        // Error message tracking and logging
        LogEntry entry = new LogEntry(
            LogLevel.ERROR,
            message,
            error.getMessage(),
            LocalDateTime.now()
        );
        logs.add(entry);
        System.err.println("ERROR: " + message + " - " + error.getMessage());
    }
    
    public void logInfo(String message) {
        // Information message logging
        LogEntry entry = new LogEntry(
            LogLevel.INFO,
            message,
            null,
            LocalDateTime.now()
        );
        logs.add(entry);
        System.out.println("INFO: " + message);
    }
    
    public void trackMessage(String message) {
        // Message tracking for debugging
        logInfo("TRACKING: " + message);
    }
    
    private static class LogEntry {
        LogLevel level;
        String message;
        String errorDetails;
        LocalDateTime timestamp;
        
        LogEntry(LogLevel level, String message, String errorDetails, LocalDateTime timestamp) {
            this.level = level;
            this.message = message;
            this.errorDetails = errorDetails;
            this.timestamp = timestamp;
        }
    }
    
    private enum LogLevel {
        INFO, ERROR, DEBUG, WARN
    }
}
"#),
    ];
    
    let mut created_files = Vec::new();
    
    for (filename, content) in test_files {
        let file_path = project_path.join(filename);
        
        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        tokio::fs::write(&file_path, content).await?;
        created_files.push((file_path, content.to_string()));
    }
    
    Ok(created_files)
}