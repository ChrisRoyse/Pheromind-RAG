use embed_search::chunking::SimpleRegexChunker;
use std::path::Path;

#[test]
fn test_python_file_chunking() {
    let chunker = SimpleRegexChunker::new();
    let path = Path::new("vectortest/auth_service.py");
    
    if path.exists() {
        let chunks = chunker.chunk_file_from_path(path).unwrap();
        
        // Python file should be chunked at method boundaries
        assert!(!chunks.is_empty(), "Python file should produce chunks");
        
        // Check that methods are properly detected
        let method_chunks: Vec<_> = chunks.iter()
            .filter(|c| c.content.contains("def "))
            .collect();
        assert!(method_chunks.len() > 5, "Should detect multiple Python methods");
        
        // Verify chunk sizes are reasonable
        for chunk in &chunks {
            let line_count = chunk.content.lines().count();
            assert!(line_count <= 100, "Chunk should not exceed 100 lines");
        }
        
        // Check specific methods are captured
        let has_hash_password = chunks.iter().any(|c| c.content.contains("def hash_password"));
        let has_create_user = chunks.iter().any(|c| c.content.contains("def create_user"));
        assert!(has_hash_password, "Should detect hash_password method");
        assert!(has_create_user, "Should detect create_user method");
    }
}

#[test]
fn test_javascript_file_chunking() {
    let chunker = SimpleRegexChunker::new();
    let path = Path::new("vectortest/user_controller.js");
    
    if path.exists() {
        let chunks = chunker.chunk_file_from_path(path).unwrap();
        
        assert!(!chunks.is_empty(), "JavaScript file should produce chunks");
        
        // Check async functions are detected
        let async_chunks: Vec<_> = chunks.iter()
            .filter(|c| c.content.contains("async "))
            .collect();
        assert!(!async_chunks.is_empty(), "Should detect async functions");
        
        // Check class methods are properly chunked
        let has_register = chunks.iter().any(|c| c.content.contains("async register"));
        let has_login = chunks.iter().any(|c| c.content.contains("async login"));
        assert!(has_register, "Should detect register method");
        assert!(has_login, "Should detect login method");
    }
}

#[test]
fn test_java_file_chunking() {
    let chunker = SimpleRegexChunker::new();
    let path = Path::new("vectortest/OrderService.java");
    
    if path.exists() {
        let chunks = chunker.chunk_file_from_path(path).unwrap();
        
        assert!(!chunks.is_empty(), "Java file should produce chunks");
        
        // Java methods with annotations
        let public_method_chunks: Vec<_> = chunks.iter()
            .filter(|c| c.content.contains("public "))
            .collect();
        assert!(!public_method_chunks.is_empty(), "Should detect public methods");
        
        // Check for specific Java patterns
        let has_transactional = chunks.iter().any(|c| c.content.contains("@Transactional"));
        assert!(has_transactional, "Should detect annotated methods");
    }
}

#[test]
fn test_go_file_chunking() {
    let chunker = SimpleRegexChunker::new();
    let path = Path::new("vectortest/analytics_dashboard.go");
    
    if path.exists() {
        let chunks = chunker.chunk_file_from_path(path).unwrap();
        
        assert!(!chunks.is_empty(), "Go file should produce chunks");
        
        // Check Go function patterns
        let func_chunks: Vec<_> = chunks.iter()
            .filter(|c| c.content.contains("func "))
            .collect();
        assert!(!func_chunks.is_empty(), "Should detect Go functions");
        
        // Check for receiver methods
        let receiver_methods: Vec<_> = chunks.iter()
            .filter(|c| c.content.contains("func ("))
            .collect();
        assert!(!receiver_methods.is_empty(), "Should detect Go receiver methods");
    }
}

#[test]
fn test_rust_file_chunking() {
    let chunker = SimpleRegexChunker::new();
    let path = Path::new("vectortest/memory_cache.rs");
    
    if path.exists() {
        let chunks = chunker.chunk_file_from_path(path).unwrap();
        
        assert!(!chunks.is_empty(), "Rust file should produce chunks");
        
        // Check impl blocks and functions
        let impl_chunks: Vec<_> = chunks.iter()
            .filter(|c| c.content.contains("impl "))
            .collect();
        assert!(!impl_chunks.is_empty(), "Should detect impl blocks");
        
        let pub_fn_chunks: Vec<_> = chunks.iter()
            .filter(|c| c.content.contains("pub fn"))
            .collect();
        assert!(!pub_fn_chunks.is_empty(), "Should detect public functions");
    }
}

