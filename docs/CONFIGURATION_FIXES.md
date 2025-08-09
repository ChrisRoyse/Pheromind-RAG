# Configuration System Architectural Fixes

## Executive Summary

The current configuration system has fundamental design flaws that create maintenance nightmares, circular dependencies, and hidden runtime failures. This document details the specific problems and provides a complete architectural solution.

## Current Configuration Problems

### 1. **Missing Default Implementation Problem**

**Issue**: Code throughout the codebase expects `Config::default()` to exist, but the main `Config` struct in `src/config/mod.rs` deliberately has no `Default` implementation.

**Evidence**:
```rust
// In tests/integration_end_to_end_validation.rs:91
let config = Config::default(); // ❌ COMPILATION ERROR - No Default impl

// In benchmarks and tests throughout the codebase:
Config::init_test().expect("Failed to initialize test config");
```

**Root Cause**: Inconsistent design philosophy between different configuration structs. Some have `Default`, others explicitly reject it.

### 2. **Global Configuration Circular Dependencies**

**Issue**: The global configuration singleton creates circular dependency issues and initialization race conditions.

**Evidence**:
```rust
// In src/config/mod.rs:46-49
pub static CONFIG: Lazy<RwLock<Option<Config>>> = Lazy::new(|| {
    RwLock::new(None)
});

// Multiple places try to access CONFIG before initialization
let config = Config::get()?; // ❌ Runtime panic if not initialized
```

**Symptoms**:
- Tests fail randomly based on execution order
- Initialization must happen in specific sequence
- Global state makes testing difficult
- Thread safety issues with multiple readers/writers

### 3. **Mixed Configuration Validation Strategies**

**Issue**: Different parts of the system use different validation approaches, leading to inconsistent error handling.

**Evidence**:
```rust
// In src/config/mod.rs - comprehensive validation
pub fn validate(&self) -> Result<()> {
    if self.chunk_size == 0 { /* detailed error */ }
    // ... many validation rules
}

// In src/config/safe_config.rs - different validation approach
pub fn validate(&self) -> Result<()> {
    if self.storage.max_connections == 0 { /* different error type */ }
    // ... different validation rules
}
```

### 4. **Test vs Production Configuration Contamination**

**Issue**: Test configurations leak into production code and vice versa.

**Evidence**:
```rust
// Test-only configs used in production paths:
#[cfg(any(test, debug_assertions))]
pub fn new_test_config() -> Self { /* ... */ }

// Production code accidentally calling test configs:
Config::init_test().expect("Failed to initialize test config");
```

## Design Flaws Analysis

### 1. **Principle Violation: Implicit vs Explicit Configuration**

The codebase violates its own "Principle 0" (no defaults) by having multiple configuration systems with different philosophies:

```rust
// Principle 0 violation - some configs have defaults:
impl Default for SearchConfig { /* ... */ }
let config = FusionConfig::default(); // Works

// But main Config rejects defaults:
// No Default for Config - compilation error
```

### 2. **Architectural Inconsistency**

Two separate configuration systems exist:
- `src/config/mod.rs` - Global singleton with complex state management
- `src/config/safe_config.rs` - Stateless configuration with explicit loading

This creates confusion about which system to use and leads to maintenance overhead.

### 3. **Error Handling Fragmentation**

Multiple error types and patterns exist:
```rust
// Different error types across config systems:
EmbedError::Configuration { /* ... */ }  // safe_config.rs
anyhow::Error                           // mod.rs
Result<Config>                          // Different Result types
```

## New Configuration Architecture

### 1. **Unified Configuration Structure**

Replace the dual-configuration system with a single, consistent approach:

