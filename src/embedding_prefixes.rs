use anyhow::Result;

/// Task types for nomic-embed-code model with correct prefixes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmbeddingTask {
    SearchQuery,     // User search queries
    SearchDocument,  // Documents for indexing
    CodeDefinition,  // Code definitions
    CodeUsage,       // Code usage examples
    Classification,  // Classification tasks
    Clustering,      // Clustering tasks
}

impl EmbeddingTask {
    /// Get the appropriate prefix for nomic-embed-code according to specifications
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::SearchQuery => "search_query: ",
            Self::SearchDocument => "search_document: ",
            Self::CodeDefinition => "def: ",
            Self::CodeUsage => "usage: ",
            Self::Classification => "classification: ",
            Self::Clustering => "clustering: ",
        }
    }
    
    /// Apply prefix to text
    pub fn apply_prefix(&self, text: &str) -> String {
        format!("{}{}", self.prefix(), text)
    }

    /// Infer task type from code content
    pub fn infer_from_code(code: &str) -> Self {
        let code_lower = code.to_lowercase();
        if code_lower.contains("fn ") || code_lower.contains("def ") || 
           code_lower.contains("function ") || code_lower.contains("class ") {
            Self::CodeDefinition
        } else if code_lower.contains("use ") || code_lower.contains("import ") ||
                  code_lower.contains("call") || code_lower.contains("invoke") {
            Self::CodeUsage
        } else {
            Self::CodeDefinition // Default for code
        }
    }
}

/// Language-specific code formatting
pub struct CodeFormatter;

impl CodeFormatter {
    /// Format code with language context
    pub fn format_code(code: &str, language: &str) -> String {
        match language.to_lowercase().as_str() {
            "rust" | "rs" => format!("// Rust\n{}", code),
            "python" | "py" => format!("# Python\n{}", code),
            "javascript" | "js" => format!("// JavaScript\n{}", code),
            "typescript" | "ts" => format!("// TypeScript\n{}", code),
            "go" => format!("// Go\n{}", code),
            "java" => format!("// Java\n{}", code),
            "cpp" | "cc" | "cxx" => format!("// C++\n{}", code),
            "c" => format!("// C\n{}", code),
            _ => code.to_string(),
        }
    }

    /// Detect language from file extension
    pub fn detect_language(filename: &str) -> Option<&'static str> {
        let ext = filename.split('.').last()?.to_lowercase();
        match ext.as_str() {
            "rs" => Some("rust"),
            "py" => Some("python"),
            "js" => Some("javascript"),
            "ts" => Some("typescript"),
            "go" => Some("go"),
            "java" => Some("java"),
            "cpp" | "cc" | "cxx" => Some("cpp"),
            "c" => Some("c"),
            _ => None,
        }
    }

    /// Format code for definition task
    pub fn format_definition(code: &str, language: Option<&str>) -> String {
        match language {
            Some(lang) => Self::format_code(code, lang),
            None => code.to_string(),
        }
    }

    /// Format code for usage task
    pub fn format_usage(code: &str, context: Option<&str>) -> String {
        match context {
            Some(ctx) => format!("Context: {}\n{}", ctx, code),
            None => code.to_string(),
        }
    }
}

/// Batch processing utilities
pub struct BatchProcessor;

impl BatchProcessor {
    /// Apply same task prefix to multiple texts
    pub fn apply_batch_prefix(texts: Vec<&str>, task: EmbeddingTask) -> Vec<String> {
        texts.into_iter()
            .map(|text| task.apply_prefix(text))
            .collect()
    }

    /// Process code snippets with language detection
    pub fn process_code_batch(
        codes: Vec<(&str, Option<&str>)>, // (code, filename)
        task: EmbeddingTask,
    ) -> Vec<String> {
        codes.into_iter()
            .map(|(code, filename)| {
                let language = filename.and_then(CodeFormatter::detect_language);
                let formatted = match task {
                    EmbeddingTask::CodeDefinition => CodeFormatter::format_definition(code, language),
                    EmbeddingTask::CodeUsage => CodeFormatter::format_usage(code, None),
                    _ => code.to_string(),
                };
                task.apply_prefix(&formatted)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_prefixes() {
        assert_eq!(EmbeddingTask::SearchQuery.prefix(), "search_query: ");
        assert_eq!(EmbeddingTask::SearchDocument.prefix(), "search_document: ");
        assert_eq!(EmbeddingTask::CodeDefinition.prefix(), "def: ");
        assert_eq!(EmbeddingTask::CodeUsage.prefix(), "usage: ");
        assert_eq!(EmbeddingTask::Classification.prefix(), "classification: ");
        assert_eq!(EmbeddingTask::Clustering.prefix(), "clustering: ");
    }

    #[test]
    fn test_apply_prefix() {
        let task = EmbeddingTask::SearchQuery;
        let result = task.apply_prefix("find rust functions");
        assert_eq!(result, "search_query: find rust functions");
    }

    #[test]
    fn test_language_detection() {
        assert_eq!(CodeFormatter::detect_language("main.rs"), Some("rust"));
        assert_eq!(CodeFormatter::detect_language("script.py"), Some("python"));
        assert_eq!(CodeFormatter::detect_language("app.js"), Some("javascript"));
        assert_eq!(CodeFormatter::detect_language("test.go"), Some("go"));
        assert_eq!(CodeFormatter::detect_language("unknown.xyz"), None);
    }

    #[test]
    fn test_code_formatting() {
        let code = "fn main() {}";
        let formatted = CodeFormatter::format_code(code, "rust");
        assert_eq!(formatted, "// Rust\nfn main() {}");

        let py_code = "def main():";
        let py_formatted = CodeFormatter::format_code(py_code, "python");
        assert_eq!(py_formatted, "# Python\ndef main():");
    }

    #[test]
    fn test_task_inference() {
        let definition = "fn calculate(x: i32) -> i32 { x * 2 }";
        assert_eq!(EmbeddingTask::infer_from_code(definition), EmbeddingTask::CodeDefinition);

        let usage = "use std::collections::HashMap;";
        assert_eq!(EmbeddingTask::infer_from_code(usage), EmbeddingTask::CodeUsage);
    }

    #[test]
    fn test_batch_processing() {
        let texts = vec!["text1", "text2", "text3"];
        let results = BatchProcessor::apply_batch_prefix(texts, EmbeddingTask::SearchDocument);
        assert_eq!(results.len(), 3);
        assert!(results[0].starts_with("search_document: "));
    }

    #[test]
    fn test_code_batch_processing() {
        let codes = vec![
            ("fn main() {}", Some("main.rs")),
            ("def hello():", Some("hello.py")),
        ];
        let results = BatchProcessor::process_code_batch(codes, EmbeddingTask::CodeDefinition);
        assert_eq!(results.len(), 2);
        assert!(results[0].starts_with("def: // Rust"));
        assert!(results[1].starts_with("def: # Python"));
    }
}