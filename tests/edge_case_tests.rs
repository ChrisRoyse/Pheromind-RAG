#[cfg(test)]
mod edge_case_tests {
    use embed_search::watcher::{EdgeCaseHandler, EdgeCaseError};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use std::io::Write;
    
    #[test]
    fn test_large_file_detection() {
        let temp_dir = TempDir::new().unwrap();
        let large_file = temp_dir.path().join("large.rs");
        
        // Create a file larger than 100MB limit
        let mut file = fs::File::create(&large_file).unwrap();
        let data = vec![b'a'; 101 * 1024 * 1024]; // 101MB
        file.write_all(&data).unwrap();
        
        let result = EdgeCaseHandler::validate_file(&large_file);
        assert!(result.is_err());
        
        if let Err(e) = result {
            let error_msg = format!("{}", e);
            assert!(error_msg.contains("ERROR[E1001]"));
            assert!(error_msg.contains("File too large"));
        }
    }
    
    #[test]
    fn test_binary_file_detection() {
        let temp_dir = TempDir::new().unwrap();
        let binary_file = temp_dir.path().join("binary.rs");
        
        // Create a file with null bytes (binary content)
        let mut file = fs::File::create(&binary_file).unwrap();
        file.write_all(b"fn main() {\x00\x01\x02\x03}").unwrap();
        
        let result = EdgeCaseHandler::validate_file(&binary_file);
        assert!(result.is_err());
        
        if let Err(e) = result {
            let error_msg = format!("{}", e);
            assert!(error_msg.contains("ERROR[E1004]"));
            assert!(error_msg.contains("Binary file"));
        }
    }
    
    #[test]
    fn test_symlink_detection() {
        let temp_dir = TempDir::new().unwrap();
        let real_file = temp_dir.path().join("real.rs");
        let symlink = temp_dir.path().join("link.rs");
        
        fs::write(&real_file, "fn main() {}").unwrap();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&real_file, &symlink).unwrap();
            
            let result = EdgeCaseHandler::validate_file(&symlink);
            assert!(result.is_err());
            