```rust
// src/config/unified.rs
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

/// Main configuration for the embed-search system
/// PRINCIPLE: All configuration must be explicit - no hidden defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub project: ProjectConfig,
    pub storage: StorageConfig,
    pub embedding: EmbeddingConfig,
    pub search: SearchConfig,
    pub logging: LoggingConfig,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project root path for search operations
    pub project_path: PathBuf,
    /// Include test files in indexing
    pub include_test_files: bool,
    /// Git watching configuration
    pub enable_git_watch: bool,
    pub git_poll_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Vector database backend
    pub backend: StorageBackend,
    /// Vector database path
    pub vector_db_path: PathBuf,
    /// Cache directory for temporary files
    pub cache_dir: PathBuf,
    /// Maximum connections to database
    pub max_connections: usize,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Cache size in number of entries
    pub cache_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackend {
    #[serde(rename = "memory")]
    Memory,
    #[serde(rename = "lancedb")]
    LanceDB,
    #[serde(rename = "sqlite")]  
    SQLite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Model configuration
    #[cfg(feature = "ml")]
    pub model_name: String,
    #[cfg(feature = "ml")]
    pub embedding_dimensions: usize,
    #[cfg(feature = "ml")]
    pub embedding_cache_size: usize,
    /// Batch processing
    pub batch_size: usize,
    /// Text chunking
    pub chunk_size: usize,
    /// Enable text preprocessing
    pub enable_stemming: bool,
    pub enable_ngrams: bool,
    pub max_ngram_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Search backends to enable
    pub backends: SearchBackends,
    /// Search result limits
    pub max_search_results: usize,
    /// BM25 configuration
    pub bm25: BM25Config,
    /// Fusion scoring weights
    pub fusion: FusionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchBackends {
    pub enable_bm25: bool,
    pub enable_tantivy: bool,
    pub enable_semantic: bool,
    pub enable_tree_sitter: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BM25Config {
    pub k1: f32,
    pub b: f32,
    pub index_path: PathBuf,
    pub cache_size: usize,
    pub min_term_length: usize,
    pub max_term_length: usize,
    pub stop_words: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionConfig {
    pub exact_weight: f32,
    pub bm25_weight: f32,
    pub semantic_weight: f32,
    pub symbol_weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (error, warn, info, debug, trace)
    pub level: String,
    /// Log format (json, text)
    pub format: LogFormat,
    /// Output destination
    pub output: LogOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "text")]
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    #[serde(rename = "stdout")]
    Stdout,
    #[serde(rename = "stderr")]
    Stderr,
    #[serde(rename = "file")]
    File(PathBuf),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Thread pool sizes
    pub search_threads: usize,
    pub indexing_threads: usize,
    /// Memory limits in MB
    pub max_memory_mb: usize,
    /// Timeout configurations in seconds
    pub search_timeout_secs: u64,
    pub indexing_timeout_secs: u64,
}
```

### 2. **Configuration Loading Strategy**

Implement a layered configuration approach with explicit precedence:

```rust
impl Config {
    /// Load configuration with explicit precedence:
    /// 1. Command line arguments (highest priority)
    /// 2. Environment variables
    /// 3. Configuration file
    /// 4. No defaults - all values must be provided
    pub fn load() -> Result<Self> {
        let mut builder = ConfigBuilder::new();
        
        // Layer 1: Find and load configuration file
        let config_file = Self::find_config_file()?;
        builder = builder.add_file_source(config_file)?;
        
        // Layer 2: Environment variable overrides
        builder = builder.add_env_source("EMBED")?;
        
        // Layer 3: Command line argument overrides (if provided)
        builder = builder.add_cli_args()?;
        
        let config = builder.build()?;
        config.validate_all()?;
        
        Ok(config)
    }
    
    /// Find configuration file using explicit search order
    fn find_config_file() -> Result<PathBuf> {
        let search_paths = vec![
            // Project-specific locations (highest priority)
            PathBuf::from(".embed/config.toml"),
            PathBuf::from(".embedrc.toml"),
            PathBuf::from("embed.toml"),
            
            // User home directory
            dirs::home_dir().map(|h| h.join(".config/embed/config.toml")),
            
            // System-wide configuration (lowest priority)
            PathBuf::from("/etc/embed/config.toml"),
        ];
        
        for path in search_paths {
            if let Some(path) = path {
                if path.exists() {
                    return Ok(path);
                }
            }
        }
        
        Err(anyhow::anyhow!(
            "No configuration file found. Please create one of:\n{}", 
            search_paths.iter()
                .filter_map(|p| p.as_ref().map(|p| format!("  - {}", p.display())))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }
    
    /// Load configuration from specific file path
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "Configuration file does not exist: {}", 
                path.display()
            ));
        }
        
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let config = match path.extension().and_then(|e| e.to_str()) {
            Some("toml") => toml::from_str(&contents)?,
            Some("json") => serde_json::from_str(&contents)?,
            Some("yaml") | Some("yml") => serde_yaml::from_str(&contents)?,
            _ => return Err(anyhow::anyhow!(
                "Unsupported config format. Supported: .toml, .json, .yaml, .yml"
            )),
        };
        
        config.validate_all()?;
        Ok(config)
    }
}
```

### 3. **Configuration Builder Pattern**

Replace global singletons with explicit dependency injection:

