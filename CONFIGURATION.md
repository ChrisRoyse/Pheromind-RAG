# Configuration System

The Rust embedding search system includes a comprehensive configuration system that allows you to customize all aspects of the application without recompiling.

## Configuration Sources (Priority Order)

The system loads configuration from multiple sources with the following priority order (highest to lowest):

1. **Command line arguments** (highest priority)
2. **Environment variables** (EMBED_* prefix)
3. **Project-specific config files** (`.embedrc` or `.embed/config.toml`)
4. **Global config file** (`config.toml`)
5. **Default values** (lowest priority)

## Configuration Parameters

### Chunking Configuration

- **`chunk_size`** (default: 100)
  - Size of text chunks for embedding (number of lines)
  - Smaller chunks provide more precise results but may miss context
  - Larger chunks provide more context but may be less precise

### Cache Configuration

- **`embedding_cache_size`** (default: 10000)
  - Maximum number of embeddings to cache in memory
  - Higher values use more memory but reduce computation time

- **`search_cache_size`** (default: 100)
  - Maximum number of search results to cache
  - Caches recent search queries for faster repeated searches

- **`cache_dir`** (default: ".embed_cache")
  - Directory for persistent cache files
  - Will be created if it doesn't exist

### Processing Configuration

- **`batch_size`** (default: 32)
  - Number of chunks to process in parallel batches
  - Higher values can be faster but use more memory/CPU

### Storage Configuration

- **`vector_db_path`** (default: ".embed_db")
  - Path to the vector database directory
  - Will be created if it doesn't exist

### Git Watching Configuration

- **`enable_git_watch`** (default: true)
  - Enable automatic file watching using git status
  - When enabled, the system monitors for file changes

- **`git_poll_interval_secs`** (default: 5)
  - How often to poll for git changes (in seconds)
  - Lower values are more responsive but use more CPU

### Search Configuration

- **`include_test_files`** (default: false)
  - Include test files in indexing and search results
  - Set to false to ignore files with test patterns (test_, _test, spec_, etc.)

- **`max_search_results`** (default: 20)
  - Maximum number of search results to return
  - Higher values provide more comprehensive results but slower response


### Model Configuration

- **`model_name`** (default: "sentence-transformers/all-MiniLM-L6-v2")
  - Name of the embedding model to use
  - ⚠️ **Warning**: Changing this requires rebuilding the entire index

- **`embedding_dimensions`** (default: 384)
  - Dimensions of the embedding vectors
  - Must match the model's output dimensions
  - all-MiniLM-L6-v2 produces 384-dimensional embeddings

### Logging Configuration

- **`log_level`** (default: "info")
  - Log level for the application
  - Valid values: error, warn, info, debug, trace
  - "info" provides balanced logging, "debug" shows detailed operation info

## Usage Examples

### 1. Global Configuration File

Create a `config.toml` file in the project root:

```toml
# Global configuration
chunk_size = 80
embedding_cache_size = 15000
log_level = "debug"
git_poll_interval_secs = 3
include_test_files = true
```

### 2. Project-Specific Configuration

Create a `.embedrc` file in your project directory:

```toml  
# Project-specific overrides
chunk_size = 50
max_search_results = 30
log_level = "trace"
```

Or create `.embed/config.toml`:

```toml
# Alternative project-specific config location
chunk_size = 120
batch_size = 64
```

### 3. Environment Variables

Set environment variables with `EMBED_` prefix:

```bash
# Linux/Mac
export EMBED_CHUNK_SIZE=150
export EMBED_LOG_LEVEL=debug
export EMBED_INCLUDE_TEST_FILES=true
export EMBED_BATCH_SIZE=64

# Windows PowerShell
$env:EMBED_CHUNK_SIZE=150
$env:EMBED_LOG_LEVEL="debug"
$env:EMBED_INCLUDE_TEST_FILES="true"
```

### 4. Custom Configuration File

Load a specific configuration file:

```bash
embed-search --config my_config.toml search "query"
```

## CLI Commands

### View Current Configuration

```bash
# Human-readable format
embed-search config

# JSON format
embed-search config --json
```

### Validate Configuration

```bash
# Validate current configuration
embed-search validate-config

# Validate specific file
embed-search validate-config my_config.toml
```

## Configuration Precedence Example

Given these configuration sources:

1. `config.toml`: `chunk_size = 100`
2. `.embedrc`: `chunk_size = 75`  
3. Environment: `EMBED_CHUNK_SIZE=50`
4. Command line: `--config special.toml` where `special.toml` has `chunk_size = 25`

The final `chunk_size` will be **25** (from the custom config file).

## Tips and Best Practices

### Performance Tuning

- **Small codebases** (< 1000 files): `chunk_size=50`, `batch_size=16`
- **Medium codebases** (1000-10000 files): `chunk_size=100`, `batch_size=32` 
- **Large codebases** (> 10000 files): `chunk_size=150`, `batch_size=64`

### Memory Management

- **8GB RAM**: `embedding_cache_size=5000`
- **16GB RAM**: `embedding_cache_size=10000` (default)
- **32GB+ RAM**: `embedding_cache_size=20000+`

### Development vs Production

**Development**:
```toml
log_level = "debug"
git_poll_interval_secs = 2
include_test_files = true
max_search_results = 50
```

**Production**:
```toml
log_level = "warn" 
git_poll_interval_secs = 10
include_test_files = false
max_search_results = 20
```

## Troubleshooting

### Configuration Not Loading

1. Check file permissions
2. Verify TOML syntax with `embed-search validate-config path/to/config.toml`
3. Check current working directory for project-specific configs

### Performance Issues

1. Monitor memory usage and adjust cache sizes
2. Tune `batch_size` based on CPU cores and memory
3. Adjust `git_poll_interval_secs` to reduce CPU usage

### Invalid Configuration

```bash
embed-search validate-config
```

Will show specific validation errors and suggestions for fixes.