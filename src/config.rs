// Configuration management - simple but flexible

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub storage: StorageConfig,
    pub search: SearchConfig,
    pub indexing: IndexingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub db_path: PathBuf,
    pub cache_size: usize,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub max_results: usize,
    pub bm25_k1: f32,
    pub bm25_b: f32,
    pub semantic_weight: f32,
    pub keyword_weight: f32,
    pub enable_fuzzy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingConfig {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub max_file_size: usize,
    pub supported_extensions: Vec<String>,
    pub enable_incremental: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage: StorageConfig {
                db_path: PathBuf::from("./embed.db"),
                cache_size: 1000,
                batch_size: 50,
            },
            search: SearchConfig {
                max_results: 20,
                bm25_k1: 1.2,
                bm25_b: 0.75,
                semantic_weight: 0.6,
                keyword_weight: 0.4,
                enable_fuzzy: true,
            },
            indexing: IndexingConfig {
                chunk_size: 512,
                chunk_overlap: 50,
                max_file_size: 10_000_000, // 10MB
                supported_extensions: vec![
                    "rs".to_string(),
                    "py".to_string(),
                    "js".to_string(),
                    "ts".to_string(),
                    "go".to_string(),
                    "java".to_string(),
                    "cpp".to_string(),
                    "c".to_string(),
                    "h".to_string(),
                ],
                enable_incremental: true,
            },
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}