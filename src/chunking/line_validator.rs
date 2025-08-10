// Line validation for code chunking

use anyhow::Result;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct LineValidator {
    max_line_length: usize,
    min_line_length: usize,
    excluded_patterns: HashSet<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Line too long: {0} characters (max: {1})")]
    LineTooLong(usize, usize),
    
    #[error("Line too short: {0} characters (min: {1})")]
    LineTooShort(usize, usize),
    
    #[error("Line matches excluded pattern: {0}")]
    ExcludedPattern(String),
}

impl Default for LineValidator {
    fn default() -> Self {
        Self {
            max_line_length: 500,
            min_line_length: 3,
            excluded_patterns: HashSet::new(),
        }
    }
}

impl LineValidator {
    pub fn new(max_length: usize, min_length: usize) -> Self {
        Self {
            max_line_length: max_length,
            min_line_length: min_length,
            excluded_patterns: HashSet::new(),
        }
    }
    
    pub fn add_excluded_pattern(&mut self, pattern: String) {
        self.excluded_patterns.insert(pattern);
    }
    
    pub fn validate_line(&self, line: &str) -> Result<(), ValidationError> {
        let line_len = line.len();
        
        if line_len > self.max_line_length {
            return Err(ValidationError::LineTooLong(line_len, self.max_line_length));
        }
        
        if line_len < self.min_line_length && !line.trim().is_empty() {
            return Err(ValidationError::LineTooShort(line_len, self.min_line_length));
        }
        
        for pattern in &self.excluded_patterns {
            if line.contains(pattern) {
                return Err(ValidationError::ExcludedPattern(pattern.clone()));
            }
        }
        
        Ok(())
    }
    
    pub fn validate_lines<'a>(&self, lines: &[&'a str]) -> Result<Vec<&'a str>> {
        let mut valid_lines = Vec::new();
        
        for line in lines {
            if self.validate_line(line).is_ok() {
                valid_lines.push(*line);
            }
        }
        
        Ok(valid_lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_line_validation() {
        let validator = LineValidator::default();
        
        assert!(validator.validate_line("fn main() {").is_ok());
        assert!(validator.validate_line("a").is_err()); // Too short
        
        let long_line = "a".repeat(501);
        assert!(validator.validate_line(&long_line).is_err()); // Too long
    }
}