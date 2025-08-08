# MCP Configuration Examples

This document provides comprehensive examples of MCP (Model Context Protocol) configuration for the embed-search system.

## Configuration File Hierarchy

The MCP configuration system looks for configuration files in the following order:

1. `.embed/mcp-config.toml` (recommended)
2. `.embed/mcp.toml`
3. `.embedrc-mcp`
4. `mcp-config.toml`

Environment variables with prefix `EMBED_MCP_` can override any configuration value.

## Required Base Configuration

Before using MCP configuration, you must have a base embed-search configuration file. See the main configuration documentation for details.

The MCP configuration extends the base configuration with MCP-specific settings.

## Configuration Examples

### 1. Basic MCP Configuration

**File: `.embed/mcp-config.toml`**

```toml
# Basic MCP server configuration
server_name = "my-project-mcp"
server_version = "1.0.0"
server_description = "MCP server for my project's code search"

# Transport configuration
[transport]
type = "Stdio"
buffer_size = 8192
line_buffering = true

# Tool configuration
[tools]
enable_search = true
enable_index = true
enable_status = true
enable_clear = true
enable_orchestrated_search = true
max_results_per_call = 100
default_search_timeout_ms = 15000
max_concurrent_operations = 5

# Performance configuration
[performance]
max_concurrent_requests = 50
request_timeout_ms = 30000
max_request_size_bytes = 1048576    # 1MB
max_response_size_bytes = 10485760  # 10MB
enable_metrics = true
metrics_interval_secs = 60

# Embedding configuration (only when ML feature is enabled)
[embedding]
enable_lazy_loading = true
init_timeout_ms = 45000
max_memory_mb = 1024
enable_health_checks = true
health_check_interval_secs = 300

# Security configuration
[security]
enable_request_validation = true
max_query_length = 1000
allowed_file_extensions = [
    "rs", "py", "js", "ts", "tsx", "jsx",
    "json", "toml", "yaml", "yml", "md",
    "txt", "csv", "sql", "go", "java", "cpp", "c", "h"
]
blocked_file_patterns = [
    "\.git/.*",
    ".*\.log$",
    ".*\.tmp$",
    ".*\.lock$",
    "node_modules/.*",
    "target/.*"
]
enable_path_protection = true
max_indexing_depth = 15

# Logging configuration
mcp_log_level = "info"
enable_request_logging = true
enable_performance_logging = false
```

### 2. High-Performance Configuration

**File: `.embed/mcp-config.toml`**

```toml
server_name = "high-performance-mcp"
server_version = "1.0.0"
server_description = "High-performance MCP server for large codebases"

[transport]
type = "Stdio"
buffer_size = 16384
line_buffering = false

[tools]
enable_search = true
enable_index = true
enable_status = true
enable_clear = false  # Disable for safety in production
enable_orchestrated_search = true
max_results_per_call = 200
default_search_timeout_ms = 5000  # Faster timeout
max_concurrent_operations = 20    # Higher concurrency

[performance]
max_concurrent_requests = 200
request_timeout_ms = 10000        # Shorter timeout for responsiveness
max_request_size_bytes = 2097152  # 2MB
max_response_size_bytes = 20971520 # 20MB
enable_metrics = true
metrics_interval_secs = 30

[embedding]
enable_lazy_loading = true
init_timeout_ms = 60000
max_memory_mb = 2048              # More memory for better performance
enable_health_checks = true
health_check_interval_secs = 180

[security]
enable_request_validation = true
max_query_length = 2000           # Longer queries allowed
allowed_file_extensions = [
    "rs", "py", "js", "ts", "tsx", "jsx", "vue", "svelte",
    "json", "toml", "yaml", "yml", "md", "mdx",
    "txt", "csv", "sql", "go", "java", "cpp", "c", "h", "hpp",
    "rb", "php", "scala", "kt", "swift", "dart", "r", "m"
]
blocked_file_patterns = [
    "\.git/.*", "\.svn/.*", "\.hg/.*",
    ".*\.log$", ".*\.tmp$", ".*\.lock$", ".*\.cache$",
    "node_modules/.*", "target/.*", "build/.*", "dist/.*",
    "\.env.*", ".*\.key$", ".*\.pem$"
]
enable_path_protection = true
max_indexing_depth = 25

mcp_log_level = "warn"            # Less verbose logging
enable_request_logging = false
enable_performance_logging = true
```

### 3. Development/Debug Configuration

**File: `.embed/mcp-config.toml`**

```toml
server_name = "development-mcp"
server_version = "0.1.0-dev"
server_description = "Development MCP server with extensive logging and debugging"

[transport]
type = "Stdio"
buffer_size = 4096
line_buffering = true

[tools]
enable_search = true
enable_index = true
enable_status = true
enable_clear = true
enable_orchestrated_search = true
max_results_per_call = 20         # Smaller for debugging
default_search_timeout_ms = 30000 # Longer timeout for debugging
max_concurrent_operations = 2     # Lower concurrency for stability

[performance]
max_concurrent_requests = 10
request_timeout_ms = 60000        # Very long timeout for debugging
max_request_size_bytes = 524288   # 512KB
max_response_size_bytes = 5242880 # 5MB
enable_metrics = true
metrics_interval_secs = 10        # Frequent metrics

[embedding]
enable_lazy_loading = true
init_timeout_ms = 120000          # Very long init timeout
max_memory_mb = 256               # Conservative memory usage
enable_health_checks = true
health_check_interval_secs = 60   # Frequent health checks

[security]
enable_request_validation = true
max_query_length = 500
allowed_file_extensions = [
    "rs", "py", "js", "ts", "json", "toml", "md", "txt"
]
blocked_file_patterns = [
    "\.git/.*", ".*\.log$", ".*\.tmp$"
]
enable_path_protection = true
max_indexing_depth = 5

mcp_log_level = "debug"           # Verbose logging
enable_request_logging = true
enable_performance_logging = true
```

