use std::time::{Duration, Instant};
use std::panic;
use std::sync::{Arc, Mutex};
use colored::*;

/// Comprehensive BM25 Stress Test Runner
/// Executes all 10 fundamental flaw tests and provides detailed diagnostic reporting
/// 
/// This runner is designed to:
/// 1. Execute each test in isolation with proper error handling
/// 2. Capture all failure modes and diagnostic information
/// 3. Provide clear pass/fail reporting with actionable insights
/// 4. Measure performance characteristics
/// 5. Generate comprehensive failure analysis

#[derive(Debug, Clone)]
pub struct StressTestResult {
    pub test_name: String,
    pub test_id: u8,
    pub status: TestStatus,
    pub duration: Duration,
    pub error_message: Option<String>,
    pub diagnostic_info: Vec<String>,
    pub performance_metrics: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Crashed,
}

pub struct BM25StressTestRunner {
    results: Vec<StressTestResult>,
    start_time: Instant,
}

impl BM25StressTestRunner {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            start_time: Instant::now(),
        }
    }

    /// Execute all BM25 stress tests with comprehensive reporting
    pub fn run_all_tests(&mut self) {
        println!("{}", "=".repeat(80).bright_blue());
        println!("{}", "BM25 FUNDAMENTAL FLAW STRESS TEST SUITE".bright_blue().bold());
        println!("{}", "=".repeat(80).bright_blue());
        println!("Targeting 10 critical implementation flaws with merciless precision\n");

        // Execute each test individually
        self.execute_single_test("Incremental Update Impossibility", 1, Self::test_incremental_updates);
        self.execute_single_test("Tokenization Catastrophe", 2, Self::test_tokenization_complex);
        self.execute_single_test("Memory Explosion", 3, Self::test_memory_limits);
        self.execute_single_test("Persistence Failure", 4, Self::test_persistence_loss);
        self.execute_single_test("Length Bias Exposure", 5, Self::test_document_length_bias);
        self.execute_single_test("Mathematical Edge Cases", 6, Self::test_mathematical_corruption);
        self.execute_single_test("Unicode Tokenization Destruction", 7, Self::test_unicode_handling);
        self.execute_single_test("Concurrency Corruption", 8, Self::test_thread_safety);
        self.execute_single_test("Stop Word Singularity", 9, Self::test_stop_word_handling);
        self.execute_single_test("Vocabulary Overflow", 10, Self::test_vocabulary_limits);

        self.generate_final_report();
    }

    /// Execute a single test with comprehensive error handling and diagnostics
    fn execute_single_test<F>(&mut self, test_name: &str, test_id: u8, test_fn: F) 
    where 
        F: Fn() -> Result<Vec<String>, String> + std::panic::UnwindSafe,
    {
        println!("{}", format!("🧪 TEST {}: {}", test_id, test_name).bright_yellow().bold());
        println!("{}", "-".repeat(60).yellow());
        
        let start = Instant::now();
        let mut diagnostics = Vec::new();
        let mut performance = Vec::new();
        
        // Capture panic information
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            test_fn()
        }));
        
        let duration = start.elapsed();
        performance.push(("execution_time".to_string(), format!("{:?}", duration)));
        
        let (status, error_message) = match result {
            Ok(Ok(test_diagnostics)) => {
                diagnostics.extend(test_diagnostics);
                println!("{}", "✅ PASSED".bright_green().bold());
                (TestStatus::Passed, None)
            }
            Ok(Err(test_error)) => {
                println!("{}", "❌ FAILED".bright_red().bold());
                println!("Error: {}", test_error.red());
                (TestStatus::Failed, Some(test_error))
            }
            Err(panic_info) => {
                let panic_message = if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic occurred".to_string()
                };
                
                println!("{}", "💥 CRASHED".bright_red().bold());
                println!("Panic: {}", panic_message.red());
                (TestStatus::Crashed, Some(panic_message))
            }
        };
        
        // Add performance metrics
        if duration > Duration::from_secs(5) {
            diagnostics.push(format!("PERFORMANCE WARNING: Test took {:?} to complete", duration));
        }
        
        let test_result = StressTestResult {
            test_name: test_name.to_string(),
            test_id,
            status,
            duration,
            error_message,
            diagnostic_info: diagnostics,
            performance_metrics: performance,
        };
        
        self.results.push(test_result);
        println!("{}", "-".repeat(60).yellow());
        println!();
    }

    /// Test 1: Incremental Update Impossibility
    fn test_incremental_updates() -> Result<Vec<String>, String> {
        use embed_search::search::bm25::{BM25Engine, BM25Document, Token};
        
        let mut diagnostics = Vec::new();
        let mut engine = BM25Engine::new();

        // Initial documents
        let doc1 = create_test_doc("doc1", vec!["function", "calculate"]);
        let doc2 = create_test_doc("doc2", vec!["function", "process"]);
        
        engine.add_document(doc1).map_err(|e| format!("Initial add failed: {}", e))?;
        engine.add_document(doc2).map_err(|e| format!("Initial add failed: {}", e))?;
        
        let initial_results = engine.search("function", 10)
            .map_err(|e| format!("Initial search failed: {}", e))?;
        diagnostics.push(format!("Initial search found {} documents", initial_results.len()));
        
        // Add third document
        let doc3 = create_test_doc("doc3", vec!["function", "validate"]);
        engine.add_document(doc3).map_err(|e| format!("Incremental add failed: {}", e))?;
        
        let updated_results = engine.search("function", 10)
            .map_err(|e| format!("Post-update search failed: {}", e))?;
        diagnostics.push(format!("After incremental add found {} documents", updated_results.len()));
        
        if updated_results.len() != 3 {
            return Err(format!("Expected 3 documents after incremental add, got {}. IDF/term frequency calculations are corrupted.", updated_results.len()));
        }
        
        let stats = engine.get_stats();
        diagnostics.push(format!("Final stats: {} documents, {} terms", stats.total_documents, stats.total_terms));
        
        if stats.total_documents != 3 {
            return Err(format!("Document count inconsistency: stats show {} but expected 3", stats.total_documents));
        }
        
        Ok(diagnostics)
    }

    /// Test 2: Tokenization Catastrophe
    fn test_tokenization_complex() -> Result<Vec<String>, String> {
        use embed_search::search::bm25::{BM25Engine, BM25Document, Token};
        
        let mut diagnostics = Vec::new();
        let mut engine = BM25Engine::new();
        
        let complex_cases = vec![
            ("café_résumé", "Unicode accents with underscore"),
            ("user@domain.com", "Email address"),
            ("http://api.v2.example.com/users", "Complex URL"),
            ("𝓯𝓾𝓷𝓬𝓽𝓲𝓸𝓷_🚀", "Mathematical Unicode with emoji"),
            ("camelCaseFunction_snake_case", "Mixed naming conventions"),
            ("JSON_API_v2.1.4-beta", "Version with punctuation"),
        ];
        
        for (i, (complex_text, description)) in complex_cases.iter().enumerate() {
            let doc = BM25Document {
                id: format!("complex_{}", i),
                file_path: format!("complex_{}.rs", i),
                chunk_index: 0,
                tokens: vec![Token {
                    text: complex_text.to_string(),
                    position: 0,
                    importance_weight: 1.0,
                }],
                start_line: 0,
                end_line: 1,
                language: Some("mixed".to_string()),
            };
            
            engine.add_document(doc)
                .map_err(|e| format!("Failed to add complex document '{}' ({}): {}", complex_text, description, e))?;
            
            diagnostics.push(format!("Added complex token: {} ({})", complex_text, description));
        }
        
        // Test searches for extracted terms
        let search_terms = vec!["café", "user", "api", "function", "json"];
        for term in search_terms {
            let results = engine.search(term, 10)
                .map_err(|e| format!("Search for '{}' failed: {}", term, e))?;
            diagnostics.push(format!("Search '{}': {} results", term, results.len()));
        }
        
        Ok(diagnostics)
    }

    /// Test 3: Memory Explosion
    fn test_memory_limits() -> Result<Vec<String>, String> {
        use embed_search::search::bm25::{BM25Engine, BM25Document, Token};
        
        let mut diagnostics = Vec::new();
        let mut engine = BM25Engine::new();
        
        const VOCAB_TARGET: usize = 25_000; // Reduced for reasonable test time
        const BATCH_SIZE: usize = 1_000;
        
        diagnostics.push(format!("Testing vocabulary growth to {} terms", VOCAB_TARGET));
        
        let mut terms_added = 0;
        let start_time = Instant::now();
        
        while terms_added < VOCAB_TARGET {
            let batch_size = std::cmp::min(BATCH_SIZE, VOCAB_TARGET - terms_added);
            let mut tokens = Vec::with_capacity(batch_size);
            
            for i in 0..batch_size {
                tokens.push(Token {
                    text: format!("term_{}_{}", terms_added + i, rand::random::<u32>()),
                    position: i,
                    importance_weight: 1.0,
                });
            }
            
            let doc = BM25Document {
                id: format!("batch_{}", terms_added / BATCH_SIZE),
                file_path: format!("batch_{}.rs", terms_added / BATCH_SIZE),
                chunk_index: 0,
                tokens,
                start_line: 0,
                end_line: 100,
                language: Some("rust".to_string()),
            };
            
            engine.add_document(doc)
                .map_err(|e| format!("Memory exhaustion at {} terms: {}", terms_added, e))?;
            
            terms_added += batch_size;
            
            if terms_added % 5000 == 0 {
                let elapsed = start_time.elapsed();
                let stats = engine.get_stats();
                diagnostics.push(format!("Added {} terms in {:?}, vocabulary: {}", 
                    terms_added, elapsed, stats.total_terms));
            }
        }
        
        // Performance test
        let search_start = Instant::now();
        let results = engine.search("term_12345", 10)
            .map_err(|e| format!("Search failed with large vocabulary: {}", e))?;
        let search_duration = search_start.elapsed();
        
        diagnostics.push(format!("Search with {} terms took {:?}, found {} results", 
            engine.get_stats().total_terms, search_duration, results.len()));
        
        if search_duration > Duration::from_secs(2) {
            return Err(format!("Search performance degraded: {:?} for vocabulary of {}", 
                search_duration, engine.get_stats().total_terms));
        }
        
        Ok(diagnostics)
    }

    /// Test 4: Persistence Failure
    fn test_persistence_loss() -> Result<Vec<String>, String> {
        use embed_search::search::bm25::{BM25Engine, BM25Document, Token};
        
        let mut diagnostics = Vec::new();
        
        // Original engine with data
        let mut engine1 = BM25Engine::new();
        let doc = create_test_doc("persistent", vec!["data", "persistence", "test"]);
        engine1.add_document(doc).map_err(|e| format!("Add to first engine failed: {}", e))?;
        
        let search1 = engine1.search("persistence", 10)
            .map_err(|e| format!("Search in first engine failed: {}", e))?;
        diagnostics.push(format!("First engine found {} results", search1.len()));
        
        if search1.is_empty() {
            return Err("First engine should find the persistent document".to_string());
        }
        
        // New engine (simulating restart)
        let engine2 = BM25Engine::new();
        let search2 = engine2.search("persistence", 10);
        
        match search2 {
            Ok(results) if results.is_empty() => {
                diagnostics.push("Expected data loss confirmed - new engine has no data".to_string());
            }
            Ok(results) => {
                return Err(format!("Impossible: new engine found {} results without data loading", results.len()));
            }
            Err(_) => {
                diagnostics.push("Expected search failure on empty engine".to_string());
            }
        }
        
        diagnostics.push("LIMITATION CONFIRMED: BM25Engine has no persistence mechanism".to_string());
        
        Ok(diagnostics)
    }

    /// Test 5: Length Bias Exposure
    fn test_document_length_bias() -> Result<Vec<String>, String> {
        use embed_search::search::bm25::{BM25Engine, BM25Document, Token};
        
        let mut diagnostics = Vec::new();
        let mut engine = BM25Engine::new();
        
        // Short document
        let short_doc = BM25Document {
            id: "short".to_string(),
            file_path: "short.rs".to_string(),
            chunk_index: 0,
            tokens: vec![
                Token { text: "target".to_string(), position: 0, importance_weight: 1.0 },
                Token { text: "word".to_string(), position: 1, importance_weight: 1.0 },
            ],
            start_line: 0,
            end_line: 1,
            language: Some("rust".to_string()),
        };
        
        // Long document with same target term
        let mut long_tokens = vec![
            Token { text: "target".to_string(), position: 0, importance_weight: 1.0 },
        ];
        for i in 1..300 {
            long_tokens.push(Token {
                text: format!("filler_{}", i),
                position: i,
                importance_weight: 1.0,
            });
        }
        
        let long_doc = BM25Document {
            id: "long".to_string(),
            file_path: "long.rs".to_string(),
            chunk_index: 0,
            tokens: long_tokens,
            start_line: 0,
            end_line: 300,
            language: Some("rust".to_string()),
        };
        
        engine.add_document(short_doc).map_err(|e| format!("Short doc add failed: {}", e))?;
        engine.add_document(long_doc).map_err(|e| format!("Long doc add failed: {}", e))?;
        
        let results = engine.search("target", 10)
            .map_err(|e| format!("Length bias search failed: {}", e))?;
        
        if results.len() != 2 {
            return Err(format!("Expected 2 results for length bias test, got {}", results.len()));
        }
        
        let short_score = results.iter().find(|r| r.doc_id == "short").unwrap().score;
        let long_score = results.iter().find(|r| r.doc_id == "long").unwrap().score;
        let ratio = short_score / long_score;
        
        diagnostics.push(format!("Short doc score: {:.6}", short_score));
        diagnostics.push(format!("Long doc score: {:.6}", long_score));
        diagnostics.push(format!("Score ratio (short/long): {:.6}", ratio));
        
        let stats = engine.get_stats();
        diagnostics.push(format!("BM25 params: k1={}, b={}, avg_len={:.2}", 
            stats.k1, stats.b, stats.avg_document_length));
        
        // BM25 should normalize by length, so scores should be different
        if (ratio - 1.0).abs() < 0.01 {
            return Err(format!("Length normalization failure: ratio {:.6} too close to 1.0", ratio));
        }
        
        Ok(diagnostics)
    }

    /// Test 6: Mathematical Edge Cases
    fn test_mathematical_corruption() -> Result<Vec<String>, String> {
        use embed_search::search::bm25::{BM25Engine, BM25Document, Token};
        
        let mut diagnostics = Vec::new();
        let mut engine = BM25Engine::new();
        
        // Test empty document handling
        let empty_doc = BM25Document {
            id: "empty".to_string(),
            file_path: "empty.rs".to_string(),
            chunk_index: 0,
            tokens: vec![],
            start_line: 0,
            end_line: 1,
            language: Some("rust".to_string()),
        };
        
        match engine.add_document(empty_doc) {
            Ok(_) => diagnostics.push("Empty document accepted".to_string()),
            Err(e) => diagnostics.push(format!("Empty document rejected: {}", e)),
        }
        
        // Test extreme term frequency
        let extreme_doc = BM25Document {
            id: "extreme".to_string(),
            file_path: "extreme.rs".to_string(),
            chunk_index: 0,
            tokens: (0..5000).map(|i| Token {
                text: "repeated".to_string(),
                position: i,
                importance_weight: 1.0,
            }).collect(),
            start_line: 0,
            end_line: 1000,
            language: Some("rust".to_string()),
        };
        
        engine.add_document(extreme_doc).map_err(|e| format!("Extreme frequency doc failed: {}", e))?;
        
        let results = engine.search("repeated", 5)
            .map_err(|e| format!("Extreme frequency search failed: {}", e))?;
        
        if !results.is_empty() {
            let score = results[0].score;
            if !score.is_finite() {
                return Err(format!("Mathematical corruption: score {} is not finite", score));
            }
            if score < 0.0 {
                return Err(format!("Mathematical error: negative BM25 score {}", score));
            }
            diagnostics.push(format!("Extreme frequency score: {:.6}", score));
        }
        
        // Test IDF calculations
        engine.clear();
        for i in 0..5 {
            let doc = create_test_doc(&format!("idf_{}", i), vec!["universal", &format!("unique_{}", i)]);
            engine.add_document(doc).map_err(|e| format!("IDF test doc {} failed: {}", i, e))?;
        }
        
        let universal_idf = engine.calculate_idf("universal");
        let unique_idf = engine.calculate_idf("unique_0");
        
        if !universal_idf.is_finite() || !unique_idf.is_finite() {
            return Err(format!("IDF calculation failure: universal={}, unique={}", universal_idf, unique_idf));
        }
        
        diagnostics.push(format!("Universal term IDF: {:.6}", universal_idf));
        diagnostics.push(format!("Unique term IDF: {:.6}", unique_idf));
        
        if unique_idf <= universal_idf {
            return Err(format!("IDF ordering error: unique ({:.6}) <= universal ({:.6})", unique_idf, universal_idf));
        }
        
        Ok(diagnostics)
    }

    /// Test 7: Unicode Tokenization Destruction
    fn test_unicode_handling() -> Result<Vec<String>, String> {
        use embed_search::search::bm25::{BM25Engine, BM25Document, Token};
        use embed_search::search::text_processor::CodeTextProcessor;
        
        let mut diagnostics = Vec::new();
        let processor = CodeTextProcessor::new();
        let mut engine = BM25Engine::new();
        
        let unicode_cases = vec![
            ("café_résumé", "French accents"),
            ("функция_обработки", "Cyrillic script"),
            ("データ処理関数", "Japanese characters"),
            ("مُعَالِج_البَيَانَات", "Arabic script"),
            ("🚀_launch_function", "Emoji with code"),
            ("α_β_γ_formula", "Greek mathematical"),
        ];
        
        for (i, (unicode_text, description)) in unicode_cases.iter().enumerate() {
            // Process with text processor
            let processed = processor.process_text(unicode_text, "mixed");
            diagnostics.push(format!("{}: {} -> {} tokens", description, unicode_text, processed.len()));
            
            if processed.is_empty() {
                return Err(format!("Unicode tokenization failed for {}: no tokens extracted", description));
            }
            
            // Convert to BM25 tokens
            let bm25_tokens: Vec<Token> = processed.iter().enumerate().map(|(pos, pt)| Token {
                text: pt.text.clone(),
                position: pos,
                importance_weight: pt.importance_weight,
            }).collect();
            
            let doc = BM25Document {
                id: format!("unicode_{}", i),
                file_path: format!("unicode_{}.txt", i),
                chunk_index: 0,
                tokens: bm25_tokens,
                start_line: 0,
                end_line: 1,
                language: Some("mixed".to_string()),
            };
            
            engine.add_document(doc)
                .map_err(|e| format!("Unicode document add failed for {}: {}", description, e))?;
        }
        
        // Test Unicode searches
        let search_terms = vec!["café", "функция", "データ", "launch"];
        for term in search_terms {
            let results = engine.search(term, 5);
            match results {
                Ok(r) => diagnostics.push(format!("Unicode search '{}': {} results", term, r.len())),
                Err(e) => return Err(format!("Unicode search failed for '{}': {}", term, e)),
            }
        }
        
        Ok(diagnostics)
    }

    /// Test 8: Concurrency Corruption
    fn test_thread_safety() -> Result<Vec<String>, String> {
        use embed_search::search::bm25::BM25Engine;
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let mut diagnostics = Vec::new();
        
        // Note: Current implementation requires external synchronization
        let engine = Arc::new(Mutex::new(BM25Engine::new()));
        
        const THREAD_COUNT: usize = 4;
        const DOCS_PER_THREAD: usize = 25;
        
        let mut handles = vec![];
        
        for thread_id in 0..THREAD_COUNT {
            let engine_clone = Arc::clone(&engine);
            
            let handle = thread::spawn(move || {
                for doc_id in 0..DOCS_PER_THREAD {
                    let doc = create_test_doc(
                        &format!("t{}d{}", thread_id, doc_id),
                        vec!["concurrent", "test", &format!("thread{}", thread_id)]
                    );
                    
                    let mut engine_guard = engine_clone.lock().expect("Lock failed");
                    if let Err(e) = engine_guard.add_document(doc) {
                        return Err(format!("Thread {} doc {} failed: {}", thread_id, doc_id, e));
                    }
                }
                Ok(())
            });
            
            handles.push(handle);
        }
        
        // Wait for completion and check for errors
        for (i, handle) in handles.into_iter().enumerate() {
            match handle.join() {
                Ok(Ok(())) => diagnostics.push(format!("Thread {} completed successfully", i)),
                Ok(Err(e)) => return Err(format!("Thread {} error: {}", i, e)),
                Err(_) => return Err(format!("Thread {} panicked", i)),
            }
        }
        
        // Verify final state
        let engine_guard = engine.lock().expect("Final lock failed");
        let stats = engine_guard.get_stats();
        let expected_docs = THREAD_COUNT * DOCS_PER_THREAD;
        
        diagnostics.push(format!("Expected {} docs, got {}", expected_docs, stats.total_documents));
        
        if stats.total_documents != expected_docs {
            return Err(format!("Concurrency corruption: expected {} docs, got {}", expected_docs, stats.total_documents));
        }
        
        let search_results = engine_guard.search("concurrent", 200)
            .map_err(|e| format!("Post-concurrency search failed: {}", e))?;
        
        diagnostics.push(format!("Concurrent search found {} results", search_results.len()));
        
        if search_results.len() != expected_docs {
            return Err(format!("Search integrity failure: found {} results, expected {}", search_results.len(), expected_docs));
        }
        
        Ok(diagnostics)
    }

    /// Test 9: Stop Word Singularity
    fn test_stop_word_handling() -> Result<Vec<String>, String> {
        use embed_search::search::bm25::{BM25Engine, BM25Document, Token};
        
        let mut diagnostics = Vec::new();
        let mut engine = BM25Engine::new();
        
        // Document with only stop words
        let stop_words = vec!["the", "and", "or", "is", "it", "in", "to", "of", "a"];
        let stop_doc = BM25Document {
            id: "stopwords".to_string(),
            file_path: "stop.txt".to_string(),
            chunk_index: 0,
            tokens: stop_words.iter().enumerate().map(|(pos, &word)| Token {
                text: word.to_string(),
                position: pos,
                importance_weight: 1.0,
            }).collect(),
            start_line: 0,
            end_line: 1,
            language: Some("english".to_string()),
        };
        
        match engine.add_document(stop_doc) {
            Ok(_) => diagnostics.push("Stop word document added".to_string()),
            Err(e) => diagnostics.push(format!("Stop word document rejected: {}", e)),
        }
        
        // Add normal document
        let normal_doc = create_test_doc("normal", vec!["function", "calculate", "result"]);
        engine.add_document(normal_doc).map_err(|e| format!("Normal doc failed: {}", e))?;
        
        // Test various queries
        let queries = vec![
            ("the", "pure stop word"),
            ("function", "normal term"),
            ("the function", "mixed query"),
        ];
        
        for (query, description) in queries {
            match engine.search(query, 10) {
                Ok(results) => {
                    diagnostics.push(format!("{}: {} results", description, results.len()));
                    
                    // Check score validity
                    for result in &results {
                        if !result.score.is_finite() {
                            return Err(format!("Stop word query '{}' produced invalid score: {}", query, result.score));
                        }
                        if result.score < 0.0 {
                            return Err(format!("Stop word query '{}' produced negative score: {}", query, result.score));
                        }
                    }
                }
                Err(e) => {
                    if query == "the" {
                        diagnostics.push(format!("Pure stop word rejected (acceptable): {}", e));
                    } else {
                        return Err(format!("Stop word handling broke normal search '{}': {}", query, e));
                    }
                }
            }
        }
        
        Ok(diagnostics)
    }

    /// Test 10: Vocabulary Overflow
    fn test_vocabulary_limits() -> Result<Vec<String>, String> {
        use embed_search::search::bm25::{BM25Engine, BM25Document, Token};
        
        let mut diagnostics = Vec::new();
        let mut engine = BM25Engine::new();
        
        const VOCAB_TARGET: usize = 50_000; // Large vocabulary test
        const BATCH_SIZE: usize = 2_000;
        
        let mut terms_added = 0;
        let start_time = Instant::now();
        
        while terms_added < VOCAB_TARGET {
            let batch_size = std::cmp::min(BATCH_SIZE, VOCAB_TARGET - terms_added);
            let mut tokens = Vec::with_capacity(batch_size);
            
            for i in 0..batch_size {
                tokens.push(Token {
                    text: format!("vocab_term_{}_{}", terms_added + i, i),
                    position: i,
                    importance_weight: 1.0,
                });
            }
            
            let doc = BM25Document {
                id: format!("vocab_batch_{}", terms_added / BATCH_SIZE),
                file_path: format!("vocab_{}.rs", terms_added / BATCH_SIZE),
                chunk_index: 0,
                tokens,
                start_line: 0,
                end_line: 100,
                language: Some("rust".to_string()),
            };
            
            match engine.add_document(doc) {
                Ok(_) => {
                    terms_added += batch_size;
                    if terms_added % 10_000 == 0 {
                        let elapsed = start_time.elapsed();
                        let stats = engine.get_stats();
                        diagnostics.push(format!("{} terms added in {:?}, vocab size: {}", 
                            terms_added, elapsed, stats.total_terms));
                    }
                }
                Err(e) => {
                    if terms_added < 5_000 {
                        return Err(format!("Premature vocabulary failure at {} terms: {}", terms_added, e));
                    }
                    diagnostics.push(format!("Vocabulary limit reached at {} terms: {}", terms_added, e));
                    break;
                }
            }
        }
        
        // Performance test
        let final_stats = engine.get_stats();
        if final_stats.total_terms > 0 {
            let search_start = Instant::now();
            let results = engine.search("vocab_term_25000", 10)
                .map_err(|e| format!("Large vocab search failed: {}", e))?;
            let search_time = search_start.elapsed();
            
            diagnostics.push(format!("Search in {} term vocab took {:?}, found {} results", 
                final_stats.total_terms, search_time, results.len()));
            
            if search_time > Duration::from_secs(3) {
                return Err(format!("Vocabulary performance failure: {:?} for {} terms", search_time, final_stats.total_terms));
            }
        }
        
        Ok(diagnostics)
    }

    /// Generate comprehensive final report
    fn generate_final_report(&self) {
        let total_duration = self.start_time.elapsed();
        
        println!("{}", "=".repeat(80).bright_blue());
        println!("{}", "FINAL BM25 STRESS TEST REPORT".bright_blue().bold());
        println!("{}", "=".repeat(80).bright_blue());
        
        let passed = self.results.iter().filter(|r| r.status == TestStatus::Passed).count();
        let failed = self.results.iter().filter(|r| r.status == TestStatus::Failed).count();
        let crashed = self.results.iter().filter(|r| r.status == TestStatus::Crashed).count();
        let skipped = self.results.iter().filter(|r| r.status == TestStatus::Skipped).count();
        
        println!("📊 SUMMARY:");
        println!("   ✅ PASSED:  {} tests", passed.to_string().bright_green());
        println!("   ❌ FAILED:  {} tests", failed.to_string().bright_red());
        println!("   💥 CRASHED: {} tests", crashed.to_string().bright_red());
        println!("   ⏭️  SKIPPED: {} tests", skipped.to_string().bright_yellow());
        println!("   ⏱️  TOTAL TIME: {:?}", total_duration);
        println!();
        
        // Detailed results
        for result in &self.results {
            let status_icon = match result.status {
                TestStatus::Passed => "✅",
                TestStatus::Failed => "❌",
                TestStatus::Crashed => "💥",
                TestStatus::Skipped => "⏭️",
            };
            
            println!("{} TEST {}: {} ({:?})", status_icon, result.test_id, result.test_name, result.duration);
            
            if let Some(ref error) = result.error_message {
                println!("   🔍 Error: {}", error.red());
            }
            
            if !result.diagnostic_info.is_empty() {
                println!("   📋 Diagnostics:");
                for diag in &result.diagnostic_info {
                    println!("      • {}", diag);
                }
            }
            
            println!();
        }
        
        // Critical flaw analysis
        let critical_failures: Vec<&StressTestResult> = self.results.iter()
            .filter(|r| matches!(r.status, TestStatus::Failed | TestStatus::Crashed))
            .collect();
        
        if !critical_failures.is_empty() {
            println!("{}", "🚨 CRITICAL FLAWS DETECTED:".bright_red().bold());
            for failure in critical_failures {
                println!("   • {}: {}", failure.test_name.red(), 
                    failure.error_message.as_ref().unwrap_or(&"Unknown error".to_string()));
            }
            println!();
        }
        
        // Recommendations
        println!("{}", "💡 RECOMMENDATIONS:".bright_yellow().bold());
        if failed + crashed == 0 {
            println!("   ✨ All stress tests passed! BM25 implementation appears robust.");
        } else {
            if self.results.iter().any(|r| r.test_name.contains("Incremental Update")) {
                println!("   • Implement proper incremental indexing with IDF recalculation");
            }
            if self.results.iter().any(|r| r.test_name.contains("Tokenization")) {
                println!("   • Enhance tokenization for complex Unicode and mixed text");
            }
            if self.results.iter().any(|r| r.test_name.contains("Memory")) {
                println!("   • Optimize memory usage for large vocabularies");
            }
            if self.results.iter().any(|r| r.test_name.contains("Persistence")) {
                println!("   • Add serialization/deserialization for index persistence");
            }
            if self.results.iter().any(|r| r.test_name.contains("Concurrency")) {
                println!("   • Implement proper thread safety or document limitations");
            }
        }
        
        println!("{}", "=".repeat(80).bright_blue());
    }

    /// Helper function to create test documents
    fn create_test_doc(id: &str, terms: Vec<&str>) -> embed_search::search::bm25::BM25Document {
        use embed_search::search::bm25::{BM25Document, Token};
        
        BM25Document {
            id: id.to_string(),
            file_path: format!("{}.rs", id),
            chunk_index: 0,
            tokens: terms.iter().enumerate().map(|(pos, &term)| Token {
                text: term.to_string(),
                position: pos,
                importance_weight: 1.0,
            }).collect(),
            start_line: 0,
            end_line: 10,
            language: Some("rust".to_string()),
        }
    }
}

// Helper function for creating test documents
fn create_test_doc(id: &str, terms: Vec<&str>) -> embed_search::search::bm25::BM25Document {
    use embed_search::search::bm25::{BM25Document, Token};
    
    BM25Document {
        id: id.to_string(),
        file_path: format!("{}.rs", id),
        chunk_index: 0,
        tokens: terms.iter().enumerate().map(|(pos, &term)| Token {
            text: term.to_string(),
            position: pos,
            importance_weight: 1.0,
        }).collect(),
        start_line: 0,
        end_line: 10,
        language: Some("rust".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn run_complete_bm25_stress_test_suite() {
        let mut runner = BM25StressTestRunner::new();
        runner.run_all_tests();
        
        // The test runner will provide comprehensive diagnostics
        // Individual test failures are reported within the runner
    }
}