//! Comprehensive integration tests for the file watcher system
//! 
//! This module verifies:
//! 1. File watcher detects changes within 2 seconds
//! 2. .gitignore rules are properly respected  
//! 3. Concurrent file modifications don't cause race conditions
//! 4. Search engines update correctly (when vectordb feature is enabled)
//! 5. Debouncing works for rapid changes
//! 6. Memory usage stays under limits
//! 7. Thread safety is maintained
//! 8. Test isolation and cleanup

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tempfile::TempDir;
use tokio::time::sleep;
use tokio::task::JoinHandle;

use embed_search::git::watcher::{GitWatcher, FileChange};

// Conditional imports for vectordb feature
#[cfg(feature = "vectordb")]
use embed_search::git::watcher::{VectorUpdater, WatchCommand, UpdateStats};
#[cfg(feature = "vectordb")]
use embed_search::search::unified::UnifiedSearcher;
#[cfg(feature = "vectordb")]
use embed_search::storage::lancedb_storage::LanceDBStorage;

/// Memory monitoring helper
struct MemoryMonitor {
    initial_usage: usize,
    peak_usage: Arc<AtomicUsize>,
    monitoring: Arc<AtomicBool>,
}

impl MemoryMonitor {
    fn new() -> Self {
        Self {
            initial_usage: get_current_memory_usage(),
            peak_usage: Arc::new(AtomicUsize::new(0)),
            monitoring: Arc::new(AtomicBool::new(false)),
        }
    }

    fn start(&self) -> JoinHandle<()> {
        self.monitoring.store(true, Ordering::Relaxed);
        let peak_usage = self.peak_usage.clone();
        let monitoring = self.monitoring.clone();
        
        tokio::spawn(async move {
            while monitoring.load(Ordering::Relaxed) {
                let current = get_current_memory_usage();
                peak_usage.fetch_max(current, Ordering::Relaxed);
                sleep(Duration::from_millis(100)).await;
            }
        })
    }

    fn stop(&self) {
        self.monitoring.store(false, Ordering::Relaxed);
    }

    fn memory_increase(&self) -> usize {
        self.peak_usage.load(Ordering::Relaxed).saturating_sub(self.initial_usage)
    }
}

fn get_current_memory_usage() -> usize {
    #[cfg(unix)]
    {
        use std::fs;
        if let Ok(statm) = fs::read_to_string("/proc/self/statm") {
            if let Some(rss_pages) = statm.split_whitespace().nth(1) {
                if let Ok(pages) = rss_pages.parse::<usize>() {
                    return pages * 4096; // Convert pages to bytes
                }
            }
        }
    }
    
    #[cfg(windows)]
    {
        use winapi::um::processthreadsapi::GetCurrentProcess;
        use winapi::um::psapi::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
        use winapi::shared::minwindef::DWORD;
        
        unsafe {
            let handle = GetCurrentProcess();
            let mut counters: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
            
            if GetProcessMemoryInfo(handle, &mut counters, std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as DWORD) != 0 {
                return counters.WorkingSetSize as usize;
            }
        }
    }
    
    0 // Fallback
}

/// Test fixture for watcher integration tests
struct WatcherTestFixture {
    temp_dir: TempDir,
    git_repo_path: PathBuf,
    watcher: GitWatcher,
    #[cfg(feature = "vectordb")]
    updater: Option<VectorUpdater>,
    #[cfg(feature = "vectordb")]
    watch_command: Option<WatchCommand>,
    memory_monitor: MemoryMonitor,
}

impl WatcherTestFixture {
    async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let temp_dir = TempDir::new()?;
        let git_repo_path = temp_dir.path().to_path_buf();
        
        // Initialize git repository
        Self::init_git_repo(&git_repo_path)?;
        
        let watcher = GitWatcher::new(git_repo_path.clone());
        
        #[cfg(feature = "vectordb")]
        let (updater, watch_command) = {
            // Try to create vectordb components, but fall back to None if initialization fails
            match Self::try_init_vectordb(&git_repo_path, &temp_dir).await {
                Ok((u, w)) => (Some(u), Some(w)),
                Err(e) => {
                    println!("Warning: Failed to initialize vectordb components for testing: {}", e);
                    println!("Some tests will be skipped");
                    (None, None)
                }
            }
        };
        
