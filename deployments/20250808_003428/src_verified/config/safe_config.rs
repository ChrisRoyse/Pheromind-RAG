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
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output: String,
}

// PRINCIPLE 0 ENFORCEMENT: No Default implementation allowed
// Configuration MUST be explicitly provided - no fallback behavior permitted

impl Config {
    /// TEST-ONLY: Create explicit test configuration - NO DEFAULT BEHAVIOR
    /// This function provides explicit values for all configuration fields
    /// MUST NOT be used in production code
    #[cfg(test)]
    fn new_explicit_test_config() -> Self {
        Self {
            storage: StorageConfig {
                backend: StorageBackend::Memory,
                path: PathBuf::from("./test_data"),
                max_connections: 10,
                connection_timeout_ms: 5000,
                cache_size: 10000,
            },
            embedding: EmbeddingConfig {
                model_path: PathBuf::from("./test_models/nomic-embed-text-v1.5.gguf"),
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
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                output: "stdout".to_string(),
            },
        }
    }

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
        
        
        Ok(())
    }
    
    /// Load configuration from environment variables - requires ALL variables to be set
    pub fn from_env() -> Result<Self> {
        // Require ALL environment variables - no fallbacks or defaults
        let storage_backend = std::env::var("EMBED_STORAGE_BACKEND")
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_STORAGE_BACKEND environment variable is required".to_string(),
                source: None,
            })?;
        
        let storage_path = std::env::var("EMBED_STORAGE_PATH")
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_STORAGE_PATH environment variable is required".to_string(),
                source: None,
            })?;
            
        let max_connections = std::env::var("EMBED_MAX_CONNECTIONS")
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_MAX_CONNECTIONS environment variable is required".to_string(),
                source: None,
            })?
            .parse()
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_MAX_CONNECTIONS must be a valid positive integer".to_string(),
                source: None,
            })?;
            
        let connection_timeout_ms = std::env::var("EMBED_CONNECTION_TIMEOUT_MS")
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_CONNECTION_TIMEOUT_MS environment variable is required".to_string(),
                source: None,
            })?
            .parse()
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_CONNECTION_TIMEOUT_MS must be a valid positive integer".to_string(),
                source: None,
            })?;
            
        let cache_size = std::env::var("EMBED_CACHE_SIZE")
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_CACHE_SIZE environment variable is required".to_string(),
                source: None,
            })?
            .parse()
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_CACHE_SIZE must be a valid positive integer".to_string(),
                source: None,
            })?;
        
        let model_path = std::env::var("EMBED_MODEL_PATH")
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_MODEL_PATH environment variable is required".to_string(),
                source: None,
            })?;
            
        let model_type = std::env::var("EMBED_MODEL_TYPE")
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_MODEL_TYPE environment variable is required".to_string(),
                source: None,
            })?;
            
        let dimension = std::env::var("EMBED_DIMENSION")
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_DIMENSION environment variable is required".to_string(),
                source: None,
            })?
            .parse()
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_DIMENSION must be a valid positive integer".to_string(),
                source: None,
            })?;
            
        let batch_size = std::env::var("EMBED_BATCH_SIZE")
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_BATCH_SIZE environment variable is required".to_string(),
                source: None,
            })?
            .parse()
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_BATCH_SIZE must be a valid positive integer".to_string(),
                source: None,
            })?;
            
        let max_sequence_length = std::env::var("EMBED_MAX_SEQUENCE_LENGTH")
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_MAX_SEQUENCE_LENGTH environment variable is required".to_string(),
                source: None,
            })?
            .parse()
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_MAX_SEQUENCE_LENGTH must be a valid positive integer".to_string(),
                source: None,
            })?;
            
        let cache_embeddings = std::env::var("EMBED_CACHE_EMBEDDINGS")
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_CACHE_EMBEDDINGS environment variable is required".to_string(),
                source: None,
            })?
            .parse()
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_CACHE_EMBEDDINGS must be 'true' or 'false'".to_string(),
                source: None,
            })?;
            
        let index_type_str = std::env::var("EMBED_INDEX_TYPE")
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_INDEX_TYPE environment variable is required".to_string(),
                source: None,
            })?;
            
        let index_type = match index_type_str.to_lowercase().as_str() {
            "flat" => IndexType::Flat,
            "ivf" => IndexType::IVF,
            "hnsw" => IndexType::HNSW,
            _ => {
                return Err(EmbedError::Configuration {
                    message: format!("Invalid index type '{}'. Valid options are: flat, ivf, hnsw", index_type_str),
                    source: None,
                });
            }
        };
        
        let top_k_default = std::env::var("EMBED_TOP_K_DEFAULT")
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_TOP_K_DEFAULT environment variable is required".to_string(),
                source: None,
            })?
            .parse()
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_TOP_K_DEFAULT must be a valid positive integer".to_string(),
                source: None,
            })?;
            
        let similarity_threshold = std::env::var("EMBED_SIMILARITY_THRESHOLD")
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_SIMILARITY_THRESHOLD environment variable is required".to_string(),
                source: None,
            })?
            .parse()
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_SIMILARITY_THRESHOLD must be a valid float between 0.0 and 1.0".to_string(),
                source: None,
            })?;
            
        let enable_reranking = std::env::var("EMBED_ENABLE_RERANKING")
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_ENABLE_RERANKING environment variable is required".to_string(),
                source: None,
            })?
            .parse()
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_ENABLE_RERANKING must be 'true' or 'false'".to_string(),
                source: None,
            })?;
            
            
        let level = std::env::var("EMBED_LOG_LEVEL")
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_LOG_LEVEL environment variable is required".to_string(),
                source: None,
            })?;
            
        let format = std::env::var("EMBED_LOG_FORMAT")
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_LOG_FORMAT environment variable is required".to_string(),
                source: None,
            })?;
            
        let output = std::env::var("EMBED_LOG_OUTPUT")
            .map_err(|_| EmbedError::Configuration {
                message: "EMBED_LOG_OUTPUT environment variable is required".to_string(),
                source: None,
            })?;
        
        let config = Self {
            storage: StorageConfig {
                backend: match storage_backend.to_lowercase().as_str() {
                    "memory" => StorageBackend::Memory,
                    "lancedb" => StorageBackend::LanceDB,
                    "sqlite" => StorageBackend::SQLite,
                    "postgresql" => StorageBackend::PostgreSQL,
                    _ => {
                        return Err(EmbedError::Configuration {
                            message: format!("Invalid storage backend '{}'. Valid options are: memory, lancedb, sqlite, postgresql", storage_backend),
                            source: None,
                        });
                    }
                },
                path: PathBuf::from(storage_path),
                max_connections,
                connection_timeout_ms,
                cache_size,
            },
            embedding: EmbeddingConfig {
                model_path: PathBuf::from(model_path),
                model_type,
                dimension,
                batch_size,
                max_sequence_length,
                cache_embeddings,
            },
            search: SearchConfig {
                index_type,
                top_k_default,
                similarity_threshold,
                enable_reranking,
            },
            logging: LoggingConfig {
                level,
                format,
                output,
            },
        };
        
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
    
    /// Load configuration with proper error handling - no fallbacks
    pub fn load_or_fail<P: AsRef<Path>>(path: P) -> Result<Arc<Config>> {
        let config = Self::load(path)?;
        Self::init(config)?;
        Self::get()
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
    fn test_explicit_config() {
        let config = Config::new_explicit_test_config();
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


[logging]
level = "debug"
format = "text"
output = "stderr"
"#).unwrap();
        
        file.flush().unwrap();
        
        let config = Config::load_from_file(file.path()).unwrap();
        assert_eq!(config.embedding.batch_size, 16);
        assert!(matches!(config.storage.backend, StorageBackend::LanceDB));
    }
    
    #[test]
    fn test_invalid_config_validation() {
        let mut config = Config::new_explicit_test_config();
        config.embedding.dimension = 0;
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("dimension"));
    }
    
    #[test]
    fn test_config_manager() {
        let config = Config::new_explicit_test_config();
        ConfigManager::init(config).unwrap();
        
        let retrieved = ConfigManager::get().unwrap();
        assert!(retrieved.storage.max_connections > 0);
    }
    
    #[test]
    fn test_env_config() {
        // Set ALL required environment variables
        std::env::set_var("EMBED_STORAGE_BACKEND", "sqlite");
        std::env::set_var("EMBED_STORAGE_PATH", "./test_data");
        std::env::set_var("EMBED_MAX_CONNECTIONS", "5");
        std::env::set_var("EMBED_CONNECTION_TIMEOUT_MS", "3000");
        std::env::set_var("EMBED_CACHE_SIZE", "5000");
        std::env::set_var("EMBED_MODEL_PATH", "./test_model.gguf");
        std::env::set_var("EMBED_MODEL_TYPE", "nomic");
        std::env::set_var("EMBED_DIMENSION", "768");
        std::env::set_var("EMBED_BATCH_SIZE", "16");
        std::env::set_var("EMBED_MAX_SEQUENCE_LENGTH", "1024");
        std::env::set_var("EMBED_CACHE_EMBEDDINGS", "true");
        std::env::set_var("EMBED_INDEX_TYPE", "flat");
        std::env::set_var("EMBED_TOP_K_DEFAULT", "20");
        std::env::set_var("EMBED_SIMILARITY_THRESHOLD", "0.8");
        std::env::set_var("EMBED_ENABLE_RERANKING", "false");
        std::env::set_var("EMBED_LOG_LEVEL", "debug");
        std::env::set_var("EMBED_LOG_FORMAT", "text");
        std::env::set_var("EMBED_LOG_OUTPUT", "stderr");
        
        let config = Config::from_env().unwrap();
        assert!(matches!(config.storage.backend, StorageBackend::SQLite));
        assert_eq!(config.embedding.batch_size, 16);
        assert_eq!(config.search.top_k_default, 20);
        
        // Clean up ALL environment variables
        std::env::remove_var("EMBED_STORAGE_BACKEND");
        std::env::remove_var("EMBED_STORAGE_PATH");
        std::env::remove_var("EMBED_MAX_CONNECTIONS");
        std::env::remove_var("EMBED_CONNECTION_TIMEOUT_MS");
        std::env::remove_var("EMBED_CACHE_SIZE");
        std::env::remove_var("EMBED_MODEL_PATH");
        std::env::remove_var("EMBED_MODEL_TYPE");
        std::env::remove_var("EMBED_DIMENSION");
        std::env::remove_var("EMBED_BATCH_SIZE");
        std::env::remove_var("EMBED_MAX_SEQUENCE_LENGTH");
        std::env::remove_var("EMBED_CACHE_EMBEDDINGS");
        std::env::remove_var("EMBED_INDEX_TYPE");
        std::env::remove_var("EMBED_TOP_K_DEFAULT");
        std::env::remove_var("EMBED_SIMILARITY_THRESHOLD");
        std::env::remove_var("EMBED_ENABLE_RERANKING");
        std::env::remove_var("EMBED_LOG_LEVEL");
        std::env::remove_var("EMBED_LOG_FORMAT");
        std::env::remove_var("EMBED_LOG_OUTPUT");
    }
}