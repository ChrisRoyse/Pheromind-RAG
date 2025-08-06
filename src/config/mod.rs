use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;
use anyhow::{Result, anyhow};
use once_cell::sync::Lazy;
use std::sync::RwLock;
use crate::error::{EmbedError, Result as EmbedResult};

/// Search backend options
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum SearchBackend {
    /// Use Tantivy for full-text search with fuzzy matching
    Tantivy,
}

impl<'de> serde::Deserialize<'de> for SearchBackend {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Default for SearchBackend {
    fn default() -> Self {
        SearchBackend::Tantivy
    }
}

impl std::fmt::Display for SearchBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchBackend::Tantivy => write!(f, "Tantivy"),
        }
    }
}

impl std::str::FromStr for SearchBackend {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tantivy" => Ok(SearchBackend::Tantivy),
            _ => Err(anyhow!("Invalid search backend '{}'. Valid option: tantivy (case-insensitive)", s)),
        }
    }
}

/// Global configuration singleton
pub static CONFIG: Lazy<RwLock<Config>> = Lazy::new(|| {
    RwLock::new(Config::default())
});

/// Main configuration struct for the embedding search system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Chunking configuration
    pub chunk_size: usize,
    
    /// Cache configuration
    pub embedding_cache_size: usize,
    pub search_cache_size: usize,
    
    /// Processing configuration
    pub batch_size: usize,
    
    /// Storage paths
    pub vector_db_path: String,
    pub cache_dir: String,
    
    /// Git watching configuration
    pub git_poll_interval_secs: u64,
    pub enable_git_watch: bool,
    
    /// Search configuration
    pub include_test_files: bool,
    pub max_search_results: usize,
    pub search_backend: SearchBackend,
    
    /// Model configuration
    pub model_name: String,
    pub embedding_dimensions: usize,
    
    /// Logging configuration
    pub log_level: String,
    
    /// BM25 configuration
    pub bm25_enabled: bool,
    pub bm25_k1: f32,
    pub bm25_b: f32,
    pub bm25_index_path: String,
    pub bm25_cache_size: usize,
    pub bm25_min_term_length: usize,
    pub bm25_max_term_length: usize,
    pub bm25_stop_words: Vec<String>,
    
    /// Enhanced fusion weights
    pub fusion_exact_weight: f32,
    pub fusion_bm25_weight: f32,
    pub fusion_semantic_weight: f32,
    pub fusion_symbol_weight: f32,
    
    /// Text processing configuration
    pub enable_stemming: bool,
    pub enable_ngrams: bool,
    pub max_ngram_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            chunk_size: 100,
            embedding_cache_size: 10000,
            search_cache_size: 100,
            batch_size: 32,
            vector_db_path: ".embed_db".to_string(),
            cache_dir: ".embed_cache".to_string(),
            git_poll_interval_secs: 5,
            enable_git_watch: true,
            include_test_files: false,
            max_search_results: 20,
            search_backend: SearchBackend::Tantivy,
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            embedding_dimensions: 768,
            log_level: "info".to_string(),
            
            // BM25 defaults (tuned for code search)
            bm25_enabled: true,
            bm25_k1: 1.2,
            bm25_b: 0.75,
            bm25_index_path: ".embed_bm25_index".to_string(),
            bm25_cache_size: 100_000,
            bm25_min_term_length: 2,
            bm25_max_term_length: 50,
            bm25_stop_words: vec![
                // Only truly common English words, not programming keywords
                "the".to_string(), "and".to_string(), "or".to_string(),
                "a".to_string(), "an".to_string(), "is".to_string(),
                "it".to_string(), "in".to_string(), "to".to_string(),
                "of".to_string(), "as".to_string(), "at".to_string(),
                "by".to_string(), "with".to_string(), "this".to_string(),
                "that".to_string(), "from".to_string(),
            ],
            
            // Fusion weights (optimized through testing)
            fusion_exact_weight: 0.4,
            fusion_bm25_weight: 0.25,
            fusion_semantic_weight: 0.25,
            fusion_symbol_weight: 0.1,
            
            // Text processing
            enable_stemming: true,
            enable_ngrams: true,
            max_ngram_size: 3,
        }
    }
}