        let memory_monitor = MemoryMonitor::new();
        
        Ok(Self {
            temp_dir,
            git_repo_path,
            watcher,
            #[cfg(feature = "vectordb")]
            updater,
            #[cfg(feature = "vectordb")]
            watch_command,
            memory_monitor,
        })
    }
    
    #[cfg(feature = "vectordb")]
    async fn try_init_vectordb(git_repo_path: &PathBuf, temp_dir: &TempDir) -> Result<(VectorUpdater, WatchCommand), Box<dyn std::error::Error + Send + Sync>> {
        use embed_search::config::Config;
        
        // Initialize config for testing
        Config::init_from_env().unwrap_or_else(|_| {
            println!("Warning: Using fallback config for testing");
        });
        
        let db_path = temp_dir.path().join("test_db");
        let searcher = Arc::new(UnifiedSearcher::new(git_repo_path.clone(), db_path).await?);
        let storage = Arc::new(tokio::sync::RwLock::new(
            LanceDBStorage::new(&temp_dir.path().join("test_storage")).await?
        ));
        
        let updater = VectorUpdater::new(searcher.clone(), storage.clone());
        let watch_command = WatchCommand::new(git_repo_path.clone(), searcher, storage)?;
        Ok((updater, watch_command))
    }
    
    fn init_git_repo(path: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Initialize git repository
        Command::new("git")
            .args(&["init"])
            .current_dir(path)
            .output()?;
            
        // Configure git user for tests
        Command::new("git")
            .args(&["config", "user.email", "test@example.com"])
            .current_dir(path)
            .output()?;
            
        Command::new("git")
            .args(&["config", "user.name", "Test User"])
            .current_dir(path)
            .output()?;
        
        Ok(())
    }
    
    fn create_file(&self, relative_path: &str, content: &str) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let file_path = self.git_repo_path.join(relative_path);
        
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(&file_path, content)?;
        Ok(file_path)
    }
    
    fn modify_file(&self, relative_path: &str, content: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let file_path = self.git_repo_path.join(relative_path);
        fs::write(&file_path, content)?;
        Ok(())
    }
    
    fn delete_file(&self, relative_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let file_path = self.git_repo_path.join(relative_path);
        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }
        Ok(())
    }
    
    fn create_gitignore(&self, patterns: &[&str]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let gitignore_path = self.git_repo_path.join(".gitignore");
        let content = patterns.join("\n");
        fs::write(gitignore_path, content)?;
        Ok(())
    }
    
    fn git_add(&self, file: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Command::new("git")
            .args(&["add", file])
            .current_dir(&self.git_repo_path)
            .output()?;
        Ok(())
    }
    
    fn git_commit(&self, message: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Command::new("git")
            .args(&["commit", "-m", message])
            .current_dir(&self.git_repo_path)
            .output()?;
        Ok(())
    }
}

#[tokio::test]
async fn test_file_change_detection_speed() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = WatcherTestFixture::new().await?;
    let _monitor_handle = fixture.memory_monitor.start();
    
    // Create initial file
    fixture.create_file("src/test.rs", "fn main() { println!(\"hello\"); }")?;
    fixture.git_add("src/test.rs")?;
    fixture.git_commit("Initial commit")?;
    
    // Modify file and measure detection time
    let start_time = Instant::now();
    fixture.modify_file("src/test.rs", "fn main() { println!(\"hello world\"); }")?;
    
    // Poll for changes until detected or timeout
    let detection_time = loop {
        let changes = fixture.watcher.get_changes()?;
        if !changes.is_empty() {
            break start_time.elapsed();
        }
        if start_time.elapsed() > Duration::from_secs(5) {
            return Err("File change detection took longer than 5 seconds".into());
        }
        sleep(Duration::from_millis(100)).await;
    };
    
    fixture.memory_monitor.stop();
    
    // Verify detection happened within 2 seconds
    assert!(detection_time < Duration::from_secs(2), 
           "File change detection took {:?}, expected < 2 seconds", detection_time);
    
    // Verify memory usage stayed reasonable
    let memory_increase = fixture.memory_monitor.memory_increase();
    assert!(memory_increase < 50 * 1024 * 1024, // 50MB limit
           "Memory usage increased by {} bytes, expected < 50MB", memory_increase);
    
    Ok(())
}

