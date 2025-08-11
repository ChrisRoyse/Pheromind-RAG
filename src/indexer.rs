// Incremental indexing with change detection

use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use ignore::WalkBuilder;

use crate::config::IndexingConfig;
use crate::chunking::{Chunk, SimpleRegexChunker, MarkdownRegexChunker};
use crate::gguf_embedder::{GGUFEmbedder, GGUFEmbedderConfig};
use crate::embedding_prefixes::{EmbeddingTask, CodeFormatter};
use crate::simple_storage::VectorStorage;
use crate::search::bm25_fixed::BM25Engine;

pub struct IncrementalIndexer {
    config: IndexingConfig,
    indexed_files: HashSet<PathBuf>,
    last_index_time: SystemTime,
    regex_chunker: SimpleRegexChunker,
    markdown_chunker: MarkdownRegexChunker,
    text_embedder: Option<GGUFEmbedder>,
    code_embedder: Option<GGUFEmbedder>,
}

impl IncrementalIndexer {
    pub fn new(config: IndexingConfig) -> Result<Self> {
        let regex_chunker = SimpleRegexChunker::with_chunk_size(config.chunk_size)?;
        let markdown_chunker = MarkdownRegexChunker::with_options(config.chunk_size, true)?;
        
        Ok(Self {
            config,
            indexed_files: HashSet::new(),
            last_index_time: SystemTime::now(),
            regex_chunker,
            markdown_chunker,
            text_embedder: None,
            code_embedder: None,
        })
    }

    /// Index only new or modified files
    pub fn init_embedders(&mut self) -> Result<()> {
        // Initialize text embedder for markdown files
        let text_config = GGUFEmbedderConfig {
            model_path: "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf".to_string(),
            ..Default::default()
        };
        self.text_embedder = Some(GGUFEmbedder::new(text_config)?);
        
        // Initialize code embedder for all code files
        let code_config = GGUFEmbedderConfig {
            model_path: "./src/model/nomic-embed-code.Q4_K_M.gguf".to_string(),
            ..Default::default()
        };
        self.code_embedder = Some(GGUFEmbedder::new(code_config)?);
        
        Ok(())
    }
    
