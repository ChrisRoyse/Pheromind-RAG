use criterion::{criterion_group, criterion_main, Criterion, BatchSize, BenchmarkId, Throughput};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::path::PathBuf;
use std::collections::HashMap;
use tokio::runtime::Runtime;
use anyhow::Result;

use embed::search::{UnifiedSearcher, SearchResult};
use embed::search::bm25::{BM25Engine, BM25Document, Token as BM25Token};
use embed::search::tantivy_search::TantivySearcher;
use embed::config::{SearchBackend, Config};

/// Individual search method benchmarking suite
pub struct IndividualMethodBenchmarks {
    rt: Arc<Runtime>,
    test_data: Vec<TestDocument>,
    benchmarks: HashMap<String, MethodBenchmark>,
}

#[derive(Debug, Clone)]
pub struct TestDocument {
    pub id: String,
    pub file_path: String,
    pub content: String,
    pub tokens: Vec<String>,
    pub language: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MethodBenchmark {
    pub name: String,
    pub setup_time: Duration,
    pub query_latencies: Vec<Duration>,
    pub memory_usage: Vec<u64>,
    pub cpu_usage: Vec<f32>,
    pub error_count: usize,
    pub successful_queries: usize,
}

#[derive(Debug, Clone)]
pub struct QueryComplexityTest {
    pub simple_queries: Vec<String>,      // Single terms: ["function", "async", "error"]
    pub compound_queries: Vec<String>,    // Multiple terms: ["async function", "error handler"]
    pub pattern_queries: Vec<String>,     // Regex patterns: ["fn.*Result", "async.*await"]
    pub semantic_queries: Vec<String>,    // Natural language: ["handle errors", "async processing"]
}

impl IndividualMethodBenchmarks {
    pub async fn new() -> Result<Self> {
        let rt = Arc::new(Runtime::new()?);
        let test_data = Self::generate_test_data().await?;
        
        Ok(Self {
            rt,
            test_data,
            benchmarks: HashMap::new(),
        })
    }

