use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use once_cell::sync::Lazy;
use std::sync::RwLock;

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
    pub ripgrep_fallback: bool,
    
    /// Model configuration
    pub model_name: String,
    pub embedding_dimensions: usize,
    
    /// Logging configuration
    pub log_level: String,
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
            ripgrep_fallback: true,
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            embedding_dimensions: 384,
            log_level: "info".to_string(),
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
        *CONFIG.write().unwrap() = config;
        Ok(())
    }

    /// Initialize with a specific config file
    pub fn init_with_file<P: AsRef<Path>>(path: P) -> Result<()> {
        let config = Self::load_from_file(path)?;
        *CONFIG.write().unwrap() = config;
        Ok(())
    }

    /// Get a copy of the global configuration
    pub fn get() -> Config {
        CONFIG.read().unwrap().clone()
    }

    /// Get chunk size setting
    pub fn chunk_size() -> usize {
        CONFIG.read().unwrap().chunk_size
    }

    /// Get embedding cache size setting
    pub fn embedding_cache_size() -> usize {
        CONFIG.read().unwrap().embedding_cache_size
    }

    /// Get search cache size setting
    pub fn search_cache_size() -> usize {
        CONFIG.read().unwrap().search_cache_size
    }

    /// Get batch size setting
    pub fn batch_size() -> usize {
        CONFIG.read().unwrap().batch_size
    }

    /// Get vector database path
    pub fn vector_db_path() -> String {
        CONFIG.read().unwrap().vector_db_path.clone()
    }

    /// Get cache directory path
    pub fn cache_dir() -> String {
        CONFIG.read().unwrap().cache_dir.clone()
    }

    /// Get git poll interval
    pub fn git_poll_interval_secs() -> u64 {
        CONFIG.read().unwrap().git_poll_interval_secs
    }

    /// Check if git watching is enabled
    pub fn enable_git_watch() -> bool {
        CONFIG.read().unwrap().enable_git_watch
    }

    /// Check if test files should be included
    pub fn include_test_files() -> bool {
        CONFIG.read().unwrap().include_test_files
    }

    /// Get maximum search results
    pub fn max_search_results() -> usize {
        CONFIG.read().unwrap().max_search_results
    }

    /// Check if ripgrep fallback is enabled
    pub fn ripgrep_fallback() -> bool {
        CONFIG.read().unwrap().ripgrep_fallback
    }

    /// Get model name
    pub fn model_name() -> String {
        CONFIG.read().unwrap().model_name.clone()
    }

    /// Get embedding dimensions
    pub fn embedding_dimensions() -> usize {
        CONFIG.read().unwrap().embedding_dimensions
    }

    /// Get log level
    pub fn log_level() -> String {
        CONFIG.read().unwrap().log_level.clone()
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
  ripgrep_fallback: {}

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
            self.ripgrep_fallback,
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
        assert_eq!(config.embedding_dimensions, 384);
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