    fn get_embedder_and_task(&self, file_path: &Path) -> (&GGUFEmbedder, EmbeddingTask) {
        // Determine which embedder and task to use based on file extension
        if let Some(ext) = file_path.extension() {
            if let Some(ext_str) = ext.to_str() {
                match ext_str.to_lowercase().as_str() {
                    "md" | "markdown" => {
                        // Use text embedder for markdown files
                        (self.text_embedder.as_ref().unwrap(), EmbeddingTask::SearchDocument)
                    },
                    "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "go" | "java" | "cpp" | "c" | "h" | "hpp" | "cc" | "cxx" | "cs" | "php" | "rb" | "swift" | "kt" | "scala" | "r" | "sh" | "bash" | "zsh" | "fish" | "ps1" | "psm1" | "lua" | "vim" | "el" => {
                        // Use code embedder for all code files
                        (self.code_embedder.as_ref().unwrap(), EmbeddingTask::CodeDefinition)
                    },
                    _ => {
                        // Default to text embedder for unknown file types
                        (self.text_embedder.as_ref().unwrap(), EmbeddingTask::SearchDocument)
                    }
                }
            } else {
                (self.text_embedder.as_ref().unwrap(), EmbeddingTask::SearchDocument)
            }
        } else {
            (self.text_embedder.as_ref().unwrap(), EmbeddingTask::SearchDocument)
        }
    }

    pub async fn index_incremental(
        &mut self,
        path: &Path,
        storage: &mut VectorStorage,
        bm25: &mut BM25Engine,
    ) -> Result<usize> {
        // Initialize embedders if not already done
        if self.text_embedder.is_none() || self.code_embedder.is_none() {
            self.init_embedders()?;
        }
        let mut indexed_count = 0;
        
        // Use ignore crate to respect .gitignore and other ignore files
        let walker = WalkBuilder::new(path)
            .hidden(false)  // Don't process hidden files by default
            .ignore(true)   // Respect .gitignore files
            .git_ignore(true)  // Respect .gitignore
            .git_global(true)  // Respect global gitignore
            .git_exclude(true) // Respect .git/info/exclude
            .parents(true)     // Respect parent .gitignore files
            .build();
        
        // Collect files to index, respecting gitignore
        let files_to_index: Vec<_> = walker
            .filter_map(|e| e.ok())
            .filter(|e| {
                let path = e.path();
                // Additional filtering for common directories to skip
                if let Some(path_str) = path.to_str() {
                    // Skip common build/dependency directories even if not in gitignore
                    if path_str.contains("/target/") ||
                       path_str.contains("/node_modules/") ||
                       path_str.contains("/.git/") ||
                       path_str.contains("/dist/") ||
                       path_str.contains("/build/") ||
                       path_str.contains("/.cache/") ||
                       path_str.contains("/__pycache__/") {
                        return false;
                    }
                }
                self.should_index(path)
            })
            .collect();
        
        for entry in files_to_index {
            let file_path = entry.path();
            
            // Check if file is new or modified
            if !self.needs_reindex(file_path)? {
                continue;
            }
            
            let content = std::fs::read_to_string(file_path)?;
            
            // Skip files that are too large
            if content.len() > self.config.max_file_size {
                continue;
            }
            
            // Create chunks with overlap for better context
            let chunks = self.create_chunks(&content, file_path)?;
            
            // Process each chunk with appropriate embedder
            for chunk in chunks {
                // Get the appropriate embedder and task based on file type
                let (embedder, task) = self.get_embedder_and_task(file_path);
                
                // For code files, optionally add language context
                let content_to_embed = if task == EmbeddingTask::CodeDefinition {
                    if let Some(lang) = CodeFormatter::detect_language(&file_path.to_string_lossy()) {
                        CodeFormatter::format_code(&chunk.content, lang)
                    } else {
                        chunk.content.clone()
                    }
                } else {
                    chunk.content.clone()
                };
                
                // Generate embedding with appropriate task prefix
                let embedding = embedder.embed(&content_to_embed, task)?;
                
                // Store original content in vector database (not the prefixed version)
                storage.store(
                    vec![chunk.content.clone()],
                    vec![embedding],
                    vec![file_path.display().to_string()],
                )?;
                
                // Index in BM25
                bm25.index_document(
                    &file_path.display().to_string(),
                    &chunk.content,
                );
                // Note: BM25 indexing returns void, no error handling needed
            }
            
            self.indexed_files.insert(file_path.to_path_buf());
            indexed_count += 1;
        }
        
        self.last_index_time = SystemTime::now();
        Ok(indexed_count)
    }
    
    fn should_index(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }
        
        // Skip files that are too large (e.g., generated files, binaries)
        if let Ok(metadata) = path.metadata() {
            if metadata.len() > self.config.max_file_size as u64 {
                return false;
            }
        }
        
        // Check if the file extension is supported
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                // Skip common non-source extensions even if in supported list
                if ext_str == "exe" || ext_str == "dll" || ext_str == "so" || 
                   ext_str == "dylib" || ext_str == "pdb" || ext_str == "lock" ||
                   ext_str == "log" || ext_str == "tmp" || ext_str == "bak" {
                    return false;
                }
                return self.config.supported_extensions.contains(&ext_str.to_string());
            }
        }
        
        false
    }
    
    fn needs_reindex(&self, path: &Path) -> Result<bool> {
        if !self.config.enable_incremental {
            return Ok(true);
        }
        
        if !self.indexed_files.contains(path) {
            return Ok(true);
        }
        
        let metadata = std::fs::metadata(path)?;
        let modified = metadata.modified()?;
        
        Ok(modified > self.last_index_time)
    }
    
    pub fn create_chunks(&self, content: &str, path: &Path) -> Result<Vec<Chunk>> {
        // Check file extension to determine which chunker to use
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                match ext_str.to_lowercase().as_str() {
                    "md" | "markdown" => {
                        // Use markdown-specific chunker
                        let markdown_chunks = self.markdown_chunker.chunk_markdown(content);
                        // Convert MarkdownChunk to Chunk
                        let chunks = markdown_chunks.into_iter().map(|mc| Chunk {
                            content: mc.content,
                            start_line: mc.start_line,
                            end_line: mc.end_line,
                        }).collect();
                        return Ok(chunks);
                    }
                    _ => {
                        // Use regex chunker for other supported files
                        return Ok(self.regex_chunker.chunk_file(content));
                    }
                }
            }
        }
        
        // Fallback to simple line-based chunking if no extension match
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        let mut i = 0;
        while i < lines.len() {
            let end = (i + self.config.chunk_size).min(lines.len());
            let chunk_lines = &lines[i..end];
            
            let chunk = Chunk {
                content: chunk_lines.join("\n"),
                start_line: i,
                end_line: end,
            };
            
            chunks.push(chunk);
            
            // Move forward with overlap
            i += self.config.chunk_size - self.config.chunk_overlap;
        }
        
        Ok(chunks)
    }
    
    /// Save index state for persistence
    pub fn save_state(&self, path: &Path) -> Result<()> {
        let state = serde_json::json!({
            "indexed_files": self.indexed_files.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            "last_index_time": self.last_index_time.duration_since(SystemTime::UNIX_EPOCH)?.as_secs(),
        });
        
        std::fs::write(path, serde_json::to_string_pretty(&state)?)?;
        Ok(())
    }
    
    /// Load index state from disk
    pub fn load_state(path: &Path, config: IndexingConfig) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let state: serde_json::Value = serde_json::from_str(&content)?;
        
        let indexed_files = state["indexed_files"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str())
            .map(PathBuf::from)
            .collect();
        
        let last_index_secs = state["last_index_time"].as_u64().unwrap_or(0);
        let last_index_time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(last_index_secs);
        
        let regex_chunker = SimpleRegexChunker::with_chunk_size(config.chunk_size)?;
        let markdown_chunker = MarkdownRegexChunker::with_options(config.chunk_size, true)?;
        
        Ok(Self {
            config,
            indexed_files,
            last_index_time,
            regex_chunker,
            markdown_chunker,
            text_embedder: None,
            code_embedder: None,
        })
    }
}