/// Integration test for MCP configuration system
/// 
/// This test verifies that the MCP configuration system properly integrates
/// with the existing Config system and handles the LazyEmbedder correctly.

#[cfg(test)]
mod mcp_config_integration_tests {
    use crate::config::Config;
    use crate::mcp::config::McpConfig;
    use crate::mcp::McpServer;
    use crate::search::unified::UnifiedSearcher;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use std::fs;

    /// Test that MCP config can be loaded and validated
    #[tokio::test]
    async fn test_mcp_config_basic_validation() {
        let config = McpConfig::new_test_config();
        assert!(config.validate().is_ok(), "Test config should be valid");
        
        // Verify LazyEmbedder integration
        #[cfg(feature = "ml")]
        {
            assert!(config.should_use_lazy_embedder(), "Should use lazy embedder in test config");
            assert_eq!(config.embedder_init_timeout_ms(), 30000);
            assert_eq!(config.embedder_max_memory_mb(), Some(512));
        }
        
        #[cfg(not(feature = "ml"))]
        {
            assert!(!config.should_use_lazy_embedder(), "Should not use lazy embedder without ML feature");
        }
    }

    /// Test MCP config loading from file
    #[tokio::test]
    async fn test_mcp_config_file_loading() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("mcp-test-config.toml");
        
        let config_content = r#"
server_name = "test-server"
server_version = "0.1.0"
server_description = "Test MCP server"

[transport]
type = "Stdio"
buffer_size = 4096
line_buffering = true

[tools]
enable_search = true
enable_index = false
enable_status = true
enable_clear = false
enable_orchestrated_search = false
max_results_per_call = 10
default_search_timeout_ms = 5000
max_concurrent_operations = 1

[performance]
max_concurrent_requests = 5
request_timeout_ms = 10000
max_request_size_bytes = 1024
max_response_size_bytes = 2048
enable_metrics = false
metrics_interval_secs = 60

[security]
enable_request_validation = true
max_query_length = 100
allowed_file_extensions = ["rs", "txt"]
blocked_file_patterns = [".*\\.tmp$"]
enable_path_protection = true
max_indexing_depth = 3

mcp_log_level = "debug"
enable_request_logging = true
enable_performance_logging = false
        "#;
        
        fs::write(&config_path, config_content).expect("Failed to write config file");
        
        let config = McpConfig::load_from_file(&config_path)
            .expect("Failed to load config from file");
        
        assert_eq!(config.server_name, "test-server");
        assert_eq!(config.server_version, "0.1.0");
        assert!(!config.tools.enable_index);
        assert_eq!(config.security.max_query_length, 100);
        assert_eq!(config.mcp_log_level, "debug");
        
