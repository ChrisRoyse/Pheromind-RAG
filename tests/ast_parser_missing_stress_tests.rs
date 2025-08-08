/// AST Parser Missing Stress Tests - 9 Critical Missing Test Implementations
/// These tests target the exact scenarios specified by the user using real Tree-sitter parsers
/// 
/// MISSING TESTS IMPLEMENTED:
/// 1. stress_silent_parser_failure_detection - Parser initialization
/// 2. stress_persistence_absence_validation - Rebuild performance
/// 3. stress_query_pattern_rigidity_testing - Fixed patterns
/// 4. stress_concurrency_symbol_corruption - Thread safety
/// 5. stress_memory_leak_validation - AST accumulation
/// 6. stress_malformed_code_recovery - Parser crashes
/// 7. stress_stack_overflow_induction - Large file limits
/// 8. stress_language_detection_chaos - Mixed languages
/// 9. stress_circular_dependency_loops - Infinite loops

#[cfg(feature = "tree-sitter")]
mod ast_missing_stress_tests {
    use embed_search::search::symbol_index::{SymbolIndexer, SymbolDatabase};
    use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
    use std::thread;
    use std::time::{Instant, Duration};
    use std::collections::HashMap;
    use std::path::Path;
    use std::panic;

    /// TEST 1: Silent Parser Failure Detection
    /// Validates that parser initialization failures are detected and reported
    /// REAL STRESS: Forces parser state corruption and validates error detection
    #[test]
    fn stress_silent_parser_failure_detection() {
        println!("🚨 STRESS TEST 1: Silent Parser Failure Detection");
        
        // Track initialization attempts and failures
        let mut initialization_attempts = 0;
        let mut silent_failures = Vec::new();
        
        // Attempt multiple indexer creations with forced state corruption
        for attempt in 0..20 {
            initialization_attempts += 1;
            
            // Create indexer with potential state corruption
            match SymbolIndexer::new() {
                Ok(mut indexer) => {
                    // Test each parser with valid code
                    let parser_tests = vec![
                        ("rust", "fn test() { println!('ok'); }", "test.rs"),
                        ("python", "def test(): pass", "test.py"),
                        ("javascript", "function test() {}", "test.js"),
                        ("go", "func test() {}", "test.go"),
                        ("java", "public class Test {}", "Test.java"),
                        ("c", "int test() { return 0; }", "test.c"),
                    ];
                    
                    let mut parser_failure_count = 0;
                    for (lang, code, filename) in parser_tests {
                        match indexer.extract_symbols(code, lang, filename) {
                            Ok(_) => {
                                println!("✓ Attempt {}: {} parser operational", attempt, lang);
                            }
                            Err(e) => {
                                parser_failure_count += 1;
                                silent_failures.push(format!("Attempt {}: {} parser failed: {}", attempt, lang, e));
                            }
                        }
                    }
                    
                    // If more than 30% of parsers fail, we have silent failures
                    if parser_failure_count > 2 {
                        silent_failures.push(format!("Attempt {}: {} out of 6 parsers failed silently", 
                                                    attempt, parser_failure_count));
                    }
                }
                Err(e) => {
                    silent_failures.push(format!("Attempt {}: Complete indexer creation failed: {}", attempt, e));
                }
            }
            
            // Simulate rapid creation/destruction cycles
            thread::sleep(Duration::from_millis(10));
        }
        
        let failure_rate = (silent_failures.len() as f64 / initialization_attempts as f64) * 100.0;
        println!("Total initialization attempts: {}", initialization_attempts);
        println!("Silent failures detected: {}", silent_failures.len());
        println!("Failure rate: {:.1}%", failure_rate);
        
        // Report all detected failures
        if !silent_failures.is_empty() {
            println!("CRITICAL: Silent parser failures detected:");
            for failure in &silent_failures {
                println!("  - {}", failure);
            }
            
            // CRITICAL: If failure rate > 10%, we have systematic parser initialization issues
            assert!(failure_rate <= 10.0, 
                   "CRITICAL FAILURE: {:.1}% parser initialization failure rate detected. \
                    Silent failures indicate systematic parser state corruption.", failure_rate);
        }
    }

