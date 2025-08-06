use std::path::{Path, PathBuf};
use std::sync::Arc;
use regex::Regex;
use rayon::prelude::*;
use walkdir::WalkDir;
use anyhow::{Result, Context};
use crate::search::ExactMatch;

/// Native Rust search implementation using parallel processing
/// Fast parallel search using rayon and regex
#[derive(Debug, Clone)]
pub struct NativeSearcher {
    case_sensitive: bool,
    max_depth: Option<usize>,
    follow_links: bool,
    ignore_hidden: bool,
}

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub column: usize,
    pub content: String,
    pub matched_text: String,
}

impl NativeSearcher {
    pub fn new() -> Self {
        Self {
            case_sensitive: false,
            max_depth: None,
            follow_links: false,
            ignore_hidden: true,
        }
    }

    pub fn case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    pub fn follow_links(mut self, follow: bool) -> Self {
        self.follow_links = follow;
        self
    }

    pub fn ignore_hidden(mut self, ignore: bool) -> Self {
        self.ignore_hidden = ignore;
        self
    }

    /// Search for a pattern in files under the given directory
    pub fn search(&self, pattern: &str, search_dir: &Path) -> Result<Vec<SearchMatch>> {
        let regex = if self.case_sensitive {
            Regex::new(pattern)?
        } else {
            Regex::new(&format!("(?i){}", pattern))?
        };

        let regex = Arc::new(regex);
        
        // Collect all files to search
        let files: Vec<PathBuf> = self.collect_files(search_dir)?;
        
        // Search files in parallel
        let matches: Vec<SearchMatch> = files
            .into_par_iter()
            .filter_map(|file_path| {
                self.search_file(&regex, &file_path).ok()
            })
            .flatten()
            .collect();

        Ok(matches)
    }

    /// Search for exact matches (for compatibility with existing interface)
    pub fn search_exact(&self, pattern: &str, search_dir: &Path) -> Result<Vec<ExactMatch>> {
        let matches = self.search(pattern, search_dir)?;
        
        Ok(matches.into_iter().map(|m| ExactMatch {
            file_path: m.file_path.to_string_lossy().to_string(),
            line_number: m.line_number,
            content: m.content.clone(),
            line_content: m.content,
        }).collect())
    }

    /// Collect all searchable files in the directory
    fn collect_files(&self, search_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut walker = WalkDir::new(search_dir)
            .follow_links(self.follow_links);
        
        if let Some(depth) = self.max_depth {
            walker = walker.max_depth(depth);
        }

        let files: Vec<PathBuf> = walker
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                let path = entry.path();
                
                // Skip directories
                if !path.is_file() {
                    return false;
                }
                
                // Skip hidden files if configured
                if self.ignore_hidden && self.is_hidden(path) {
                    return false;
                }
                
                // Skip binary files and common non-text files
                if !self.is_text_file(path) {
                    return false;
                }
                
                true
            })
            .map(|entry| entry.path().to_path_buf())
            .collect();

        Ok(files)
    }

    /// Search within a single file
    fn search_file(&self, regex: &Regex, file_path: &Path) -> Result<Vec<SearchMatch>> {
        let content = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        let mut matches = Vec::new();
        
        for (line_number, line) in content.lines().enumerate() {
            if let Some(mat) = regex.find(line) {
                matches.push(SearchMatch {
                    file_path: file_path.to_path_buf(),
                    line_number: line_number + 1, // 1-based line numbers
                    column: mat.start() + 1, // 1-based column numbers
                    content: line.to_string(),
                    matched_text: mat.as_str().to_string(),
                });
            }
        }

        Ok(matches)
    }

    /// Check if a path represents a hidden file/directory
    fn is_hidden(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with('.'))
            .unwrap_or(false)
    }

    /// Check if a file is likely to be a text file
    fn is_text_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            matches!(extension.to_lowercase().as_str(),
                "txt" | "md" | "rs" | "py" | "js" | "ts" | "java" | "cpp" | "c" | "h" |
                "hpp" | "cs" | "rb" | "go" | "php" | "html" | "css" | "xml" | "json" |
                "yml" | "yaml" | "toml" | "cfg" | "conf" | "ini" | "sh" | "bat" |
                "dockerfile" | "makefile" | "cmake" | "sql" | "r" | "scala" | "kt" |
                "swift" | "dart" | "vue" | "jsx" | "tsx" | "sass" | "scss" | "less"
            )
        } else {
            // Files without extension - check if they have common names
            if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                matches!(name.to_lowercase().as_str(),
                    "makefile" | "dockerfile" | "readme" | "license" | "changelog" |
                    "authors" | "contributors" | "copying" | "install" | "news" | "todo"
                )
            } else {
                false
            }
        }
    }
}

impl Default for NativeSearcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_basic_search() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello world\nThis is a test\nAnother line").unwrap();

        let searcher = NativeSearcher::new();
        let matches = searcher.search("test", temp_dir.path()).unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].line_number, 2);
        assert_eq!(matches[0].matched_text, "test");
    }

    #[test]
    fn test_case_sensitive_search() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello World\nHELLO world\nhello WORLD").unwrap();

        let searcher = NativeSearcher::new().case_sensitive(true);
        let matches = searcher.search("Hello", temp_dir.path()).unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].line_number, 1);
    }

    #[test]
    fn test_case_insensitive_search() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello World\nHELLO world\nhello WORLD").unwrap();

        let searcher = NativeSearcher::new().case_sensitive(false);
        let matches = searcher.search("hello", temp_dir.path()).unwrap();

        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_regex_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test123\ntest456\nabc789").unwrap();

        let searcher = NativeSearcher::new();
        let matches = searcher.search(r"test\d+", temp_dir.path()).unwrap();

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].matched_text, "test123");
        assert_eq!(matches[1].matched_text, "test456");
    }

    #[test]
    fn test_hidden_files_ignored() {
        let temp_dir = TempDir::new().unwrap();
        let visible_file = temp_dir.path().join("visible.txt");
        let hidden_file = temp_dir.path().join(".hidden.txt");
        
        fs::write(&visible_file, "visible content").unwrap();
        fs::write(&hidden_file, "hidden content").unwrap();

        let searcher = NativeSearcher::new().ignore_hidden(true);
        let matches = searcher.search("content", temp_dir.path()).unwrap();

        assert_eq!(matches.len(), 1);
        assert!(matches[0].file_path.ends_with("visible.txt"));
    }
}