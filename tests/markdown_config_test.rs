use embed_search::config::Config;

#[test]
fn test_markdown_extensions_included() {
    let config = Config::default();
    
    // Verify that both "md" and "markdown" extensions are supported
    assert!(config.indexing.supported_extensions.contains(&"md".to_string()),
           "Configuration should include 'md' extension");
    
    assert!(config.indexing.supported_extensions.contains(&"markdown".to_string()),
           "Configuration should include 'markdown' extension");
    
    // Verify total number of extensions is correct (11 total now)
    assert_eq!(config.indexing.supported_extensions.len(), 11,
              "Should have 11 supported extensions including markdown files");
}

#[test]
fn test_markdown_extensions_order() {
    let config = Config::default();
    let extensions = &config.indexing.supported_extensions;
    
    // Find positions of markdown extensions
    let md_pos = extensions.iter().position(|x| x == "md");
    let markdown_pos = extensions.iter().position(|x| x == "markdown");
    
    assert!(md_pos.is_some(), "md extension should be present");
    assert!(markdown_pos.is_some(), "markdown extension should be present");
    
    // Verify they are at the end as expected
    assert_eq!(md_pos.unwrap(), 9, "md should be at position 9");
    assert_eq!(markdown_pos.unwrap(), 10, "markdown should be at position 10");
}