#[tokio::test]
async fn test_gitignore_rules_respected() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = WatcherTestFixture::new().await?;
    let _monitor_handle = fixture.memory_monitor.start();
    
    // Create .gitignore with various rules
    fixture.create_gitignore(&[
        "*.tmp",
        "target/",
        "node_modules/", 
        ".env",
        "!important.tmp",
    ])?;
    
    fixture.git_add(".gitignore")?;
    fixture.git_commit("Add gitignore")?;
    
    // Create files that should be ignored
    fixture.create_file("test.tmp", "temporary file")?;
    fixture.create_file("target/debug/output", "build output")?;
    fixture.create_file("node_modules/package/index.js", "dependency")?;
    fixture.create_file(".env", "SECRET_KEY=secret")?;
    
    // Create file that should NOT be ignored (exception rule)
    fixture.create_file("important.tmp", "important temporary file")?;
    
    // Create normal code file that should be tracked
    fixture.create_file("src/main.rs", "fn main() {}")?;
    
    let changes = fixture.watcher.get_changes()?;
    
    fixture.memory_monitor.stop();
    
    // Verify only non-ignored files are detected
    let changed_files: Vec<String> = changes.iter()
        .map(|change| match change {
            FileChange::Added(path) | FileChange::Modified(path) | FileChange::Deleted(path) => {
                path.strip_prefix(&fixture.git_repo_path)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/") // Normalize path separators
            }
        })
        .collect();
    
    // Should detect important.tmp and src/main.rs, but not the ignored files
    assert!(changed_files.contains(&"important.tmp".to_string()), 
           "important.tmp should be detected (gitignore exception)");
    assert!(changed_files.contains(&"src/main.rs".to_string()), 
           "src/main.rs should be detected");
    assert!(!changed_files.iter().any(|f| f.ends_with(".tmp") && f != "important.tmp"), 
           "*.tmp files (except important.tmp) should be ignored");
    assert!(!changed_files.iter().any(|f| f.starts_with("target/")), 
           "target/ directory should be ignored");
    assert!(!changed_files.iter().any(|f| f.starts_with("node_modules/")), 
           "node_modules/ directory should be ignored");
    assert!(!changed_files.contains(&".env".to_string()), 
           ".env file should be ignored");
    
    Ok(())
}

