use regex::Regex;
use std::path::Path;
use crate::config::Config;

// Language-specific patterns
const FUNCTION_PATTERNS: &[&str] = &[
    r"^\s*(pub|public|private|protected|static|async)?\s*(fn|func|function|def)\s+\w+",  // Rust, Go, Python, JS
    r"^\s*(public|private|protected|static)?\s*\w+\s+\w+\s*\([^)]*\)\s*\{",  // Java, C#, C++
    r"^\s*def\s+\w+\s*\(",  // Python
    r"^\s*(async\s+)?function\s+\w+",  // JavaScript
    r"^\s*func\s+(\(\w+\s+\*?\w+\)\s+)?\w+\s*\(",  // Go
];

const CLASS_PATTERNS: &[&str] = &[
    r"^\s*(pub|public|private|protected)?\s*(class|struct|interface|enum|trait)\s+\w+",
    r"^\s*type\s+\w+\s+(struct|interface)",  // Go
    r"^\s*CREATE\s+TABLE",  // SQL
];

pub struct SimpleRegexChunker {
    function_patterns: Vec<Regex>,
    class_patterns: Vec<Regex>,
    chunk_size_target: usize,
}

impl SimpleRegexChunker {
    /// Create a new regex chunker using configured chunk size
    /// Returns an error if configuration is not properly initialized  
    pub fn new() -> Result<Self, crate::error::EmbedError> {
        let chunk_size = Config::chunk_size()?;
        Self::with_chunk_size(chunk_size)
    }
    
    pub fn with_chunk_size(chunk_size: usize) -> Result<Self, crate::error::EmbedError> {
        let function_patterns = FUNCTION_PATTERNS
            .iter()
            .map(|p| Regex::new(p).map_err(|e| crate::error::EmbedError::Internal {
                message: format!("Invalid regex pattern '{}': {}", p, e),
                backtrace: None,
            }))
            .collect::<Result<Vec<_>, _>>()?;
            
        let class_patterns = CLASS_PATTERNS
            .iter()
            .map(|p| Regex::new(p).map_err(|e| crate::error::EmbedError::Internal {
                message: format!("Invalid regex pattern '{}': {}", p, e),
                backtrace: None,
            }))
            .collect::<Result<Vec<_>, _>>()?;
            
        Ok(Self {
            function_patterns,
            class_patterns,
            chunk_size_target: chunk_size,
        })
    }
    
    pub fn chunk_file(&self, content: &str) -> Vec<Chunk> {
        let lines: Vec<&str> = content.lines().collect();
        let mut chunks = Vec::new();
        let mut current_chunk_lines = Vec::new();
        let mut start_line = 0;
        
        for (i, line) in lines.iter().enumerate() {
            if i > 0 && self.is_chunk_boundary(line) && !current_chunk_lines.is_empty() {
                let chunk_content = self.build_chunk_content(&lines, start_line, i - 1);
                chunks.push(Chunk {
                    content: chunk_content,
                    start_line,
                    end_line: i - 1,
                });
                current_chunk_lines.clear();
                start_line = i;
            }
            
            current_chunk_lines.push(*line);
            
            if current_chunk_lines.len() >= self.chunk_size_target {
                let chunk_content = self.build_chunk_content(&lines, start_line, i);
                chunks.push(Chunk {
                    content: chunk_content,
                    start_line,
                    end_line: i,
                });
                current_chunk_lines.clear();
                start_line = i + 1;
            }
        }
        
        if !current_chunk_lines.is_empty() {
            let end_line = lines.len() - 1;
            let chunk_content = self.build_chunk_content(&lines, start_line, end_line);
            chunks.push(Chunk {
                content: chunk_content,
                start_line,
                end_line,
            });
        }
        
        chunks
    }
    
    /// Build chunk content that exactly matches the original file's line structure
    fn build_chunk_content(&self, lines: &[&str], start_line: usize, end_line: usize) -> String {
        lines[start_line..=end_line].join("\n")
    }
    
    fn is_chunk_boundary(&self, line: &str) -> bool {
        self.function_patterns.iter().any(|p| p.is_match(line)) || 
        self.class_patterns.iter().any(|p| p.is_match(line))
    }
    
    pub fn chunk_file_from_path(&self, path: &Path) -> std::io::Result<Vec<Chunk>> {
        let content = std::fs::read_to_string(path)?;
        Ok(self.chunk_file(&content))
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Chunk {
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init() {
        INIT.call_once(|| {
            // Initialize config for tests
            let config = crate::config::Config::new_test_config();
            crate::config::CONFIG.write().unwrap().replace(config);
        });
    }

    #[test]
    fn test_basic_chunking() {
        init();
        let chunker = SimpleRegexChunker::new().expect("Failed to create chunker");
        let content = "line1\nline2\nfn test() {\n    body\n}\nline6";
        let chunks = chunker.chunk_file(content);
        
        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].start_line, 0);
    }
    
    #[test]
    fn test_chunk_size_limit() {
        init();
        let chunker = SimpleRegexChunker::new().expect("Failed to create chunker");
        let mut content = String::new();
        for i in 0..150 {
            content.push_str(&format!("line {}\n", i));
        }
        
        let chunks = chunker.chunk_file(&content);
        assert!(chunks.len() > 1);
        assert!(chunks[0].content.lines().count() <= 100);
    }
    
    #[test]
    fn test_function_boundary_detection() {
        init();
        let chunker = SimpleRegexChunker::new().expect("Failed to create chunker");
        let content = "// comment\nfn first() {\n}\nfn second() {\n}";
        let chunks = chunker.chunk_file(content);
        
        // The chunker creates chunks at boundaries - this is correct behavior
        // The 3-chunk context expansion happens during search, not during chunking
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].content, "// comment");
        assert!(chunks[1].content.contains("fn first"));
        assert!(chunks[2].content.contains("fn second"));
    }
}