### 4. Production Security-Focused Configuration

**File: `.embed/mcp-config.toml`**

```toml
server_name = "secure-production-mcp"
server_version = "1.0.0"
server_description = "Production MCP server with enhanced security"

[transport]
type = "Stdio"
buffer_size = 8192
line_buffering = true

[tools]
enable_search = true
enable_index = false              # Disable indexing in production
enable_status = true
enable_clear = false              # Disable clear for safety
enable_orchestrated_search = false # Disable complex operations
max_results_per_call = 50         # Limit results
default_search_timeout_ms = 10000 # Quick timeout
max_concurrent_operations = 3     # Conservative concurrency

[performance]
max_concurrent_requests = 25      # Conservative limit
request_timeout_ms = 15000        # Quick timeout
max_request_size_bytes = 262144   # 256KB - small requests
max_response_size_bytes = 2621440 # 2.5MB - limited response size
enable_metrics = false            # Disable metrics for performance
metrics_interval_secs = 300

[embedding]
enable_lazy_loading = true
init_timeout_ms = 30000
max_memory_mb = 512
enable_health_checks = false     # Disable for performance
health_check_interval_secs = 600

[security]
enable_request_validation = true
max_query_length = 200            # Very short queries only
allowed_file_extensions = [       # Minimal set
    "rs", "py", "js", "ts", "json", "md"
]
blocked_file_patterns = [
    "\.git/.*", "\.env.*", ".*\.key$", ".*\.pem$", ".*\.log$",
    ".*\.tmp$", ".*\.lock$", ".*\.cache$", ".*\.bak$",
    "node_modules/.*", "target/.*", "build/.*", "dist/.*",
    "\.ssh/.*", "\.aws/.*", "\.docker/.*"
]
enable_path_protection = true
max_indexing_depth = 3            # Very shallow indexing

mcp_log_level = "error"           # Only errors
enable_request_logging = false
enable_performance_logging = false
```

## Environment Variable Overrides

You can override any configuration value using environment variables with the `EMBED_MCP_` prefix:

```bash
# Override server name
export EMBED_MCP_SERVER_NAME="my-custom-server"

# Override transport buffer size
export EMBED_MCP_TRANSPORT_BUFFER_SIZE=16384

# Override tool settings
export EMBED_MCP_TOOLS_MAX_RESULTS_PER_CALL=150

# Override performance settings
export EMBED_MCP_PERFORMANCE_MAX_CONCURRENT_REQUESTS=100

# Override embedding settings (when ML feature enabled)
export EMBED_MCP_EMBEDDING_MAX_MEMORY_MB=2048

# Override security settings
export EMBED_MCP_SECURITY_MAX_QUERY_LENGTH=1500

# Override logging
export EMBED_MCP_MCP_LOG_LEVEL=debug
```

## Configuration Validation

All configuration values are validated when the MCP server starts. The system will:

1. Check that all required fields are present
2. Validate that numeric values are within reasonable ranges
3. Ensure security settings are properly configured
4. Verify that the transport configuration is valid

If validation fails, the server will not start and will provide clear error messages about what needs to be fixed.

## Integration with Base Configuration

The MCP configuration works alongside your main embed-search configuration:

```toml
# Main configuration: .embed/config.toml
project_path = "/path/to/project"
chunk_size = 1000
# ... other base settings

# MCP configuration: .embed/mcp-config.toml  
server_name = "my-project-mcp"
# ... MCP-specific settings
```

Both configurations must be present for the MCP server to function properly.

## LazyEmbedder Integration

The MCP configuration includes specific settings for the LazyEmbedder system:

- `enable_lazy_loading`: Use lazy loading to prevent memory issues in Node.js environments
- `init_timeout_ms`: How long to wait for embedder initialization
- `max_memory_mb`: Memory limit for the embedder (optional)
- `enable_health_checks`: Monitor embedder health
- `health_check_interval_secs`: How often to check embedder health

This is especially important in MCP server contexts where memory management is critical.

## Troubleshooting

### Common Issues

1. **Configuration file not found**: Ensure your MCP config file is in one of the expected locations
2. **Base configuration missing**: Make sure you have a valid base embed-search configuration
3. **Validation errors**: Check that all numeric values are positive and strings are non-empty
4. **LazyEmbedder initialization fails**: Increase `init_timeout_ms` or check memory limits
5. **Performance issues**: Adjust concurrency and timeout settings for your hardware

### Debug Configuration

Use the development configuration example above for debugging issues, then optimize for your specific use case.