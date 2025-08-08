use embed_search::config::{Config, SearchBackend};
use std::str::FromStr;

#[test]
fn test_search_backend_parsing() {
    // Test case-insensitive parsing
    assert_eq!(SearchBackend::from_str("tantivy").unwrap(), SearchBackend::Tantivy);
    assert_eq!(SearchBackend::from_str("TANTIVY").unwrap(), SearchBackend::Tantivy);
    assert_eq!(SearchBackend::from_str("Tantivy").unwrap(), SearchBackend::Tantivy);
    assert_eq!(SearchBackend::from_str("TaNtIvY").unwrap(), SearchBackend::Tantivy);
    
    // Test invalid parsing
    assert!(SearchBackend::from_str("invalid").is_err());
    assert!(SearchBackend::from_str("invalid_backend").is_err());
    assert!(SearchBackend::from_str("unsupported").is_err());
    assert!(SearchBackend::from_str("ripgrep").is_err()); // Ensure legacy backend is rejected
}

#[test]
fn test_config_serialization_deserialization() {
    // Test with new search_backend field and minimal required fields
    let config_toml = r#"
project_path = "."
chunk_size = 200
embedding_cache_size = 5000
search_cache_size = 50
batch_size = 16
vector_db_path = "test_db"
cache_dir = "test_cache"
git_poll_interval_secs = 10
enable_git_watch = false
include_test_files = true
max_search_results = 50
search_backend = "tantivy"
model_name = "test-model"
embedding_dimensions = 768
log_level = "debug"
bm25_enabled = false
bm25_k1 = 1.0
bm25_b = 0.5
bm25_index_path = "test_bm25"
bm25_cache_size = 1000
bm25_min_term_length = 1
bm25_max_term_length = 20
bm25_stop_words = ["test"]
fusion_exact_weight = 0.3
fusion_bm25_weight = 0.2
fusion_semantic_weight = 0.3
fusion_symbol_weight = 0.2
enable_stemming = false
enable_ngrams = false
max_ngram_size = 2
"#;
    
    let config: Config = toml::from_str(config_toml).expect("Failed to parse config");
    assert_eq!(config.chunk_size, 200);
    assert_eq!(config.search_backend, SearchBackend::Tantivy);
    assert_eq!(config.max_search_results, 50);
    #[cfg(feature = "ml")]
    assert_eq!(config.embedding_cache_size, 5000);
}

#[test]
fn test_legacy_compatibility() {
    // Test with legacy configuration format - only essential fields to test backward compatibility
    let mut config = Config::new_test_config();
    
    // Override specific fields like an old config file might
    config.chunk_size = 150;
    config.search_backend = SearchBackend::Tantivy;
    config.max_search_results = 30;
    
    // Test serialization and deserialization
    let serialized = toml::to_string(&config).expect("Failed to serialize config");
    let deserialized: Config = toml::from_str(&serialized).expect("Failed to deserialize config");
    
    assert_eq!(deserialized.chunk_size, 150);
    assert_eq!(deserialized.search_backend, SearchBackend::Tantivy);
    assert_eq!(deserialized.max_search_results, 30);
}

#[test]
fn test_config_without_search_backend() {
    // Test config without explicit search_backend field uses default
    let mut base_config = Config::new_test_config();
    base_config.chunk_size = 100;
    base_config.max_search_results = 20;
    
    // Serialize config (without explicitly setting search_backend)
    let serialized = toml::to_string(&base_config).expect("Failed to serialize config");
    
    // Deserialize should use default search_backend
    let deserialized: Config = toml::from_str(&serialized).expect("Failed to parse config");
    assert_eq!(deserialized.chunk_size, 100);
    assert_eq!(deserialized.search_backend, SearchBackend::Tantivy); // Should use default
    assert_eq!(deserialized.max_search_results, 20);
}

#[test]
fn test_search_backend_display() {
    // Tantivy backend display format
    assert_eq!(SearchBackend::Tantivy.to_string(), "Tantivy");
}