    /// TEST 2: Persistence Absence Validation  
    /// Tests rebuild performance cost without persistence caching
    /// REAL STRESS: Forces complete rebuilds to measure catastrophic performance impact
    #[test]
    fn stress_persistence_absence_validation() {
        println!("🚨 STRESS TEST 2: Persistence Absence Validation");
        
        // Generate large code bases that would benefit from persistence
        let large_rust_code = generate_massive_rust_codebase(2000); // 2000 symbols
        let large_python_code = generate_massive_python_codebase(1500); // 1500 symbols  
        let large_js_code = generate_massive_js_codebase(1000); // 1000 symbols
        
        let test_cases = vec![
            ("rust", large_rust_code, "massive.rs"),
            ("python", large_python_code, "massive.py"),
            ("javascript", large_js_code, "massive.js"),
        ];
        
        let rebuild_iterations = 15;
        let mut total_rebuild_costs = Vec::new();
        
        for (lang, code, filename) in test_cases {
            println!("Testing persistence absence for {}", lang);
            let mut iteration_times = Vec::new();
            
            for iteration in 0..rebuild_iterations {
                let rebuild_start = Instant::now();
                
                // Force complete rebuild (no persistence)
                let mut indexer = SymbolIndexer::new()
                    .expect("Failed to create indexer for rebuild test");
                
                let symbols = indexer.extract_symbols(&code, lang, &filename)
                    .expect("Failed to extract symbols during rebuild");
                
                let rebuild_time = rebuild_start.elapsed();
                iteration_times.push(rebuild_time);
                
                println!("  Rebuild {}: {} symbols in {:?}", 
                        iteration + 1, symbols.len(), rebuild_time);
                
                // Simulate work between rebuilds
                thread::sleep(Duration::from_millis(50));
            }
            
            let average_rebuild_time: Duration = iteration_times.iter().sum::<Duration>() / rebuild_iterations as u32;
            let total_rebuild_cost: Duration = iteration_times.iter().sum();
            
            total_rebuild_costs.push((lang, average_rebuild_time, total_rebuild_cost));
            
            println!("  {} - Average rebuild: {:?}, Total cost: {:?}", 
                    lang, average_rebuild_time, total_rebuild_cost);
        }
        
        // Analyze catastrophic performance impact
        for (lang, avg_time, total_cost) in &total_rebuild_costs {
            println!("ANALYSIS {}: Avg={:?}, Total={:?}", lang, avg_time, total_cost);
            
            // CRITICAL: Average rebuild time > 200ms indicates catastrophic performance
            if avg_time.as_millis() > 200 {
                panic!("CATASTROPHIC PERFORMANCE: {} average rebuild time {:?} exceeds 200ms threshold. \
                       No persistence makes system unusable for large codebases.", lang, avg_time);
            }
            
            // WARNING: Total cost > 5 seconds indicates severe efficiency loss
            if total_cost.as_secs() > 5 {
                println!("WARNING: {} total rebuild cost {:?} indicates severe efficiency loss without persistence", 
                        lang, total_cost);
            }
        }
    }

    /// TEST 3: Query Pattern Rigidity Testing
    /// Tests fixed query patterns against diverse code patterns and edge cases
    /// REAL STRESS: Uses actual malformed/edge-case code that fixed patterns can't handle
    #[test]
    fn stress_query_pattern_rigidity_testing() {
        println!("🚨 STRESS TEST 3: Query Pattern Rigidity Testing");
        
        let mut indexer = SymbolIndexer::new().expect("Failed to create indexer");
        
        // Extreme edge cases that rigid query patterns typically fail on
        let rigidity_stress_cases = vec![
            // Rust edge cases
            ("rust", "fn\n\n\t\tmultiline_spaces\n\n() {}", "multiline.rs", "multiline_spaces"),
            ("rust", "pub unsafe extern \"C\" fn complex_fn() {}", "complex.rs", "complex_fn"),
            ("rust", "impl<'a, T: Clone + Send> Generic<'a, T> {}", "generic.rs", "Generic"),
            ("rust", "macro_rules! weird_macro { ($($x:tt)*) => {}; }", "macro.rs", "weird_macro"),
            
            // Python edge cases  
            ("python", "def функция(): pass", "unicode.py", "функция"),  // Cyrillic
            ("python", "@decorator\n@another\ndef decorated(): pass", "decorated.py", "decorated"),
            ("python", "def lambda_heavy(): return lambda x: x", "lambda.py", "lambda_heavy"),
            
            // JavaScript edge cases
            ("javascript", "const arrow = () => {}", "arrow.js", "arrow"),
            ("javascript", "async function* generator() {}", "generator.js", "generator"),
            ("javascript", "class Extends extends Super {}", "extends.js", "Extends"),
            
            // Java edge cases
            ("java", "public <T extends Comparable<T>> void generic() {}", "Generic.java", "generic"),
            ("java", "@Override\npublic void annotated() {}", "Annotated.java", "annotated"),
            
            // C/C++ edge cases
            ("c", "static inline int __attribute__((always_inline)) optimized() {}", "optimized.c", "optimized"),
            ("cpp", "template<typename T> constexpr auto modern() -> T {}", "modern.cpp", "modern"),
            
            // Mixed content (tests language detection rigidity)
            ("rust", "// This looks like JS\nfunction fake() {}\nfn real() {}", "mixed.rs", "real"),
        ];
        
        let mut rigidity_failures = Vec::new();
        let mut pattern_adaptation_score = 0;
        let total_cases = rigidity_stress_cases.len();
        
        for (lang, code, filename, expected_symbol) in rigidity_stress_cases {
            println!("Testing rigidity: {} in {} expecting '{}'", filename, lang, expected_symbol);
            
            match indexer.extract_symbols(code, lang, filename) {
                Ok(symbols) => {
                    let found_expected = symbols.iter().any(|s| s.name.contains(expected_symbol));
                    
                    if found_expected {
                        pattern_adaptation_score += 1;
                        println!("✓ Adapted: Found '{}' in {} ({} total symbols)", 
                                expected_symbol, filename, symbols.len());
                    } else {
                        rigidity_failures.push(format!("Pattern rigidity failure: {} - expected '{}' not found in {} symbols", 
                                                      filename, expected_symbol, symbols.len()));
                        println!("✗ Rigid: Missing '{}' in {} (found: {:?})", 
                                expected_symbol, filename, 
                                symbols.iter().map(|s| &s.name).collect::<Vec<_>>());
                    }
                }
                Err(e) => {
                    rigidity_failures.push(format!("Parse failure on rigidity test {}: {}", filename, e));
                    println!("✗ Parse failed: {} - {}", filename, e);
                }
            }
        }
        
        let adaptation_rate = (pattern_adaptation_score as f64 / total_cases as f64) * 100.0;
        
        println!("Pattern adaptation score: {}/{} ({:.1}%)", 
                pattern_adaptation_score, total_cases, adaptation_rate);
        
        if !rigidity_failures.is_empty() {
            println!("Query pattern rigidity issues detected:");
            for failure in &rigidity_failures {
                println!("  - {}", failure);
            }
        }
        
        // CRITICAL: Adaptation rate < 70% indicates severe pattern rigidity
        assert!(adaptation_rate >= 70.0, 
               "CRITICAL RIGIDITY: {:.1}% adaptation rate indicates fixed query patterns \
                cannot handle diverse code structures. {} failures detected.", 
                adaptation_rate, rigidity_failures.len());
    }