        assert!(config.validate().is_ok(), "Loaded config should be valid");
    }

    /// Test MCP config validation catches errors
    #[tokio::test]
    async fn test_mcp_config_validation_errors() {
        let mut config = McpConfig::new_test_config();
        
        // Test empty server name
        config.server_name = "".to_string();
        assert!(config.validate().is_err(), "Should reject empty server name");
        
        // Reset and test invalid log level
        config = McpConfig::new_test_config();
        config.mcp_log_level = "invalid_level".to_string();
        assert!(config.validate().is_err(), "Should reject invalid log level");
        
        // Reset and test zero timeout
        config = McpConfig::new_test_config();
        config.performance.request_timeout_ms = 0;
        assert!(config.validate().is_err(), "Should reject zero timeout");
        
        // Reset and test zero query length
        config = McpConfig::new_test_config();
        config.security.max_query_length = 0;
        assert!(config.validate().is_err(), "Should reject zero query length");
    }

    /// Test MCP server creation with config
    #[tokio::test]
    async fn test_mcp_server_with_config() {
        // Initialize base config first
        Config::init_test().expect("Failed to init test config");
        
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_path_buf();
        let db_path = project_path.join(".embed-search");
        
        let searcher = UnifiedSearcher::new(project_path.clone(), db_path)
            .await
            .expect("Failed to create UnifiedSearcher");
        
        let mcp_config = McpConfig::new_test_config();
        
        let server = McpServer::new(searcher, mcp_config)
            .await
            .expect("Failed to create MCP server with config");
        
        // Verify server was created successfully
        // The fact that we got here without panicking means the integration worked
    }

    /// Test that MCP config properly handles LazyEmbedder settings
    #[cfg(feature = "ml")]
    #[tokio::test]
    async fn test_lazy_embedder_integration() {
        let config = McpConfig::new_test_config();
        
        // Test LazyEmbedder specific methods
        assert!(config.should_use_lazy_embedder());
        assert_eq!(config.embedder_init_timeout_ms(), 30000);
        assert_eq!(config.embedder_max_memory_mb(), Some(512));
        
        // Test that embedding config is properly set
        assert!(config.embedding.enable_lazy_loading);
        assert!(config.embedding.enable_health_checks);
        assert_eq!(config.embedding.health_check_interval_secs, 300);
    }

    /// Test configuration summary generation
    #[tokio::test]
    async fn test_config_summary() {
        let config = McpConfig::new_test_config();
        let summary = config.summary();
        
        assert!(summary.contains("MCP Configuration Summary"));
        assert!(summary.contains(&config.server_name));
        assert!(summary.contains(&config.server_version));
        assert!(summary.contains("Transport:"));
        assert!(summary.contains("Tools:"));
        assert!(summary.contains("Performance:"));
        assert!(summary.contains("Security:"));
        assert!(summary.contains("Logging:"));
    }

    /// Test transport configuration validation
    #[tokio::test]
    async fn test_transport_config_validation() {
        use crate::mcp::config::McpTransportConfig;
        
        let mut config = McpConfig::new_test_config();
        
        // Test invalid stdio buffer size
        config.transport = McpTransportConfig::Stdio {
            buffer_size: 0,
            line_buffering: true,
        };
        assert!(config.validate().is_err(), "Should reject zero buffer size");
        
        // Test valid TCP config (if we implement it in the future)
        config.transport = McpTransportConfig::Tcp {
            port: 8080,
            host: "localhost".to_string(),
        };
        assert!(config.validate().is_ok(), "Should accept valid TCP config");
        
        // Test invalid TCP port
        config.transport = McpTransportConfig::Tcp {
            port: 0,
            host: "localhost".to_string(),
        };
        assert!(config.validate().is_err(), "Should reject zero port");
        
        // Test empty TCP host
        config.transport = McpTransportConfig::Tcp {
            port: 8080,
            host: "".to_string(),
        };
        assert!(config.validate().is_err(), "Should reject empty host");
    }

    /// Test security configuration validation
    #[tokio::test]
    async fn test_security_config_validation() {
        let mut config = McpConfig::new_test_config();
        
        // Test that empty allowed extensions are handled
        config.security.allowed_file_extensions.clear();
        assert!(config.validate().is_ok(), "Empty extensions should be valid");
        
        // Test that empty blocked patterns are handled
        config.security.blocked_file_patterns.clear();
        assert!(config.validate().is_ok(), "Empty blocked patterns should be valid");
        
        // Test zero max indexing depth
        config.security.max_indexing_depth = 0;
        assert!(config.validate().is_err(), "Should reject zero max indexing depth");
    }

    /// Test performance configuration edge cases
    #[tokio::test]
    async fn test_performance_config_edge_cases() {
        let mut config = McpConfig::new_test_config();
        
        // Test very large values (should be accepted)
        config.performance.max_concurrent_requests = 1000;
        config.performance.max_request_size_bytes = 100_000_000;  // 100MB
        assert!(config.validate().is_ok(), "Should accept large but reasonable values");
        
        // Test zero concurrent requests
        config.performance.max_concurrent_requests = 0;
        assert!(config.validate().is_err(), "Should reject zero concurrent requests");
        
        // Reset and test zero timeout
        config = McpConfig::new_test_config();
        config.performance.request_timeout_ms = 0;
        assert!(config.validate().is_err(), "Should reject zero timeout");
    }
}