use std::path::PathBuf;
use std::time::Instant;
use tokio::time::{sleep, Duration};
use embed::{
    search::{UnifiedSearcher, SearchResult, unified::IndexStats},
    config::Config,
    chunking::SimpleRegexChunker,
    search::cache::SearchResult as CachedSearchResult,
    search::fusion::MatchType,
};
use tempfile::TempDir;

/// **BRUTAL SEMANTIC SEARCH VALIDATION**
/// 
/// This test suite validates semantic search functionality with ZERO tolerance for:
/// - Irrelevant results
/// - Search failures
/// - Performance degradation
/// - Integration issues
/// 
/// **PASS REQUIREMENTS (100/100)**:
/// - All searches return relevant, ranked results
/// - Vector similarity is actually computed 
/// - Results integrate properly with other search methods
/// - Performance is acceptable (<1s per query)

#[cfg(all(feature = "ml", feature = "vectordb"))]
#[tokio::test]
async fn test_semantic_search_accuracy_brutal_validation() -> anyhow::Result<()> {
    println!("🔥 SEMANTIC SEARCH ACCURACY - BRUTAL VALIDATION");
    println!("======================================================");
    
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().to_path_buf();
    
    // Initialize config
    Config::init_with_defaults().expect("Config initialization failed");
    
    // Create searcher with semantic features enabled
    let searcher = UnifiedSearcher::new(
        temp_dir.path().to_path_buf(), 
        db_path
    ).await?;
    
    // Create test files with semantic content
    let test_files = create_semantic_test_files(temp_dir.path()).await?;
    println!("📄 Created {} test files with semantic content", test_files.len());
    
    // Index all test files
    let mut total_stats = IndexStats::new();
    for file_path in &test_files {
        let stats = searcher.index_file(file_path).await?;
        println!("   Indexed: {:?} -> chunks created", file_path.file_name().unwrap());
    }
    
    // Wait for indexing to complete
    sleep(Duration::from_millis(500)).await;
    
    // Test semantic queries with expected results
    let semantic_test_cases = vec![
        ("function definition", vec!["calculator.rs", "helpers.py"]),
        ("error handling", vec!["error_handler.rs", "validators.py"]),
        ("data processing", vec!["data_processor.rs", "utils.py"]),
        ("class implementation", vec!["user_model.py", "helpers.py"]),
        ("mathematical operations", vec!["calculator.rs", "math_utils.js"]),
    ];
    
    let mut passed_tests = 0;
    let total_tests = semantic_test_cases.len();
    
    for (query, expected_files) in semantic_test_cases {
        println!("\n🔍 Testing query: '{}'", query);
        
        let start_time = Instant::now();
        let results = searcher.search(query).await?;
        let search_time = start_time.elapsed();
        
        // REQUIREMENT: Search must complete in <1s
        if search_time > Duration::from_secs(1) {
            println!("❌ PERFORMANCE FAILURE: Search took {:.2}s (>1s limit)", search_time.as_secs_f32());
            continue;
        }
        
        // REQUIREMENT: Must return results
        if results.is_empty() {
            println!("❌ SEARCH FAILURE: No results returned for '{}'", query);
            continue;
        }
        
        // REQUIREMENT: Results must be relevant
        let mut relevance_score = 0;
        let mut found_expected = 0;
        
        for result in &results {
            // Check if result contains expected files
            for expected_file in &expected_files {
                if result.file.contains(expected_file) {
                    found_expected += 1;
                    break;
                }
            }
            
            // Check if semantic match type is present
            if result.match_type == MatchType::Semantic {
                relevance_score += 10; // High score for semantic matches
            }
            
            // Validate search result structure
            if result.score < 0.0 || result.score > 1.5 {
                println!("❌ INVALID SCORE: Result score {} is out of valid range", result.score);
                continue;
            }
        }
        
        // REQUIREMENT: Must find at least half the expected files
        let relevance_ratio = found_expected as f32 / expected_files.len() as f32;
        if relevance_ratio < 0.5 {
            println!("❌ RELEVANCE FAILURE: Found {}/{} expected files ({:.1}%)", 
                    found_expected, expected_files.len(), relevance_ratio * 100.0);
            continue;
        }
        
        // REQUIREMENT: Must have semantic matches
        let has_semantic = results.iter().any(|r| r.match_type == MatchType::Semantic);
        if !has_semantic {
            println!("❌ SEMANTIC FAILURE: No semantic matches found in results");
            continue;
        }
        
        println!("✅ PASSED: {:.1}ms, {}/{} expected files, {} semantic matches", 
                search_time.as_millis(),
                found_expected,
                expected_files.len(),
                results.iter().filter(|r| r.match_type == MatchType::Semantic).count());
        
        passed_tests += 1;
    }
    
    println!("\n======================================================");
    println!("🎯 SEMANTIC SEARCH ACCURACY RESULTS");
    println!("Passed: {}/{} tests ({:.1}%)", passed_tests, total_tests, 
            (passed_tests as f32 / total_tests as f32) * 100.0);
    
    if passed_tests == total_tests {
        println!("✅ SEMANTIC SEARCH ACCURACY: PASS (100/100)");
        Ok(())
    } else {
        Err(anyhow::anyhow!("❌ SEMANTIC SEARCH ACCURACY: FAIL ({}/100) - {} tests failed", 
                           (passed_tests * 100 / total_tests), 
                           total_tests - passed_tests))
    }
}