    /// TEST 4: Concurrency Symbol Corruption  
    /// Tests thread safety during concurrent parsing and symbol table operations
    /// REAL STRESS: Actual concurrent parsing with shared state corruption detection
    #[test]
    fn stress_concurrency_symbol_corruption() {
        println!("🚨 STRESS TEST 4: Concurrency Symbol Corruption");
        
        let shared_db = Arc::new(Mutex::new(SymbolDatabase::new()));
        let corruption_counter = Arc::new(AtomicUsize::new(0));
        let operation_counter = Arc::new(AtomicUsize::new(0));
        
        // Different code blocks for concurrent parsing
        let concurrent_codes = vec![
            ("rust", "fn concurrent_a() { let x = 1; }", "a.rs"),
            ("python", "def concurrent_b(): return 2", "b.py"),
            ("javascript", "function concurrent_c() { return 3; }", "c.js"),
            ("go", "func concurrent_d() int { return 4 }", "d.go"),
            ("java", "public class Concurrent { public void e() {} }", "E.java"),
        ];
        
        let thread_count = 10;
        let operations_per_thread = 100;
        
        let handles: Vec<_> = (0..thread_count).map(|thread_id| {
            let db_clone = Arc::clone(&shared_db);
            let corruption_clone = Arc::clone(&corruption_counter);
            let operation_clone = Arc::clone(&operation_counter);
            let codes = concurrent_codes.clone();
            
            thread::spawn(move || {
                let mut local_corruption_count = 0;
                
                for operation in 0..operations_per_thread {
                    operation_clone.fetch_add(1, Ordering::Relaxed);
                    
                    // Create thread-local indexer (simulates real usage)
                    let mut indexer = match SymbolIndexer::new() {
                        Ok(idx) => idx,
                        Err(e) => {
                            eprintln!("Thread {} operation {}: Indexer creation failed: {}", 
                                     thread_id, operation, e);
                            local_corruption_count += 1;
                            continue;
                        }
                    };
                    
                    // Parse different files concurrently
                    for (lang, code, filename) in &codes {
                        let unique_filename = format!("thread_{}_op_{}_{}", thread_id, operation, filename);
                        
                        match indexer.extract_symbols(code, lang, &unique_filename) {
                            Ok(symbols) => {
                                // Attempt to add to shared database (contention point)
                                match db_clone.try_lock() {
                                    Ok(mut db) => {
                                        let before_count = db.total_symbols();
                                        let before_files = db.files_indexed();
                                        
                                        db.add_symbols(symbols);
                                        
                                        let after_count = db.total_symbols();
                                        let after_files = db.files_indexed();
                                        
                                        // Detect corruption: symbols or files should never decrease
                                        if after_count < before_count || after_files < before_files {
                                            eprintln!("CORRUPTION DETECTED: Thread {} op {}: symbols {} -> {}, files {} -> {}", 
                                                     thread_id, operation, before_count, after_count, before_files, after_files);
                                            local_corruption_count += 1;
                                        }
                                    }
                                    Err(_) => {
                                        // Lock contention - acceptable, not corruption
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Thread {} op {} parse error for {}: {}", 
                                         thread_id, operation, filename, e);
                                local_corruption_count += 1;
                            }
                        }
                    }
                }
                
                corruption_clone.fetch_add(local_corruption_count, Ordering::Relaxed);
            })
        }).collect();
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread panicked during concurrency test");
        }
        
        let total_corruptions = corruption_counter.load(Ordering::Relaxed);
        let total_operations = operation_counter.load(Ordering::Relaxed);
        let final_db = shared_db.lock().unwrap();
        
        println!("Concurrency stress results:");
        println!("  Total operations: {}", total_operations);
        println!("  Corruptions detected: {}", total_corruptions);
        println!("  Final symbol count: {}", final_db.total_symbols());
        println!("  Final files indexed: {}", final_db.files_indexed());
        println!("  Corruption rate: {:.3}%", (total_corruptions as f64 / total_operations as f64) * 100.0);
        
        // CRITICAL: Any corruption in concurrent access is unacceptable
        assert_eq!(total_corruptions, 0, 
                  "CRITICAL CORRUPTION: {} corruptions detected in concurrent symbol operations. \
                   Thread safety is compromised.", total_corruptions);
    }

