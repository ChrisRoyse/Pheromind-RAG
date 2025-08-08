/// MCP-specific configuration that integrates with the existing Config system
/// 
/// This module provides configuration specific to the MCP (Model Context Protocol) server,
/// building on top of the base configuration system. Following the existing pattern,
/// NO DEFAULT VALUES are provided - all configuration must be explicit.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::config::Config;
use crate::error::{EmbedError, Result as EmbedResult};

/// Transport configuration options for MCP server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum McpTransportConfig {
    /// Standard I/O transport (default for MCP)
    Stdio {
        /// Buffer size for stdio operations
        buffer_size: usize,
        /// Enable line buffering
        line_buffering: bool,
    },
    /// TCP transport (future extension)
    Tcp {
        /// Port to bind to
        port: u16,
        /// Host address to bind to
        host: String,
    },
    /// Unix socket transport (future extension)
    #[cfg(unix)]
    UnixSocket {
        /// Path to unix socket
        socket_path: PathBuf,
    },
}

/// Tool-specific configuration for MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolsConfig {
    /// Enable search tool
    pub enable_search: bool,
    /// Enable indexing tool
    pub enable_index: bool,
    /// Enable status/stats tool
    pub enable_status: bool,
    /// Enable clear/reset tool
    pub enable_clear: bool,
    /// Enable orchestrated search tool
    pub enable_orchestrated_search: bool,
    /// Maximum number of search results per tool call
    pub max_results_per_call: usize,
    /// Default search timeout in milliseconds
    pub default_search_timeout_ms: u64,
    /// Maximum concurrent tool operations
    pub max_concurrent_operations: usize,
}

/// Performance and resource limits for MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPerformanceConfig {
    /// Maximum number of concurrent requests
    pub max_concurrent_requests: u32,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Maximum request size in bytes
    pub max_request_size_bytes: usize,
    /// Maximum response size in bytes
    pub max_response_size_bytes: usize,
    /// Enable request metrics collection
    pub enable_metrics: bool,
    /// Metrics collection interval in seconds
    pub metrics_interval_secs: u64,
}

/// LazyEmbedder specific configuration for MCP environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpEmbeddingConfig {
    /// Enable lazy loading of embedder (recommended for MCP)
    pub enable_lazy_loading: bool,
    /// Embedder initialization timeout in milliseconds
    pub init_timeout_ms: u64,
    /// Maximum memory usage for embedder in MB
    pub max_memory_mb: Option<usize>,
    /// Enable embedder health checks
    pub enable_health_checks: bool,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
}

/// Security configuration for MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSecurityConfig {
    /// Enable request validation
    pub enable_request_validation: bool,
    /// Maximum query length
    pub max_query_length: usize,
    /// Allowed file extensions for indexing
    pub allowed_file_extensions: Vec<String>,
    /// Blocked file patterns (regex)
    pub blocked_file_patterns: Vec<String>,
    /// Enable path traversal protection
    pub enable_path_protection: bool,
    /// Maximum indexing depth
    pub max_indexing_depth: usize,
}

/// Complete MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Server identity
    pub server_name: String,
    pub server_version: String,
    pub server_description: String,
    
    /// Transport configuration
    pub transport: McpTransportConfig,
    
    /// Tool configuration
    pub tools: McpToolsConfig,
    
    /// Performance configuration
    pub performance: McpPerformanceConfig,
    
    /// Embedding configuration (only when ml feature is enabled)
    #[cfg(feature = "ml")]
    pub embedding: McpEmbeddingConfig,
    
    /// Security configuration
    pub security: McpSecurityConfig,
    
    /// Logging specific to MCP operations
    pub mcp_log_level: String,
    pub enable_request_logging: bool,
    pub enable_performance_logging: bool,
}

// PRINCIPLE 0 ENFORCEMENT: No Default implementation
// All MCP configuration MUST be explicit - no fallback values allowed