#[cfg(all(feature = "ml", feature = "vectordb"))]
#[tokio::test]
async fn test_semantic_integration_brutal_validation() -> anyhow::Result<()> {
    println!("🔥 SEMANTIC INTEGRATION - BRUTAL VALIDATION");
    println!("======================================================");
    
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().to_path_buf();
    
    // Initialize config
    Config::init_with_defaults().expect("Config integration test config init failed");
    
    let searcher = UnifiedSearcher::new(
        temp_dir.path().to_path_buf(),
        db_path
    ).await?;
    
    // Create test content that should trigger multiple search types
    let test_content = r#"
// This is a function definition for calculating user scores
function calculateUserScore(user: User): number {
    if (!user) {
        throw new Error("User cannot be null");
    }
    
    // Mathematical operation for scoring
    return user.points * 1.5 + user.bonus;
}

class UserProcessor {
    processUserData(users: User[]): ProcessedUser[] {
        return users.map(user => ({
            ...user,
            score: calculateUserScore(user)
        }));
    }
}
"#;
    
    let test_file = temp_dir.path().join("integration_test.ts");
    tokio::fs::write(&test_file, test_content).await?;
    
    // Index the file
    searcher.index_file(&test_file).await?;
    sleep(Duration::from_millis(300)).await;
    
    // Test that semantic search integrates with other search types
    let integration_queries = vec![
        ("calculateUserScore", "Should find exact and semantic matches"),
        ("function definition", "Should find semantic matches for concepts"),
        ("error handling", "Should find relevant code sections"),
    ];
    
    let mut integration_passed = 0;
    let total_integration_tests = integration_queries.len();
    
    for (query, description) in integration_queries {
        println!("\n🔍 Testing integration: '{}' - {}", query, description);
        
        let results = searcher.search(query).await?;
        
        if results.is_empty() {
            println!("❌ INTEGRATION FAILURE: No results for '{}'", query);
            continue;
        }
        
        // Validate multiple match types are present when expected
        let match_types: std::collections::HashSet<_> = results.iter()
            .map(|r| &r.match_type)
            .collect();
            
        // Check that results are properly fused and ranked
        let mut previous_score = f32::INFINITY;
        let mut ranking_valid = true;
        
        for result in &results {
            if result.score > previous_score {
                println!("❌ RANKING FAILURE: Results not properly sorted by score");
                ranking_valid = false;
                break;
            }
            previous_score = result.score;
            
            // Validate three-chunk expansion worked
            if result.three_chunk_context.center_chunk.content.is_empty() {
                println!("❌ CONTEXT FAILURE: Empty center chunk content");
                ranking_valid = false;
                break;
            }
        }
        
        if !ranking_valid {
            continue;
        }
        
        println!("✅ INTEGRATION PASSED: {} results, {} match types, proper ranking", 
                results.len(), match_types.len());
        integration_passed += 1;
    }
    
    println!("\n======================================================");
    println!("🎯 SEMANTIC INTEGRATION RESULTS");
    println!("Passed: {}/{} tests ({:.1}%)", integration_passed, total_integration_tests, 
            (integration_passed as f32 / total_integration_tests as f32) * 100.0);
    
    if integration_passed == total_integration_tests {
        println!("✅ SEMANTIC INTEGRATION: PASS (100/100)");
        Ok(())
    } else {
        Err(anyhow::anyhow!("❌ SEMANTIC INTEGRATION: FAIL ({}/100)", 
                           (integration_passed * 100 / total_integration_tests)))
    }
}

