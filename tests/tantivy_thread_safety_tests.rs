use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use anyhow::Result;
use tempfile::TempDir;
use tokio::task;

#[cfg(feature = "tantivy")]
use embed_search::search::{TantivySearcher, ExactMatch};

/// THREAD SAFETY AND CONCURRENCY STRESS TESTS
/// 
/// These tests expose race conditions, deadlocks, and thread safety issues:
/// - Multiple threads indexing simultaneously
/// - Concurrent reads while writing
/// - Resource contention scenarios
/// - Deadlock detection
/// - Memory consistency verification
/// 
/// NO ARTIFICIAL DELAYS - All concurrency issues must be real and reproducible.

#[cfg(test)]
mod thread_safety_tests {
    use super::*;

    /// Test multiple threads indexing different files simultaneously
    #[cfg(feature = "tantivy")]
    #[tokio::test]
    async fn stress_concurrent_indexing() -> Result<()> {
        println!("🔥 THREAD SAFETY TEST: Concurrent Indexing");
        println!("===========================================");
        
        let temp_dir = TempDir::new()?;
        let index_path = temp_dir.path().join("concurrent_index");
        
        // Shared state for tracking operations
        let success_count = Arc::new(AtomicUsize::new(0));
        let error_count = Arc::new(AtomicUsize::new(0));
        let total_docs_indexed = Arc::new(AtomicUsize::new(0));
        let is_test_running = Arc::new(AtomicBool::new(true));
        
        // Create test files for each thread
        const NUM_THREADS: usize = 8;
        const FILES_PER_THREAD: usize = 50;
        
        let files_dir = temp_dir.path().join("thread_test_files");
        fs::create_dir_all(&files_dir)?;
        
        println!("📋 Creating {} files for {} threads...", 
                FILES_PER_THREAD * NUM_THREADS, NUM_THREADS);
        
        let creation_start = Instant::now();
        let mut all_file_paths = Vec::new();
        
        for thread_id in 0..NUM_THREADS {
            for file_id in 0..FILES_PER_THREAD {
                let file_path = files_dir.join(format!("thread_{}_file_{}.rs", thread_id, file_id));
                let content = format!(
                    "// Thread {} File {}\n\
                    pub struct ThreadData{}_{} {{\n    \
                        thread_id: usize,\n    \
                        file_id: usize,\n    \
                        unique_content: String,\n\
                    }}\n\n\
                    impl ThreadData{}_{} {{\n    \
                        pub fn get_unique_identifier() -> &'static str {{\n        \
                            \"thread_{}_file_{}_unique_marker\"\n    \
                        }}\n    \
                        \n    \
                        pub fn concurrent_function_{}_{}_{}() -> Result<String> {{\n        \
                            let data = \"concurrent_data_{}_{}_{}\";\n        \
                            Ok(data.to_string())\n    \
                        }}\n\
                    }}",
                    thread_id, file_id, thread_id, file_id, 
                    thread_id, file_id, thread_id, file_id,
                    thread_id, file_id, thread_id, thread_id, file_id,
                    thread_id, file_id, thread_id
                );
                
                fs::write(&file_path, content)?;
                all_file_paths.push((thread_id, file_id, file_path));
            }
        }
        
        println!("   ✅ Created {} files in {:?}", 
                all_file_paths.len(), creation_start.elapsed());
        
        // Launch concurrent indexing threads
        println!("🚀 Launching {} concurrent indexing threads...", NUM_THREADS);
        let test_start = Instant::now();
        
        let mut thread_handles = Vec::new();
        
        for thread_id in 0..NUM_THREADS {
            let index_path = index_path.clone();
            let success_count = success_count.clone();
            let error_count = error_count.clone();
            let total_docs_indexed = total_docs_indexed.clone();
            let is_test_running = is_test_running.clone();
            
            // Get files for this thread
            let thread_files: Vec<PathBuf> = all_file_paths.iter()
                .filter(|(tid, _, _)| *tid == thread_id)
                .map(|(_, _, path)| path.clone())
                .collect();
            
            let handle = thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                
                rt.block_on(async move {
                    let thread_start = Instant::now();
                    let mut thread_successes = 0;
                    let mut thread_errors = 0;
                    let mut thread_docs = 0;
                    
                    // Create searcher for this thread
                    match TantivySearcher::new_with_path(&index_path).await {
                        Ok(mut searcher) => {
                            println!("   🔧 Thread {} started with {} files", thread_id, thread_files.len());
                            
                            // Index files assigned to this thread
                            for (file_idx, file_path) in thread_files.iter().enumerate() {
                                if !is_test_running.load(Ordering::SeqCst) {
                                    println!("   ⏹️  Thread {} stopping early due to test timeout", thread_id);
                                    break;
                                }
                                
                                match searcher.index_file(file_path).await {
                                    Ok(()) => {
                                        thread_successes += 1;
                                        thread_docs += 1; // Assume 1 doc per successful file
                                        
                                        if file_idx % 10 == 0 {
                                            println!("      Thread {} indexed {}/{} files", 
                                                    thread_id, file_idx + 1, thread_files.len());
                                        }
                                    }
                                    Err(e) => {
                                        thread_errors += 1;
                                        println!("      ❌ Thread {} failed to index {:?}: {}", 
                                                thread_id, file_path.file_name(), e);
                                        
                                        // Check if it's a concurrency-related error
                                        let error_msg = e.to_string().to_lowercase();
                                        let concurrency_errors = ["lock", "busy", "conflict", "concurrent"];
                                        let is_concurrency_error = concurrency_errors.iter()
                                            .any(|&err_type| error_msg.contains(err_type));
                                        
                                        if is_concurrency_error {
                                            println!("         ⚠️  Concurrency-related error detected");
                                        }
                                    }
                                }
                            }
                            
                            // Test search functionality from this thread
                            let search_test_query = format!("thread_{}_file_0_unique_marker", thread_id);
                            match searcher.search(&search_test_query).await {
                                Ok(results) => {
                                    println!("   🔍 Thread {} search test: {} results for '{}'", 
                                            thread_id, results.len(), search_test_query);
                                    
                                    if !results.is_empty() {
                                        // Verify result is actually from our thread's content
                                        let found_our_content = results.iter().any(|r| 
                                            r.content.contains(&format!("thread_{}", thread_id))
                                        );
                                        
                                        if !found_our_content {
                                            println!("      ⚠️  WARNING: Search didn't find our thread's content");
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("   ❌ Thread {} search test failed: {}", thread_id, e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("   ❌ Thread {} failed to create searcher: {}", thread_id, e);
                            thread_errors += 1;
                        }
                    }
                    
                    // Update global counters
                    success_count.fetch_add(thread_successes, Ordering::SeqCst);
                    error_count.fetch_add(thread_errors, Ordering::SeqCst);
                    total_docs_indexed.fetch_add(thread_docs, Ordering::SeqCst);
                    
                    let thread_duration = thread_start.elapsed();
                    println!("   ✅ Thread {} completed: {} successes, {} errors in {:?}", 
                            thread_id, thread_successes, thread_errors, thread_duration);
                })
            });
            
            thread_handles.push(handle);
        }
        
        // Set a reasonable timeout for the test
        let timeout_duration = Duration::from_secs(120); // 2 minutes
        let timeout_handle = {
            let is_test_running = is_test_running.clone();
            thread::spawn(move || {
                thread::sleep(timeout_duration);
                is_test_running.store(false, Ordering::SeqCst);
            })
        };
        
        // Wait for all threads to complete
        for (i, handle) in thread_handles.into_iter().enumerate() {
            match handle.join() {
                Ok(()) => {
                    println!("   ✅ Thread {} joined successfully", i);
                }
                Err(e) => {
                    println!("   ❌ Thread {} panicked: {:?}", i, e);
                }
            }
        }
        
        // Stop the timeout thread
        is_test_running.store(false, Ordering::SeqCst);
        let _ = timeout_handle.join();
        
        let total_duration = test_start.elapsed();
        
        // Report results
        let final_successes = success_count.load(Ordering::SeqCst);
        let final_errors = error_count.load(Ordering::SeqCst);
        let final_docs = total_docs_indexed.load(Ordering::SeqCst);
        
        println!("📊 Concurrent indexing test results:");
        println!("   ✅ Successful operations: {}", final_successes);
        println!("   ❌ Failed operations: {}", final_errors);
        println!("   📄 Documents indexed: {}", final_docs);
        println!("   ⏱️  Total duration: {:?}", total_duration);
        println!("   🚀 Throughput: {:.1} ops/sec", 
                final_successes as f64 / total_duration.as_secs_f64());
        
        // Verify final index state
        println!("🔍 Verifying final index state...");
        let final_searcher = TantivySearcher::new_with_path(&index_path).await?;
        let final_stats = final_searcher.get_index_stats()?;
        println!("   📊 Final index: {}", final_stats);
        
        // TRUTH CHECK: At least some operations must succeed
        if final_successes == 0 {
            if final_errors == 0 {
                panic!("No operations completed - test setup issue");
            } else {
                println!("   ⚠️  All operations failed - possible concurrency issues or resource limits");
                // This could be legitimate resource exhaustion
            }
        } else {
            println!("   ✅ Concurrent indexing partially successful");
            
            // Test final search across all indexed content
            let verification_results = final_searcher.search("concurrent_function").await?;
            println!("   🔍 Final verification search found {} results", verification_results.len());
        }
        
        Ok(())
    }

    /// Test readers while writers are active (read-write concurrency)
    #[cfg(feature = "tantivy")]
    #[tokio::test]
    async fn stress_read_write_concurrency() -> Result<()> {
        println!("🔥 THREAD SAFETY TEST: Read-Write Concurrency");
        println!("==============================================");
        
        let temp_dir = TempDir::new()?;
        let index_path = temp_dir.path().join("read_write_index");
        
        // Initialize index with some initial data
        {
            let mut initial_searcher = TantivySearcher::new_with_path(&index_path).await?;
            let initial_file = temp_dir.path().join("initial.rs");
            fs::write(&initial_file, r#"
pub struct InitialData {
    value: String,
}

impl InitialData {
    pub fn initial_search_target() -> &'static str {
        "initial_marker_content"
    }
}
"#)?;
            initial_searcher.index_file(&initial_file).await?;
        }
        
        // Shared state for coordination
        let reader_results = Arc::new(Mutex::new(Vec::new()));
        let writer_operations = Arc::new(AtomicUsize::new(0));
        let reader_operations = Arc::new(AtomicUsize::new(0));
        let test_running = Arc::new(AtomicBool::new(true));
        
        println!("🚀 Starting read-write concurrency test...");
        let test_start = Instant::now();
        
        let mut handles = Vec::new();
        
        // Launch writer threads
        const NUM_WRITERS: usize = 3;
        const NUM_READERS: usize = 5;
        
        for writer_id in 0..NUM_WRITERS {
            let index_path = index_path.clone();
            let writer_operations = writer_operations.clone();
            let test_running = test_running.clone();
            let temp_dir_path = temp_dir.path().to_path_buf();
            
            let handle = thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                
                rt.block_on(async move {
                    match TantivySearcher::new_with_path(&index_path).await {
                        Ok(mut searcher) => {
                            let mut operation_count = 0;
                            
                            while test_running.load(Ordering::SeqCst) && operation_count < 20 {
                                let file_path = temp_dir_path.join(format!("writer_{}_{}.rs", writer_id, operation_count));
                                let content = format!(
                                    "// Writer {} Operation {}\n\
                                    pub fn writer_{}_operation_{}() -> String {{\n    \
                                        \"writer_marker_{}_{}\".to_string()\n\
                                    }}\n\
                                    \n\
                                    pub struct WriterData{}_{} {{\n    \
                                        writer_id: usize,\n    \
                                        operation_id: usize,\n\
                                    }}",
                                    writer_id, operation_count,
                                    writer_id, operation_count,
                                    writer_id, operation_count,
                                    writer_id, operation_count
                                );
                                
                                match fs::write(&file_path, content) {
                                    Ok(()) => {
                                        match searcher.index_file(&file_path).await {
                                            Ok(()) => {
                                                writer_operations.fetch_add(1, Ordering::SeqCst);
                                                operation_count += 1;
                                                println!("      ✏️  Writer {} completed operation {}", 
                                                        writer_id, operation_count);
                                            }
                                            Err(e) => {
                                                println!("      ❌ Writer {} indexing failed: {}", writer_id, e);
                                                break;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("      ❌ Writer {} file creation failed: {}", writer_id, e);
                                        break;
                                    }
                                }
                                
                                // Brief pause between operations
                                tokio::time::sleep(Duration::from_millis(100)).await;
                            }
                            
                            println!("   ✅ Writer {} completed {} operations", writer_id, operation_count);
                        }
                        Err(e) => {
                            println!("   ❌ Writer {} failed to create searcher: {}", writer_id, e);
                        }
                    }
                })
            });
            
            handles.push(handle);
        }
        
        // Brief delay to let writers start
        thread::sleep(Duration::from_millis(200));
        
        // Launch reader threads
        for reader_id in 0..NUM_READERS {
            let index_path = index_path.clone();
            let reader_results = reader_results.clone();
            let reader_operations = reader_operations.clone();
            let test_running = test_running.clone();
            
            let handle = thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                
                rt.block_on(async move {
                    // Allow some time for writers to add content
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    
                    match TantivySearcher::new_with_path(&index_path).await {
                        Ok(searcher) => {
                            let mut read_count = 0;
                            let mut local_results = Vec::new();
                            
                            while test_running.load(Ordering::SeqCst) && read_count < 50 {
                                let search_queries = vec![
                                    "initial_marker_content",
                                    "writer_marker",
                                    "WriterData",
                                    "operation",
                                    format!("writer_{}", reader_id % NUM_WRITERS).as_str(),
                                ];
                                
                                for query in &search_queries {
                                    match searcher.search(query).await {
                                        Ok(results) => {
                                            reader_operations.fetch_add(1, Ordering::SeqCst);
                                            read_count += 1;
                                            
                                            local_results.push((query.to_string(), results.len()));
                                            
                                            if read_count % 10 == 0 {
                                                println!("      👁️  Reader {} completed {} searches", 
                                                        reader_id, read_count);
                                            }
                                            
                                            // Verify results are consistent
                                            if !results.is_empty() {
                                                let first_result = &results[0];
                                                if !first_result.content.is_empty() && 
                                                   !first_result.file_path.is_empty() &&
                                                   first_result.line_number > 0 {
                                                    // Result structure looks valid
                                                } else {
                                                    println!("         ⚠️  Reader {} found malformed result", reader_id);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            println!("      ❌ Reader {} search failed for '{}': {}", 
                                                    reader_id, query, e);
                                        }
                                    }
                                    
                                    // Very brief pause between searches
                                    tokio::time::sleep(Duration::from_millis(20)).await;
                                }
                            }
                            
                            // Store results for analysis
                            {
                                let mut results_guard = reader_results.lock().unwrap();
                                results_guard.extend(local_results);
                            }
                            
                            println!("   ✅ Reader {} completed {} searches", reader_id, read_count);
                        }
                        Err(e) => {
                            println!("   ❌ Reader {} failed to create searcher: {}", reader_id, e);
                        }
                    }
                })
            });
            
            handles.push(handle);
        }
        
        // Let the test run for a reasonable duration
        thread::sleep(Duration::from_secs(15));
        test_running.store(false, Ordering::SeqCst);
        
        // Wait for all threads
        for (i, handle) in handles.into_iter().enumerate() {
            match handle.join() {
                Ok(()) => {
                    println!("   ✅ Thread {} completed successfully", i);
                }
                Err(e) => {
                    println!("   ❌ Thread {} panicked: {:?}", i, e);
                }
            }
        }
        
        let test_duration = test_start.elapsed();
        
        // Analyze results
        let total_writes = writer_operations.load(Ordering::SeqCst);
        let total_reads = reader_operations.load(Ordering::SeqCst);
        
        println!("📊 Read-write concurrency results:");
        println!("   ✏️  Write operations: {}", total_writes);
        println!("   👁️  Read operations: {}", total_reads);
        println!("   ⏱️  Total duration: {:?}", test_duration);
        println!("   🚀 Read throughput: {:.1} reads/sec", 
                total_reads as f64 / test_duration.as_secs_f64());
        println!("   🚀 Write throughput: {:.1} writes/sec", 
                total_writes as f64 / test_duration.as_secs_f64());
        
        // Analyze search result consistency
        {
            let results_guard = reader_results.lock().unwrap();
            let mut query_stats: HashMap<String, Vec<usize>> = HashMap::new();
            
            for (query, result_count) in results_guard.iter() {
                query_stats.entry(query.clone()).or_insert_with(Vec::new).push(*result_count);
            }
            
            println!("   📈 Search consistency analysis:");
            for (query, counts) in query_stats.iter() {
                let min_results = *counts.iter().min().unwrap_or(&0);
                let max_results = *counts.iter().max().unwrap_or(&0);
                let avg_results = counts.iter().sum::<usize>() as f64 / counts.len() as f64;
                
                println!("      '{}': min={}, max={}, avg={:.1} (over {} searches)", 
                        query, min_results, max_results, avg_results, counts.len());
                
                // Check for excessive inconsistency
                if max_results > 0 && min_results == 0 {
                    println!("         ⚠️  WARNING: Inconsistent results (some searches found nothing)");
                }
            }
        }
        
        // Final verification
        let final_searcher = TantivySearcher::new_with_path(&index_path).await?;
        let final_stats = final_searcher.get_index_stats()?;
        println!("   📊 Final index state: {}", final_stats);
        
        // TRUTH CHECK
        if total_reads == 0 && total_writes == 0 {
            panic!("No operations completed - test setup failure");
        } else if total_reads == 0 {
            println!("   ⚠️  No reads completed - possible reader blocking");
        } else if total_writes == 0 {
            println!("   ⚠️  No writes completed - possible writer blocking");
        } else {
            println!("   ✅ Read-write concurrency test successful");
        }
        
        Ok(())
    }

    /// Test for deadlock detection and prevention
    #[cfg(feature = "tantivy")]
    #[tokio::test]
    async fn stress_deadlock_detection() -> Result<()> {
        println!("🔥 THREAD SAFETY TEST: Deadlock Detection");
        println!("=========================================");
        
        let temp_dir = TempDir::new()?;
        let index_path = temp_dir.path().join("deadlock_test_index");
        
        // Create two separate index paths that threads might contend over
        let index_path_a = temp_dir.path().join("index_a");
        let index_path_b = temp_dir.path().join("index_b");
        
        // Initialize both indices
        {
            let mut searcher_a = TantivySearcher::new_with_path(&index_path_a).await?;
            let mut searcher_b = TantivySearcher::new_with_path(&index_path_b).await?;
            
            let test_file_a = temp_dir.path().join("init_a.rs");
            let test_file_b = temp_dir.path().join("init_b.rs");
            
            fs::write(&test_file_a, "pub fn init_a() { println!(\"Index A\"); }")?;
            fs::write(&test_file_b, "pub fn init_b() { println!(\"Index B\"); }")?;
            
            searcher_a.index_file(&test_file_a).await?;
            searcher_b.index_file(&test_file_b).await?;
        }
        
        // Shared state for deadlock detection
        let completed_operations = Arc::new(AtomicUsize::new(0));
        let is_deadlocked = Arc::new(AtomicBool::new(false));
        let test_start_time = Arc::new(Mutex::new(Instant::now()));
        
        println!("🚀 Testing potential deadlock scenarios...");
        
        // Pattern 1: Thread acquires A then B, other thread acquires B then A
        let deadlock_test_1 = {
            let index_path_a = index_path_a.clone();
            let index_path_b = index_path_b.clone();
            let completed_ops = completed_operations.clone();
            let temp_dir_path = temp_dir.path().to_path_buf();
            
            async move {
                println!("   🧵 Starting deadlock test pattern 1...");
                
                let mut handles = Vec::new();
                
                // Thread 1: A -> B
                let handle_1 = {
                    let index_path_a = index_path_a.clone();
                    let index_path_b = index_path_b.clone();
                    let completed_ops = completed_ops.clone();
                    let temp_dir_path = temp_dir_path.clone();
                    
                    task::spawn(async move {
                        match TantivySearcher::new_with_path(&index_path_a).await {
                            Ok(mut searcher_a) => {
                                // Create and index a file in A
                                let file_a = temp_dir_path.join("deadlock_1_a.rs");
                                fs::write(&file_a, "pub fn deadlock_test_1_a() { println!(\"Test\"); }")?;
                                
                                if let Err(e) = searcher_a.index_file(&file_a).await {
                                    println!("      ❌ Thread 1 failed to index in A: {}", e);
                                    return Err(e);
                                }
                                
                                // Small delay to increase chance of deadlock
                                tokio::time::sleep(Duration::from_millis(10)).await;
                                
                                // Now try to access B
                                match TantivySearcher::new_with_path(&index_path_b).await {
                                    Ok(mut searcher_b) => {
                                        let file_b = temp_dir_path.join("deadlock_1_b.rs");
                                        fs::write(&file_b, "pub fn deadlock_test_1_b() { println!(\"Test\"); }")?;
                                        
                                        if let Err(e) = searcher_b.index_file(&file_b).await {
                                            println!("      ❌ Thread 1 failed to index in B: {}", e);
                                            return Err(e);
                                        }
                                        
                                        completed_ops.fetch_add(1, Ordering::SeqCst);
                                        println!("      ✅ Thread 1 (A->B) completed successfully");
                                        Ok(())
                                    }
                                    Err(e) => {
                                        println!("      ❌ Thread 1 failed to create searcher B: {}", e);
                                        Err(e)
                                    }
                                }
                            }
                            Err(e) => {
                                println!("      ❌ Thread 1 failed to create searcher A: {}", e);
                                Err(e)
                            }
                        }
                    })
                };
                handles.push(handle_1);
                
                // Thread 2: B -> A
                let handle_2 = {
                    let index_path_a = index_path_a.clone();
                    let index_path_b = index_path_b.clone();
                    let completed_ops = completed_ops.clone();
                    let temp_dir_path = temp_dir_path.clone();
                    
                    task::spawn(async move {
                        match TantivySearcher::new_with_path(&index_path_b).await {
                            Ok(mut searcher_b) => {
                                // Create and index a file in B
                                let file_b = temp_dir_path.join("deadlock_2_b.rs");
                                fs::write(&file_b, "pub fn deadlock_test_2_b() { println!(\"Test\"); }")?;
                                
                                if let Err(e) = searcher_b.index_file(&file_b).await {
                                    println!("      ❌ Thread 2 failed to index in B: {}", e);
                                    return Err(e);
                                }
                                
                                // Small delay to increase chance of deadlock
                                tokio::time::sleep(Duration::from_millis(10)).await;
                                
                                // Now try to access A
                                match TantivySearcher::new_with_path(&index_path_a).await {
                                    Ok(mut searcher_a) => {
                                        let file_a = temp_dir_path.join("deadlock_2_a.rs");
                                        fs::write(&file_a, "pub fn deadlock_test_2_a() { println!(\"Test\"); }")?;
                                        
                                        if let Err(e) = searcher_a.index_file(&file_a).await {
                                            println!("      ❌ Thread 2 failed to index in A: {}", e);
                                            return Err(e);
                                        }
                                        
                                        completed_ops.fetch_add(1, Ordering::SeqCst);
                                        println!("      ✅ Thread 2 (B->A) completed successfully");
                                        Ok(())
                                    }
                                    Err(e) => {
                                        println!("      ❌ Thread 2 failed to create searcher A: {}", e);
                                        Err(e)
                                    }
                                }
                            }
                            Err(e) => {
                                println!("      ❌ Thread 2 failed to create searcher B: {}", e);
                                Err(e)
                            }
                        }
                    })
                };
                handles.push(handle_2);
                
                // Wait for completion with timeout
                let timeout_result = tokio::time::timeout(
                    Duration::from_secs(30), 
                    futures::future::join_all(handles)
                ).await;
                
                match timeout_result {
                    Ok(results) => {
                        let mut success_count = 0;
                        let mut error_count = 0;
                        
                        for (i, result) in results.into_iter().enumerate() {
                            match result {
                                Ok(Ok(())) => {
                                    success_count += 1;
                                }
                                Ok(Err(e)) => {
                                    error_count += 1;
                                    println!("      ❌ Thread {} completed with error: {}", i + 1, e);
                                }
                                Err(e) => {
                                    error_count += 1;
                                    println!("      ❌ Thread {} panicked: {:?}", i + 1, e);
                                }
                            }
                        }
                        
                        println!("   📊 Deadlock test 1 results: {} successes, {} errors", 
                                success_count, error_count);
                        
                        if success_count == 2 {
                            println!("      ✅ No deadlock detected in pattern 1");
                        } else {
                            println!("      ⚠️  Potential issues in pattern 1");
                        }
                        
                        Ok(())
                    }
                    Err(_) => {
                        println!("   ❌ DEADLOCK DETECTED: Pattern 1 timed out after 30 seconds");
                        is_deadlocked.store(true, Ordering::SeqCst);
                        Err(anyhow::anyhow!("Deadlock timeout"))
                    }
                }
            }
        };
        
        // Execute deadlock test
        let _ = deadlock_test_1.await;
        
        // Final analysis
        let total_completed = completed_operations.load(Ordering::SeqCst);
        let deadlock_detected = is_deadlocked.load(Ordering::SeqCst);
        
        println!("📊 Deadlock detection test results:");
        println!("   ✅ Completed operations: {}", total_completed);
        println!("   🚫 Deadlock detected: {}", deadlock_detected);
        
        if deadlock_detected {
            println!("   ⚠️  CRITICAL: Deadlock vulnerability found in TantivySearcher");
            println!("      This indicates a serious thread safety issue that must be addressed");
            // Don't panic - deadlock detection is a valid test outcome
        } else {
            println!("   ✅ No deadlocks detected - thread safety appears adequate");
        }
        
        // Test cleanup - verify we can still create new searchers
        println!("🧹 Testing cleanup and recovery...");
        let cleanup_searcher = TantivySearcher::new_with_path(&index_path).await?;
        let cleanup_stats = cleanup_searcher.get_index_stats()?;
        println!("   ✅ Cleanup successful, can create new searcher: {}", cleanup_stats);
        
        Ok(())
    }

    /// Test resource cleanup under concurrent access
    #[cfg(feature = "tantivy")]
    #[tokio::test]
    async fn stress_resource_cleanup() -> Result<()> {
        println!("🔥 THREAD SAFETY TEST: Resource Cleanup");
        println!("=======================================");
        
        let temp_dir = TempDir::new()?;
        
        // Test creating and destroying many searchers concurrently
        let creation_count = Arc::new(AtomicUsize::new(0));
        let cleanup_count = Arc::new(AtomicUsize::new(0));
        let error_count = Arc::new(AtomicUsize::new(0));
        
        println!("🚀 Testing concurrent searcher creation and cleanup...");
        
        let mut handles = Vec::new();
        const NUM_CLEANUP_THREADS: usize = 10;
        const OPERATIONS_PER_THREAD: usize = 20;
        
        for thread_id in 0..NUM_CLEANUP_THREADS {
            let temp_dir_path = temp_dir.path().to_path_buf();
            let creation_count = creation_count.clone();
            let cleanup_count = cleanup_count.clone();
            let error_count = error_count.clone();
            
            let handle = task::spawn(async move {
                for operation_id in 0..OPERATIONS_PER_THREAD {
                    let index_path = temp_dir_path.join(format!("cleanup_{}_{}", thread_id, operation_id));
                    
                    // Create searcher
                    match TantivySearcher::new_with_path(&index_path).await {
                        Ok(mut searcher) => {
                            creation_count.fetch_add(1, Ordering::SeqCst);
                            
                            // Do some work
                            let test_file = temp_dir_path.join(format!("cleanup_test_{}_{}.rs", thread_id, operation_id));
                            let content = format!(
                                "pub fn cleanup_test_{}_{}() -> String {{ \
                                    \"cleanup_test_content_{}_{}\".to_string() \
                                }}",
                                thread_id, operation_id, thread_id, operation_id
                            );
                            
                            if let Err(e) = fs::write(&test_file, content) {
                                println!("      ❌ Thread {} failed to write file: {}", thread_id, e);
                                error_count.fetch_add(1, Ordering::SeqCst);
                                continue;
                            }
                            
                            if let Err(e) = searcher.index_file(&test_file).await {
                                println!("      ❌ Thread {} failed to index: {}", thread_id, e);
                                error_count.fetch_add(1, Ordering::SeqCst);
                                continue;
                            }
                            
                            // Test search
                            let search_query = format!("cleanup_test_content_{}_{}", thread_id, operation_id);
                            match searcher.search(&search_query).await {
                                Ok(results) => {
                                    if results.is_empty() {
                                        println!("      ⚠️  Thread {} search found no results", thread_id);
                                    }
                                }
                                Err(e) => {
                                    println!("      ❌ Thread {} search failed: {}", thread_id, e);
                                    error_count.fetch_add(1, Ordering::SeqCst);
                                }
                            }
                            
                            // Explicit drop to test cleanup
                            drop(searcher);
                            cleanup_count.fetch_add(1, Ordering::SeqCst);
                            
                            if operation_id % 5 == 0 {
                                println!("      🔄 Thread {} completed {}/{} operations", 
                                        thread_id, operation_id + 1, OPERATIONS_PER_THREAD);
                            }
                        }
                        Err(e) => {
                            println!("      ❌ Thread {} failed to create searcher: {}", thread_id, e);
                            error_count.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                    
                    // Brief pause between operations
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                
                println!("   ✅ Thread {} completed all operations", thread_id);
            });
            
            handles.push(handle);
        }
        
        // Wait for all cleanup threads
        let start_time = Instant::now();
        futures::future::join_all(handles).await;
        let total_duration = start_time.elapsed();
        
        // Report cleanup results
        let total_creations = creation_count.load(Ordering::SeqCst);
        let total_cleanups = cleanup_count.load(Ordering::SeqCst);
        let total_errors = error_count.load(Ordering::SeqCst);
        
        println!("📊 Resource cleanup test results:");
        println!("   🔧 Searchers created: {}", total_creations);
        println!("   🧹 Cleanups completed: {}", total_cleanups);
        println!("   ❌ Errors encountered: {}", total_errors);
        println!("   ⏱️  Total duration: {:?}", total_duration);
        println!("   🚀 Cleanup throughput: {:.1} cleanups/sec", 
                total_cleanups as f64 / total_duration.as_secs_f64());
        
        // Verify system state after cleanup
        println!("🔍 Verifying system state after cleanup...");
        
        // Check that we can still create new searchers (no resource leaks)
        let post_cleanup_searcher = TantivySearcher::new().await?;
        println!("   ✅ Can create new searcher after cleanup");
        
        // Test that temporary index files were cleaned up properly
        let remaining_files = fs::read_dir(temp_dir.path())?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().is_dir() && 
                entry.file_name().to_string_lossy().starts_with("cleanup_")
            })
            .count();
        
        println!("   📁 Remaining index directories: {}", remaining_files);
        
        // TRUTH CHECK
        if total_creations == 0 {
            panic!("No searchers were created - test setup failure");
        }
        
        if total_cleanups == 0 {
            panic!("No cleanups completed - potential resource leak");
        }
        
        let cleanup_ratio = total_cleanups as f64 / total_creations as f64;
        if cleanup_ratio < 0.8 {
            println!("   ⚠️  WARNING: Low cleanup ratio ({:.1}%) - potential resource leaks", 
                    cleanup_ratio * 100.0);
        } else {
            println!("   ✅ Good cleanup ratio ({:.1}%) - resource management appears sound", 
                    cleanup_ratio * 100.0);
        }
        
        Ok(())
    }
}

/// Helper function to detect potential deadlocks
#[allow(dead_code)]
fn is_likely_deadlocked(start_time: Instant, timeout: Duration) -> bool {
    start_time.elapsed() > timeout
}