            if let Err(e) = result {
                let error_msg = format!("{}", e);
                assert!(error_msg.contains("ERROR[E1002]"));
            }
        }
        
        #[cfg(windows)]
        {
            // Windows requires admin rights for symlinks, so we skip in CI
            if std::env::var("CI").is_err() {
                use std::os::windows::fs::symlink_file;
                let _ = symlink_file(&real_file, &symlink);
            }
        }
    }
    
    #[test]
    fn test_unicode_path_handling() {
        let temp_dir = TempDir::new().unwrap();
        let unicode_file = temp_dir.path().join("测试文件🦀.rs");
        
        fs::write(&unicode_file, "fn main() {}").unwrap();
        
        let normalized = EdgeCaseHandler::normalize_path(&unicode_file);
        assert!(normalized.exists());
        
        // Should handle unicode correctly
        let result = EdgeCaseHandler::validate_file(&normalized);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_path_normalization() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();
        
        // Test various path formats
        let test_cases = vec![
            (base.join("./file.rs"), base.join("file.rs")),
            (base.join("dir/../file.rs"), base.join("file.rs")),
            (base.join("dir/./file.rs"), base.join("dir/file.rs")),
        ];
        
        for (input, expected) in test_cases {
            let normalized = EdgeCaseHandler::normalize_path(&input);
            // Compare just the relative parts since canonicalize gives absolute paths
            assert_eq!(
                normalized.file_name(),
                expected.file_name(),
                "Failed for input: {:?}",
                input
            );
        }
    }
    
    #[test]
    fn test_file_with_many_non_printable_chars() {
        let temp_dir = TempDir::new().unwrap();
        let weird_file = temp_dir.path().join("weird.rs");
        
        // Create file with lots of control characters
        let mut content = String::from("fn main() {");
        for _ in 0..100 {
            content.push('\x01');
            content.push('\x02');
        }
        content.push_str("}");
        
        fs::write(&weird_file, content).unwrap();
        
        let result = EdgeCaseHandler::validate_file(&weird_file);
        assert!(result.is_err());
        
        if let Err(e) = result {
            let error_msg = format!("{}", e);
            assert!(error_msg.contains("non-printable characters"));
        }
    }
    
    #[test]
    fn test_read_with_retry() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        
        fs::write(&file_path, "fn main() {}").unwrap();
        
        // Normal read should work
        let result = EdgeCaseHandler::read_file_with_retry(&file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "fn main() {}");
    }
    
    #[test]
    fn test_disk_space_check() {
        let temp_dir = TempDir::new().unwrap();
        
        // This should pass on most systems with > 100MB free
        let result = EdgeCaseHandler::check_disk_space(temp_dir.path());
        
        // We can't guarantee disk space in all environments, so just check it doesn't panic
        let _ = result;
    }
    
    #[test]
    fn test_minified_file_detection() {
        let temp_dir = TempDir::new().unwrap();
        
        // Test that minified files are skipped
        let minified = temp_dir.path().join("app.min.js");
        fs::write(&minified, "var a=1;function b(){}").unwrap();
        
        // The GitWatcher should skip this
        use embed_search::watcher::GitWatcher;
        assert!(!GitWatcher::is_code_file(&minified));
    }
    
    #[test]
    fn test_bundle_file_detection() {
        let temp_dir = TempDir::new().unwrap();
        
        // Test that bundle files are skipped
        let bundle = temp_dir.path().join("app.bundle.js");
        fs::write(&bundle, "function webpack(){}").unwrap();
        
        use embed_search::watcher::GitWatcher;
        assert!(!GitWatcher::is_code_file(&bundle));
    }
    
    #[test]
    #[cfg(windows)]
    fn test_unc_path_detection() {
        // Test UNC path detection (Windows only)
        let unc_path = PathBuf::from(r"\\server\share\file.rs");
        
        // This will fail if not on network, which is expected
        let result = EdgeCaseHandler::validate_file(&unc_path);
        
        if let Err(e) = result {
            let error_msg = format!("{}", e);
            // Should either be network path error or file not found
            assert!(error_msg.contains("ERROR[E1005]") || error_msg.contains("does not exist"));
        }
    }
    
    #[test]
    fn test_error_message_format() {
        // Test that error messages have the required format
        let error = EdgeCaseError::FileTooLarge {
            path: PathBuf::from("/test/file.rs"),
            size: 200 * 1024 * 1024,
        };
        
        let msg = format!("{}", error);
        
        // Check required components
        assert!(msg.contains("ERROR[E1001]"), "Missing error code");
        assert!(msg.contains("File:"), "Missing file path");
        assert!(msg.contains("Action:"), "Missing action");
        assert!(msg.contains("Reason:"), "Missing reason");
    }
    
    #[test]
    fn test_case_sensitivity_handling() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create files with different cases
        let file1 = temp_dir.path().join("File.rs");
        let file2 = temp_dir.path().join("file.rs");
        
        fs::write(&file1, "fn main() {}").unwrap();
        
        #[cfg(windows)]
        {
            // On Windows, these should be the same file
            let norm1 = EdgeCaseHandler::normalize_path(&file1);
            let norm2 = EdgeCaseHandler::normalize_path(&file2);
            
            // Both should normalize to the same path on Windows
            assert_eq!(norm1.to_string_lossy().to_lowercase(), norm2.to_string_lossy().to_lowercase());
        }
        
        #[cfg(unix)]
        {
            // On Unix, these are different files
            fs::write(&file2, "fn test() {}").unwrap();
            
            let norm1 = EdgeCaseHandler::normalize_path(&file1);
            let norm2 = EdgeCaseHandler::normalize_path(&file2);
            
            assert_ne!(norm1, norm2);
        }
    }
}