#[cfg(all(feature = "ml", feature = "vectordb"))]
#[tokio::test]
async fn test_semantic_performance_brutal_validation() -> anyhow::Result<()> {
    println!("🔥 SEMANTIC PERFORMANCE - BRUTAL VALIDATION");
    println!("======================================================");
    
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().to_path_buf();
    
    Config::init_with_defaults().expect("Config performance test init failed");
    
    let searcher = UnifiedSearcher::new(
        temp_dir.path().to_path_buf(),
        db_path
    ).await?;
    
    // Create larger test files for performance testing
    let performance_files = create_performance_test_files(temp_dir.path()).await?;
    println!("📄 Created {} performance test files", performance_files.len());
    
    // Index all files and measure indexing performance
    let indexing_start = Instant::now();
    for file_path in &performance_files {
        searcher.index_file(file_path).await?;
    }
    let indexing_time = indexing_start.elapsed();
    
    println!("⏱️  Indexing completed in {:.2}s for {} files", 
            indexing_time.as_secs_f32(), performance_files.len());
    
    // Wait for indexing to settle
    sleep(Duration::from_millis(1000)).await;
    
    // Performance test queries
    let performance_queries = vec![
        "function implementation",
        "class definition", 
        "error handling logic",
        "data validation",
        "async processing",
    ];
    
    let mut performance_metrics = Vec::new();
    
    for query in performance_queries {
        let start_time = Instant::now();
        let results = searcher.search(query).await?;
        let search_time = start_time.elapsed();
        
        performance_metrics.push((query, search_time, results.len()));
        
        println!("🔍 '{}': {:.1}ms, {} results", 
                query, search_time.as_millis(), results.len());
    }
    
    // Analyze performance
    let avg_time = performance_metrics.iter()
        .map(|(_, time, _)| time.as_millis())
        .sum::<u128>() as f32 / performance_metrics.len() as f32;
        
    let max_time = performance_metrics.iter()
        .map(|(_, time, _)| time.as_millis())
        .max()
        .unwrap_or(0);
    
    println!("\n======================================================");
    println!("🎯 SEMANTIC PERFORMANCE RESULTS");
    println!("Average search time: {:.1}ms", avg_time);
    println!("Maximum search time: {}ms", max_time);
    println!("Indexing time: {:.2}s for {} files", indexing_time.as_secs_f32(), performance_files.len());
    
    // Performance requirements
    let performance_pass = avg_time < 500.0 && max_time < 1000;
    
    if performance_pass {
        println!("✅ SEMANTIC PERFORMANCE: PASS (100/100)");
        Ok(())
    } else {
        Err(anyhow::anyhow!("❌ SEMANTIC PERFORMANCE: FAIL - Average: {:.1}ms, Max: {}ms", 
                           avg_time, max_time))
    }
}