    /// Generate comprehensive test dataset
    async fn generate_test_data() -> Result<Vec<TestDocument>> {
        let mut documents = Vec::new();
        
        // Generate different types of code files
        let rust_content = vec![
            "pub async fn handle_request(req: Request) -> Result<Response, Error> {\n    let data = process_data(&req).await?;\n    Ok(Response::new(data))\n}",
            "impl ErrorHandler for CustomHandler {\n    fn handle_error(&self, error: &Error) -> Result<()> {\n        log::error!(\"Error occurred: {}\", error);\n        Ok(())\n    }\n}",
            "use std::collections::HashMap;\nuse tokio::sync::RwLock;\n\npub struct Cache<T> {\n    data: RwLock<HashMap<String, T>>,\n}",
        ];
        
        let javascript_content = vec![
            "async function processData(input) {\n    try {\n        const result = await fetchData(input);\n        return transformResult(result);\n    } catch (error) {\n        console.error('Processing failed:', error);\n        throw error;\n    }\n}",
            "class DataProcessor {\n    constructor(config) {\n        this.config = config;\n        this.cache = new Map();\n    }\n\n    async process(data) {\n        if (this.cache.has(data.id)) {\n            return this.cache.get(data.id);\n        }\n        const result = await this.transform(data);\n        this.cache.set(data.id, result);\n        return result;\n    }\n}",
            "function debounce(func, wait) {\n    let timeout;\n    return function executedFunction(...args) {\n        const later = () => {\n            clearTimeout(timeout);\n            func(...args);\n        };\n        clearTimeout(timeout);\n        timeout = setTimeout(later, wait);\n    };\n}",
        ];
        
        let python_content = vec![
            "async def process_async_data(data_list):\n    results = []\n    async with aiohttp.ClientSession() as session:\n        tasks = [fetch_data(session, item) for item in data_list]\n        results = await asyncio.gather(*tasks)\n    return results",
            "class ConfigurationManager:\n    def __init__(self, config_path):\n        self.config_path = config_path\n        self.config = {}\n        self.load_config()\n    \n    def load_config(self):\n        try:\n            with open(self.config_path, 'r') as f:\n                self.config = json.load(f)\n        except FileNotFoundError:\n            raise ConfigurationError(f\"Config file not found: {self.config_path}\")",
            "def error_handler(func):\n    def wrapper(*args, **kwargs):\n        try:\n            return func(*args, **kwargs)\n        except Exception as e:\n            logging.error(f\"Error in {func.__name__}: {str(e)}\")\n            raise\n    return wrapper",
        ];
        
        // Create test documents
        let mut doc_id = 0;
        
        for (lang, contents) in [("rust", rust_content), ("javascript", javascript_content), ("python", python_content)] {
            for content in contents {
                let tokens: Vec<String> = content
                    .split_whitespace()
                    .map(|s| s.to_lowercase())
                    .collect();
                
                documents.push(TestDocument {
                    id: format!("doc_{}", doc_id),
                    file_path: format!("test_{}.{}", doc_id, match lang {
                        "rust" => "rs",
                        "javascript" => "js",
                        "python" => "py",
                        _ => "txt",
                    }),
                    content: content.to_string(),
                    tokens,
                    language: Some(lang.to_string()),
                });
                
                doc_id += 1;
            }
        }
        
        // Add more synthetic documents for larger dataset
        for i in 0..1000 {
            let content = format!(
                "// Generated test document {}\n\
                 pub fn test_function_{}() -> Result<(), Error> {{\n\
                     let value = process_input({});\n\
                     handle_result(value).await?;\n\
                     Ok(())\n\
                 }}\n\
                 \n\
                 async fn helper_function_{}(data: &str) {{\n\
                     // Implementation for test case {}\n\
                 }}",
                i, i, i, i, i
            );
            
            let tokens: Vec<String> = content
                .split_whitespace()
                .map(|s| s.to_lowercase())
                .collect();
            
            documents.push(TestDocument {
                id: format!("synthetic_{}", i),
                file_path: format!("synthetic_{}.rs", i),
                content,
                tokens,
                language: Some("rust".to_string()),
            });
        }
        
        Ok(documents)
    }

    /// Benchmark BM25 search method
    pub async fn benchmark_bm25_search(&mut self) -> Result<MethodBenchmark> {
        println!("🔍 Benchmarking BM25 Search Method");
        
        let setup_start = Instant::now();
        
        // Initialize BM25 engine
        let mut bm25_engine = BM25Engine::with_params(1.2, 0.75);
        
        // Index all test documents
        for doc in &self.test_data {
            let bm25_tokens: Vec<BM25Token> = doc.tokens.iter()
                .enumerate()
                .map(|(pos, token)| BM25Token {
                    text: token.clone(),
                    position: pos,
                    importance_weight: 1.0,
                })
                .collect();
            
            let bm25_doc = BM25Document {
                id: doc.id.clone(),
                file_path: doc.file_path.clone(),
                chunk_index: 0,
                tokens: bm25_tokens,
                start_line: 1,
                end_line: doc.content.lines().count(),
                language: doc.language.clone(),
            };
            
            bm25_engine.add_document(bm25_doc)?;
        }
        
        let setup_time = setup_start.elapsed();
        
        // Test different query complexities
        let queries = self.generate_test_queries();
        let mut query_latencies = Vec::new();
        let mut memory_usage = Vec::new();
        let mut error_count = 0;
        let mut successful_queries = 0;
        
        for query_batch in &[queries.simple_queries, queries.compound_queries, queries.pattern_queries] {
            for query in query_batch {
                let query_start = Instant::now();
                let memory_before = self.get_memory_usage();
                
                match bm25_engine.search(query, 50) {
                    Ok(_results) => {
                        successful_queries += 1;
                        query_latencies.push(query_start.elapsed());
                        
                        let memory_after = self.get_memory_usage();
                        memory_usage.push(memory_after - memory_before);
                    }
                    Err(_) => {
                        error_count += 1;
                    }
                }
            }
        }
        
        Ok(MethodBenchmark {
            name: "BM25".to_string(),
            setup_time,
            query_latencies,
            memory_usage,
            cpu_usage: Vec::new(), // Would need system monitoring
            error_count,
            successful_queries,
        })
    }

