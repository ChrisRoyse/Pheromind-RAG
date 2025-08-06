use embed_search::config::{Config, SearchBackend};
use std::str::FromStr;

#[test]
fn test_search_backend_parsing() {
    // Test valid parsing
    assert_eq!(SearchBackend::from_str("ripgrep").unwrap(), SearchBackend::Ripgrep);
    assert_eq!(SearchBackend::from_str("TANTIVY").unwrap(), SearchBackend::Tantivy);
    assert_eq!(SearchBackend::from_str("Auto").unwrap(), SearchBackend::Auto);
    
    // Test invalid parsing
    assert!(SearchBackend::from_str("invalid").is_err());
    assert!(SearchBackend::from_str("grep").is_err());
}

#[test]
fn test_config_serialization_deserialization() {
    // Test with new search_backend field
    let config_toml = r#"
chunk_size = 200
search_backend = "tantivy"
max_search_results = 50
"#;
    
    let config: Config = toml::from_str(config_toml).expect("Failed to parse config");
    assert_eq!(config.chunk_size, 200);
    assert_eq!(config.search_backend, SearchBackend::Tantivy);
    assert_eq!(config.max_search_results, 50);
    assert_eq!(config.ripgrep_fallback, None);
}

#[test]
fn test_legacy_compatibility() {
    // Test with legacy ripgrep_fallback setting
    let legacy_config_toml = r#"
chunk_size = 150
search_backend = "auto"
ripgrep_fallback = false
max_search_results = 30
"#;
    
    let config: Config = toml::from_str(legacy_config_toml).expect("Failed to parse legacy config");
    assert_eq!(config.chunk_size, 150);
    assert_eq!(config.search_backend, SearchBackend::Auto);
    assert_eq!(config.max_search_results, 30);
    assert_eq!(config.ripgrep_fallback, Some(false));
}

#[test]
fn test_config_without_search_backend() {
    // Test config without search_backend field uses default
    let minimal_config_toml = r#"
chunk_size = 100
max_search_results = 20
"#;
    
    let config: Config = toml::from_str(minimal_config_toml).expect("Failed to parse minimal config");
    assert_eq!(config.chunk_size, 100);
    assert_eq!(config.search_backend, SearchBackend::Auto); // Default value
    assert_eq!(config.max_search_results, 20);
    assert_eq!(config.ripgrep_fallback, None);
}

#[test]
fn test_search_backend_display() {
    assert_eq!(SearchBackend::Ripgrep.to_string(), "Ripgrep");
    assert_eq!(SearchBackend::Tantivy.to_string(), "Tantivy");
    assert_eq!(SearchBackend::Auto.to_string(), "Auto");
}