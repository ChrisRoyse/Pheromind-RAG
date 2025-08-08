use embed_search::watcher::{GitWatcher, EdgeCaseHandler};
use embed_search::search::unified::UnifiedSearcher;
use embed_search::config::Config;
use std::sync::{Arc, RwLock};
use std::fs;
use std::io::Write;
use tempfile::TempDir;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Git Watcher Edge Case Demo\n");
    println!("This demo shows how the watcher handles various edge cases with proper error messages.\n");
    
    // Create a test directory
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();
    
    // Initialize git repo
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(&repo_path)
        .output()?;
    
    // Create config and searcher
    let config = Config::new_test_config();
    let searcher = Arc::new(RwLock::new(UnifiedSearcher::new(config)?));
    
    // Initialize watcher
    let mut watcher = GitWatcher::new(repo_path, searcher)?;
    
    println!("📁 Starting watcher on: {:?}\n", repo_path);
    watcher.start_watching()?;
    
    // Demo 1: Large file
    println!("❌ Demo 1: Large File Detection");
    let large_file = repo_path.join("large.rs");
    let mut file = fs::File::create(&large_file)?;
    
    // Create 101MB file
    println!("Creating 101MB file...");
    let chunk = vec![b'a'; 1024 * 1024]; // 1MB chunk
    for _ in 0..101 {
        file.write_all(&chunk)?;
    }
    drop(file);
    
    // Try to validate - will show error
    match EdgeCaseHandler::validate_file(&large_file) {
        Ok(_) => println!("✅ File validated"),
        Err(e) => println!("{}\n", e),
    }
    fs::remove_file(large_file)?;
    
    // Demo 2: Binary file
    println!("❌ Demo 2: Binary File Detection");
    let binary_file = repo_path.join("binary.rs");
    fs::write(&binary_file, b"fn main() {\x00\x01\x02\x03 binary content }")?;
    
    match EdgeCaseHandler::validate_file(&binary_file) {
        Ok(_) => println!("✅ File validated"),
        Err(e) => println!("{}\n", e),
    }
    fs::remove_file(binary_file)?;
    
    // Demo 3: Unicode paths
    println!("✅ Demo 3: Unicode Path Handling");
    let unicode_file = repo_path.join("测试文件🦀.rs");
    fs::write(&unicode_file, "fn main() { println!(\"Unicode works!\"); }")?;
    
    match EdgeCaseHandler::validate_file(&unicode_file) {
        Ok(_) => println!("✅ Unicode file validated successfully: {:?}\n", unicode_file),
        Err(e) => println!("{}\n", e),
    }
    
    // Demo 4: Minified files (should be skipped)
    println!("⚠️  Demo 4: Minified File Filtering");
    let minified = repo_path.join("app.min.js");
    fs::write(&minified, "var a=1;function b(){}")?;
    
    if GitWatcher::is_code_file(&minified) {
        println!("❌ Minified file incorrectly accepted");
    } else {
        println!("✅ Minified file correctly skipped: {:?}\n", minified);
    }
    
    // Demo 5: Check disk space
    println!("✅ Demo 5: Disk Space Check");
    match EdgeCaseHandler::check_disk_space(repo_path) {
        Ok(_) => println!("✅ Sufficient disk space available\n"),
        Err(e) => println!("{}\n", e),
    }
    
    // Demo 6: Path normalization
    println!("✅ Demo 6: Path Normalization");
    let weird_path = repo_path.join("./dir/../file.rs");
    let normalized = EdgeCaseHandler::normalize_path(&weird_path);
    println!("Original: {:?}", weird_path);
    println!("Normalized: {:?}\n", normalized);
    
    // Demo 7: Error tracking
    println!("📊 Demo 7: Error Tracking");
    println!("Current error count: {}", watcher.get_error_count());
    
    // Create a file that will trigger validation errors
    let problem_file = repo_path.join("problem.rs");
    
    // Write a file with control characters
    let mut content = String::from("fn main() {");
    for _ in 0..100 {
        content.push('\x01');
    }
    content.push_str("}");
    fs::write(&problem_file, content)?;
    
    // Wait for watcher to process
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    println!("Error count after problematic file: {}\n", watcher.get_error_count());
    
    // Demo 8: Symlink detection
    #[cfg(unix)]
    {
        println!("❌ Demo 8: Symlink Detection");
        let real_file = repo_path.join("real.rs");
        let symlink = repo_path.join("link.rs");
        
        fs::write(&real_file, "fn main() {}")?;
        std::os::unix::fs::symlink(&real_file, &symlink)?;
        
        match EdgeCaseHandler::validate_file(&symlink) {
            Ok(_) => println!("✅ Symlink validated"),
            Err(e) => println!("{}\n", e),
        }
    }
    
    // Demo 9: File with retry
    println!("✅ Demo 9: File Read with Retry");
    let normal_file = repo_path.join("normal.rs");
    fs::write(&normal_file, "fn main() { println!(\"Hello!\"); }")?;
    
    match EdgeCaseHandler::read_file_with_retry(&normal_file) {
        Ok(content) => println!("✅ Successfully read file: {} bytes\n", content.len()),
        Err(e) => println!("{}\n", e),
    }
    
    // Stop watcher
    watcher.stop_watching();
    
    println!("✅ Demo complete! The watcher handled all edge cases with proper error messages.");
    println!("\nKey features demonstrated:");
    println!("  • Large file detection (>100MB)");
    println!("  • Binary file detection");
    println!("  • Unicode path support");
    println!("  • Minified file filtering");
    println!("  • Disk space checking");
    println!("  • Path normalization");
    println!("  • Error tracking");
    println!("  • Symlink detection (Unix)");
    println!("  • File read with retry");
    
    Ok(())
}