impl McpConfig {
    /// Load MCP configuration from the main configuration system
    /// This integrates with the existing Config::load() pattern
    pub fn from_base_config() -> EmbedResult<Self> {
        let _base_config = Config::get()?;
        
        // Look for MCP-specific configuration in the same places as base config
        let current_dir = std::env::current_dir()
            .map_err(|e| EmbedError::Configuration {
                message: format!("Failed to get current directory: {}", e),
                source: Some(Box::new(e)),
            })?;
        
        let possible_mcp_config_files = vec![
            current_dir.join(".embed").join("mcp-config.toml"),
            current_dir.join(".embed").join("mcp.toml"),
            current_dir.join(".embedrc-mcp"),
            current_dir.join("mcp-config.toml"),
        ];
        
        let mut config_file_found = false;
        let mut settings = config::Config::builder();
        
        for config_path in possible_mcp_config_files {
            if config_path.exists() {
                settings = settings.add_source(
                    config::File::from(config_path.clone())
                        .format(config::FileFormat::Toml)
                        .required(true)
                );
                config_file_found = true;
                break;
            }
        }
        
        if !config_file_found {
            return Err(EmbedError::Configuration {
                message: "No MCP configuration file found. Please create one of: .embed/mcp-config.toml, .embed/mcp.toml, .embedrc-mcp, or mcp-config.toml".to_string(),
                source: None,
            });
        }

        // Add environment variables with MCP prefix
        settings = settings.add_source(
            config::Environment::with_prefix("EMBED_MCP")
                .try_parsing(true)
                .separator("_")
                .list_separator(",")
        );

        let mcp_config = settings.build()
            .map_err(|e| EmbedError::Configuration {
                message: format!("Failed to build MCP configuration: {}", e),
                source: Some(Box::new(e)),
            })?
            .try_deserialize()
            .map_err(|e| EmbedError::Configuration {
                message: format!("Failed to deserialize MCP configuration: {}", e),
                source: Some(Box::new(e)),
            })?;

        Ok(mcp_config)
    }

    /// Load MCP configuration from a specific file
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> EmbedResult<Self> {
        let mut settings = config::Config::builder()
            .add_source(
                config::File::from(path.as_ref())
                    .format(config::FileFormat::Toml)
                    .required(true)
            );

        // Still add environment variables for overrides
        settings = settings.add_source(
            config::Environment::with_prefix("EMBED_MCP")
                .try_parsing(true)
                .separator("_")
        );

        let mcp_config = settings.build()
            .map_err(|e| EmbedError::Configuration {
                message: format!("Failed to build MCP configuration from file: {}", e),
                source: Some(Box::new(e)),
            })?
            .try_deserialize()
            .map_err(|e| EmbedError::Configuration {
                message: format!("Failed to deserialize MCP configuration from file: {}", e),
                source: Some(Box::new(e)),
            })?;

        Ok(mcp_config)
    }

    /// TEST-ONLY: Create a test MCP configuration
    /// This should NEVER be used in production code
    /// Made public for integration examples but should not be used in production
    pub fn new_test_config() -> Self {
        Self {
            server_name: "embed-search-mcp-test".to_string(),
            server_version: "0.1.0-test".to_string(),
            server_description: "Test MCP server for embed-search".to_string(),
            
            transport: McpTransportConfig::Stdio {
                buffer_size: 8192,
                line_buffering: true,
            },
            
            tools: McpToolsConfig {
                enable_search: true,
                enable_index: true,
                enable_status: true,
                enable_clear: true,
                enable_orchestrated_search: true,
                max_results_per_call: 50,
                default_search_timeout_ms: 10000,
                max_concurrent_operations: 10,
            },
            
            performance: McpPerformanceConfig {
                max_concurrent_requests: 50,
                request_timeout_ms: 30000,
                max_request_size_bytes: 1048576, // 1MB
                max_response_size_bytes: 10485760, // 10MB
                enable_metrics: true,
                metrics_interval_secs: 30,
            },
            
            #[cfg(feature = "ml")]
            embedding: McpEmbeddingConfig {
                enable_lazy_loading: true,
                init_timeout_ms: 30000,
                max_memory_mb: Some(512),
                enable_health_checks: true,
                health_check_interval_secs: 300, // 5 minutes
            },
            
            security: McpSecurityConfig {
                enable_request_validation: true,
                max_query_length: 1000,
                allowed_file_extensions: vec![
                    "rs".to_string(), "py".to_string(), "js".to_string(), 
                    "ts".to_string(), "json".to_string(), "toml".to_string(),
                    "yaml".to_string(), "yml".to_string(), "md".to_string(),
                    "txt".to_string(), "csv".to_string(), "sql".to_string(),
                ],
                blocked_file_patterns: vec![
                    r"\.git/.*".to_string(),
                    r".*\.log$".to_string(),
                    r".*\.tmp$".to_string(),
                    r".*\.lock$".to_string(),
                ],
                enable_path_protection: true,
                max_indexing_depth: 10,
            },
            
            mcp_log_level: "info".to_string(),
            enable_request_logging: true,
            enable_performance_logging: false,
        }
    }

