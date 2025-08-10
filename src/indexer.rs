// Incremental indexing with change detection

use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

use crate::config::IndexingConfig;
use crate::chunking::Chunk;
use crate::simple_embedder::NomicEmbedder;
use crate::simple_storage::VectorStorage;
use crate::search::bm25_fixed::BM25Engine;

pub struct IncrementalIndexer {
    config: IndexingConfig,
    indexed_files: HashSet<PathBuf>,
    last_index_time: SystemTime,
}

impl IncrementalIndexer {
    pub fn new(config: IndexingConfig) -> Self {
        Self {
            config,
            indexed_files: HashSet::new(),
            last_index_time: SystemTime::now(),
        }
    }

    /// Index only new or modified files
    pub async fn index_incremental(
        &mut self,
        path: &Path,
        embedder: &mut NomicEmbedder,
        storage: &mut VectorStorage,
        bm25: &mut BM25Engine,
    ) -> Result<usize> {
        let mut indexed_count = 0;
        
        // Collect files to index first to avoid borrowing issues
        let files_to_index: Vec<_> = WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| self.should_index(e.path()))
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
            
            // Process each chunk
            for chunk in chunks {
                // Generate embedding
                let embedding = embedder.embed(&chunk.content)?;
                
                // Store in vector database
                storage.store(
                    vec![chunk.content.clone()],
                    vec![embedding],
                    vec![file_path.display().to_string()],
                ).await?;
                
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
        
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
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
    
    fn create_chunks(&self, content: &str, _path: &Path) -> Result<Vec<Chunk>> {
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
        
        Ok(Self {
            config,
            indexed_files,
            last_index_time,
        })
    }
}