#[test]
fn test_sql_file_chunking() {
    let chunker = SimpleRegexChunker::new();
    let path = Path::new("vectortest/database_migration.sql");
    
    if path.exists() {
        let chunks = chunker.chunk_file_from_path(path).unwrap();
        
        assert!(!chunks.is_empty(), "SQL file should produce chunks");
        
        // Check for CREATE TABLE statements
        let table_chunks: Vec<_> = chunks.iter()
            .filter(|c| c.content.contains("CREATE TABLE"))
            .collect();
        assert!(!table_chunks.is_empty(), "Should detect CREATE TABLE statements");
    }
}

#[test]
fn test_markdown_file_chunking() {
    let chunker = SimpleRegexChunker::new();
    let path = Path::new("vectortest/API_DOCUMENTATION.md");
    
    if path.exists() {
        let chunks = chunker.chunk_file_from_path(path).unwrap();
        
        assert!(!chunks.is_empty(), "Markdown file should produce chunks");
        
        // Markdown files should chunk based on size since they don't have functions
        for chunk in &chunks {
            let line_count = chunk.content.lines().count();
            assert!(line_count <= 100, "Markdown chunks should respect size limit");
        }
    }
}

#[test]
fn test_chunk_line_numbers() {
    let chunker = SimpleRegexChunker::new();
    let content = r#"// File header
fn first_function() {
    println!("First");
}

fn second_function() {
    println!("Second");
}

fn third_function() {
    println!("Third");
}"#;
    
    let chunks = chunker.chunk_file(content);
    
    // Verify line numbers are correct and continuous
    for i in 0..chunks.len() {
        assert!(chunks[i].start_line <= chunks[i].end_line, 
                "Start line should be <= end line");
        
        if i > 0 {
            assert!(chunks[i].start_line > chunks[i-1].end_line,
                    "Chunks should not overlap");
        }
    }
    
    // First chunk should start at line 0
    assert_eq!(chunks[0].start_line, 0);
    
    // Last chunk should end at the last line
    let total_lines = content.lines().count();
    assert_eq!(chunks.last().unwrap().end_line, total_lines - 1);
}

#[test]
fn test_large_file_chunking() {
    let chunker = SimpleRegexChunker::with_chunk_size(50); // Smaller chunks for testing
    
    // Create a large synthetic file
    let mut content = String::new();
    for i in 0..10 {
        content.push_str(&format!("fn function_{:03}() {{\n", i));
        for j in 0..60 {
            content.push_str(&format!("    // Line {} in function {}\n", j, i));
        }
        content.push_str("}\n\n");
    }
    
    let chunks = chunker.chunk_file(&content);
    
    // Should create multiple chunks due to size limit
    assert!(chunks.len() > 10, "Large file should produce many chunks");
    
    // Each chunk should respect the size limit
    for chunk in &chunks {
        assert!(chunk.content.lines().count() <= 50, 
                "Chunk should not exceed size limit");
    }
}

#[test]
fn test_mixed_language_patterns() {
    let chunker = SimpleRegexChunker::new();
    
    // Test content with mixed language patterns
    let content = r#"
// JavaScript style
async function processOrder(order) {
    return await db.save(order);
}

# Python style
def calculate_total(items):
    return sum(item.price for item in items)

// Java style
public class OrderProcessor {
    private void validateOrder(Order order) {
        // validation logic
    }
}

// Go style
func (s *Server) HandleRequest(w http.ResponseWriter, r *http.Request) {
    fmt.Fprintf(w, "Hello")
}

// Rust style
pub fn process_payment(amount: f64) -> Result<Receipt, Error> {
    Ok(Receipt::new(amount))
}
"#;

    let chunks = chunker.chunk_file(content);
    
    // Should detect all different function patterns
    assert!(chunks.len() >= 5, "Should detect functions from different languages");
    
    // Verify each pattern is detected
    let patterns = vec![
        "async function",
        "def calculate_total",
        "class OrderProcessor",
        "func (s *Server)",
        "pub fn process_payment"
    ];
    
    for pattern in patterns {
        let found = chunks.iter().any(|c| c.content.contains(pattern));
        assert!(found, "Should detect pattern: {}", pattern);
    }
}