    /// Validate MCP configuration settings
    pub fn validate(&self) -> EmbedResult<()> {
        // Validate server identity
        if self.server_name.is_empty() {
            return Err(EmbedError::Configuration {
                message: "server_name cannot be empty".to_string(),
                source: None,
            });
        }
        
        if self.server_version.is_empty() {
            return Err(EmbedError::Configuration {
                message: "server_version cannot be empty".to_string(),
                source: None,
            });
        }

        // Validate transport config
        match &self.transport {
            McpTransportConfig::Stdio { buffer_size, .. } => {
                if *buffer_size == 0 {
                    return Err(EmbedError::Configuration {
                        message: "transport.buffer_size must be greater than 0".to_string(),
                        source: None,
                    });
                }
            },
            McpTransportConfig::Tcp { port, host } => {
                if *port == 0 {
                    return Err(EmbedError::Configuration {
                        message: "transport.port must be greater than 0".to_string(),
                        source: None,
                    });
                }
                if host.is_empty() {
                    return Err(EmbedError::Configuration {
                        message: "transport.host cannot be empty".to_string(),
                        source: None,
                    });
                }
            },
            #[cfg(unix)]
            McpTransportConfig::UnixSocket { socket_path } => {
                if socket_path.as_os_str().is_empty() {
                    return Err(EmbedError::Configuration {
                        message: "transport.socket_path cannot be empty".to_string(),
                        source: None,
                    });
                }
            },
        }

        // Validate tools config
        if self.tools.max_results_per_call == 0 {
            return Err(EmbedError::Configuration {
                message: "tools.max_results_per_call must be greater than 0".to_string(),
                source: None,
            });
        }
        
        if self.tools.default_search_timeout_ms == 0 {
            return Err(EmbedError::Configuration {
                message: "tools.default_search_timeout_ms must be greater than 0".to_string(),
                source: None,
            });
        }

        // Validate performance config
        if self.performance.max_concurrent_requests == 0 {
            return Err(EmbedError::Configuration {
                message: "performance.max_concurrent_requests must be greater than 0".to_string(),
                source: None,
            });
        }
        
        if self.performance.request_timeout_ms == 0 {
            return Err(EmbedError::Configuration {
                message: "performance.request_timeout_ms must be greater than 0".to_string(),
                source: None,
            });
        }

        // Validate embedding config (only when ml feature is enabled)
        #[cfg(feature = "ml")]
        {
            if self.embedding.init_timeout_ms == 0 {
                return Err(EmbedError::Configuration {
                    message: "embedding.init_timeout_ms must be greater than 0".to_string(),
                    source: None,
                });
            }
            
            if self.embedding.health_check_interval_secs == 0 {
                return Err(EmbedError::Configuration {
                    message: "embedding.health_check_interval_secs must be greater than 0".to_string(),
                    source: None,
                });
            }
        }

        // Validate security config
        if self.security.max_query_length == 0 {
            return Err(EmbedError::Configuration {
                message: "security.max_query_length must be greater than 0".to_string(),
                source: None,
            });
        }
        
        if self.security.max_indexing_depth == 0 {
            return Err(EmbedError::Configuration {
                message: "security.max_indexing_depth must be greater than 0".to_string(),
                source: None,
            });
        }

        // Validate log level
        match self.mcp_log_level.to_lowercase().as_str() {
            "error" | "warn" | "info" | "debug" | "trace" => {},
            _ => return Err(EmbedError::Configuration {
                message: "mcp_log_level must be one of: error, warn, info, debug, trace".to_string(),
                source: None,
            }),
        }

        Ok(())
    }