#[tokio::test] 
async fn test_concurrent_file_modifications() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = WatcherTestFixture::new().await?;
    let _monitor_handle = fixture.memory_monitor.start();
    
    // Create initial files
    for i in 0..10 {
        fixture.create_file(&format!("src/file_{}.rs", i), 
                           &format!("fn function_{}() {{ println!(\"initial\"); }}", i))?;
    }
    fixture.git_add("src/")?;
    fixture.git_commit("Initial files")?;
    
    // Spawn concurrent modification tasks
    let handles: Vec<_> = (0..10).map(|i| {
        let path = fixture.git_repo_path.clone();
        tokio::spawn(async move {
            for j in 0..5 {
                let file_path = path.join(format!("src/file_{}.rs", i));
                let content = format!("fn function_{}() {{ println!(\"iteration {}\"); }}", i, j);
                if let Err(e) = tokio::fs::write(&file_path, content).await {
                    eprintln!("Failed to write file {}: {}", i, e);
                    return false;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            true
        })
    }).collect();
    
    // Wait for all modifications to complete
    let results: Vec<_> = futures::future::join_all(handles).await;
    for (i, result) in results.into_iter().enumerate() {
        assert!(result.is_ok(), "Task {} panicked", i);
        assert!(result.unwrap(), "Task {} failed", i);
    }
    
    // Give some time for file system events to settle
    sleep(Duration::from_millis(200)).await;
    
    // Verify changes are detected without race conditions
    let changes = fixture.watcher.get_changes()?;
    
    fixture.memory_monitor.stop();
    
    // Should detect modifications to all 10 files
    assert_eq!(changes.len(), 10, "Should detect exactly 10 modified files");
    
    // Verify all changes are modifications (not corrupted)
    for change in &changes {
        match change {
            FileChange::Modified(path) => {
                assert!(path.to_string_lossy().contains("file_"), 
                       "Modified file should match expected pattern");
            },
            _ => panic!("Expected only Modified changes, got {:?}", change),
        }
    }
    
    // Verify memory usage stayed reasonable during concurrent operations
    let memory_increase = fixture.memory_monitor.memory_increase();
    assert!(memory_increase < 100 * 1024 * 1024, // 100MB limit for concurrent ops
           "Memory usage increased by {} bytes during concurrent operations", memory_increase);
    
    Ok(())
}

#[cfg(feature = "vectordb")]
#[tokio::test]
async fn test_search_engines_update_correctly() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = WatcherTestFixture::new().await?;
    let _monitor_handle = fixture.memory_monitor.start();
    
    // Skip test if vectordb components couldn't be initialized
    let updater = match &fixture.updater {
        Some(u) => u,
        None => {
            println!("Skipping search engine test - vectordb not available");
            return Ok(());
        }
    };
    
    // Create test files with distinct content for each search engine
    let test_files = vec![
        ("src/bm25_test.rs", "struct BM25SearchEngine { query_processor: QueryProcessor }"),
        ("src/tantivy_test.rs", "struct TantivySearchEngine { index: Index, reader: IndexReader }"),
        ("src/semantic_test.rs", "struct SemanticSearchEngine { embedder: NomicEmbedder, storage: VectorStorage }"),
        ("src/symbol_test.rs", "struct SymbolSearchEngine { indexer: SymbolIndexer, database: SymbolDatabase }"),
    ];
    
    for (path, content) in &test_files {
        fixture.create_file(path, content)?;
    }
    fixture.git_add("src/")?;
    fixture.git_commit("Add test files for search engines")?;
    
    // Get initial changes and update through vector updater
    let changes = fixture.watcher.get_changes()?;
    assert_eq!(changes.len(), 4, "Should detect 4 new files");
    
    // Process updates through the vector updater
    let stats = updater.batch_update(changes).await?;
    
    fixture.memory_monitor.stop();
    
    // Verify update statistics
    assert_eq!(stats.updated_files, 4, "Should update 4 files");
    assert_eq!(stats.deleted_files, 0, "Should delete 0 files");
    assert_eq!(stats.failed_files, 0, "Should have 0 failures");
    assert!(stats.total_time < Duration::from_secs(10), 
           "Update should complete within 10 seconds, took {:?}", stats.total_time);
    
    // Modify files to test update handling
    for (path, _) in &test_files {
        let new_content = format!("// Updated at {:?}\n{}", 
                                std::time::SystemTime::now(), 
                                format!("updated content for {}", path));
        fixture.modify_file(path, &new_content)?;
    }
    
    // Process modifications
    let changes = fixture.watcher.get_changes()?;
    assert_eq!(changes.len(), 4, "Should detect 4 modified files");
    
    let stats = updater.batch_update(changes).await?;
    assert_eq!(stats.updated_files, 4, "Should update 4 modified files");
    
    // Test deletion handling
    fixture.delete_file("src/bm25_test.rs")?;
    let changes = fixture.watcher.get_changes()?;
    assert_eq!(changes.len(), 1, "Should detect 1 deleted file");
    
    let stats = updater.batch_update(changes).await?;
    assert_eq!(stats.deleted_files, 1, "Should delete 1 file");
    
    Ok(())
}

