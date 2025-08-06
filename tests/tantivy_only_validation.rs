// Validation tests to ensure only Tantivy search backend is available
// and that all ripgrep functionality has been removed

use embed_search::config::{Config, SearchBackend};
use std::str::FromStr;

#[test]
fn test_only_tantivy_backend_exists() {
    // Test that Tantivy is the only valid backend
    assert_eq!(SearchBackend::from_str("tantivy").unwrap(), SearchBackend::Tantivy);
    assert_eq!(SearchBackend::from_str("TANTIVY").unwrap(), SearchBackend::Tantivy);
    
    // Test that legacy backends are rejected
    assert!(SearchBackend::from_str("ripgrep").is_err());
    assert!(SearchBackend::from_str("auto").is_err());
    assert!(SearchBackend::from_str("native").is_err());
    
    // Test that invalid backends are rejected
    assert!(SearchBackend::from_str("invalid").is_err());
    assert!(SearchBackend::from_str("").is_err());
}

#[test] 
fn test_default_backend_is_tantivy() {
    let config = Config::default();
    assert_eq!(config.search_backend, SearchBackend::Tantivy);
}

#[test]
fn test_config_parsing_rejects_invalid_backends() {
    // Test parsing config with invalid backend
    let config_toml_invalid = r#"
chunk_size = 100
search_backend = "ripgrep"
"#;
    
    // This should fail to parse
    let result: Result<Config, _> = toml::from_str(config_toml_invalid);
    assert!(result.is_err());
    
    // Test parsing config with auto backend
    let config_toml_auto = r#"
chunk_size = 100  
search_backend = "auto"
"#;
    
    // This should also fail to parse
    let result: Result<Config, _> = toml::from_str(config_toml_auto);
    assert!(result.is_err());
}

#[test]
fn test_config_parsing_accepts_tantivy() {
    // Test that a config can be parsed and the backend is properly set
    let mut config = Config::default();
    config.search_backend = SearchBackend::Tantivy;
    config.chunk_size = 100;
    
    // Test serialization and deserialization
    let serialized = toml::to_string(&config).expect("Should serialize config");
    let parsed: Config = toml::from_str(&serialized).expect("Should parse valid config");
    
    assert_eq!(parsed.search_backend, SearchBackend::Tantivy);
    assert_eq!(parsed.chunk_size, 100);
}

#[test]
fn test_search_backend_display() {
    assert_eq!(SearchBackend::Tantivy.to_string(), "Tantivy");
}

#[test]
fn test_search_backend_serialization() {
    let backend = SearchBackend::Tantivy;
    let serialized = serde_json::to_string(&backend).unwrap();
    assert_eq!(serialized, "\"Tantivy\"");
    
    let deserialized: SearchBackend = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, SearchBackend::Tantivy);
}