    /// Check if LazyEmbedder should be used based on configuration
    pub fn should_use_lazy_embedder(&self) -> bool {
        #[cfg(feature = "ml")]
        {
            self.embedding.enable_lazy_loading
        }
        #[cfg(not(feature = "ml"))]
        {
            false
        }
    }

    /// Get embedder initialization timeout
    #[cfg(feature = "ml")]
    pub fn embedder_init_timeout_ms(&self) -> u64 {
        self.embedding.init_timeout_ms
    }

    /// Get maximum memory limit for embedder
    #[cfg(feature = "ml")]
    pub fn embedder_max_memory_mb(&self) -> Option<usize> {
        self.embedding.max_memory_mb
    }

    /// Get a configuration summary as a formatted string
    pub fn summary(&self) -> String {
        format!(
            r#"MCP Configuration Summary:
============================
Server:
  name: {}
  version: {}
  description: {}

Transport: {:?}

Tools:
  search: {}, index: {}, status: {}, clear: {}, orchestrated: {}
  max_results_per_call: {}
  default_timeout_ms: {}
  max_concurrent_ops: {}

Performance:
  max_concurrent_requests: {}
  request_timeout_ms: {}
  max_request_size_bytes: {}
  max_response_size_bytes: {}
  metrics_enabled: {}

Security:
  request_validation: {}
  max_query_length: {}
  path_protection: {}
  max_indexing_depth: {}

Logging:
  mcp_log_level: {}
  request_logging: {}
  performance_logging: {}
"#,
            self.server_name,
            self.server_version,
            self.server_description,
            self.transport,
            self.tools.enable_search,
            self.tools.enable_index,
            self.tools.enable_status,
            self.tools.enable_clear,
            self.tools.enable_orchestrated_search,
            self.tools.max_results_per_call,
            self.tools.default_search_timeout_ms,
            self.tools.max_concurrent_operations,
            self.performance.max_concurrent_requests,
            self.performance.request_timeout_ms,
            self.performance.max_request_size_bytes,
            self.performance.max_response_size_bytes,
            self.performance.enable_metrics,
            self.security.enable_request_validation,
            self.security.max_query_length,
            self.security.enable_path_protection,
            self.security.max_indexing_depth,
            self.mcp_log_level,
            self.enable_request_logging,
            self.enable_performance_logging
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_config_validation() {
        let config = McpConfig::new_test_config();
        assert!(config.validate().is_ok());
    }

    #[test] 
    fn test_mcp_config_invalid_server_name() {
        let mut config = McpConfig::new_test_config();
        config.server_name = "".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_mcp_config_invalid_log_level() {
        let mut config = McpConfig::new_test_config();
        config.mcp_log_level = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_lazy_embedder_config() {
        let config = McpConfig::new_test_config();
        
        #[cfg(feature = "ml")]
        {
            assert!(config.should_use_lazy_embedder());
            assert_eq!(config.embedder_init_timeout_ms(), 30000);
            assert_eq!(config.embedder_max_memory_mb(), Some(512));
        }
        
        #[cfg(not(feature = "ml"))]
        {
            assert!(!config.should_use_lazy_embedder());
        }
    }

    #[test]
    fn test_transport_config_validation() {
        let mut config = McpConfig::new_test_config();
        
        // Test invalid stdio buffer size
        config.transport = McpTransportConfig::Stdio {
            buffer_size: 0,
            line_buffering: true,
        };
        assert!(config.validate().is_err());
        
        // Test invalid TCP config
        config.transport = McpTransportConfig::Tcp {
            port: 0,
            host: "localhost".to_string(),
        };
        assert!(config.validate().is_err());
        
        config.transport = McpTransportConfig::Tcp {
            port: 8080,
            host: "".to_string(),
        };
        assert!(config.validate().is_err());
    }
}