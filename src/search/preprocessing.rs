use std::collections::HashSet;

pub struct QueryPreprocessor;

impl QueryPreprocessor {
    pub fn new() -> Self {
        Self
    }
    
    pub fn preprocess(&self, query: &str) -> String {
        let mut processed = query.to_lowercase();
        
        // Remove common noise words
        let noise_words: HashSet<&str> = [
            "the", "a", "an", "in", "of", "for", "to", "with", "by", "at", "from"
        ].iter().cloned().collect();
        
        // Split into words and filter
        let words: Vec<String> = processed
            .split_whitespace()
            .filter(|word| !noise_words.contains(word))
            .map(|s| s.to_string())
            .collect();
        
        processed = words.join(" ");
        
        // Expand common programming abbreviations
        // Use word boundary approach to prevent double replacement
        let words: Vec<String> = processed
            .split_whitespace()
            .map(|word| {
                match word {
                    "fn" => "function",
                    "impl" => "implementation", 
                    "struct" => "structure",
                    "auth" => "authentication",
                    "config" => "configuration",
                    "db" => "database",
                    "api" => "application programming interface",
                    "ui" => "user interface",
                    "ux" => "user experience",
                    _ => word,
                }
            })
            .map(|s| s.to_string())
            .collect();
        
        processed = words.join(" ");
        
        // Remove excessive whitespace
        processed = processed.split_whitespace().collect::<Vec<_>>().join(" ");
        
        processed.trim().to_string()
    }
    
    pub fn extract_keywords(&self, query: &str) -> Vec<String> {
        let processed = self.preprocess(query);
        processed
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_preprocessing_removes_noise_words() {
        let preprocessor = QueryPreprocessor::new();
        let result = preprocessor.preprocess("find the function in the database");
        assert_eq!(result, "find function database");
    }
    
    #[test]
    fn test_preprocessing_expands_abbreviations() {
        let preprocessor = QueryPreprocessor::new();
        let result = preprocessor.preprocess("fn auth db");
        assert_eq!(result, "function authentication database");
    }
    
    #[test]
    fn test_preprocessing_normalizes_whitespace() {
        let preprocessor = QueryPreprocessor::new();
        let result = preprocessor.preprocess("  multiple   spaces   here  ");
        assert_eq!(result, "multiple spaces here");
    }
}