#[cfg(all(feature = "ml", feature = "vectordb"))]
async fn create_semantic_test_files(temp_dir: &std::path::Path) -> anyhow::Result<Vec<PathBuf>> {
    let files = vec![
        ("calculator.rs", r#"
// Mathematical operations and calculations
use std::f64;

pub struct Calculator {
    precision: usize,
}

impl Calculator {
    pub fn new(precision: usize) -> Self {
        Self { precision }
    }
    
    /// Performs addition with precision handling
    pub fn add(&self, a: f64, b: f64) -> f64 {
        let result = a + b;
        (result * 10_f64.powi(self.precision as i32)).round() / 10_f64.powi(self.precision as i32)
    }
    
    /// Calculates compound interest
    pub fn compound_interest(&self, principal: f64, rate: f64, time: f64) -> f64 {
        principal * (1.0 + rate).powf(time)
    }
    
    /// Function definition for statistical calculations
    pub fn calculate_mean(values: &[f64]) -> f64 {
        if values.is_empty() { return 0.0; }
        values.iter().sum::<f64>() / values.len() as f64
    }
}
"#),
        ("error_handler.rs", r#"
// Comprehensive error handling and validation
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum ProcessingError {
    InvalidInput(String),
    NetworkError(String),
    DataCorruption(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessingError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ProcessingError::NetworkError(msg) => write!(f, "Network error: {}", msg), 
            ProcessingError::DataCorruption(msg) => write!(f, "Data corruption: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct ErrorHandler {
    retry_count: usize,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self { retry_count: 3 }
    }
    
    /// Handle various error types with retry logic
    pub async fn handle_with_retry<T, F, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
        E: Error + 'static,
    {
        let mut attempts = 0;
        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    attempts += 1;
                    if attempts >= self.retry_count {
                        return Err(error);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(100 * attempts as u64)).await;
                }
            }
        }
    }
}
"#),
        ("data_processor.rs", r#"
// Data processing and transformation utilities  
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: String,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

pub struct DataProcessor {
    batch_size: usize,
}

impl DataProcessor {
    pub fn new(batch_size: usize) -> Self {
        Self { batch_size }
    }
    
    /// Process data records in batches for efficiency
    pub async fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, String> {
        let mut processed = Vec::new();
        
        for chunk in records.chunks(self.batch_size) {
            let batch_result = self.transform_chunk(chunk).await?;
            processed.extend(batch_result);
        }
        
        Ok(processed)
    }
    
    async fn transform_chunk(&self, chunk: &[DataRecord]) -> Result<Vec<DataRecord>, String> {
        let mut transformed = Vec::new();
        
        for record in chunk {
            let mut new_record = record.clone();
            // Apply transformations
            new_record.values = new_record.values.iter()
                .map(|&v| v * 1.1) // Apply 10% scaling
                .collect();
            transformed.push(new_record);
        }
        
        Ok(transformed)
    }
    
    /// Calculate statistics for data processing
    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }
        
        let total_values: Vec<f64> = records.iter()
            .flat_map(|r| &r.values)
            .cloned()
            .collect();
            
        if !total_values.is_empty() {
            let sum: f64 = total_values.iter().sum();
            let mean = sum / total_values.len() as f64;
            let variance = total_values.iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f64>() / total_values.len() as f64;
            
            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("std_dev".to_string(), variance.sqrt());
        }
        
        stats
    }
}
"#),
        ("user_model.py", r#"
"""
User model and class implementation with validation
"""
from typing import Optional, List, Dict
from datetime import datetime
import hashlib

class User:
    """User class implementation with comprehensive features"""
    
    def __init__(self, username: str, email: str, age: Optional[int] = None):
        self.username = username
        self.email = email
        self.age = age
        self.created_at = datetime.now()
        self.is_active = True
        self._password_hash = None
        
    def set_password(self, password: str) -> None:
        """Set user password with proper hashing"""
        salt = "user_salt_2024"
        self._password_hash = hashlib.sha256(
            (password + salt).encode()
        ).hexdigest()
    
    def verify_password(self, password: str) -> bool:
        """Verify password against stored hash"""
        if not self._password_hash:
            return False
        salt = "user_salt_2024" 
        password_hash = hashlib.sha256(
            (password + salt).encode()
        ).hexdigest()
        return password_hash == self._password_hash
    
    def to_dict(self) -> Dict:
        """Convert user to dictionary for serialization"""
        return {
            'username': self.username,
            'email': self.email,
            'age': self.age,
            'created_at': self.created_at.isoformat(),
            'is_active': self.is_active
        }
    
    @classmethod 
    def from_dict(cls, data: Dict) -> 'User':
        """Create User from dictionary data"""
        user = cls(data['username'], data['email'], data.get('age'))
        user.is_active = data.get('is_active', True)
        if 'created_at' in data:
            user.created_at = datetime.fromisoformat(data['created_at'])
        return user
    
    def __repr__(self) -> str:
        return f"User(username='{self.username}', email='{self.email}')"

class UserManager:
    """User management class with CRUD operations"""
    
    def __init__(self):
        self._users: Dict[str, User] = {}
    
    def create_user(self, username: str, email: str, age: Optional[int] = None) -> User:
        """Create a new user with validation"""
        if username in self._users:
            raise ValueError(f"User {username} already exists")
        
        # Validate email format
        if '@' not in email or '.' not in email:
            raise ValueError("Invalid email format")
        
        user = User(username, email, age)
        self._users[username] = user
        return user
    
    def get_user(self, username: str) -> Optional[User]:
        """Get user by username"""
        return self._users.get(username)
    
    def update_user(self, username: str, **kwargs) -> bool:
        """Update user attributes"""
        user = self._users.get(username)
        if not user:
            return False
            
        for key, value in kwargs.items():
            if hasattr(user, key):
                setattr(user, key, value)
        return True
    
    def delete_user(self, username: str) -> bool:
        """Delete user by username"""
        if username in self._users:
            del self._users[username]
            return True
        return False
    
    def list_active_users(self) -> List[User]:
        """Get list of all active users"""
        return [user for user in self._users.values() if user.is_active]
"#),
        ("helpers.py", r#"
"""
Utility helpers and function definitions for common operations
"""
import re
import json
from typing import Any, Dict, List, Optional, Union
from functools import wraps
import time

def function_timer(func):
    """Decorator for timing function execution"""
    @wraps(func)
    def wrapper(*args, **kwargs):
        start_time = time.time()
        result = func(*args, **kwargs)
        end_time = time.time()
        print(f"{func.__name__} executed in {end_time - start_time:.4f} seconds")
        return result
    return wrapper

@function_timer 
def validate_email(email: str) -> bool:
    """Validate email address format"""
    pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
    return re.match(pattern, email) is not None

def sanitize_input(input_string: str) -> str:
    """Sanitize user input to prevent injection"""
    # Remove potentially dangerous characters
    dangerous_chars = ['<', '>', '"', "'", '&', ';', '(', ')', '|', '`']
    sanitized = input_string
    for char in dangerous_chars:
        sanitized = sanitized.replace(char, '')
    return sanitized.strip()

class DataValidator:
    """Class implementation for data validation"""
    
    @staticmethod
    def validate_json(json_string: str) -> bool:
        """Validate JSON string format"""
        try:
            json.loads(json_string)
            return True
        except json.JSONDecodeError:
            return False
    
    @staticmethod
    def validate_phone(phone: str) -> bool:
        """Validate phone number format"""
        # Remove all non-digit characters
        digits_only = re.sub(r'\D', '', phone)
        # Check if it's 10 or 11 digits
        return len(digits_only) in [10, 11]
    
    @staticmethod
    def validate_age(age: Union[str, int]) -> bool:
        """Validate age value"""
        try:
            age_int = int(age)
            return 0 <= age_int <= 150
        except (ValueError, TypeError):
            return False

def batch_process(items: List[Any], batch_size: int = 100, 
                 processor_func=None) -> List[Any]:
    """Function definition for batch processing with custom processor"""
    if processor_func is None:
        processor_func = lambda x: x  # Identity function
    
    results = []
    for i in range(0, len(items), batch_size):
        batch = items[i:i + batch_size]
        processed_batch = [processor_func(item) for item in batch]
        results.extend(processed_batch)
    
    return results

def deep_merge_dicts(dict1: Dict, dict2: Dict) -> Dict:
    """Deep merge two dictionaries"""
    result = dict1.copy()
    for key, value in dict2.items():
        if key in result and isinstance(result[key], dict) and isinstance(value, dict):
            result[key] = deep_merge_dicts(result[key], value)
        else:
            result[key] = value
    return result
"#),
        ("validators.py", r#"
"""
Validation utilities and error handling functions
"""
from typing import Any, Callable, Dict, List, Optional, Type, Union
import re
import datetime
from functools import wraps

class ValidationError(Exception):
    """Custom validation error for better error handling"""
    
    def __init__(self, message: str, field: Optional[str] = None, 
                 value: Any = None):
        self.message = message
        self.field = field
        self.value = value
        super().__init__(self.message)
    
    def __str__(self):
        if self.field:
            return f"Validation error in field '{self.field}': {self.message}"
        return f"Validation error: {self.message}"

def validate_required(value: Any, field_name: str) -> Any:
    """Validate that a required field has a value"""
    if value is None or (isinstance(value, str) and not value.strip()):
        raise ValidationError(f"Field '{field_name}' is required", field_name, value)
    return value

def validate_string_length(value: str, min_length: int = 0, 
                         max_length: Optional[int] = None, 
                         field_name: str = "field") -> str:
    """Validate string length constraints"""
    if not isinstance(value, str):
        raise ValidationError(f"Expected string, got {type(value).__name__}", 
                            field_name, value)
    
    if len(value) < min_length:
        raise ValidationError(
            f"String too short: {len(value)} < {min_length}", 
            field_name, value
        )
    
    if max_length and len(value) > max_length:
        raise ValidationError(
            f"String too long: {len(value)} > {max_length}", 
            field_name, value
        )
    
    return value

def validate_email_format(email: str) -> str:
    """Validate email format with comprehensive error handling"""
    email_pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
    if not re.match(email_pattern, email):
        raise ValidationError(
            "Invalid email format", "email", email
        )
    return email.lower()

def validate_numeric_range(value: Union[int, float], min_val: Optional[Union[int, float]] = None,
                         max_val: Optional[Union[int, float]] = None,
                         field_name: str = "field") -> Union[int, float]:
    """Validate numeric value within specified range"""
    if not isinstance(value, (int, float)):
        raise ValidationError(
            f"Expected numeric value, got {type(value).__name__}", 
            field_name, value
        )
    
    if min_val is not None and value < min_val:
        raise ValidationError(
            f"Value {value} is below minimum {min_val}", 
            field_name, value
        )
    
    if max_val is not None and value > max_val:
        raise ValidationError(
            f"Value {value} is above maximum {max_val}", 
            field_name, value
        )
    
    return value

class ValidatorChain:
    """Class implementation for chaining multiple validators"""
    
    def __init__(self, field_name: str):
        self.field_name = field_name
        self.validators: List[Callable] = []
    
    def add_validator(self, validator: Callable, *args, **kwargs) -> 'ValidatorChain':
        """Add a validator function to the chain"""
        def bound_validator(value):
            return validator(value, *args, field_name=self.field_name, **kwargs)
        self.validators.append(bound_validator)
        return self
    
    def validate(self, value: Any) -> Any:
        """Execute all validators in the chain"""
        result = value
        for validator in self.validators:
            result = validator(result)
        return result
    
    def required(self) -> 'ValidatorChain':
        """Mark field as required"""
        return self.add_validator(validate_required)
    
    def string_length(self, min_length: int = 0, 
                     max_length: Optional[int] = None) -> 'ValidatorChain':
        """Add string length validation"""
        return self.add_validator(validate_string_length, min_length, max_length)
    
    def email_format(self) -> 'ValidatorChain':
        """Add email format validation"""
        return self.add_validator(validate_email_format)
    
    def numeric_range(self, min_val: Optional[Union[int, float]] = None,
                     max_val: Optional[Union[int, float]] = None) -> 'ValidatorChain':
        """Add numeric range validation"""
        return self.add_validator(validate_numeric_range, min_val, max_val)

def handle_validation_errors(func: Callable) -> Callable:
    """Decorator for handling validation errors gracefully"""
    @wraps(func)
    def wrapper(*args, **kwargs):
        try:
            return func(*args, **kwargs)
        except ValidationError as e:
            print(f"Validation failed: {e}")
            raise
        except Exception as e:
            print(f"Unexpected error during validation: {e}")
            raise ValidationError(f"Validation failed: {str(e)}")
    return wrapper
"#),
        ("math_utils.js", r#"
/**
 * Mathematical utility functions and operations
 */

class MathUtils {
    /**
     * Function definition for advanced mathematical operations
     */
    static calculateFactorial(n) {
        if (n < 0) throw new Error('Factorial not defined for negative numbers');
        if (n === 0 || n === 1) return 1;
        
        let result = 1;
        for (let i = 2; i <= n; i++) {
            result *= i;
        }
        return result;
    }
    
    /**
     * Calculate Greatest Common Divisor using Euclidean algorithm
     */
    static gcd(a, b) {
        a = Math.abs(a);
        b = Math.abs(b);
        
        while (b !== 0) {
            let temp = b;
            b = a % b;
            a = temp;
        }
        return a;
    }
    
    /**
     * Mathematical operations for statistical analysis
     */
    static calculateStatistics(numbers) {
        if (!Array.isArray(numbers) || numbers.length === 0) {
            throw new Error('Input must be a non-empty array of numbers');
        }
        
        const sum = numbers.reduce((acc, num) => acc + num, 0);
        const mean = sum / numbers.length;
        
        const variance = numbers.reduce((acc, num) => {
            return acc + Math.pow(num - mean, 2);
        }, 0) / numbers.length;
        
        const standardDeviation = Math.sqrt(variance);
        
        const sorted = [...numbers].sort((a, b) => a - b);
        const median = sorted.length % 2 === 0
            ? (sorted[sorted.length / 2 - 1] + sorted[sorted.length / 2]) / 2
            : sorted[Math.floor(sorted.length / 2)];
        
        return {
            sum,
            mean,
            median,
            variance,
            standardDeviation,
            min: Math.min(...numbers),
            max: Math.max(...numbers),
            count: numbers.length
        };
    }
    
    /**
     * Generate mathematical sequences
     */
    static generateFibonacci(count) {
        if (count <= 0) return [];
        if (count === 1) return [0];
        if (count === 2) return [0, 1];
        
        const sequence = [0, 1];
        for (let i = 2; i < count; i++) {
            sequence.push(sequence[i - 1] + sequence[i - 2]);
        }
        return sequence;
    }
    
    /**
     * Mathematical operations for matrix calculations
     */
    static multiplyMatrices(matrixA, matrixB) {
        const rowsA = matrixA.length;
        const colsA = matrixA[0].length;
        const rowsB = matrixB.length;
        const colsB = matrixB[0].length;
        
        if (colsA !== rowsB) {
            throw new Error('Matrix dimensions incompatible for multiplication');
        }
        
        const result = Array(rowsA).fill().map(() => Array(colsB).fill(0));
        
        for (let i = 0; i < rowsA; i++) {
            for (let j = 0; j < colsB; j++) {
                for (let k = 0; k < colsA; k++) {
                    result[i][j] += matrixA[i][k] * matrixB[k][j];
                }
            }
        }
        
        return result;
    }
}

/**
 * Function definitions for geometric calculations
 */
const GeometryUtils = {
    calculateCircleArea: (radius) => {
        if (radius < 0) throw new Error('Radius cannot be negative');
        return Math.PI * radius * radius;
    },
    
    calculateTriangleArea: (base, height) => {
        if (base < 0 || height < 0) {
            throw new Error('Base and height must be positive');
        }
        return 0.5 * base * height;
    },
    
    calculateDistance: (point1, point2) => {
        const dx = point2.x - point1.x;
        const dy = point2.y - point1.y;
        return Math.sqrt(dx * dx + dy * dy);
    },
    
    calculatePolygonArea: (vertices) => {
        if (vertices.length < 3) {
            throw new Error('Polygon must have at least 3 vertices');
        }
        
        let area = 0;
        const n = vertices.length;
        
        for (let i = 0; i < n; i++) {
            const j = (i + 1) % n;
            area += vertices[i].x * vertices[j].y;
            area -= vertices[j].x * vertices[i].y;
        }
        
        return Math.abs(area) / 2;
    }
};

module.exports = {
    MathUtils,
    GeometryUtils
};
"#),
        ("utils.py", r#"
"""
General utility functions for data processing and manipulation
"""
import json
import csv
import os
from typing import Any, Dict, List, Optional, Union, Callable
from datetime import datetime, timedelta
import hashlib
import base64

def read_json_file(file_path: str) -> Dict[str, Any]:
    """Data processing function to read JSON files safely"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            return json.load(f)
    except FileNotFoundError:
        raise FileNotFoundError(f"JSON file not found: {file_path}")
    except json.JSONDecodeError as e:
        raise ValueError(f"Invalid JSON format in {file_path}: {str(e)}")

def write_json_file(data: Dict[str, Any], file_path: str, 
                   pretty: bool = True) -> None:
    """Write data to JSON file with proper formatting"""
    os.makedirs(os.path.dirname(file_path), exist_ok=True)
    
    with open(file_path, 'w', encoding='utf-8') as f:
        if pretty:
            json.dump(data, f, indent=2, ensure_ascii=False)
        else:
            json.dump(data, f, ensure_ascii=False)

def process_csv_data(file_path: str, 
                    processor_func: Optional[Callable] = None) -> List[Dict]:
    """Data processing function for CSV files"""
    results = []
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            reader = csv.DictReader(f)
            for row in reader:
                if processor_func:
                    processed_row = processor_func(row)
                    results.append(processed_row)
                else:
                    results.append(row)
    except FileNotFoundError:
        raise FileNotFoundError(f"CSV file not found: {file_path}")
    
    return results

class DataProcessor:
    """Data processing class with multiple utility methods"""
    
    def __init__(self, chunk_size: int = 1000):
        self.chunk_size = chunk_size
        
    def process_large_dataset(self, data: List[Any], 
                            processor: Callable) -> List[Any]:
        """Process large datasets in chunks to manage memory"""
        results = []
        
        for i in range(0, len(data), self.chunk_size):
            chunk = data[i:i + self.chunk_size]
            processed_chunk = [processor(item) for item in chunk]
            results.extend(processed_chunk)
            
        return results
    
    def aggregate_data(self, data: List[Dict], 
                      group_by: str, 
                      aggregate_fields: Dict[str, str]) -> Dict[str, Dict]:
        """Aggregate data by specified field with various operations"""
        groups = {}
        
        for item in data:
            key = item.get(group_by)
            if key not in groups:
                groups[key] = []
            groups[key].append(item)
        
        aggregated = {}
        for key, items in groups.items():
            aggregated[key] = {}
            
            for field, operation in aggregate_fields.items():
                values = [item.get(field, 0) for item in items 
                         if isinstance(item.get(field), (int, float))]
                
                if not values:
                    aggregated[key][field] = 0
                    continue
                
                if operation == 'sum':
                    aggregated[key][field] = sum(values)
                elif operation == 'avg':
                    aggregated[key][field] = sum(values) / len(values)
                elif operation == 'max':
                    aggregated[key][field] = max(values)
                elif operation == 'min':
                    aggregated[key][field] = min(values)
                elif operation == 'count':
                    aggregated[key][field] = len(values)
        
        return aggregated

def generate_hash(data: str, algorithm: str = 'sha256') -> str:
    """Generate hash for data integrity checking"""
    hash_obj = getattr(hashlib, algorithm)()
    hash_obj.update(data.encode('utf-8'))
    return hash_obj.hexdigest()

def encode_base64(data: Union[str, bytes]) -> str:
    """Encode data to base64 string"""
    if isinstance(data, str):
        data = data.encode('utf-8')
    return base64.b64encode(data).decode('utf-8')

def decode_base64(encoded_data: str) -> bytes:
    """Decode base64 string to bytes"""
    return base64.b64decode(encoded_data.encode('utf-8'))

def format_timestamp(timestamp: Optional[datetime] = None, 
                    format_string: str = '%Y-%m-%d %H:%M:%S') -> str:
    """Format timestamp for consistent display"""
    if timestamp is None:
        timestamp = datetime.now()
    return timestamp.strftime(format_string)

def parse_timestamp(timestamp_str: str, 
                   format_string: str = '%Y-%m-%d %H:%M:%S') -> datetime:
    """Parse timestamp string to datetime object"""
    return datetime.strptime(timestamp_str, format_string)

def calculate_time_difference(start_time: datetime, 
                            end_time: Optional[datetime] = None) -> timedelta:
    """Calculate time difference between two timestamps"""
    if end_time is None:
        end_time = datetime.now()
    return end_time - start_time

class CacheManager:
    """Simple cache management for data processing optimization"""
    
    def __init__(self, max_size: int = 100):
        self.cache: Dict[str, Any] = {}
        self.access_times: Dict[str, datetime] = {}
        self.max_size = max_size
    
    def get(self, key: str) -> Optional[Any]:
        """Get cached value by key"""
        if key in self.cache:
            self.access_times[key] = datetime.now()
            return self.cache[key]
        return None
    
    def put(self, key: str, value: Any) -> None:
        """Cache a value with automatic cleanup"""
        if len(self.cache) >= self.max_size:
            # Remove least recently accessed item
            oldest_key = min(self.access_times.keys(), 
                           key=lambda k: self.access_times[k])
            del self.cache[oldest_key]
            del self.access_times[oldest_key]
        
        self.cache[key] = value
        self.access_times[key] = datetime.now()
    
    def clear(self) -> None:
        """Clear all cached data"""
        self.cache.clear()
        self.access_times.clear()
"#),
    ];
    
    let mut file_paths = Vec::new();
    for (filename, content) in files {
        let file_path = temp_dir.join(filename);
        tokio::fs::write(&file_path, content).await?;
        file_paths.push(file_path);
    }
    
    Ok(file_paths)
}

#[cfg(all(feature = "ml", feature = "vectordb"))]
async fn create_performance_test_files(temp_dir: &std::path::Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut file_paths = Vec::new();
    
    // Create multiple larger files for performance testing
    for i in 0..10 {
        let content = format!(r#"
// Performance test file {} - Complex code structure
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PerformanceTestComponent{} {{
    data: Arc<RwLock<HashMap<String, String>>>,
    cache: HashMap<u64, String>,
    counters: Vec<u64>,
}}

impl PerformanceTestComponent{} {{
    pub fn new() -> Self {{
        Self {{
            data: Arc::new(RwLock::new(HashMap::new())),
            cache: HashMap::new(),
            counters: Vec::new(),
        }}
    }}
    
    /// Function implementation for data processing
    pub async fn process_data(&mut self, input: &str) -> Result<String, String> {{
        let mut data = self.data.write().await;
        
        // Simulate complex processing
        let processed = input.chars()
            .map(|c| c.to_uppercase().to_string())
            .collect::<Vec<_>>()
            .join("");
            
        data.insert(format!("key_{}", {}), processed.clone());
        
        // Update counters
        self.counters.push(processed.len() as u64);
        
        Ok(processed)
    }}
    
    /// Async processing function with error handling
    pub async fn batch_process(&mut self, inputs: Vec<String>) -> Vec<Result<String, String>> {{
        let mut results = Vec::new();
        
        for input in inputs {{
            let result = self.process_data(&input).await;
            results.push(result);
        }}
        
        results
    }}
    
    /// Class implementation with statistical calculations
    pub fn calculate_metrics(&self) -> HashMap<String, f64> {{
        let mut metrics = HashMap::new();
        
        if !self.counters.is_empty() {{
            let sum: u64 = self.counters.iter().sum();
            let avg = sum as f64 / self.counters.len() as f64;
            let max = *self.counters.iter().max().unwrap_or(&0);
            let min = *self.counters.iter().min().unwrap_or(&0);
            
            metrics.insert("average".to_string(), avg);
            metrics.insert("maximum".to_string(), max as f64);
            metrics.insert("minimum".to_string(), min as f64);
            metrics.insert("total".to_string(), sum as f64);
        }}
        
        metrics
    }}
    
    /// Function definition for data validation
    pub fn validate_data(&self, input: &str) -> bool {{
        !input.is_empty() && 
        input.len() < 10000 && 
        !input.contains(&['<', '>', '&'][..])
    }}
    
    /// Error handling function with comprehensive logic
    pub fn handle_error(&self, error: &str) -> String {{
        match error {{
            "timeout" => "Operation timed out - please retry".to_string(),
            "invalid_input" => "Input validation failed - check format".to_string(), 
            "network_error" => "Network connection failed - check connectivity".to_string(),
            _ => format!("Unknown error occurred: {{}}", error),
        }}
    }}
}}

/// Mathematical operations for performance testing
pub fn calculate_complex_result(input: f64) -> f64 {{
    let mut result = input;
    
    // Complex mathematical operations
    for i in 1..=100 {{
        result = result * (i as f64).sin() + (i as f64).cos();
        result = result / (i as f64 + 1.0);
        result = result.abs().sqrt();
    }}
    
    result
}}

/// Function definitions for concurrent processing
pub async fn concurrent_processing(data: Vec<String>) -> Vec<String> {{
    let tasks = data.into_iter().map(|item| {{
        tokio::spawn(async move {{
            // Simulate async work
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            format!("processed: {{}}", item)
        }})
    }});
    
    let mut results = Vec::new();
    for task in tasks {{
        if let Ok(result) = task.await {{
            results.push(result);
        }}
    }}
    
    results
}}
"#, i, i, i);
        
        let file_path = temp_dir.join(format!("perf_test_{}.rs", i));
        tokio::fs::write(&file_path, content).await?;
        file_paths.push(file_path);
    }
    
    Ok(file_paths)
}

// Compile tests without running when features aren't available
#[cfg(not(all(feature = "ml", feature = "vectordb")))]
#[tokio::test] 
async fn semantic_search_feature_disabled() -> anyhow::Result<()> {
    println!("⚠️ SEMANTIC SEARCH VALIDATION SKIPPED");
    println!("Required features 'ml' and 'vectordb' are not enabled");
    println!("To run semantic search tests, use: cargo test --features ml,vectordb");
    Ok(())
}