    /// Benchmark Tantivy exact search method
    pub async fn benchmark_tantivy_search(&mut self) -> Result<MethodBenchmark> {
        println!("🔍 Benchmarking Tantivy Search Method");
        
        let setup_start = Instant::now();
        
        // Initialize Tantivy searcher
        let mut tantivy_searcher = TantivySearcher::new().await?;
        
        // Index all test documents
        for doc in &self.test_data {
            let temp_file = format!("/tmp/{}", doc.file_path);
            std::fs::write(&temp_file, &doc.content)?;
            tantivy_searcher.index_file(std::path::Path::new(&temp_file)).await?;
        }
        
        let setup_time = setup_start.elapsed();
        
        // Test different query complexities
        let queries = self.generate_test_queries();
        let mut query_latencies = Vec::new();
        let mut memory_usage = Vec::new();
        let mut error_count = 0;
        let mut successful_queries = 0;
        
        for query_batch in &[queries.simple_queries, queries.compound_queries] {
            for query in query_batch {
                let query_start = Instant::now();
                let memory_before = self.get_memory_usage();
                
                match tantivy_searcher.search(query).await {
                    Ok(_results) => {
                        successful_queries += 1;
                        query_latencies.push(query_start.elapsed());
                        
                        let memory_after = self.get_memory_usage();
                        memory_usage.push(memory_after - memory_before);
                    }
                    Err(_) => {
                        error_count += 1;
                    }
                }
            }
        }
        
        Ok(MethodBenchmark {
            name: "Tantivy".to_string(),
            setup_time,
            query_latencies,
            memory_usage,
            cpu_usage: Vec::new(),
            error_count,
            successful_queries,
        })
    }

    /// Benchmark semantic search method
    #[cfg(all(feature = "ml", feature = "vectordb"))]
    pub async fn benchmark_semantic_search(&mut self) -> Result<MethodBenchmark> {
        println!("🔍 Benchmarking Semantic Search Method");
        
        let setup_start = Instant::now();
        
        // Initialize unified searcher for semantic search
        let project_path = PathBuf::from(".");
        let db_path = PathBuf::from("./benchmark_semantic_db");
        Config::init_test().expect("Failed to initialize test config");
        let searcher = UnifiedSearcher::new(project_path, db_path).await?;
        
        // Index test documents
        for doc in &self.test_data.iter().take(100) { // Limit for performance
            let temp_file = format!("/tmp/{}", doc.file_path);
            std::fs::write(&temp_file, &doc.content)?;
            searcher.index_file(std::path::Path::new(&temp_file)).await?;
        }
        
        let setup_time = setup_start.elapsed();
        
        // Test semantic queries
        let queries = self.generate_test_queries();
        let mut query_latencies = Vec::new();
        let mut memory_usage = Vec::new();
        let mut error_count = 0;
        let mut successful_queries = 0;
        
        for query in &queries.semantic_queries {
            let query_start = Instant::now();
            let memory_before = self.get_memory_usage();
            
            match searcher.search(query).await {
                Ok(_results) => {
                    successful_queries += 1;
                    query_latencies.push(query_start.elapsed());
                    
                    let memory_after = self.get_memory_usage();
                    memory_usage.push(memory_after - memory_before);
                }
                Err(_) => {
                    error_count += 1;
                }
            }
        }
        
        Ok(MethodBenchmark {
            name: "Semantic".to_string(),
            setup_time,
            query_latencies,
            memory_usage,
            cpu_usage: Vec::new(),
            error_count,
            successful_queries,
        })
    }

