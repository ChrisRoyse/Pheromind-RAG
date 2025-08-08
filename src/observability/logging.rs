use tracing::Level;
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use std::io;
use crate::error::LoggingError;

/// Configuration for logging setup
/// 
/// **IMPORTANT**: This struct requires explicit configuration - no defaults are provided.
/// Either provide an explicit filter via `filter()` method or set the RUST_LOG environment variable.
/// 
/// # Example
/// 
/// ```
/// use tracing::Level;
/// use your_crate::observability::logging::LogConfig;
/// 
/// // Explicit configuration required
/// let config = LogConfig::new(Level::INFO)
///     .filter("your_crate=debug")
///     .colors(true);
/// ```
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub level: Level,
    pub enable_colors: bool,
    pub enable_timestamps: bool,
    pub show_target: bool,
    pub show_thread_ids: bool,
    pub json_format: bool,
    pub filter: Option<String>,
}


impl LogConfig {
    pub fn new(level: Level) -> Self {
        Self {
            level,
            enable_colors: true,
            enable_timestamps: true,
            show_target: false,
            show_thread_ids: false,
            json_format: false,
            filter: None,
        }
    }

    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    pub fn debug() -> Self {
        Self::new(Level::DEBUG)
    }

    pub fn trace() -> Self {
        Self::new(Level::TRACE)
    }

    pub fn colors(mut self, enable: bool) -> Self {
        self.enable_colors = enable;
        self
    }

    pub fn timestamps(mut self, enable: bool) -> Self {
        self.enable_timestamps = enable;
        self
    }

    pub fn show_target(mut self, show: bool) -> Self {
        self.show_target = show;
        self
    }

    pub fn show_thread_ids(mut self, show: bool) -> Self {
        self.show_thread_ids = show;
        self
    }

    pub fn json_format(mut self, enable: bool) -> Self {
        self.json_format = enable;
        self
    }

    pub fn filter<S: Into<String>>(mut self, filter: S) -> Self {
        self.filter = Some(filter.into());
        self
    }
}

/// Initialize logging with the given configuration
/// 
/// **IMPORTANT**: No fallback behavior is provided. This function will fail explicitly if:
/// - No filter is provided via `LogConfig::filter()` AND
/// - No RUST_LOG environment variable is set
/// 
/// This ensures all logging configuration is explicit and intentional.
pub fn init_logging(config: LogConfig) -> Result<(), LoggingError> {
    // Build the env filter - require explicit configuration
    let env_filter = if let Some(filter) = &config.filter {
        EnvFilter::try_new(filter).map_err(|e| LoggingError::InvalidFilter {
            filter: filter.clone(),
            source: Some(Box::new(e)),
        })?
    } else {
        // Check if RUST_LOG environment variable is set
        match std::env::var("RUST_LOG") {
            Ok(env_filter_str) => EnvFilter::try_new(&env_filter_str).map_err(|_e| {
                LoggingError::EnvironmentError {
                    variable: "RUST_LOG".to_string(),
                    value: Some(env_filter_str.clone()),
                }
            })?,
            Err(_) => {
                return Err(LoggingError::InitializationFailed {
                    reason: "No logging filter configured".to_string(),
                    config_detail: Some("Either provide explicit filter via LogConfig::filter() or set RUST_LOG environment variable".to_string()),
                });
            }
        }
    };

    if config.json_format {
        // JSON format for structured logging
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .json()
                    .with_current_span(true)
                    .with_writer(io::stdout)
            )
            .try_init()
            .map_err(|e| LoggingError::InitializationFailed {
                reason: "Failed to initialize JSON logger".to_string(),
                config_detail: Some(format!("Error: {}", e)),
            })?;
    } else {
        // Pretty format for human-readable logging - use default timer to avoid type issues
        let fmt_layer = fmt::layer()
            .with_ansi(config.enable_colors)
            .with_target(config.show_target)
            .with_thread_ids(config.show_thread_ids)
            .with_writer(io::stdout);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .try_init()
            .map_err(|e| LoggingError::InitializationFailed {
                reason: "Failed to initialize formatted logger".to_string(),
                config_detail: Some(format!("Error: {}", e)),
            })?;
    }

    Ok(())
}


/// Initialize development logging (debug level with colors)
pub fn init_dev_logging() -> Result<(), LoggingError> {
    init_logging(
        LogConfig::debug()
            .colors(true)
            .timestamps(true)
            .show_target(true)
    )
}

