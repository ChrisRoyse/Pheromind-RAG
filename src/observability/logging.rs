use tracing::Level;
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use std::io;

/// Configuration for logging setup
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

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            enable_colors: true,
            enable_timestamps: true,
            show_target: false,
            show_thread_ids: false,
            json_format: false,
            filter: None,
        }
    }
}

impl LogConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    pub fn debug() -> Self {
        Self::new().level(Level::DEBUG)
    }

    pub fn trace() -> Self {
        Self::new().level(Level::TRACE)
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
pub fn init_logging(config: LogConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Build the env filter
    let env_filter = if let Some(filter) = &config.filter {
        EnvFilter::try_new(filter)?
    } else {
        // Try to read from environment variable first
        match EnvFilter::try_from_default_env() {
            Ok(env_filter) => env_filter,
            Err(_) => {
                // No RUST_LOG environment variable set - use configured level
                EnvFilter::new(config.level.to_string().to_lowercase())
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
            .try_init()?;
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
            .try_init()?;
    }

    Ok(())
}

/// Initialize logging with default configuration
pub fn init_default_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_logging(LogConfig::default())
}

/// Initialize development logging (debug level with colors)
pub fn init_dev_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_logging(
        LogConfig::debug()
            .colors(true)
            .timestamps(true)
            .show_target(true)
    )
}

/// Initialize production logging (info level, JSON format)
pub fn init_prod_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_logging(
        LogConfig::new()
            .level(Level::INFO)
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
        let config = LogConfig::new()
            .level(Level::DEBUG)
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
        // Initialize logging for testing
        let _ = init_logging(LogConfig::new().level(Level::DEBUG));

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
        // Initialize logging for testing
        let _ = init_logging(LogConfig::new().level(Level::DEBUG));

        log_search_operation(
            "test query",
            5,
            std::time::Duration::from_millis(100),
            "test_source"
        );

        log_embedding_operation(
            256,
            std::time::Duration::from_millis(50),
            true,
            Some(384)
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