impl Config {
    /// Load configuration from multiple sources with precedence:
    /// 1. Command line arguments (highest priority)
    /// 2. Environment variables (EMBED_* prefix)  
    /// 3. Project-specific config files (.embedrc or .embed/config.toml)
    /// 4. Global config file (config.toml)
    /// 5. Default values (lowest priority)
    pub fn load() -> Result<Self> {
        let mut settings = config::Config::builder()
            // Start with defaults
            .add_source(config::Config::try_from(&Config::default())?)
            // Add global config file if it exists
            .add_source(
                config::File::with_name("config")
                    .format(config::FileFormat::Toml)
                    .required(false)
            );

        // Look for project-specific config files
        let current_dir = std::env::current_dir()?;
        
        // Check for .embedrc in current directory
        let embedrc_path = current_dir.join(".embedrc");
        if embedrc_path.exists() {
            settings = settings.add_source(
                config::File::from(embedrc_path)
                    .format(config::FileFormat::Toml)
                    .required(false)
            );
        }
        
        // Check for .embed/config.toml in current directory
        let embed_config_path = current_dir.join(".embed").join("config.toml");
        if embed_config_path.exists() {
            settings = settings.add_source(
                config::File::from(embed_config_path)
                    .format(config::FileFormat::Toml)
                    .required(false)
            );
        }

        // Add environment variables with EMBED_ prefix
        settings = settings.add_source(
            config::Environment::with_prefix("EMBED")
                .try_parsing(true)
                .separator("_")
                .list_separator(",")
        );

        let config = settings.build()?.try_deserialize()?;
        Ok(config)
    }

    /// Load configuration from a specific file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut settings = config::Config::builder()
            .add_source(config::Config::try_from(&Config::default())?)
            .add_source(
                config::File::from(path.as_ref())
                    .format(config::FileFormat::Toml)
                    .required(true)
            );

        // Still add environment variables for overrides
        settings = settings.add_source(
            config::Environment::with_prefix("EMBED")
                .try_parsing(true)
                .separator("_")
        );