/// Initialize production logging (info level, JSON format)
pub fn init_prod_logging() -> Result<(), LoggingError> {
    init_logging(
        LogConfig::new(Level::INFO)
            .json_format(true)
            .colors(false)
            .timestamps(true)
    )
}

/// Macro to log performance of a function
#[macro_export]
macro_rules! log_performance {
    ($operation:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        tracing::debug!("{} completed in {:.3}s", $operation, duration.as_secs_f64());
        result
    }};
}

/// Macro to log and measure async performance
#[macro_export]
macro_rules! log_async_performance {
    ($operation:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block.await;
        let duration = start.elapsed();
        tracing::debug!("{} completed in {:.3}s", $operation, duration.as_secs_f64());
        result
    }};
}

/// Structured logging helper for search operations
pub fn log_search_operation(
    query: &str,
    result_count: usize,
    duration: std::time::Duration,
    source: &str,
) {
    tracing::info!(
        query = %query,
        result_count = result_count,
        duration_ms = duration.as_millis(),
        source = source,
        "Search operation completed"
    );
}

/// Structured logging helper for embedding operations
#[cfg(feature = "ml")]
pub fn log_embedding_operation(
    text_length: usize,
    duration: std::time::Duration,
    from_cache: bool,
    embedding_dimension: Option<usize>,
) {
    tracing::info!(
        text_length = text_length,
        duration_ms = duration.as_millis(),
        from_cache = from_cache,
        embedding_dimension = embedding_dimension,
        "Embedding operation completed"
    );
}

/// Structured logging helper for cache operations
pub fn log_cache_operation(
    cache_name: &str,
    operation: &str,
    hit: bool,
    size: Option<usize>,
    capacity: Option<usize>,
) {
    tracing::debug!(
        cache_name = cache_name,
        operation = operation,
        hit = hit,
        size = size,
        capacity = capacity,
        "Cache operation"
    );
}

/// Structured logging helper for system metrics
pub fn log_system_metrics(
    memory_usage_mb: u64,
    memory_pressure: &str,
    cache_hit_rate: f64,
    active_searches: usize,
) {
    tracing::info!(
        memory_usage_mb = memory_usage_mb,
        memory_pressure = memory_pressure,
        cache_hit_rate = cache_hit_rate,
        active_searches = active_searches,
        "System metrics"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::{info, debug, error};

    #[test]
    fn test_log_config_builder() {
        let config = LogConfig::new(Level::DEBUG)
            .colors(false)
            .json_format(true)
            .filter("embed_search=debug");

        assert_eq!(config.level, Level::DEBUG);
        assert!(!config.enable_colors);
        assert!(config.json_format);
        assert_eq!(config.filter, Some("embed_search=debug".to_string()));
    }

    #[test]
    fn test_preset_configs() {
        let debug_config = LogConfig::debug();
        assert_eq!(debug_config.level, Level::DEBUG);

        let trace_config = LogConfig::trace();
        assert_eq!(trace_config.level, Level::TRACE);
    }

    #[tokio::test]
    async fn test_logging_macros() {
        // Skip initialization if already done - common in test environments
        let _ = init_logging(LogConfig::new(Level::DEBUG).filter("test=debug"));
        // Test will continue regardless of initialization status

        // Test sync performance macro
        let result = log_performance!("test_operation", {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });
        assert_eq!(result, 42);

        // Test async performance macro
        let result = log_async_performance!("async_test_operation", {
            tokio::time::sleep(std::time::Duration::from_millis(10))
        });
        // Should complete without panicking
    }

    #[test]
    fn test_structured_logging_helpers() {
        // Skip initialization if already done - common in test environments
        let _ = init_logging(LogConfig::new(Level::DEBUG).filter("test=debug"));

        log_search_operation(
            "test query",
            5,
            std::time::Duration::from_millis(100),
            "test_source"
        );

        // Test embedding operation logging (when ml feature is available)
        #[cfg(feature = "ml")]
        log_embedding_operation(
            256,
            std::time::Duration::from_millis(50),
            true,
            Some(768)
        );

        log_cache_operation(
            "embedding_cache",
            "get",
            true,
            Some(100),
            Some(1000)
        );

        log_system_metrics(
            512,
            "low",
            0.85,
            3
        );

        // Should complete without panicking
    }
}