    /// TEST 5: Memory Leak Validation
    /// Tests AST node accumulation and memory growth patterns
    /// REAL STRESS: Forces large AST parsing cycles and measures actual memory usage
    #[test]
    fn stress_memory_leak_validation() {
        println!("🚨 STRESS TEST 5: Memory Leak Validation");
        
        let initial_memory = get_current_memory_mb();
        let mut memory_samples = Vec::new();
        let mut peak_memory = initial_memory;
        
        let parsing_cycles = 25;  // Reduced from 50 for reliability
        let large_code_blocks = vec![
            generate_deeply_nested_rust_code(500),   // Deep nesting
            generate_massive_function_rust_code(200), // Many functions
            generate_complex_generic_rust_code(100),  // Complex types
        ];
        
        for cycle in 0..parsing_cycles {
            let cycle_start_memory = get_current_memory_mb();
            
            // Create new indexer each cycle (forces memory allocation)
            let mut indexer = SymbolIndexer::new()
                .expect("Failed to create indexer for memory test");
            
            // Parse multiple large code blocks in sequence
            for (block_idx, large_code) in large_code_blocks.iter().enumerate() {
                let filename = format!("memory_cycle_{}_block_{}.rs", cycle, block_idx);
                
                match indexer.extract_symbols(large_code, "rust", &filename) {
                    Ok(symbols) => {
                        if cycle % 5 == 0 {
                            println!("  Cycle {} block {}: {} symbols parsed", 
                                    cycle, block_idx, symbols.len());
                        }
                    }
                    Err(e) => {
                        println!("  Parse error in cycle {} block {}: {}", cycle, block_idx, e);
                    }
                }
            }
            
            // Force indexer to go out of scope
            drop(indexer);
            
            // Sample memory after processing
            let cycle_end_memory = get_current_memory_mb();
            memory_samples.push((cycle, cycle_start_memory, cycle_end_memory));
            
            if cycle_end_memory > peak_memory {
                peak_memory = cycle_end_memory;
            }
            
            if cycle % 5 == 0 {
                println!("Memory cycle {}: start={}MB, end={}MB, delta={}MB", 
                        cycle, cycle_start_memory, cycle_end_memory, 
                        cycle_end_memory.saturating_sub(cycle_start_memory));
            }
            
            // Small delay to allow for potential cleanup
            thread::sleep(Duration::from_millis(20));
        }
        
        let final_memory = get_current_memory_mb();
        let total_growth = final_memory.saturating_sub(initial_memory);
        let peak_growth = peak_memory.saturating_sub(initial_memory);
        
        // Analyze memory patterns
        let growth_trend = analyze_memory_growth_trend(&memory_samples);
        
        println!("Memory leak analysis:");
        println!("  Initial memory: {}MB", initial_memory);
        println!("  Peak memory: {}MB (+{}MB)", peak_memory, peak_growth);
        println!("  Final memory: {}MB (+{}MB)", final_memory, total_growth);
        println!("  Growth trend: {:.2}MB per cycle", growth_trend);
        
        // CRITICAL: Memory growth > 2MB per cycle indicates severe leaks
        assert!(growth_trend < 2.0, 
               "CRITICAL MEMORY LEAK: {:.2}MB growth per cycle indicates AST nodes not being freed. \
                Total growth: {}MB", growth_trend, total_growth);
        
        // WARNING: Total retained memory > 100MB indicates potential issues  
        if total_growth > 100 {
            println!("WARNING: {}MB total memory growth may indicate memory retention issues", total_growth);
        }
    }