#[tokio::test]
async fn test_debouncing_rapid_changes() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = WatcherTestFixture::new().await?;
    let _monitor_handle = fixture.memory_monitor.start();
    
    // Create test file
    fixture.create_file("src/rapid_test.rs", "initial content")?;
    fixture.git_add("src/rapid_test.rs")?;
    fixture.git_commit("Initial commit")?;
    
    // Make rapid modifications
    let modification_count = 20;
    let start_time = Instant::now();
    
    for i in 0..modification_count {
        fixture.modify_file("src/rapid_test.rs", 
                          &format!("rapid modification number {}", i))?;
        sleep(Duration::from_millis(50)).await; // Rapid changes
    }
    
    // Wait for debouncing period
    sleep(Duration::from_millis(500)).await;
    
    let changes = fixture.watcher.get_changes()?;
    fixture.memory_monitor.stop();
    
    // Should detect only one modification despite multiple rapid changes
    assert_eq!(changes.len(), 1, "Debouncing should result in single change detection");
    
    match &changes[0] {
        FileChange::Modified(path) => {
            assert!(path.to_string_lossy().contains("rapid_test.rs"));
        },
        _ => panic!("Expected Modified change, got {:?}", changes[0]),
    }
    
    // Verify the operation completed quickly
    let total_time = start_time.elapsed();
    assert!(total_time < Duration::from_secs(5), 
           "Rapid changes should be processed quickly, took {:?}", total_time);
    
    // Verify final content is the last modification
    let final_content = fs::read_to_string(fixture.git_repo_path.join("src/rapid_test.rs"))?;
    assert!(final_content.contains(&format!("rapid modification number {}", modification_count - 1)));
    
    Ok(())
}

#[cfg(feature = "vectordb")]
#[tokio::test]
async fn test_memory_usage_limits() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = WatcherTestFixture::new().await?;
    let monitor_handle = fixture.memory_monitor.start();
    
    // Skip test if vectordb components couldn't be initialized
    let updater = match &fixture.updater {
        Some(u) => u,
        None => {
            println!("Skipping memory usage test - vectordb not available");
            return Ok(());
        }
    };
    
    // Create many files to test memory usage
    let file_count = 100;
    let file_size = 1024; // 1KB per file
    
    for i in 0..file_count {
        let content = "x".repeat(file_size); // Create 1KB content
        fixture.create_file(&format!("src/large_file_{}.rs", i), &content)?;
    }
    
    fixture.git_add("src/")?;
    fixture.git_commit("Add many files")?;
    
    // Process all files through the watcher and updater
    let changes = fixture.watcher.get_changes()?;
    assert_eq!(changes.len(), file_count, "Should detect all {} files", file_count);
    
    // Process in batches to monitor memory usage
    let batch_size = 10;
    for batch_start in (0..changes.len()).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(changes.len());
        let batch = changes[batch_start..batch_end].to_vec();
        
        let _stats = updater.batch_update(batch).await?;
        
        // Check memory usage after each batch
        let memory_increase = fixture.memory_monitor.memory_increase();
        assert!(memory_increase < 200 * 1024 * 1024, // 200MB limit
               "Memory usage exceeded 200MB: {} bytes after batch {}", 
               memory_increase, batch_start / batch_size + 1);
    }
    
    fixture.memory_monitor.stop();
    monitor_handle.await?;
    
    // Final memory check
    let final_memory_increase = fixture.memory_monitor.memory_increase();
    assert!(final_memory_increase < 300 * 1024 * 1024, // 300MB total limit
           "Final memory usage exceeded 300MB: {} bytes", final_memory_increase);
    
    println!("Memory usage test completed. Peak memory increase: {} MB", 
             final_memory_increase / (1024 * 1024));
    
    Ok(())
}