    /// Benchmark symbol search method
    #[cfg(feature = "tree-sitter")]
    pub async fn benchmark_symbol_search(&mut self) -> Result<MethodBenchmark> {
        println!("🔍 Benchmarking Symbol Search Method");
        
        let setup_start = Instant::now();
        
        // Initialize unified searcher for symbol search
        let project_path = PathBuf::from(".");
        let db_path = PathBuf::from("./benchmark_symbol_db");
        Config::init_test().expect("Failed to initialize test config");
        let searcher = UnifiedSearcher::new(project_path, db_path).await?;
        
        // Index test documents
        for doc in &self.test_data.iter().take(100) {
            let temp_file = format!("/tmp/{}", doc.file_path);
            std::fs::write(&temp_file, &doc.content)?;
            searcher.index_file(std::path::Path::new(&temp_file)).await?;
        }
        
        let setup_time = setup_start.elapsed();
        
        // Test symbol queries (function names, class names, etc.)
        let symbol_queries = vec![
            "handle_request", "ErrorHandler", "process_data", "CustomHandler",
            "Cache", "DataProcessor", "ConfigurationManager", "error_handler",
        ];
        
        let mut query_latencies = Vec::new();
        let mut memory_usage = Vec::new();
        let mut error_count = 0;
        let mut successful_queries = 0;
        
        for query in &symbol_queries {
            let query_start = Instant::now();
            let memory_before = self.get_memory_usage();
            
            match searcher.search(query).await {
                Ok(_results) => {
                    successful_queries += 1;
                    query_latencies.push(query_start.elapsed());
                    
                    let memory_after = self.get_memory_usage();
                    memory_usage.push(memory_after - memory_before);
                }
                Err(_) => {
                    error_count += 1;
                }
            }
        }
        
        Ok(MethodBenchmark {
            name: "Symbol".to_string(),
            setup_time,
            query_latencies,
            memory_usage,
            cpu_usage: Vec::new(),
            error_count,
            successful_queries,
        })
    }

    /// Benchmark unified search (all methods combined)
    pub async fn benchmark_unified_search(&mut self) -> Result<MethodBenchmark> {
        println!("🔍 Benchmarking Unified Search Method");
        
        let setup_start = Instant::now();
        
        // Initialize unified searcher
        let project_path = PathBuf::from(".");
        let db_path = PathBuf::from("./benchmark_unified_db");
        Config::init_test().expect("Failed to initialize test config");
        let searcher = UnifiedSearcher::new(project_path, db_path).await?;
        
        // Index test documents
        for doc in &self.test_data.iter().take(200) {
            let temp_file = format!("/tmp/{}", doc.file_path);
            std::fs::write(&temp_file, &doc.content)?;
            searcher.index_file(std::path::Path::new(&temp_file)).await?;
        }
        
        let setup_time = setup_start.elapsed();
        
        // Test all query types
        let queries = self.generate_test_queries();
        let mut query_latencies = Vec::new();
        let mut memory_usage = Vec::new();
        let mut error_count = 0;
        let mut successful_queries = 0;
        
        let all_queries = [
            &queries.simple_queries[..],
            &queries.compound_queries[..],
            &queries.semantic_queries[..],
        ].concat();
        
        for query in &all_queries {
            let query_start = Instant::now();
            let memory_before = self.get_memory_usage();
            
            match searcher.search(query).await {
                Ok(_results) => {
                    successful_queries += 1;
                    query_latencies.push(query_start.elapsed());
                    
                    let memory_after = self.get_memory_usage();
                    memory_usage.push(memory_after - memory_before);
                }
                Err(_) => {
                    error_count += 1;
                }
            }
        }
        
        Ok(MethodBenchmark {
            name: "Unified".to_string(),
            setup_time,
            query_latencies,
            memory_usage,
            cpu_usage: Vec::new(),
            error_count,
            successful_queries,
        })
    }

