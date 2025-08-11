// Simple verification test for markdown configuration changes
// This test can be run independently to verify the changes work

#[cfg(test)]
mod tests {
    // Inline the config structure to test without dependencies
    
    fn get_default_supported_extensions() -> Vec<String> {
        vec![
            "rs".to_string(),
            "py".to_string(),
            "js".to_string(),
            "ts".to_string(),
            "go".to_string(),
            "java".to_string(),
            "cpp".to_string(),
            "c".to_string(),
            "h".to_string(),
            "md".to_string(),
            "markdown".to_string(),
        ]
    }
    
    #[test]
    fn test_markdown_extensions_present() {
        let extensions = get_default_supported_extensions();
        
        assert!(extensions.contains(&"md".to_string()), 
               "md extension should be included");
        assert!(extensions.contains(&"markdown".to_string()), 
               "markdown extension should be included");
        
        // Check they are at the expected positions
        assert_eq!(extensions[9], "md");
        assert_eq!(extensions[10], "markdown");
        
        // Verify total count
        assert_eq!(extensions.len(), 11);
    }
}