```rust
/// Configuration builder for composing config from multiple sources
pub struct ConfigBuilder {
    layers: Vec<ConfigLayer>,
}

enum ConfigLayer {
    File(PathBuf),
    Environment(String), // prefix
    CliArgs(Vec<String>),
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }
    
    pub fn add_file_source<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        self.layers.push(ConfigLayer::File(path.as_ref().to_path_buf()));
        Ok(self)
    }
    
    pub fn add_env_source<S: Into<String>>(mut self, prefix: S) -> Result<Self> {
        self.layers.push(ConfigLayer::Environment(prefix.into()));
        Ok(self)
    }
    
    pub fn add_cli_args(mut self) -> Result<Self> {
        let args: Vec<String> = std::env::args().collect();
        self.layers.push(ConfigLayer::CliArgs(args));
        Ok(self)
    }
    
    pub fn build(self) -> Result<Config> {
        let mut config_map = std::collections::HashMap::new();
        
        // Process layers in order (later layers override earlier ones)
        for layer in &self.layers {
            match layer {
                ConfigLayer::File(path) => {
                    self.merge_file_config(&mut config_map, path)?;
                }
                ConfigLayer::Environment(prefix) => {
                    self.merge_env_config(&mut config_map, prefix)?;
                }
                ConfigLayer::CliArgs(args) => {
                    self.merge_cli_config(&mut config_map, args)?;
                }
            }
        }
        
        // Convert merged map to Config struct
        let config_value = serde_json::to_value(config_map)?;
        let config: Config = serde_json::from_value(config_value)?;
        
        Ok(config)
    }
    
    // Implementation details for merge_file_config, merge_env_config, merge_cli_config...
}
```

### 4. **Dependency Injection Container**

Replace global state with explicit dependency management:

```rust
/// Application context containing all configured services
/// Replaces global singletons with explicit dependency injection
pub struct AppContext {
    config: Config,
    // Lazily initialized services
    storage: OnceCell<Arc<dyn VectorStorage>>,
    embedder: OnceCell<Arc<dyn Embedder>>,
    searcher: OnceCell<Arc<dyn SearchEngine>>,
}

impl AppContext {
    /// Create new application context from configuration
    pub fn new(config: Config) -> Self {
        Self {
            config,
            storage: OnceCell::new(),
            embedder: OnceCell::new(),
            searcher: OnceCell::new(),
        }
    }
    
    /// Get configuration reference
    pub fn config(&self) -> &Config {
        &self.config
    }
    
    /// Get or create storage service
    pub async fn storage(&self) -> Result<Arc<dyn VectorStorage>> {
        if let Some(storage) = self.storage.get() {
            return Ok(Arc::clone(storage));
        }
        
        let storage = match self.config.storage.backend {
            StorageBackend::Memory => {
                Arc::new(MemoryStorage::new()) as Arc<dyn VectorStorage>
            }
            StorageBackend::LanceDB => {
                Arc::new(LanceDBStorage::new(&self.config.storage.vector_db_path).await?)
            }
            StorageBackend::SQLite => {
                Arc::new(SQLiteStorage::new(&self.config.storage.vector_db_path).await?)
            }
        };
        
        let _ = self.storage.set(Arc::clone(&storage));
        Ok(storage)
    }
    
    /// Get or create embedding service
    #[cfg(feature = "ml")]
    pub async fn embedder(&self) -> Result<Arc<dyn Embedder>> {
        if let Some(embedder) = self.embedder.get() {
            return Ok(Arc::clone(embedder));
        }
        
        let embedder = Arc::new(
            NomicEmbedder::new(
                &self.config.embedding.model_name,
                self.config.embedding.embedding_cache_size,
            ).await?
        ) as Arc<dyn Embedder>;
        
        let _ = self.embedder.set(Arc::clone(&embedder));
        Ok(embedder)
    }
    
    /// Create a new search session with this context
    pub async fn create_search_session(&self) -> Result<SearchSession> {
        SearchSession::new(
            Arc::clone(&self.storage().await?),
            #[cfg(feature = "ml")]
            Arc::clone(&self.embedder().await?),
            &self.config.search,
        ).await
    }
}

/// Individual search session - no global state
pub struct SearchSession {
    storage: Arc<dyn VectorStorage>,
    #[cfg(feature = "ml")]
    embedder: Arc<dyn Embedder>,
    config: SearchConfig,
    // Session-specific state
    bm25_engine: OnceCell<BM25Engine>,
    tantivy_searcher: OnceCell<TantivySearcher>,
}
```