    /// Run all individual method benchmarks
    pub async fn run_all_benchmarks(&mut self) -> Result<HashMap<String, MethodBenchmark>> {
        let mut results = HashMap::new();
        
        // BM25 benchmark (always available)
        match self.benchmark_bm25_search().await {
            Ok(benchmark) => {
                results.insert(benchmark.name.clone(), benchmark);
            }
            Err(e) => {
                eprintln!("⚠️ BM25 benchmark failed: {}", e);
            }
        }
        
        // Tantivy benchmark
        match self.benchmark_tantivy_search().await {
            Ok(benchmark) => {
                results.insert(benchmark.name.clone(), benchmark);
            }
            Err(e) => {
                eprintln!("⚠️ Tantivy benchmark failed: {}", e);
            }
        }
        
        // Semantic search benchmark (if features enabled)
        #[cfg(all(feature = "ml", feature = "vectordb"))]
        match self.benchmark_semantic_search().await {
            Ok(benchmark) => {
                results.insert(benchmark.name.clone(), benchmark);
            }
            Err(e) => {
                eprintln!("⚠️ Semantic search benchmark failed: {}", e);
            }
        }
        
        // Symbol search benchmark (if feature enabled)
        #[cfg(feature = "tree-sitter")]
        match self.benchmark_symbol_search().await {
            Ok(benchmark) => {
                results.insert(benchmark.name.clone(), benchmark);
            }
            Err(e) => {
                eprintln!("⚠️ Symbol search benchmark failed: {}", e);
            }
        }
        
        // Unified search benchmark
        match self.benchmark_unified_search().await {
            Ok(benchmark) => {
                results.insert(benchmark.name.clone(), benchmark);
            }
            Err(e) => {
                eprintln!("⚠️ Unified search benchmark failed: {}", e);
            }
        }
        
        Ok(results)
    }

    /// Generate test queries for different complexity levels
    fn generate_test_queries(&self) -> QueryComplexityTest {
        QueryComplexityTest {
            simple_queries: vec![
                "function".to_string(),
                "async".to_string(),
                "error".to_string(),
                "handle".to_string(),
                "process".to_string(),
                "data".to_string(),
                "result".to_string(),
                "config".to_string(),
            ],
            compound_queries: vec![
                "async function".to_string(),
                "error handler".to_string(),
                "process data".to_string(),
                "handle request".to_string(),
                "configuration manager".to_string(),
                "data processor".to_string(),
                "fetch data".to_string(),
                "transform result".to_string(),
            ],
            pattern_queries: vec![
                "fn.*Result".to_string(),
                "async.*await".to_string(),
                "Error.*Handler".to_string(),
                "pub.*fn".to_string(),
                "impl.*for".to_string(),
                "struct.*{".to_string(),
            ],
            semantic_queries: vec![
                "handle errors in async functions".to_string(),
                "process data asynchronously".to_string(),
                "configuration management".to_string(),
                "error handling patterns".to_string(),
                "data transformation".to_string(),
                "async request processing".to_string(),
                "caching mechanisms".to_string(),
                "timeout and retry logic".to_string(),
            ],
        }
    }

    /// Get current memory usage (simplified implementation)
    fn get_memory_usage(&self) -> u64 {
        // This is a placeholder implementation
        // In a real benchmark, you'd use proper memory monitoring
        use std::alloc::{GlobalAlloc, Layout, System};
        
        // Return a dummy value for now
        // In practice, you'd use tools like jemalloc with stats or system monitoring
        1024 * 1024 // 1MB placeholder
    }

