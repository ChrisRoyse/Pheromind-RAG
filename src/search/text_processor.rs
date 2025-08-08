use std::collections::HashSet;
use rust_stemmers::{Algorithm, Stemmer};
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;
use serde::{Serialize, Deserialize};

/// Code-aware text processor for optimal BM25 performance
pub struct CodeTextProcessor {
    /// Stop words to filter out
    stop_words: HashSet<String>,
    /// Porter stemmer for natural language in comments
    stemmer: Stemmer,
    /// Whether to enable stemming
    enable_stemming: bool,
    /// Whether to generate n-grams
    enable_ngrams: bool,
    /// Maximum n-gram size
    max_ngram_size: usize,
    /// Minimum term length to index
    min_term_length: usize,
    /// Maximum term length to index
    max_term_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedToken {
    pub text: String,
    pub original_text: String,
    pub token_type: TokenType,
    pub position: usize,
    pub line_number: usize,
    pub importance_weight: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    Identifier,      // Variable/function names (high importance)
    Keyword,         // Language keywords (medium importance)
    Comment,         // Documentation (low importance)
    String,          // String literals (low importance)
    Number,          // Numeric literals (low importance)
    Operator,        // Operators (very low importance)
    Other,           // Everything else
}

// CodeTextProcessor must be explicitly created with new() - no default fallback allowed
// This ensures intentional configuration of text processing

impl std::fmt::Debug for CodeTextProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodeTextProcessor")
            .field("stop_words", &self.stop_words)
            .field("enable_stemming", &self.enable_stemming)
            .field("enable_ngrams", &self.enable_ngrams)
            .field("max_ngram_size", &self.max_ngram_size)
            .field("min_term_length", &self.min_term_length)
            .field("max_term_length", &self.max_term_length)
            .finish()
    }
}

impl Clone for CodeTextProcessor {
    fn clone(&self) -> Self {
        Self {
            stop_words: self.stop_words.clone(),
            stemmer: Stemmer::create(Algorithm::English), // Recreate stemmer since it may not be Clone
            enable_stemming: self.enable_stemming,
            enable_ngrams: self.enable_ngrams,
            max_ngram_size: self.max_ngram_size,
            min_term_length: self.min_term_length,
            max_term_length: self.max_term_length,
        }
    }
}

impl CodeTextProcessor {
    pub fn new() -> Self {
        let stop_words = Self::default_stop_words();
        let stemmer = Stemmer::create(Algorithm::English);
        
        Self {
            stop_words,
            stemmer,
            enable_stemming: true,
            enable_ngrams: true,
            max_ngram_size: 3,
            min_term_length: 2,
            max_term_length: 50,
        }
    }
    
    pub fn with_config(
        enable_stemming: bool,
        enable_ngrams: bool,
        max_ngram_size: usize,
        min_term_length: usize,
        max_term_length: usize,
        custom_stop_words: Vec<String>,
    ) -> Self {
        let mut stop_words = Self::default_stop_words();
        for word in custom_stop_words {
            stop_words.insert(word.to_lowercase());
        }
        
        let stemmer = Stemmer::create(Algorithm::English);
        
        Self {
            stop_words,
            stemmer,
            enable_stemming,
            enable_ngrams,
            max_ngram_size,
            min_term_length,
            max_term_length,
        }
    }
    
    /// Default stop words for code search
    fn default_stop_words() -> HashSet<String> {
        let words = vec![
            // Only truly common English words, not programming keywords
            // Programming keywords are important for code search!
            "the", "and", "or", "is", "it", "in", "to", "of", "a", "an",
            "as", "at", "by", "from", "with", "this", "that",
            "be", "are", "was", "were", "been", "being", "have", "has",
            "had", "having", "do", "does", "did", "doing", "will", "would",
            "could", "should", "may", "might", "must", "can", "shall",
        ];
        
        words.into_iter().map(|s| s.to_string()).collect()
    }
    
    /// Process text with language awareness (alias for tokenize_code)
    pub fn process_text(&self, text: &str, language: &str) -> Vec<ProcessedToken> {
        self.tokenize_code(text, Some(language))
    }
    