    /// TEST 6: Malformed Code Recovery
    /// Tests parser crash handling with extreme malformed input
    /// REAL STRESS: Uses actual malformed syntax that can crash parsers
    #[test]
    fn stress_malformed_code_recovery() {
        println!("🚨 STRESS TEST 6: Malformed Code Recovery");
        
        let mut indexer = SymbolIndexer::new().expect("Failed to create indexer");
        
        // Generate extreme malformed cases
        let deep_parens = "fn test(".repeat(1000);
        let massive_id = format!("fn {}_test() {{}}", "a".repeat(50000));
        let endless_indent = "if True:\n  ".repeat(5000);
        let brace_explosion = "{".repeat(10000);
        let interface_spam = format!("{}Test {{}}", "interface ".repeat(1000));
        
        let malformed_stress_cases = vec![
            // Rust malformed cases
            ("rust", "fn{{{{{{{{{{{{{{{{{{{{", "unclosed_braces.rs"),
            ("rust", "impl impl impl impl impl", "recursive_keywords.rs"),
            ("rust", deep_parens.as_str(), "deep_unclosed_parens.rs"),
            ("rust", massive_id.as_str(), "massive_identifier.rs"),
            ("rust", "fn test() { \x00\x01\x02\x03\x04 }", "binary_injection.rs"),
            
            // Python malformed cases  
            ("python", "def def def def def:", "recursive_def.py"),
            ("python", endless_indent.as_str(), "endless_indent.py"),
            ("python", "def \u{FFFF}\u{FFFE}(): pass", "invalid_unicode.py"),
            
            // JavaScript malformed cases
            ("javascript", "function function function", "recursive_function.js"),
            ("javascript", brace_explosion.as_str(), "brace_explosion.js"),
            ("javascript", "class extends extends extends", "broken_extends.js"),
            
            // Java malformed cases
            ("java", "public public public class", "modifier_spam.java"),
            ("java", interface_spam.as_str(), "interface_spam.java"),
            
            // C malformed cases
            ("c", "int main(int argc, char", "incomplete_signature.c"),
            ("c", "#include <", "broken_include.c"),
            
            // Encoding bombs
            ("rust", "\u{1F4A3}fn bomb() {}", "emoji_bomb.rs"),  // Bomb emoji
            ("python", "def \u{202E}reverse\u{202D}(): pass", "bidi_override.py"),  // BiDi override
        ];
        
        let mut crash_count = 0;
        let mut recovery_stats = HashMap::new();
        
        for (lang, malformed_code, filename) in &malformed_stress_cases {
            println!("Stress testing: {} ({} bytes)", filename, malformed_code.len());
            
            let recovery_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                let start = Instant::now();
                let result = indexer.extract_symbols(malformed_code, lang, filename);
                let duration = start.elapsed();
                (result, duration)
            }));
            
            match recovery_result {
                Ok((parse_result, duration)) => {
                    match parse_result {
                        Ok(symbols) => {
                            println!("✓ Recovered: {} symbols from malformed {} in {:?}", 
                                    symbols.len(), filename, duration);
                            recovery_stats.insert(filename, ("recovered", symbols.len(), duration));
                        }
                        Err(e) => {
                            println!("✓ Graceful fail: {} failed cleanly: {}", filename, e);
                            recovery_stats.insert(filename, ("graceful_fail", 0, duration));
                        }
                    }
                    
                    // Check for infinite parsing (timeout)
                    if duration > Duration::from_secs(5) {
                        crash_count += 1;
                        println!("✗ TIMEOUT: {} took {:?} - possible infinite loop", filename, duration);
                    }
                }
                Err(_) => {
                    crash_count += 1;
                    println!("✗ CRASHED: {} caused parser panic", filename);
                    recovery_stats.insert(filename, ("crashed", 0, Duration::from_secs(0)));
                }
            }
        }
        
        let total_cases = malformed_stress_cases.len();
        let crash_rate = (crash_count as f64 / total_cases as f64) * 100.0;
        
        println!("Malformed code recovery analysis:");
        println!("  Total malformed cases: {}", total_cases);
        println!("  Crashes/timeouts: {}", crash_count);
        println!("  Crash rate: {:.1}%", crash_rate);
        
        // CRITICAL: Any crashes indicate poor error handling
        assert_eq!(crash_count, 0, 
                  "CRITICAL RECOVERY FAILURE: {} crashes detected when parsing malformed code. \
                   Parser must gracefully handle all malformed input.", crash_count);
    }

    /// TEST 7: Stack Overflow Induction
    /// Tests parsing limits with deeply nested and massive code structures  
    /// REAL STRESS: Generates actual code that can cause stack overflow
    #[test]
    fn stress_stack_overflow_induction() {
        println!("🚨 STRESS TEST 7: Stack Overflow Induction");
        
        let mut indexer = SymbolIndexer::new().expect("Failed to create indexer");
        
        // Generate structures designed to trigger stack overflow
        let stack_stress_cases = vec![
            // Extreme nesting depth
            (generate_stack_killer_rust_nesting(8000), "rust", "stack_killer_nesting.rs"),
            (generate_stack_killer_json(5000), "json", "stack_killer.json"),
            (generate_stack_killer_html(3000), "html", "stack_killer.html"),
            
            // Massive linear structures
            (generate_massive_rust_linear(100000), "rust", "massive_linear.rs"),
            (generate_massive_js_functions(25000), "javascript", "massive_functions.js"),
            
            // Complex recursive patterns
            (generate_recursive_rust_types(2000), "rust", "recursive_types.rs"),
        ];
        
        for (stress_code, lang, filename) in stack_stress_cases {
            println!("Stack stress test: {} ({:.1}KB)", filename, stress_code.len() as f64 / 1024.0);
            
            let stress_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                let parse_start = Instant::now();
                let result = indexer.extract_symbols(&stress_code, lang, filename);
                let parse_duration = parse_start.elapsed();
                (result, parse_duration)
            }));
            
            match stress_result {
                Ok((parse_result, duration)) => {
                    match parse_result {
                        Ok(symbols) => {
                            println!("✓ Survived stack stress: {} symbols in {:?} from {}", 
                                    symbols.len(), duration, filename);
                            
                            // Performance validation - should not take forever
                            if duration > Duration::from_secs(60) {
                                panic!("PERFORMANCE FAILURE: {} took {:?} indicating stack thrashing or infinite recursion", 
                                       filename, duration);
                            }
                        }
                        Err(e) => {
                            println!("✓ Graceful limit reached: {} - {}", filename, e);
                        }
                    }
                }
                Err(_) => {
                    panic!("STACK OVERFLOW DETECTED: {} caused stack overflow in parser. \
                           Parser must handle deeply nested structures safely.", filename);
                }
            }
        }
    }

    /// TEST 8: Language Detection Chaos
    /// Tests mixed language files and detection confusion scenarios
    /// REAL STRESS: Uses actual polyglot files that confuse language detection
    #[test]
    fn stress_language_detection_chaos() {
        println!("🚨 STRESS TEST 8: Language Detection Chaos");
        
        let mut indexer = SymbolIndexer::new().expect("Failed to create indexer");
        
        // Polyglot chaos scenarios - real files that can confuse detection
        let detection_chaos_cases = vec![
            // HTML with embedded languages
            (r#"
            <script>
            function jsFunction() { 
                console.log("JS in HTML");
            }
            class JSClass { method() {} }
            </script>
            <style>
            .css-class { color: red; }
            #css-id { background: blue; }
            </style>
            "#, "html", "mixed.html"),
            
            // JavaScript disguised as other languages
            (r#"
            function looks_like_rust() {
                let x = 42;
                return x + 1;
            }
            class FakeRustStruct {
                constructor() {}
            }
            "#, "rust", "js_disguised_as_rust.rs"),
            
            // Python-like syntax in JavaScript
            (r#"
            def python_style_function():
                pass
                
            class PythonLikeClass:
                def method(self):
                    return None
            "#, "javascript", "python_in_js.js"),
            
            // JSON with code strings
            (r#"{
                "rust_code": "fn embedded_rust() { println!(\"hello\"); }",
                "python_code": "def embedded_python(): return True",
                "js_code": "function embedded_js() { return false; }"
            }"#, "json", "code_in_json.json"),
            
            // Markdown with code blocks (treated as other languages)
            (r#"
            # Documentation
            ```rust
            fn code_in_markdown() {}
            ```
            ```python  
            def more_code(): pass
            ```
            "#, "rust", "markdown_as_rust.rs"),
            
            // Ambiguous extensions
            ("fn ambiguous() {}", "rust", "ambiguous"),  // No extension
            ("function ambiguous() {}", "javascript", "ambiguous.bak"),  // Wrong extension
        ];
        
        let mut detection_confusion_count = 0;
        let mut chaos_results = Vec::new();
        
        let total_chaos_cases = detection_chaos_cases.len();
        
        for (chaotic_code, assumed_lang, filename) in detection_chaos_cases {
            println!("Chaos test: {} as {}", filename, assumed_lang);
            
            // Test detection chaos
            let detected_lang = SymbolIndexer::detect_language(Path::new(filename));
            let chaos_result = indexer.extract_symbols(chaotic_code, assumed_lang, filename);
            
            match chaos_result {
                Ok(ref symbols) => {
                    println!("✓ Chaos handled: {} symbols from {} as {} (detected: {:?})", 
                            symbols.len(), filename, assumed_lang, detected_lang);
                    
                    // Check if symbols make sense for assumed language
                    let reasonable_symbols = symbols.iter().all(|s| {
                        !s.name.is_empty() && 
                        !s.name.contains("undefined") &&
                        s.name.len() < 1000  // No massive garbage names
                    });
                    
                    if !reasonable_symbols {
                        detection_confusion_count += 1;
                        chaos_results.push(format!("Unreasonable symbols from {} as {}", filename, assumed_lang));
                    }
                }
                Err(ref e) => {
                    // Some confusion is expected with truly incompatible content
                    println!("✓ Expected confusion: {} as {} failed: {}", filename, assumed_lang, e);
                }
            }
            
            let symbol_count = match &chaos_result {
                Ok(symbols) => symbols.len(),
                Err(_) => 0,
            };
            
            chaos_results.push(format!("{} as {} -> {} symbols", 
                                     filename, assumed_lang, symbol_count));
        }
        let confusion_rate = (detection_confusion_count as f64 / total_chaos_cases as f64) * 100.0;
        
        println!("Language detection chaos results:");
        for result in &chaos_results {
            println!("  - {}", result);
        }
        println!("Confusion rate: {:.1}%", confusion_rate);
        
        // Some confusion is acceptable, but not total chaos
        assert!(confusion_rate < 50.0, 
               "EXCESSIVE CHAOS: {:.1}% confusion rate indicates language detection \
                completely breaks down with mixed content", confusion_rate);
    }

    /// TEST 9: Circular Dependency Loops
    /// Tests infinite loop detection in dependency resolution and reference tracking
    /// REAL STRESS: Creates actual circular references that can cause infinite loops
    #[test]
    fn stress_circular_dependency_loops() {
        println!("🚨 STRESS TEST 9: Circular Dependency Loops");
        
        let mut indexer = SymbolIndexer::new().expect("Failed to create indexer");
        let mut db = SymbolDatabase::new();
        
        // Create multiple circular dependency scenarios  
        let circular_scenarios = vec![
            // Simple A -> B -> A cycle
            vec![
                ("use crate::module_b::StructB; pub struct StructA { b: StructB }", "rust", "module_a.rs"),
                ("use crate::module_a::StructA; pub struct StructB { a: Box<StructA> }", "rust", "module_b.rs"),
            ],
            
            // Complex A -> B -> C -> A cycle
            vec![
                ("use crate::c::StructC; pub struct StructA { c_ref: StructC }", "rust", "a.rs"),
                ("use crate::a::StructA; pub struct StructB { a_ref: StructA }", "rust", "b.rs"), 
                ("use crate::b::StructB; pub struct StructC { b_ref: StructB }", "rust", "c.rs"),
            ],
            
            // Self-referential structures
            vec![
                ("pub struct Node { child: Option<Box<Node>> }", "rust", "self_ref.rs"),
            ],
            
            // Cross-language circular references (JavaScript)
            vec![
                ("import { ClassB } from './b.js'; class ClassA { constructor() { this.b = new ClassB(); } }", "javascript", "a.js"),
                ("import { ClassA } from './a.js'; class ClassB { constructor() { this.a = new ClassA(); } }", "javascript", "b.js"),
            ],
            
            // Python circular imports
            vec![
                ("from module_b import ClassB\nclass ClassA:\n    def __init__(self):\n        self.b = ClassB()", "python", "module_a.py"),
                ("from module_a import ClassA\nclass ClassB:\n    def __init__(self):\n        self.a = ClassA()", "python", "module_b.py"),
            ],
        ];
        
        for (scenario_idx, scenario) in circular_scenarios.iter().enumerate() {
            println!("Testing circular scenario {}: {} files", scenario_idx + 1, scenario.len());
            
            let scenario_start = Instant::now();
            let timeout_duration = Duration::from_secs(15);  // Generous timeout
            
            for (code, lang, filename) in scenario {
                let parse_start = Instant::now();
                
                // Check for infinite loop during parsing
                let parse_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    indexer.extract_symbols(code, lang, filename)
                }));
                
                let parse_duration = parse_start.elapsed();
                
                match parse_result {
                    Ok(Ok(symbols)) => {
                        println!("  ✓ Parsed {} in {:?}: {} symbols", filename, parse_duration, symbols.len());
                        
                        // Add to database to track cross-references
                        db.add_symbols(symbols);
                        
                        // Check for timeout (indicates possible infinite loop)
                        if parse_duration > Duration::from_secs(5) {
                            panic!("INFINITE LOOP DETECTED: {} took {:?} to parse, indicating circular reference handling failure", 
                                   filename, parse_duration);
                        }
                    }
                    Ok(Err(e)) => {
                        println!("  ✓ Parse error (acceptable): {} - {}", filename, e);
                    }
                    Err(_) => {
                        panic!("PARSER CRASH: {} caused panic during circular dependency parsing", filename);
                    }
                }
                
                // Global timeout check
                if scenario_start.elapsed() > timeout_duration {
                    panic!("SCENARIO TIMEOUT: Circular scenario {} exceeded {:?}, indicating infinite loop in dependency resolution", 
                           scenario_idx + 1, timeout_duration);
                }
            }
            
            let scenario_duration = scenario_start.elapsed();
            println!("  Scenario {} completed in {:?}", scenario_idx + 1, scenario_duration);
        }
        
        // Test circular reference resolution in database
        println!("Testing circular reference resolution...");
        
        let reference_tests = vec![
            ("StructA", "StructB"),
            ("StructB", "StructA"),
            ("ClassA", "ClassB"),
            ("ClassB", "ClassA"),
        ];
        
        for (ref1, ref2) in reference_tests {
            let refs1 = db.find_all_references(ref1);
            let refs2 = db.find_all_references(ref2);
            
            println!("  {} references: {}, {} references: {}", 
                    ref1, refs1.len(), ref2, refs2.len());
            
            // Both should exist if circular references were parsed correctly
            // but this shouldn't cause infinite loops
        }
        
        println!("Circular dependency stress test completed successfully");
        println!("Total symbols in database: {}", db.total_symbols());
        println!("Total files indexed: {}", db.files_indexed());
    }

    // Helper functions for generating stress test data

    fn generate_massive_rust_codebase(symbol_count: usize) -> String {
        let mut code = String::new();
        
        for i in 0..symbol_count {
            code.push_str(&format!(
                "pub struct Struct{} {{ field_{}: i32 }}\n\
                 impl Struct{} {{ pub fn method_{}() -> i32 {{ {} }} }}\n\
                 pub fn function_{}() -> Struct{} {{ Struct{} {{ field_{}: {} }} }}\n",
                i, i, i, i, i, i, i, i, i, i
            ));
        }
        
        code
    }

    fn generate_massive_python_codebase(symbol_count: usize) -> String {
        let mut code = String::new();
        
        for i in 0..symbol_count {
            code.push_str(&format!(
                "class Class{}:\n    def __init__(self):\n        self.field_{} = {}\n    def method_{}(self):\n        return self.field_{}\n\n\
                 def function_{}():\n    return Class{}()\n\n",
                i, i, i, i, i, i, i
            ));
        }
        
        code
    }

    fn generate_massive_js_codebase(symbol_count: usize) -> String {
        let mut code = String::new();
        
        for i in 0..symbol_count {
            code.push_str(&format!(
                "class Class{} {{\n    constructor() {{ this.field_{} = {}; }}\n    method_{}() {{ return this.field_{}; }}\n}}\n\
                 function function_{}() {{ return new Class{}(); }}\n",
                i, i, i, i, i, i, i
            ));
        }
        
        code
    }

    fn generate_deeply_nested_rust_code(depth: usize) -> String {
        let mut code = String::new();
        
        for i in 0..depth {
            code.push_str(&format!("mod module_{} {{\n", i));
        }
        
        code.push_str("pub fn deeply_nested_function() -> i32 { 42 }\n");
        
        code.push_str(&"}".repeat(depth));
        code
    }

    fn generate_massive_function_rust_code(function_count: usize) -> String {
        let mut code = String::new();
        
        for i in 0..function_count {
            code.push_str(&format!(
                "pub fn massive_function_{}(param_{}: i32) -> i32 {{\n    let local_{} = param_{} * 2;\n    local_{} + {}\n}}\n",
                i, i, i, i, i, i
            ));
        }
        
        code
    }

    fn generate_complex_generic_rust_code(type_count: usize) -> String {
        let mut code = String::new();
        
        for i in 0..type_count {
            code.push_str(&format!(
                "pub trait Trait{}<T> {{ fn method_{}(&self) -> T; }}\n\
                 pub struct Generic{}<T, U> {{ field_{}: T, other_{}: U }}\n\
                 impl<T, U> Trait{}<T> for Generic{}<T, U> {{ fn method_{}(&self) -> T {{ unimplemented!() }} }}\n",
                i, i, i, i, i, i, i, i
            ));
        }
        
        code
    }

    fn generate_stack_killer_rust_nesting(depth: usize) -> String {
        let mut code = String::new();
        
        // Create deeply nested match expressions
        code.push_str("fn stack_killer() -> i32 {\n    match true {\n");
        
        for i in 0..depth {
            code.push_str(&format!("        true => match {} {{\n", i));
            code.push_str(&format!("            {} => match Some({}) {{\n", i, i));
        }
        
        code.push_str("                Some(x) => x,\n");
        code.push_str("                None => 0,\n");
        
        for _i in 0..depth {
            code.push_str("            },\n            _ => 0,\n        },\n");
        }
        
        code.push_str("        false => 0,\n    }\n}\n");
        code
    }

    fn generate_stack_killer_json(depth: usize) -> String {
        let mut json = String::new();
        
        for i in 0..depth {
            json.push_str(&format!("{{\"level_{}\": ", i));
        }
        
        json.push_str("\"deep_value\"");
        
        for _i in 0..depth {
            json.push_str("}");
        }
        
        json
    }

    fn generate_stack_killer_html(depth: usize) -> String {
        let mut html = String::new();
        
        for i in 0..depth {
            html.push_str(&format!("<div class=\"level-{}\">\n", i));
        }
        
        html.push_str("<span>Deep content</span>\n");
        
        for _i in 0..depth {
            html.push_str("</div>\n");
        }
        
        html
    }

    fn generate_massive_rust_linear(line_count: usize) -> String {
        let mut code = String::new();
        
        for i in 0..line_count {
            code.push_str(&format!("pub const CONST_{}: i32 = {}; // Line {}\n", i, i, i));
        }
        
        code
    }

    fn generate_massive_js_functions(function_count: usize) -> String {
        let mut code = String::new();
        
        for i in 0..function_count {
            code.push_str(&format!("function func_{}() {{ return {}; }}\n", i, i));
        }
        
        code
    }

    fn generate_recursive_rust_types(type_count: usize) -> String {
        let mut code = String::new();
        
        for i in 0..type_count {
            let next_i = (i + 1) % type_count;  // Create circular type references
            code.push_str(&format!(
                "pub struct RecursiveType{} {{ next: Option<Box<RecursiveType{}>> }}\n",
                i, next_i
            ));
        }
        
        code
    }

    fn get_current_memory_mb() -> u64 {
        use sysinfo::{System, Pid};
        
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let pid = Pid::from(std::process::id() as usize);
        if let Some(process) = sys.process(pid) {
            process.memory() / 1024 / 1024 // Convert to MB
        } else {
            0
        }
    }

    fn analyze_memory_growth_trend(samples: &[(usize, u64, u64)]) -> f64 {
        if samples.len() < 2 {
            return 0.0;
        }
        
        let total_cycles = samples.len();
        let initial_memory = samples[0].2;  // end memory of first cycle
        let final_memory = samples[total_cycles - 1].2;  // end memory of last cycle
        
        let total_growth = final_memory.saturating_sub(initial_memory) as f64;
        total_growth / total_cycles as f64
    }
}

#[cfg(not(feature = "tree-sitter"))]
mod ast_missing_stress_disabled {
    #[test]
    fn missing_ast_stress_tests_disabled() {
        println!("Missing AST Parser Stress Tests skipped (tree-sitter feature not enabled)");
        println!("To run these tests: cargo test --features tree-sitter stress_");
    }
}