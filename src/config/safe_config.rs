// Safe Configuration Management - Phase 1: Foundation & Safety
// This module provides safe configuration loading without unwrap() calls

use std::path::{Path, PathBuf};
use std::fs;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

use crate::error::{EmbedError, Result, SafeUnwrap, ErrorContext};

/// Global configuration instance with thread-safe access
static GLOBAL_CONFIG: Lazy<Arc<RwLock<Option<Arc<Config>>>>> = 
    Lazy::new(|| Arc::new(RwLock::new(None)));

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub storage: StorageConfig,
    pub embedding: EmbeddingConfig,
    pub search: SearchConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub backend: StorageBackend,
    pub path: PathBuf,
    pub max_connections: usize,
    pub connection_timeout_ms: u64,
    pub cache_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Memory,
    LanceDB,
    SQLite,
    PostgreSQL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model_path: PathBuf,
    pub model_type: String,
    pub dimension: usize,
    pub batch_size: usize,
    pub max_sequence_length: usize,
    pub cache_embeddings: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub index_type: IndexType,
    pub top_k_default: usize,
    pub similarity_threshold: f32,
    pub enable_reranking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IndexType {
    Flat,
    IVF,
    HNSW,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub max_request_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage: StorageConfig {
                backend: StorageBackend::Memory,
                path: PathBuf::from("./data"),
                max_connections: 10,
                connection_timeout_ms: 5000,
                cache_size: 10000,
            },
            embedding: EmbeddingConfig {
                model_path: PathBuf::from("./models/nomic-embed-text-v1.5.gguf"),
                model_type: "nomic".to_string(),
                dimension: 768,
                batch_size: 32,
                max_sequence_length: 2048,
                cache_embeddings: true,
            },
            search: SearchConfig {
                index_type: IndexType::Flat,
                top_k_default: 10,
                similarity_threshold: 0.7,
                enable_reranking: false,
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                workers: 4,
                max_request_size: 10 * 1024 * 1024,  // 10MB
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                output: "stdout".to_string(),
            },
        }
    }
}

impl Config {
    /// Load configuration from file with proper error handling
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        // Check if file exists
        if !path.exists() {
            return Err(EmbedError::Configuration {
                message: format!("Configuration file not found: {}", path.display()),
                source: None,
            });
        }
        
        // Read file contents
        let contents = fs::read_to_string(path)
            .map_err(|e| EmbedError::Configuration {
                message: format!("Failed to read configuration file: {}", path.display()),
                source: Some(Box::new(e)),
            })?;
        
        // Determine format based on extension
        let config = match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => Self::parse_toml(&contents)?,
            Some("json") => Self::parse_json(&contents)?,
            Some("yaml") | Some("yml") => Self::parse_yaml(&contents)?,
            _ => {
                return Err(EmbedError::Configuration {
                    message: format!(
                        "Unsupported configuration format: {}",
                        path.display()
                    ),
                    source: None,
                });
            }
        };
        
        // Validate configuration
        config.validate()?;
        
        Ok(config)
    }
    
    /// Parse TOML configuration
    fn parse_toml(contents: &str) -> Result<Self> {
        toml::from_str(contents)
            .map_err(|e| EmbedError::Configuration {
                message: "Failed to parse TOML configuration".to_string(),
                source: Some(Box::new(e)),
            })
    }
    
    /// Parse JSON configuration
    fn parse_json(contents: &str) -> Result<Self> {
        serde_json::from_str(contents)
            .map_err(|e| EmbedError::Configuration {
                message: "Failed to parse JSON configuration".to_string(),
                source: Some(Box::new(e)),
            })
    }
    
    /// Parse YAML configuration
    fn parse_yaml(contents: &str) -> Result<Self> {
        serde_yaml::from_str(contents)
            .map_err(|e| EmbedError::Configuration {
                message: "Failed to parse YAML configuration".to_string(),
                source: Some(Box::new(e)),
            })
    }
    
    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        // Validate storage configuration
        if self.storage.max_connections == 0 {
            return Err(EmbedError::Validation {
                field: "storage.max_connections".to_string(),
                reason: "Must be greater than 0".to_string(),
                value: Some("0".to_string()),
            });
        }
        
        // Validate embedding configuration
        if self.embedding.dimension == 0 {
            return Err(EmbedError::Validation {
                field: "embedding.dimension".to_string(),
                reason: "Must be greater than 0".to_string(),
                value: Some("0".to_string()),
            });
        }
        
        if self.embedding.batch_size == 0 {
            return Err(EmbedError::Validation {
                field: "embedding.batch_size".to_string(),
                reason: "Must be greater than 0".to_string(),
                value: Some("0".to_string()),
            });
        }
        
        // Validate search configuration
        if self.search.similarity_threshold < 0.0 || self.search.similarity_threshold > 1.0 {
            return Err(EmbedError::Validation {
                field: "search.similarity_threshold".to_string(),
                reason: "Must be between 0.0 and 1.0".to_string(),
                value: Some(self.search.similarity_threshold.to_string()),
            });
        }
        
        // Validate server configuration
        if self.server.workers == 0 {
            return Err(EmbedError::Validation {
                field: "server.workers".to_string(),
                reason: "Must be greater than 0".to_string(),
                value: Some("0".to_string()),
            });
        }
        
        Ok(())
    }
    
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();
        
        // Override with environment variables (safe parsing)
        if let Ok(val) = std::env::var("EMBED_STORAGE_BACKEND") {
            config.storage.backend = match val.to_lowercase().as_str() {
                "memory" => StorageBackend::Memory,
                "lancedb" => StorageBackend::LanceDB,
                "sqlite" => StorageBackend::SQLite,
                "postgresql" => StorageBackend::PostgreSQL,
                _ => {
                    return Err(EmbedError::Configuration {
                        message: format!("Invalid storage backend: {}", val),
                        source: None,
                    });
                }
            };
        }
        
        if let Ok(val) = std::env::var("EMBED_SERVER_PORT") {
            config.server.port = val.parse()
                .map_err(|e| EmbedError::Configuration {
                    message: format!("Invalid server port: {}", val),
                    source: Some(Box::new(e)),
                })?;
        }
        
        if let Ok(val) = std::env::var("EMBED_SERVER_WORKERS") {
            config.server.workers = val.parse()
                .map_err(|e| EmbedError::Configuration {
                    message: format!("Invalid worker count: {}", val),
                    source: Some(Box::new(e)),
                })?;
        }
        
        config.validate()?;
        Ok(config)
    }
    
    /// Merge configuration from multiple sources
    pub fn merge(&mut self, other: Config) {
        // This is a simple merge - could be enhanced with more sophisticated logic
        *self = other;
    }
}