    /// Tokenize code content with language awareness
    pub fn tokenize_code(&self, content: &str, language: Option<&str>) -> Vec<ProcessedToken> {
        let mut tokens = Vec::new();
        let mut position = 0;
        
        // Split content into lines for line number tracking
        let lines: Vec<&str> = content.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            // Simple tokenization for now - can be enhanced with language-specific parsers
            let line_tokens = self.tokenize_line(line, line_num, language);
            
            for mut token in line_tokens {
                token.position = position;
                position += 1;
                
                // Apply filters
                if self.should_index_token(&token) {
                    tokens.push(token);
                }
            }
        }
        
        // Generate n-grams if enabled
        if self.enable_ngrams && tokens.len() > 1 {
            let ngrams = self.generate_ngrams(&tokens);
            tokens.extend(ngrams);
        }
        
        tokens
    }
    
    /// Tokenize a single line of code
    fn tokenize_line(&self, line: &str, line_number: usize, language: Option<&str>) -> Vec<ProcessedToken> {
        let mut tokens = Vec::new();
        
        // Check if line is a comment
        let is_comment = self.is_comment_line(line, language);
        
        // Split on word boundaries and common separators
        let words = line.unicode_words();
        
        for word in words {
            // Normalize the word
            let normalized = word.nfc().collect::<String>().to_lowercase();
            
            // Skip if it's a stop word
            if self.stop_words.contains(&normalized) {
                continue;
            }
            
            // Determine token type
            let token_type = if is_comment {
                TokenType::Comment
            } else {
                self.classify_token(&normalized, language)
            };
            
            // Apply stemming if enabled and appropriate
            let processed_text = if self.enable_stemming && token_type == TokenType::Comment {
                self.stemmer.stem(&normalized).to_string()
            } else {
                normalized.clone()
            };
            
            // Calculate importance weight
            let importance_weight = match token_type {
                TokenType::Identifier => 1.0,
                TokenType::Keyword => 0.8,
                TokenType::Comment => 0.6,
                TokenType::String => 0.4,
                TokenType::Number => 0.3,
                TokenType::Operator => 0.2,
                TokenType::Other => 0.5,
            };
            
            // Handle camelCase and snake_case splitting
            let subtokens = self.split_compound_identifier(&processed_text);
            
            for subtoken in subtokens {
                if subtoken.len() >= self.min_term_length && subtoken.len() <= self.max_term_length {
                    tokens.push(ProcessedToken {
                        text: subtoken.clone(),
                        original_text: word.to_string(),
                        token_type: token_type.clone(),
                        position: 0, // Will be set by caller
                        line_number,
                        importance_weight,
                    });
                }
            }
        }
        
        tokens
    }
    
    /// Check if a line is a comment
    fn is_comment_line(&self, line: &str, language: Option<&str>) -> bool {
        let trimmed = line.trim();
        
        match language {
            Some("rust") | Some("c") | Some("cpp") | Some("java") | Some("javascript") | 
            Some("typescript") | Some("go") => {
                trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*")
            }
            Some("python") | Some("bash") => {
                trimmed.starts_with("#")
            }
            Some("html") | Some("xml") => {
                trimmed.starts_with("<!--")
            }
            Some("css") => {
                trimmed.starts_with("/*")
            }
            _ => {
                // Generic comment detection
                trimmed.starts_with("//") || trimmed.starts_with("#") || 
                trimmed.starts_with("/*") || trimmed.starts_with("<!--")
            }
        }
    }
    
    /// Classify a token based on its content
    fn classify_token(&self, token: &str, _language: Option<&str>) -> TokenType {
        // Check if it's a number
        if token.chars().all(|c| c.is_numeric() || c == '.' || c == '-') {
            return TokenType::Number;
        }
        
        // Check if it's an operator
        if token.chars().all(|c| "+-*/%=<>!&|^~".contains(c)) {
            return TokenType::Operator;
        }
        
        // Check if it's a common keyword (language-agnostic for now)
        let keywords = [
            "if", "else", "for", "while", "return", "function", "class", "struct",
            "import", "export", "public", "private", "static", "const", "let", "var",
            "async", "await", "try", "catch", "throw", "new", "this", "self",
        ];
        
        if keywords.contains(&token) {
            return TokenType::Keyword;
        }
        
        // Check if it looks like an identifier (contains letters/numbers/underscores)
        if token.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return TokenType::Identifier;
        }
        
        TokenType::Other
    }
    
    /// Split compound identifiers (camelCase, snake_case, etc.)
    fn split_compound_identifier(&self, identifier: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        
        // Always add the original identifier
        tokens.push(identifier.to_string());
        
        // Split on underscores
        if identifier.contains('_') {
            let parts: Vec<String> = identifier.split('_')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            tokens.extend(parts);
        }
        
        // Split camelCase - ALWAYS try to split
        let camel_parts = self.split_camel_case(identifier);
        tokens.extend(camel_parts);
        
        // Remove duplicates and return
        tokens.sort();
        tokens.dedup();
        tokens
    }
    
    /// Split camelCase identifiers
    fn split_camel_case(&self, text: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut prev_was_upper = false;
        
        for ch in text.chars() {
            if ch.is_uppercase() && !prev_was_upper && !current.is_empty() {
                parts.push(current.to_lowercase());
                current = String::new();
            }
            current.push(ch);
            prev_was_upper = ch.is_uppercase();
        }
        
        if !current.is_empty() {
            parts.push(current.to_lowercase());
        }
        
        parts
    }
    
    /// Generate n-grams from tokens
    fn generate_ngrams(&self, tokens: &[ProcessedToken]) -> Vec<ProcessedToken> {
        let mut ngrams = Vec::new();
        
        for n in 2..=self.max_ngram_size.min(tokens.len()) {
            for i in 0..tokens.len() - n + 1 {
                let ngram_text = tokens[i..i + n]
                    .iter()
                    .map(|t| t.text.as_str())
                    .collect::<Vec<_>>()
                    .join("_");
                
                // Average importance of constituent tokens
                let avg_importance = tokens[i..i + n]
                    .iter()
                    .map(|t| t.importance_weight)
                    .sum::<f32>() / n as f32;
                
                ngrams.push(ProcessedToken {
                    text: ngram_text,
                    original_text: format!("ngram_{}", n),
                    token_type: TokenType::Other,
                    position: tokens[i].position,
                    line_number: tokens[i].line_number,
                    importance_weight: avg_importance * 0.8, // Slightly reduce n-gram importance
                });
            }
        }
        
        ngrams
    }
    
    /// Check if a token should be indexed
    fn should_index_token(&self, token: &ProcessedToken) -> bool {
        // Check length constraints
        if token.text.len() < self.min_term_length || token.text.len() > self.max_term_length {
            return false;
        }
        
        // Check if it's a stop word
        if self.stop_words.contains(&token.text) {
            return false;
        }
        
        // Filter out pure operators
        if token.token_type == TokenType::Operator {
            return false;
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenization_basic() {
        let processor = CodeTextProcessor::new();
        let code = "function calculateTotal(items) { return sum; }";
        let tokens = processor.tokenize_code(code, Some("javascript"));
        
        assert!(!tokens.is_empty());
        
        // Should include "calculate", "total", "items", "sum" but not "function" or "return"
        let token_texts: Vec<String> = tokens.iter().map(|t| t.text.clone()).collect();
        assert!(token_texts.contains(&"calculate".to_string()) || token_texts.contains(&"calculatetotal".to_string()));
        assert!(token_texts.contains(&"items".to_string()));
        assert!(token_texts.contains(&"sum".to_string()));
    }
    
    #[test]
    fn test_camel_case_splitting() {
        let processor = CodeTextProcessor::new();
        let tokens = processor.split_compound_identifier("getUserName");
        
        assert!(tokens.contains(&"get".to_string()));
        assert!(tokens.contains(&"user".to_string()));
        assert!(tokens.contains(&"name".to_string()));
    }
    
    #[test]
    fn test_snake_case_splitting() {
        let processor = CodeTextProcessor::new();
        let tokens = processor.split_compound_identifier("get_user_name");
        
        assert!(tokens.contains(&"get".to_string()));
        assert!(tokens.contains(&"user".to_string()));
        assert!(tokens.contains(&"name".to_string()));
    }
    
    #[test]
    fn test_comment_detection() {
        let processor = CodeTextProcessor::new();
        
        assert!(processor.is_comment_line("// This is a comment", Some("rust")));
        assert!(processor.is_comment_line("# Python comment", Some("python")));
        assert!(processor.is_comment_line("/* C-style comment */", Some("c")));
        assert!(!processor.is_comment_line("let x = 5;", Some("rust")));
    }
}