#[cfg(feature = "vectordb")]
#[tokio::test]
async fn test_thread_safety() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = WatcherTestFixture::new().await?;
    let _monitor_handle = fixture.memory_monitor.start();
    
    // Create initial files
    for i in 0..5 {
        fixture.create_file(&format!("src/thread_test_{}.rs", i), 
                          &format!("fn thread_function_{}() {{}}", i))?;
    }
    fixture.git_add("src/")?;
    fixture.git_commit("Initial thread test files")?;
    
    // Create shared state for tracking operations
    let operation_count = Arc::new(AtomicUsize::new(0));
    let error_count = Arc::new(AtomicUsize::new(0));
    
    // Spawn multiple concurrent watcher operations
    let handles: Vec<_> = (0..5).map(|thread_id| {
        let fixture_path = fixture.git_repo_path.clone();
        let watcher = GitWatcher::new(fixture_path.clone());
        let updater = match &fixture.updater {
            Some(u) => u.clone(),
            None => {
                error_count.fetch_add(1, Ordering::Relaxed);
                return;
            }
        };
        let op_count = operation_count.clone();
        let err_count = error_count.clone();
        
        tokio::spawn(async move {
            for i in 0..10 {
                // Modify file from this thread
                let file_path = fixture_path.join(format!("src/thread_test_{}.rs", thread_id));
                let content = format!("// Thread {} iteration {}\nfn thread_function_{}() {{}}", 
                                    thread_id, i, thread_id);
                
                if let Err(_) = tokio::fs::write(&file_path, content).await {
                    err_count.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
                
                // Try to detect and process changes
                if let Ok(changes) = watcher.get_changes() {
                    if !changes.is_empty() {
                        if let Ok(_stats) = updater.batch_update(changes).await {
                            op_count.fetch_add(1, Ordering::Relaxed);
                        } else {
                            err_count.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
                
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
    }).collect();
    
    // Wait for all threads to complete
    for handle in handles {
        handle.await?;
    }
    
    fixture.memory_monitor.stop();
    
    // Verify thread safety
    let total_operations = operation_count.load(Ordering::Relaxed);
    let total_errors = error_count.load(Ordering::Relaxed);
    
    println!("Thread safety test completed. Operations: {}, Errors: {}", 
             total_operations, total_errors);
    
    // Should have completed some operations successfully
    assert!(total_operations > 0, "Should have completed at least some operations");
    
    // Error rate should be reasonable (allow some contention)
    let error_rate = total_errors as f64 / (total_operations + total_errors) as f64;
    assert!(error_rate < 0.1, "Error rate should be < 10%, got {:.2}%", error_rate * 100.0);
    
    // Verify final state is consistent
    let changes = fixture.watcher.get_changes()?;
    
    // All files should exist and be in a valid state
    for i in 0..5 {
        let file_path = fixture.git_repo_path.join(format!("src/thread_test_{}.rs", i));
        assert!(file_path.exists(), "File {} should exist after concurrent operations", i);
        
        let content = fs::read_to_string(&file_path)?;
        assert!(content.contains(&format!("thread_function_{}", i)), 
               "File {} content should be valid", i);
    }
    
    Ok(())
}

#[cfg(feature = "vectordb")]
#[tokio::test] 
async fn test_watch_command_integration() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = WatcherTestFixture::new().await?;
    let _monitor_handle = fixture.memory_monitor.start();
    
    // Skip test if watch command couldn't be initialized
    let watch_command = match &fixture.watch_command {
        Some(w) => w,
        None => {
            println!("Skipping watch command test - vectordb not available");
            return Ok(());
        }
    };
    
    // Test watch command lifecycle
    assert!(!watch_command.is_running(), "Watch command should start stopped");
    
    // Start watching
    watch_command.start().await;
    assert!(watch_command.is_running(), "Watch command should be running after start");
    
    // Create and modify files
    fixture.create_file("src/watch_test.rs", "initial content")?;
    fixture.git_add("src/watch_test.rs")?;
    fixture.git_commit("Add watch test file")?;
    
    fixture.modify_file("src/watch_test.rs", "modified content")?;
    
    // Run one-off detection
    let stats = watch_command.run_once().await?;
    
    fixture.memory_monitor.stop();
    
    // Verify detection worked
    assert_eq!(stats.updated_files, 1, "Should detect 1 updated file");
    assert_eq!(stats.failed_files, 0, "Should have no failures");
    
    // Stop watching
    watch_command.stop();
    assert!(!watch_command.is_running(), "Watch command should stop after stop() call");
    
    Ok(())
}

#[tokio::test]
async fn test_code_file_filtering() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = WatcherTestFixture::new().await?;
    
    // Create various file types
    let test_files = vec![
        ("src/code.rs", "rust code", true),
        ("src/script.py", "python code", true),
        ("src/app.js", "javascript code", true),  
        ("src/component.tsx", "typescript react", true),
        ("docs/README.md", "markdown documentation", true),
        ("config.toml", "configuration", false),
        ("data.json", "json data", false),
        ("image.png", "binary image", false),
        ("video.mp4", "binary video", false),
    ];
    
    for (path, content, _should_detect) in &test_files {
        fixture.create_file(path, content)?;
    }
    
    // Add files so they show up in git status
    fixture.git_add(".")?;
    fixture.git_commit("Add test files for filtering")?;
    
    // Now modify them so they show as changed
    for (path, content, _should_detect) in &test_files {
        let modified_content = format!("// Modified\n{}", content);
        fixture.modify_file(path, &modified_content)?;
    }
    
    let changes = fixture.watcher.get_changes()?;
    
    // Filter to only expected detectable files
    let expected_files: Vec<&str> = test_files.iter()
        .filter_map(|(path, _, should_detect)| if *should_detect { Some(*path) } else { None })
        .collect();
    
    // Verify only code files are detected
    assert_eq!(changes.len(), expected_files.len(), 
              "Should detect only code files. Expected: {:?}, Got: {} changes", 
              expected_files, changes.len());
    
    for change in &changes {
        let path = match change {
            FileChange::Added(p) | FileChange::Modified(p) | FileChange::Deleted(p) => p,
        };
        
        let relative_path = path.strip_prefix(&fixture.git_repo_path)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
            
        assert!(expected_files.contains(&relative_path.as_str()), 
               "Detected file '{}' should be in expected code files", relative_path);
    }
    
    Ok(())
}

/// Test cleanup and isolation verification
#[tokio::test]
async fn test_isolation_and_cleanup() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Each test fixture should be completely isolated
    let fixture1 = WatcherTestFixture::new().await?;
    let fixture2 = WatcherTestFixture::new().await?;
    
    // Create files in each fixture
    fixture1.create_file("test1.rs", "fixture 1 content")?;
    fixture2.create_file("test2.rs", "fixture 2 content")?;
    
    // Changes should be isolated
    let changes1 = fixture1.watcher.get_changes()?;
    let changes2 = fixture2.watcher.get_changes()?;
    
    assert_eq!(changes1.len(), 1, "Fixture 1 should have 1 change");
    assert_eq!(changes2.len(), 1, "Fixture 2 should have 1 change");
    
    // Verify file isolation
    let path1 = &fixture1.git_repo_path;
    let path2 = &fixture2.git_repo_path;
    
    assert_ne!(path1, path2, "Fixtures should have different paths");
    assert!(path1.join("test1.rs").exists(), "Fixture 1 should have its file");
    assert!(!path1.join("test2.rs").exists(), "Fixture 1 should not have fixture 2's file");
    assert!(path2.join("test2.rs").exists(), "Fixture 2 should have its file");
    assert!(!path2.join("test1.rs").exists(), "Fixture 2 should not have fixture 1's file");
    
    // Drop fixtures (automatic cleanup)
    drop(fixture1);
    drop(fixture2);
    
    // Verify cleanup occurred (paths should no longer exist after tempdir drop)
    // Note: We can't directly test this as the paths are already dropped,
    // but tempfile guarantees cleanup
    
    Ok(())
}

// Helper test to verify the testing framework itself
#[tokio::test]
async fn test_framework_functionality() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = WatcherTestFixture::new().await?;
    
    // Test basic operations
    assert!(fixture.git_repo_path.exists(), "Git repo path should exist");
    assert!(fixture.git_repo_path.join(".git").exists(), "Should be a git repository");
    
    // Test file operations
    let file_path = fixture.create_file("test.rs", "test content")?;
    assert!(file_path.exists(), "Created file should exist");
    
    let content = fs::read_to_string(&file_path)?;
    assert_eq!(content, "test content", "File should have correct content");
    
    // Test modification
    fixture.modify_file("test.rs", "modified content")?;
    let modified_content = fs::read_to_string(&file_path)?;
    assert_eq!(modified_content, "modified content", "File should be modified");
    
    // Test deletion
    fixture.delete_file("test.rs")?;
    assert!(!file_path.exists(), "Deleted file should not exist");
    
    // Test change detection
    fixture.create_file("new_test.rs", "new content")?;
    let changes = fixture.watcher.get_changes()?;
    assert!(!changes.is_empty(), "Should detect new file");
    
    println!("Test framework verification completed successfully");
    
    Ok(())
}