/// Configuration manager for global access
pub struct ConfigManager;

impl ConfigManager {
    /// Initialize global configuration
    pub fn init(config: Config) -> Result<()> {
        let mut global = GLOBAL_CONFIG.write();
        *global = Some(Arc::new(config));
        Ok(())
    }
    
    /// Get the global configuration
    pub fn get() -> Result<Arc<Config>> {
        let global = GLOBAL_CONFIG.read();
        global.as_ref()
            .cloned()
            .ok_or_else(|| EmbedError::Configuration {
                message: "Configuration not initialized".to_string(),
                source: None,
            })
    }
    
    /// Load and initialize configuration from file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Arc<Config>> {
        let config = Config::load_from_file(path)?;
        Self::init(config)?;
        Self::get()
    }
    
    /// Load configuration with fallback to defaults
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Arc<Config> {
        match Self::load(path) {
            Ok(config) => config,
            Err(e) => {
                log::warn!("Failed to load configuration, using defaults: {}", e);
                let config = Arc::new(Config::default());
                let _ = Self::init((*config).clone());
                config
            }
        }
    }
    
    /// Reload configuration from file
    pub fn reload<P: AsRef<Path>>(path: P) -> Result<()> {
        let config = Config::load_from_file(path)?;
        Self::init(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_load_toml_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"
[storage]
backend = "lancedb"
path = "./test_data"
max_connections = 20
connection_timeout_ms = 3000
cache_size = 5000

[embedding]
model_path = "./test_model.gguf"
model_type = "nomic"
dimension = 768
batch_size = 16
max_sequence_length = 512
cache_embeddings = true

[search]
index_type = "ivf"
top_k_default = 5
similarity_threshold = 0.8
enable_reranking = true

[server]
host = "0.0.0.0"
port = 9090
workers = 8
max_request_size = 5242880

[logging]
level = "debug"
format = "text"
output = "stderr"
"#).unwrap();
        
        file.flush().unwrap();
        
        let config = Config::load_from_file(file.path()).unwrap();
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.embedding.batch_size, 16);
        assert!(matches!(config.storage.backend, StorageBackend::LanceDB));
    }
    
    #[test]
    fn test_invalid_config_validation() {
        let mut config = Config::default();
        config.embedding.dimension = 0;
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("dimension"));
    }
    
    #[test]
    fn test_config_manager() {
        let config = Config::default();
        ConfigManager::init(config).unwrap();
        
        let retrieved = ConfigManager::get().unwrap();
        assert_eq!(retrieved.server.port, 8080);
    }
    
    #[test]
    fn test_env_config() {
        std::env::set_var("EMBED_SERVER_PORT", "3333");
        std::env::set_var("EMBED_STORAGE_BACKEND", "sqlite");
        
        let config = Config::from_env().unwrap();
        assert_eq!(config.server.port, 3333);
        assert!(matches!(config.storage.backend, StorageBackend::SQLite));
        
        // Clean up
        std::env::remove_var("EMBED_SERVER_PORT");
        std::env::remove_var("EMBED_STORAGE_BACKEND");
    }
}