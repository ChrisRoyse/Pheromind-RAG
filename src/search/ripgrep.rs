use std::path::Path;
use std::process::Command;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExactMatch {
    pub file_path: String,
    pub line_number: usize,
    pub content: String,
    pub line_content: String,
}

#[derive(Debug, Deserialize)]
struct RgMatch {
    #[serde(rename = "type")]
    type_field: String,
    data: Option<RgMatchData>,
}

#[derive(Debug, Deserialize)]
struct RgMatchData {
    path: RgPath,
    lines: RgLines,
    line_number: usize,
}

#[derive(Debug, Deserialize)]
struct RgPath {
    text: String,
}

#[derive(Debug, Deserialize)]
struct RgLines {
    text: String,
}

pub struct RipgrepSearcher;

impl RipgrepSearcher {
    pub fn new() -> Self {
        Self
    }
    
    pub fn search(&self, query: &str, path: &Path) -> Result<Vec<ExactMatch>> {
        // Build ripgrep command
        let output = Command::new("rg")
            .args(&[
                "--json",
                "--max-count", "100",
                "--no-heading",
                "--with-filename",
                "--line-number",
                query,
                path.to_str().unwrap()
            ])
            .output();
        
        // Handle case where ripgrep is not installed
        let output = match output {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Warning: ripgrep not found, using fallback search: {}", e);
                return self.fallback_search(query, path);
            }
        };
        
        let mut matches = Vec::new();
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            if let Ok(json) = serde_json::from_str::<RgMatch>(line) {
                if json.type_field == "match" {
                    if let Some(data) = json.data {
                        matches.push(ExactMatch {
                            file_path: data.path.text.clone(),
                            line_number: data.line_number,
                            content: data.lines.text.clone(),
                            line_content: data.lines.text.clone(),
                        });
                    }
                }
            }
        }
        
        Ok(matches)
    }
    
    // Fallback implementation when ripgrep is not available
    fn fallback_search(&self, query: &str, path: &Path) -> Result<Vec<ExactMatch>> {
        let mut matches = Vec::new();
        
        // Walk directory recursively
        self.search_in_dir(path, query, &mut matches)?;
        
        // Limit to 100 matches
        matches.truncate(100);
        Ok(matches)
    }
    
    fn search_in_dir(&self, dir: &Path, query: &str, matches: &mut Vec<ExactMatch>) -> Result<()> {
        if dir.is_file() {
            self.search_in_file(dir, query, matches)?;
            return Ok(());
        }
        
        let entries = std::fs::read_dir(dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            // Skip hidden files and directories
            if let Some(name) = path.file_name() {
                if name.to_string_lossy().starts_with('.') {
                    continue;
                }
            }
            
            if path.is_dir() {
                // Skip common non-code directories
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if matches!(name_str.as_ref(), "target" | "node_modules" | ".git" | "dist" | "build") {
                        continue;
                    }
                }
                self.search_in_dir(&path, query, matches)?;
            } else if self.is_code_file(&path) {
                self.search_in_file(&path, query, matches)?;
            }
        }
        
        Ok(())
    }
    
    fn search_in_file(&self, file_path: &Path, query: &str, matches: &mut Vec<ExactMatch>) -> Result<()> {
        let content = std::fs::read_to_string(file_path)?;
        let query_lower = query.to_lowercase();
        
        for (line_num, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&query_lower) {
                matches.push(ExactMatch {
                    file_path: file_path.to_string_lossy().to_string(),
                    line_number: line_num + 1, // 1-indexed
                    content: line.to_string(),
                    line_content: line.to_string(),
                });
                
                // Stop if we have enough matches
                if matches.len() >= 100 {
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    fn is_code_file(&self, path: &Path) -> bool {
        match path.extension().and_then(|s| s.to_str()) {
            Some(ext) => matches!(
                ext,
                "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | 
                "go" | "java" | "cpp" | "c" | "h" | "hpp" |
                "rb" | "php" | "swift" | "kt" | "scala" | "cs" |
                "sql" | "md" | "json" | "yaml" | "yml" | "toml"
            ),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_fallback_search() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        
        let content = r#"
fn main() {
    println!("Hello, world!");
}

fn test_function() {
    let x = 42;
    println!("The answer is {}", x);
}
"#;
        
        fs::write(&test_file, content).unwrap();
        
        let searcher = RipgrepSearcher::new();
        let matches = searcher.fallback_search("println", temp_dir.path()).unwrap();
        
        assert_eq!(matches.len(), 2);
        assert!(matches[0].line_content.contains("println"));
        assert!(matches[1].line_content.contains("println"));
    }
}