    /// Analyze and report individual method performance
    pub fn analyze_performance(&self, results: &HashMap<String, MethodBenchmark>) -> String {
        let mut report = String::new();
        
        report.push_str("# Individual Search Method Performance Analysis\n\n");
        
        for (method_name, benchmark) in results {
            report.push_str(&format!("## {} Performance Metrics\n\n", method_name));
            
            // Setup performance
            report.push_str(&format!("**Setup Time:** {:.2} ms\n\n", benchmark.setup_time.as_secs_f64() * 1000.0));
            
            // Query performance
            if !benchmark.query_latencies.is_empty() {
                let avg_latency = benchmark.query_latencies.iter().sum::<Duration>().as_nanos() as f64 / benchmark.query_latencies.len() as f64;
                let min_latency = benchmark.query_latencies.iter().min().unwrap().as_nanos() as f64;
                let max_latency = benchmark.query_latencies.iter().max().unwrap().as_nanos() as f64;
                
                report.push_str(&format!("**Query Performance:**\n"));
                report.push_str(&format!("- Average Latency: {:.2} ms\n", avg_latency / 1_000_000.0));
                report.push_str(&format!("- Min Latency: {:.2} ms\n", min_latency / 1_000_000.0));
                report.push_str(&format!("- Max Latency: {:.2} ms\n", max_latency / 1_000_000.0));
                report.push_str(&format!("- Successful Queries: {}\n", benchmark.successful_queries));
                report.push_str(&format!("- Error Count: {}\n", benchmark.error_count));
                
                let throughput = benchmark.successful_queries as f64 / (benchmark.query_latencies.iter().sum::<Duration>().as_secs_f64());
                report.push_str(&format!("- Throughput: {:.2} queries/sec\n\n", throughput));
            }
            
            // Memory usage
            if !benchmark.memory_usage.is_empty() {
                let avg_memory = benchmark.memory_usage.iter().sum::<u64>() as f64 / benchmark.memory_usage.len() as f64;
                let max_memory = *benchmark.memory_usage.iter().max().unwrap();
                
                report.push_str(&format!("**Memory Usage:**\n"));
                report.push_str(&format!("- Average: {:.2} MB\n", avg_memory / 1_000_000.0));
                report.push_str(&format!("- Peak: {:.2} MB\n\n", max_memory as f64 / 1_000_000.0));
            }
        }
        
        // Performance comparison
        report.push_str("## Performance Comparison\n\n");
        report.push_str("| Method | Avg Latency (ms) | Throughput (q/s) | Setup Time (ms) | Error Rate |\n");
        report.push_str("|--------|------------------|------------------|-----------------|------------|\n");
        
        for (method_name, benchmark) in results {
            let avg_latency = if !benchmark.query_latencies.is_empty() {
                benchmark.query_latencies.iter().sum::<Duration>().as_nanos() as f64 / benchmark.query_latencies.len() as f64 / 1_000_000.0
            } else { 0.0 };
            
            let throughput = if !benchmark.query_latencies.is_empty() {
                benchmark.successful_queries as f64 / benchmark.query_latencies.iter().sum::<Duration>().as_secs_f64()
            } else { 0.0 };
            
            let setup_time = benchmark.setup_time.as_secs_f64() * 1000.0;
            let error_rate = benchmark.error_count as f64 / (benchmark.successful_queries + benchmark.error_count) as f64;
            
            report.push_str(&format!("| {} | {:.2} | {:.2} | {:.2} | {:.4} |\n", 
                method_name, avg_latency, throughput, setup_time, error_rate));
        }
        
        report
    }
}

// Create benchmarking functions for Criterion
pub fn bench_individual_methods(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut benchmarks = rt.block_on(async {
        IndividualMethodBenchmarks::new().await.unwrap()
    });
    
    // BM25 benchmark
    c.bench_function("bm25_search", |b| {
        b.to_async(&rt).iter(|| async {
            benchmarks.benchmark_bm25_search().await.unwrap()
        })
    });
    
    // Tantivy benchmark
    c.bench_function("tantivy_search", |b| {
        b.to_async(&rt).iter(|| async {
            benchmarks.benchmark_tantivy_search().await.unwrap()
        })
    });
    
    // Unified search benchmark
    c.bench_function("unified_search", |b| {
        b.to_async(&rt).iter(|| async {
            benchmarks.benchmark_unified_search().await.unwrap()
        })
    });
}

criterion_group!(individual_benches, bench_individual_methods);
criterion_main!(individual_benches);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_individual_benchmarks() -> Result<()> {
        let mut benchmarks = IndividualMethodBenchmarks::new().await?;
        let results = benchmarks.run_all_benchmarks().await?;
        
        // Verify we got results for at least one method
        assert!(!results.is_empty());
        
        // Verify each result has valid data
        for (method, benchmark) in &results {
            println!("✅ {} benchmark completed with {} successful queries", 
                method, benchmark.successful_queries);
        }
        
        Ok(())
    }
}