## Migration Strategy

### Phase 1: Create Unified Configuration (Week 1)

1. **Create new unified config module**:
   ```bash
   # Create new configuration system
   src/config/unified.rs     # New unified config structs
   src/config/builder.rs     # Configuration builder
   src/config/context.rs     # Application context
   ```

2. **Implement backward compatibility layer**:
   ```rust
   // src/config/compat.rs - Temporary compatibility
   #[deprecated(note = "Use AppContext instead")]
   pub fn init_global_config() -> Result<()> {
       // Bridge old global config to new system
   }
   ```

3. **Create example configuration files**:
   ```bash
   config/examples/development.toml
   config/examples/production.toml
   config/examples/testing.toml
   ```

### Phase 2: Migrate Core Services (Week 2)

1. **Update search services**:
   - Modify `UnifiedSearcher` to accept `AppContext`
   - Remove global config dependencies
   - Update all search-related modules

2. **Update storage services**:
   - Modify storage backends to use explicit config
   - Remove global state access
   - Update connection management

### Phase 3: Test Migration (Week 3)

1. **Update test infrastructure**:
   ```rust
   // tests/helpers/mod.rs
   pub fn create_test_context() -> AppContext {
       let config = Config::load_from_file("config/examples/testing.toml")
           .expect("Test config must exist");
       AppContext::new(config)
   }
   ```

2. **Migrate all tests to use AppContext**:
   - Remove `Config::init_test()` calls
   - Use `create_test_context()` helper
   - Ensure tests are isolated

### Phase 4: Production Migration (Week 4)

1. **Update binary entry points**:
   - CLI applications
   - MCP server
   - Benchmark tools

2. **Remove legacy configuration**:
   - Delete `src/config/mod.rs`
   - Delete `src/config/safe_config.rs`
   - Remove global CONFIG static

## Validation Rules

### 1. **Comprehensive Validation Framework**

```rust
impl Config {
    /// Validate entire configuration with detailed error reporting
    pub fn validate_all(&self) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();
        
        // Validate each section
        self.project.validate(&mut report);
        self.storage.validate(&mut report);
        self.embedding.validate(&mut report);
        self.search.validate(&mut report);
        self.logging.validate(&mut report);
        self.performance.validate(&mut report);
        
        // Cross-section validation
        self.validate_cross_dependencies(&mut report);
        
        if report.has_errors() {
            Err(anyhow::anyhow!("Configuration validation failed:\n{}", report))
        } else {
            Ok(report)
        }
    }
    
    fn validate_cross_dependencies(&self, report: &mut ValidationReport) {
        // Example: Ensure embedding dimensions match storage expectations
        #[cfg(feature = "ml")]
        {
            if matches!(self.storage.backend, StorageBackend::LanceDB) {
                if self.embedding.embedding_dimensions == 0 {
                    report.add_error(
                        "embedding.embedding_dimensions",
                        "Must be > 0 when using LanceDB storage"
                    );
                }
            }
        }
        
        // Ensure performance settings are reasonable
        if self.performance.search_threads == 0 {
            report.add_error(
                "performance.search_threads",
                "Must be > 0"
            );
        }
        
        // Validate path accessibility
        if !self.project.project_path.exists() {
            report.add_warning(
                "project.project_path",
                format!("Path does not exist: {}", self.project.project_path.display())
            );
        }
    }
}

pub struct ValidationReport {
    errors: Vec<ValidationError>,
    warnings: Vec<ValidationWarning>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn add_error<S1, S2>(&mut self, field: S1, message: S2)
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.errors.push(ValidationError {
            field: field.into(),
            message: message.into(),
        });
    }
    
    pub fn add_warning<S1, S2>(&mut self, field: S1, message: S2)
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.warnings.push(ValidationWarning {
            field: field.into(),
            message: message.into(),
        });
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl std::fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.errors.is_empty() {
            writeln!(f, "Errors:")?;
            for error in &self.errors {
                writeln!(f, "  - {}: {}", error.field, error.message)?;
            }
        }
        
        if !self.warnings.is_empty() {
            writeln!(f, "Warnings:")?;
            for warning in &self.warnings {
                writeln!(f, "  - {}: {}", warning.field, warning.message)?;
            }
        }
        
        Ok(())
    }
}
```

### 2. **Field-Specific Validation Rules**