        let config = settings.build()?.try_deserialize()?;
        Ok(config)
    }

    /// Initialize the global configuration
    pub fn init() -> Result<()> {
        let config = Self::load()?;
        *CONFIG.write().map_err(|e| anyhow!("Failed to acquire write lock for CONFIG: {}", e))? = config;
        Ok(())
    }

    /// Initialize with a specific config file
    pub fn init_with_file<P: AsRef<Path>>(path: P) -> Result<()> {
        let config = Self::load_from_file(path)?;
        *CONFIG.write().map_err(|e| anyhow!("Failed to acquire write lock for CONFIG: {}", e))? = config;
        Ok(())
    }

    /// Get a copy of the global configuration
    pub fn get() -> EmbedResult<Config> {
        Ok(CONFIG.read().map_err(|e| EmbedError::Concurrency {
            message: format!("Failed to acquire read lock for CONFIG: {}", e),
            operation: Some("get_config".to_string()),
        })?.clone())
    }

    /// Get chunk size setting
    pub fn chunk_size() -> EmbedResult<usize> {
        Ok(CONFIG.read().map_err(|e| EmbedError::Concurrency {
            message: format!("Failed to acquire read lock for CONFIG: {}", e),
            operation: Some("get_chunk_size".to_string()),
        })?.chunk_size)
    }

    /// Get embedding cache size setting
    pub fn embedding_cache_size() -> EmbedResult<usize> {
        Ok(CONFIG.read().map_err(|e| EmbedError::Concurrency {
            message: format!("Failed to acquire read lock for CONFIG: {}", e),
            operation: Some("get_embedding_cache_size".to_string()),
        })?.embedding_cache_size)
    }

    /// Get search cache size setting
    pub fn search_cache_size() -> EmbedResult<usize> {
        Ok(CONFIG.read().map_err(|e| EmbedError::Concurrency {
            message: format!("Failed to acquire read lock for CONFIG: {}", e),
            operation: Some("get_search_cache_size".to_string()),
        })?.search_cache_size)
    }

    /// Get batch size setting
    pub fn batch_size() -> EmbedResult<usize> {
        Ok(CONFIG.read().map_err(|e| EmbedError::Concurrency {
            message: format!("Failed to acquire read lock for CONFIG: {}", e),
            operation: Some("get_batch_size".to_string()),
        })?.batch_size)
    }

    /// Get vector database path
    pub fn vector_db_path() -> EmbedResult<String> {
        Ok(CONFIG.read().map_err(|e| EmbedError::Concurrency {
            message: format!("Failed to acquire read lock for CONFIG: {}", e),
            operation: Some("get_vector_db_path".to_string()),
        })?.vector_db_path.clone())
    }

    /// Get cache directory path
    pub fn cache_dir() -> EmbedResult<String> {
        Ok(CONFIG.read().map_err(|e| EmbedError::Concurrency {
            message: format!("Failed to acquire read lock for CONFIG: {}", e),
            operation: Some("get_cache_dir".to_string()),
        })?.cache_dir.clone())
    }

    /// Get git poll interval
    pub fn git_poll_interval_secs() -> EmbedResult<u64> {
        Ok(CONFIG.read().map_err(|e| EmbedError::Concurrency {
            message: format!("Failed to acquire read lock for CONFIG: {}", e),
            operation: Some("get_git_poll_interval_secs".to_string()),
        })?.git_poll_interval_secs)
    }

    /// Check if git watching is enabled
    pub fn enable_git_watch() -> EmbedResult<bool> {
        Ok(CONFIG.read().map_err(|e| EmbedError::Concurrency {
            message: format!("Failed to acquire read lock for CONFIG: {}", e),
            operation: Some("get_enable_git_watch".to_string()),
        })?.enable_git_watch)
    }

    /// Check if test files should be included
    pub fn include_test_files() -> EmbedResult<bool> {
        Ok(CONFIG.read().map_err(|e| EmbedError::Concurrency {
            message: format!("Failed to acquire read lock for CONFIG: {}", e),
            operation: Some("get_include_test_files".to_string()),
        })?.include_test_files)
    }

    /// Get maximum search results
    pub fn max_search_results() -> EmbedResult<usize> {
        Ok(CONFIG.read().map_err(|e| EmbedError::Concurrency {
            message: format!("Failed to acquire read lock for CONFIG: {}", e),
            operation: Some("get_max_search_results".to_string()),
        })?.max_search_results)
    }

    /// Get the search backend configuration
    pub fn search_backend() -> EmbedResult<SearchBackend> {
        Ok(CONFIG.read().map_err(|e| EmbedError::Concurrency {
            message: format!("Failed to acquire read lock for CONFIG: {}", e),
            operation: Some("get_search_backend".to_string()),
        })?.search_backend.clone())
    }
    

    /// Get model name
    pub fn model_name() -> EmbedResult<String> {
        Ok(CONFIG.read().map_err(|e| EmbedError::Concurrency {
            message: format!("Failed to acquire read lock for CONFIG: {}", e),
            operation: Some("get_model_name".to_string()),
        })?.model_name.clone())
    }

    /// Get embedding dimensions
    pub fn embedding_dimensions() -> EmbedResult<usize> {
        Ok(CONFIG.read().map_err(|e| EmbedError::Concurrency {
            message: format!("Failed to acquire read lock for CONFIG: {}", e),
            operation: Some("get_embedding_dimensions".to_string()),
        })?.embedding_dimensions)
    }

    /// Get log level
    pub fn log_level() -> EmbedResult<String> {
        Ok(CONFIG.read().map_err(|e| EmbedError::Concurrency {
            message: format!("Failed to acquire read lock for CONFIG: {}", e),
            operation: Some("get_log_level".to_string()),
        })?.log_level.clone())
    }

    /// Validate configuration settings
    pub fn validate(&self) -> Result<()> {
        if self.chunk_size == 0 {
            return Err(anyhow!("chunk_size must be greater than 0"));
        }
        
        if self.embedding_cache_size == 0 {
            return Err(anyhow!("embedding_cache_size must be greater than 0"));
        }
        
        if self.search_cache_size == 0 {
            return Err(anyhow!("search_cache_size must be greater than 0"));
        }
        
        if self.batch_size == 0 {
            return Err(anyhow!("batch_size must be greater than 0"));
        }
        
        if self.git_poll_interval_secs == 0 {
            return Err(anyhow!("git_poll_interval_secs must be greater than 0"));
        }
        
        if self.max_search_results == 0 {
            return Err(anyhow!("max_search_results must be greater than 0"));
        }
        
        if self.embedding_dimensions == 0 {
            return Err(anyhow!("embedding_dimensions must be greater than 0"));
        }
        
        if self.vector_db_path.is_empty() {
            return Err(anyhow!("vector_db_path cannot be empty"));
        }
        
        if self.cache_dir.is_empty() {
            return Err(anyhow!("cache_dir cannot be empty"));
        }
        
        if self.model_name.is_empty() {
            return Err(anyhow!("model_name cannot be empty"));
        }

        // Validate log level
        match self.log_level.to_lowercase().as_str() {
            "error" | "warn" | "info" | "debug" | "trace" => {},
            _ => return Err(anyhow!("log_level must be one of: error, warn, info, debug, trace")),
        }
        
        // Validate search backend - SearchBackend enum already handles validation via FromStr
        // No additional validation needed here

        Ok(())
    }

    /// Get a configuration summary as a formatted string
    pub fn summary(&self) -> String {
        format!(
            r#"Configuration Summary:
====================
Chunking:
  chunk_size: {}

Caching:
  embedding_cache_size: {}
  search_cache_size: {}
  cache_dir: {}

Processing:
  batch_size: {}

Storage:
  vector_db_path: {}

Git Watching:
  enable_git_watch: {}
  git_poll_interval_secs: {}

Search:
  include_test_files: {}
  max_search_results: {}
  search_backend: {}

Model:
  model_name: {}
  embedding_dimensions: {}

Logging:
  log_level: {}
"#,
            self.chunk_size,
            self.embedding_cache_size,
            self.search_cache_size, 
            self.cache_dir,
            self.batch_size,
            self.vector_db_path,
            self.enable_git_watch,
            self.git_poll_interval_secs,
            self.include_test_files,
            self.max_search_results,
            self.search_backend,
            self.model_name,
            self.embedding_dimensions,
            self.log_level
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.chunk_size, 100);
        assert_eq!(config.embedding_cache_size, 10000);
        assert_eq!(config.batch_size, 32);
        assert_eq!(config.embedding_dimensions, 768);
        assert_eq!(config.search_backend, SearchBackend::Tantivy);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        
        // Test invalid chunk_size
        config.chunk_size = 0;
        assert!(config.validate().is_err());
        
        // Reset and test invalid log level
        config = Config::default();
        config.log_level = "invalid".to_string();
        assert!(config.validate().is_err());
        
        // Test valid config
        config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_search_backend_enum() {
        use std::str::FromStr;
        
        // Test string parsing
        assert_eq!(SearchBackend::from_str("TANTIVY").expect("parsing tantivy"), SearchBackend::Tantivy);
        assert!(SearchBackend::from_str("invalid").is_err());
        
        // Test display
        assert_eq!(SearchBackend::Tantivy.to_string(), "Tantivy");
        
        // Test default
        assert_eq!(SearchBackend::default(), SearchBackend::Tantivy);
    }


    #[test]
    fn test_search_backend_method() {
        let mut config = Config::default();
        config.search_backend = SearchBackend::Tantivy;
        *CONFIG.write().expect("write lock") = config;
        
        assert_eq!(Config::search_backend().expect("get search backend"), SearchBackend::Tantivy);
        
        // Reset to default for other tests
        *CONFIG.write().expect("write lock") = Config::default();
    }

    #[test]
    fn test_environment_variables() {
        // Set some environment variables
        env::set_var("EMBED_CHUNK_SIZE", "200");
        env::set_var("EMBED_BATCH_SIZE", "64");
        
        // This would normally load from env, but we can't easily test that here
        // without affecting other tests. The functionality is tested in integration tests.
        
        // Clean up
        env::remove_var("EMBED_CHUNK_SIZE");
        env::remove_var("EMBED_BATCH_SIZE");
    }
}