```rust
impl ProjectConfig {
    fn validate(&self, report: &mut ValidationReport) {
        if !self.project_path.exists() {
            report.add_error(
                "project.project_path",
                format!("Project path does not exist: {}", self.project_path.display())
            );
        }
        
        if self.git_poll_interval_secs == 0 {
            report.add_error(
                "project.git_poll_interval_secs",
                "Git poll interval must be > 0 seconds"
            );
        }
        
        if self.git_poll_interval_secs < 5 {
            report.add_warning(
                "project.git_poll_interval_secs", 
                "Git poll interval < 5 seconds may cause high CPU usage"
            );
        }
    }
}

impl StorageConfig {
    fn validate(&self, report: &mut ValidationReport) {
        // Validate paths
        if let Some(parent) = self.vector_db_path.parent() {
            if !parent.exists() {
                report.add_error(
                    "storage.vector_db_path",
                    format!("Parent directory does not exist: {}", parent.display())
                );
            }
        }
        
        // Validate numeric constraints
        if self.max_connections == 0 {
            report.add_error(
                "storage.max_connections",
                "Maximum connections must be > 0"
            );
        }
        
        if self.connection_timeout_ms < 1000 {
            report.add_warning(
                "storage.connection_timeout_ms",
                "Connection timeout < 1000ms may cause frequent timeouts"
            );
        }
        
        if self.cache_size == 0 {
            report.add_error(
                "storage.cache_size",
                "Cache size must be > 0"
            );
        }
    }
}

impl SearchConfig {
    fn validate(&self, report: &mut ValidationReport) {
        // Validate that at least one backend is enabled
        if !self.backends.enable_bm25 
           && !self.backends.enable_tantivy
           && !self.backends.enable_semantic
           && !self.backends.enable_tree_sitter {
            report.add_error(
                "search.backends",
                "At least one search backend must be enabled"
            );
        }
        
        // Validate fusion weights
        let total_weight = self.fusion.exact_weight 
            + self.fusion.bm25_weight
            + self.fusion.semantic_weight
            + self.fusion.symbol_weight;
            
        if (total_weight - 1.0).abs() > 0.01 {
            report.add_warning(
                "search.fusion",
                format!("Fusion weights sum to {:.3}, not 1.0", total_weight)
            );
        }
        
        // Validate BM25 parameters
        if self.bm25.k1 <= 0.0 {
            report.add_error("search.bm25.k1", "k1 parameter must be > 0");
        }
        
        if self.bm25.b < 0.0 || self.bm25.b > 1.0 {
            report.add_error("search.bm25.b", "b parameter must be between 0 and 1");
        }
    }
}
```

## Testing Configuration

### 1. **Isolated Test Configuration**

Create completely isolated test configurations that don't interfere with each other:

```rust
// tests/helpers/config.rs
use embed_search::config::unified::*;
use tempfile::TempDir;

/// Create isolated test configuration with temporary directories
pub fn create_test_config() -> (Config, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path().to_path_buf();
    
    let config = Config {
        project: ProjectConfig {
            project_path: temp_path.clone(),
            include_test_files: true,
            enable_git_watch: false,
            git_poll_interval_secs: 30,
        },
        storage: StorageConfig {
            backend: StorageBackend::Memory,
            vector_db_path: temp_path.join("vectors.db"),
            cache_dir: temp_path.join("cache"),
            max_connections: 4,
            connection_timeout_ms: 5000,
            cache_size: 1000,
        },
        embedding: EmbeddingConfig {
            #[cfg(feature = "ml")]
            model_name: "test-model".to_string(),
            #[cfg(feature = "ml")]
            embedding_dimensions: 384,
            #[cfg(feature = "ml")]
            embedding_cache_size: 100,
            batch_size: 16,
            chunk_size: 200,
            enable_stemming: false,
            enable_ngrams: false,
            max_ngram_size: 2,
        },
        search: SearchConfig {
            backends: SearchBackends {
                enable_bm25: true,
                enable_tantivy: false,
                enable_semantic: false,
                enable_tree_sitter: false,
            },
            max_search_results: 10,
            bm25: BM25Config {
                k1: 1.2,
                b: 0.75,
                index_path: temp_path.join("bm25_index"),
                cache_size: 1000,
                min_term_length: 2,
                max_term_length: 30,
                stop_words: vec!["the".to_string(), "and".to_string()],
            },
            fusion: FusionConfig {
                exact_weight: 0.4,
                bm25_weight: 0.6,
                semantic_weight: 0.0,
                symbol_weight: 0.0,
            },
        },
        logging: LoggingConfig {
            level: "info".to_string(),
            format: LogFormat::Text,
            output: LogOutput::Stderr,
        },
        performance: PerformanceConfig {
            search_threads: 2,
            indexing_threads: 2,
            max_memory_mb: 512,
            search_timeout_secs: 10,
            indexing_timeout_secs: 30,
        },
    };
    
    (config, temp_dir)
}

/// Create test application context
pub fn create_test_context() -> (AppContext, TempDir) {
    let (config, temp_dir) = create_test_config();
    let context = AppContext::new(config);
    (context, temp_dir)
}

/// Create test configuration with specific overrides
pub fn create_test_config_with<F>(modifier: F) -> (Config, TempDir)
where
    F: FnOnce(&mut Config),
{
    let (mut config, temp_dir) = create_test_config();
    modifier(&mut config);
    config.validate_all()
        .expect("Modified test config must be valid");
    (config, temp_dir)
}
```

### 2. **Configuration Test Macros**

Create macros to simplify common test patterns:

```rust
// tests/helpers/macros.rs
/// Test a configuration scenario with automatic cleanup
#[macro_export]
macro_rules! test_with_config {
    ($config_modifier:expr, $test_body:expr) => {
        let (_config, _temp_dir) = create_test_config_with($config_modifier);
        let context = AppContext::new(_config);
        $test_body(context).await
    };
}

/// Test multiple search backends with the same test logic
#[macro_export] 
macro_rules! test_all_backends {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn test_bm25() -> Result<()> {
            test_with_config!(
                |config| {
                    config.search.backends.enable_bm25 = true;
                    config.search.backends.enable_tantivy = false;
                    config.search.backends.enable_semantic = false;
                },
                $test_body
            )
        }
        
        #[tokio::test]
        #[cfg(feature = "tantivy")]
        async fn test_tantivy() -> Result<()> {
            test_with_config!(
                |config| {
                    config.search.backends.enable_bm25 = false;
                    config.search.backends.enable_tantivy = true;
                    config.search.backends.enable_semantic = false;
                },
                $test_body
            )
        }
        
        #[tokio::test]
        #[cfg(feature = "ml")]
        async fn test_semantic() -> Result<()> {
            test_with_config!(
                |config| {
                    config.search.backends.enable_bm25 = false;
                    config.search.backends.enable_tantivy = false;
                    config.search.backends.enable_semantic = true;
                },
                $test_body
            )
        }
    };
}
```

### 3. **Example Test Usage**

```rust
// tests/search/integration_test.rs
use crate::helpers::*;

test_all_backends!(basic_search, |context: AppContext| async move {
    let session = context.create_search_session().await?;
    
    // Add test documents
    session.index_document("test.rs", "fn main() { println!(\"hello\"); }").await?;
    
    // Search
    let results = session.search("function").await?;
    assert!(!results.is_empty());
    
    Ok(())
});

#[tokio::test]
async fn test_configuration_validation() -> Result<()> {
    // Test invalid config
    let (mut config, _temp_dir) = create_test_config();
    config.storage.max_connections = 0; // Invalid
    
    let validation_result = config.validate_all();
    assert!(validation_result.is_err());
    
    let error_message = validation_result.unwrap_err().to_string();
    assert!(error_message.contains("max_connections"));
    
    Ok(())
}

#[tokio::test] 
async fn test_environment_variable_override() -> Result<()> {
    std::env::set_var("EMBED_STORAGE_MAX_CONNECTIONS", "20");
    
    // Config loaded from environment should override file values
    let config = Config::load()?;
    assert_eq!(config.storage.max_connections, 20);
    
    std::env::remove_var("EMBED_STORAGE_MAX_CONNECTIONS");
    Ok(())
}
```

## Implementation Timeline

- **Week 1**: Create unified configuration structures and builder pattern
- **Week 2**: Implement AppContext and migrate core services  
- **Week 3**: Update all tests to use new configuration system
- **Week 4**: Remove legacy configuration and deploy to production

## Benefits

1. **Eliminates Circular Dependencies**: No more global state initialization races
2. **Explicit Configuration**: All configuration values must be explicitly provided
3. **Better Testing**: Isolated test configurations prevent interference
4. **Type Safety**: Comprehensive validation with detailed error messages
5. **Maintainability**: Single source of truth for configuration structure
6. **Flexibility**: Support for multiple configuration sources with clear precedence

The new architecture transforms configuration from a source of bugs into a robust, maintainable foundation for the application.