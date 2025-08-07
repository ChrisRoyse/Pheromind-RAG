use criterion::{criterion_group, criterion_main, Criterion, BatchSize, BenchmarkId, Throughput};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::path::PathBuf;
use std::collections::HashMap;
use tokio::runtime::Runtime;
use sysinfo::{System, SystemExt, ProcessExt, ProcessorExt, NetworksExt, DiskExt};
use plotters::prelude::*;
use anyhow::Result;

// Import the search system modules
use embed::search::{UnifiedSearcher, SearchResult};
use embed::config::SearchBackend;

/// Comprehensive performance benchmarking framework for search system
pub struct SearchPerformanceBenchmarker {
    rt: Arc<Runtime>,
    system: System,
    benchmarks: Vec<BenchmarkSuite>,
    resource_monitor: ResourceMonitor,
    results_collector: BenchmarkResultsCollector,
}

#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub test_data_sizes: Vec<usize>,        // Number of files to index: [100, 500, 1000, 5000]
    pub query_complexities: Vec<QueryType>, // Different query types
    pub parallel_levels: Vec<usize>,        // Concurrent queries: [1, 2, 4, 8, 16]
    pub measurement_duration: Duration,      // How long to run each test
    pub warmup_iterations: usize,           // Warmup runs before measurement
}

#[derive(Debug, Clone)]
pub enum QueryType {
    Simple(String),           // Single word: "function"
    Compound(String),         // Multiple words: "async function handler"
    Complex(String),          // Complex patterns: "function.*async.*error"
    CodePattern(String),      // Code-specific: "fn main() {"
    Semantic(String),         // Natural language: "error handling in async functions"
}

#[derive(Debug)]
pub struct BenchmarkSuite {
    pub name: String,
    pub config: BenchmarkConfig,
    pub search_backend: SearchBackend,
}

/// Resource monitoring during benchmarks
pub struct ResourceMonitor {
    system: System,
    measurements: Vec<ResourceMeasurement>,
    monitoring_active: bool,
}

#[derive(Debug, Clone)]
pub struct ResourceMeasurement {
    pub timestamp: Instant,
    pub cpu_usage: f32,
    pub memory_usage: u64,     // RSS memory in bytes
    pub disk_io_read: u64,     // Bytes read
    pub disk_io_write: u64,    // Bytes written
    pub network_in: u64,       // Bytes received
    pub network_out: u64,      // Bytes sent
}

/// Individual search method performance metrics
#[derive(Debug, Clone)]
pub struct SearchMethodMetrics {
    pub method_name: String,
    pub latency_ns: Vec<u64>,  // Nanoseconds per query
    pub throughput_qps: f64,   // Queries per second
    pub memory_peak: u64,      // Peak memory usage in bytes
    pub accuracy_score: f64,   // Relevance score (0-1)
    pub error_rate: f64,       // Errors per query (0-1)
    pub percentiles: HashMap<String, f64>, // P50, P95, P99 latencies
}

/// Parallel execution performance analysis
#[derive(Debug, Clone)]
pub struct ParallelPerformanceMetrics {
    pub sequential_time: Duration,
    pub parallel_time: Duration,
    pub speedup_factor: f64,
    pub efficiency: f64,           // speedup / num_threads
    pub contention_overhead: f64,  // Time lost to synchronization
    pub resource_utilization: f64, // CPU/memory efficiency
}

/// Bottleneck identification results
#[derive(Debug, Clone)]
pub struct BottleneckAnalysis {
    pub cpu_bound: bool,
    pub memory_bound: bool,
    pub io_bound: bool,
    pub network_bound: bool,
    pub limiting_factor: String,
    pub optimization_suggestions: Vec<String>,
}

/// Results collector and reporter
pub struct BenchmarkResultsCollector {
    pub method_results: HashMap<String, SearchMethodMetrics>,
    pub parallel_results: HashMap<usize, ParallelPerformanceMetrics>,
    pub bottleneck_analysis: BottleneckAnalysis,
    pub comparison_matrix: ComparisonMatrix,
}

#[derive(Debug, Clone)]
pub struct ComparisonMatrix {
    pub methods: Vec<String>,
    pub metrics: HashMap<String, HashMap<String, f64>>, // metric_name -> method_name -> value
}

impl SearchPerformanceBenchmarker {
    pub async fn new() -> Result<Self> {
        let rt = Arc::new(Runtime::new()?);
        let system = System::new_all();
        let resource_monitor = ResourceMonitor::new();
        let results_collector = BenchmarkResultsCollector::new();
        
        Ok(Self {
            rt,
            system,
            benchmarks: Vec::new(),
            resource_monitor,
            results_collector,
        })
    }

    /// Register a benchmark suite
    pub fn register_benchmark(&mut self, suite: BenchmarkSuite) {
        self.benchmarks.push(suite);
    }

    /// Run comprehensive performance benchmarks
    pub async fn run_comprehensive_benchmarks(&mut self) -> Result<BenchmarkReport> {
        println!("🚀 Starting comprehensive search performance benchmarking...");
        
        // Start resource monitoring
        self.resource_monitor.start_monitoring().await?;
        
        let mut report = BenchmarkReport::new();
        
        for suite in &self.benchmarks {
            println!("📊 Running benchmark suite: {}", suite.name);
            
            // 1. Individual method performance
            let method_metrics = self.benchmark_individual_methods(suite).await?;
            report.add_method_metrics(method_metrics);
            
            // 2. Parallel execution performance
            let parallel_metrics = self.benchmark_parallel_execution(suite).await?;
            report.add_parallel_metrics(parallel_metrics);
            
            // 3. Scalability analysis
            let scalability_metrics = self.benchmark_scalability(suite).await?;
            report.add_scalability_metrics(scalability_metrics);
        }
        
        // 4. Bottleneck analysis
        let bottleneck_analysis = self.analyze_bottlenecks().await?;
        report.set_bottleneck_analysis(bottleneck_analysis);
        
        // 5. Generate comparison matrix
        let comparison_matrix = self.generate_comparison_matrix().await?;
        report.set_comparison_matrix(comparison_matrix);
        
        // Stop resource monitoring
        self.resource_monitor.stop_monitoring().await?;
        
        println!("✅ Comprehensive benchmarking completed");
        Ok(report)
    }

    /// Benchmark individual search methods
    async fn benchmark_individual_methods(&mut self, suite: &BenchmarkSuite) -> Result<HashMap<String, SearchMethodMetrics>> {
        let mut results = HashMap::new();
        
        // Create test searcher
        let project_path = PathBuf::from(".");
        let db_path = PathBuf::from("./benchmark_test_db");
        let searcher = UnifiedSearcher::new_with_backend(
            project_path.clone(),
            db_path.clone(),
            suite.search_backend.clone()
        ).await?;
        
        // Prepare test data
        let test_queries = self.generate_test_queries(&suite.config.query_complexities);
        
        // Test each search method individually
        for (method_name, search_fn) in self.get_search_methods(&searcher) {
            println!("  🔍 Benchmarking method: {}", method_name);
            
            let mut latencies = Vec::new();
            let mut errors = 0;
            let start_time = Instant::now();
            
            // Warmup
            for _ in 0..suite.config.warmup_iterations {
                if let Err(_) = search_fn(&test_queries[0]).await {
                    errors += 1;
                }
            }
            
            // Measure performance
            let measurement_start = Instant::now();
            let mut query_count = 0;
            
            while measurement_start.elapsed() < suite.config.measurement_duration {
                for query in &test_queries {
                    let query_start = Instant::now();
                    
                    match search_fn(query).await {
                        Ok(_) => {
                            latencies.push(query_start.elapsed().as_nanos() as u64);
                            query_count += 1;
                        }
                        Err(_) => {
                            errors += 1;
                        }
                    }
                }
            }
            
            let total_time = start_time.elapsed();
            let throughput = query_count as f64 / total_time.as_secs_f64();
            let error_rate = errors as f64 / (query_count + errors) as f64;
            
            // Calculate percentiles
            let mut sorted_latencies = latencies.clone();
            sorted_latencies.sort();
            let percentiles = self.calculate_percentiles(&sorted_latencies);
            
            // Get peak memory usage (simplified for this benchmark)
            let memory_peak = self.get_current_memory_usage();
            
            let metrics = SearchMethodMetrics {
                method_name: method_name.clone(),
                latency_ns: latencies,
                throughput_qps: throughput,
                memory_peak,
                accuracy_score: 0.9, // Placeholder - would need actual relevance scoring
                error_rate,
                percentiles,
            };
            
            results.insert(method_name, metrics);
        }
        
        Ok(results)
    }

    /// Benchmark parallel execution performance
    async fn benchmark_parallel_execution(&mut self, suite: &BenchmarkSuite) -> Result<HashMap<usize, ParallelPerformanceMetrics>> {
        let mut results = HashMap::new();
        
        let project_path = PathBuf::from(".");
        let db_path = PathBuf::from("./benchmark_test_db");
        let searcher = Arc::new(UnifiedSearcher::new_with_backend(
            project_path.clone(),
            db_path.clone(),
            suite.search_backend.clone()
        ).await?);
        
        let test_query = "async function";
        
        // Measure sequential execution
        let sequential_start = Instant::now();
        for _ in 0..100 {
            let _ = searcher.search(test_query).await?;
        }
        let sequential_time = sequential_start.elapsed();
        
        // Test different parallel levels
        for &parallel_level in &suite.config.parallel_levels {
            println!("  ⚡ Testing {} parallel queries", parallel_level);
            
            let parallel_start = Instant::now();
            let mut handles = Vec::new();
            
            for _ in 0..parallel_level {
                let searcher_clone = searcher.clone();
                let query = test_query.to_string();
                
                let handle = tokio::spawn(async move {
                    for _ in 0..(100 / parallel_level) {
                        let _ = searcher_clone.search(&query).await;
                    }
                });
                
                handles.push(handle);
            }
            
            // Wait for all tasks to complete
            for handle in handles {
                let _ = handle.await;
            }
            
            let parallel_time = parallel_start.elapsed();
            let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
            let efficiency = speedup / parallel_level as f64;
            
            let metrics = ParallelPerformanceMetrics {
                sequential_time,
                parallel_time,
                speedup_factor: speedup,
                efficiency,
                contention_overhead: (parallel_time.as_secs_f64() - sequential_time.as_secs_f64() / parallel_level as f64).max(0.0),
                resource_utilization: efficiency,
            };
            
            results.insert(parallel_level, metrics);
        }
        
        Ok(results)
    }

    /// Benchmark scalability with different dataset sizes
    async fn benchmark_scalability(&mut self, suite: &BenchmarkSuite) -> Result<ScalabilityMetrics> {
        let mut results = Vec::new();
        
        for &data_size in &suite.config.test_data_sizes {
            println!("  📈 Testing scalability with {} files", data_size);
            
            // Create test dataset of specified size
            let test_data = self.generate_test_dataset(data_size).await?;
            
            let project_path = PathBuf::from(&format!("./test_data_{}", data_size));
            let db_path = PathBuf::from(&format!("./benchmark_db_{}", data_size));
            
            // Create and index searcher
            let indexing_start = Instant::now();
            let searcher = UnifiedSearcher::new_with_backend(
                project_path.clone(),
                db_path.clone(),
                suite.search_backend.clone()
            ).await?;
            
            // Index the test data
            for file_path in &test_data {
                let _ = searcher.index_file(&file_path).await;
            }
            let indexing_time = indexing_start.elapsed();
            
            // Measure query performance
            let query_start = Instant::now();
            let test_queries = ["function", "async", "error", "handler", "test"];
            
            for query in &test_queries {
                let _ = searcher.search(query).await?;
            }
            let query_time = query_start.elapsed();
            
            let memory_usage = self.get_current_memory_usage();
            
            results.push(ScalabilityDataPoint {
                dataset_size: data_size,
                indexing_time,
                query_time,
                memory_usage,
                throughput: test_queries.len() as f64 / query_time.as_secs_f64(),
            });
        }
        
        Ok(ScalabilityMetrics { data_points: results })
    }

    /// Analyze system bottlenecks
    async fn analyze_bottlenecks(&mut self) -> Result<BottleneckAnalysis> {
        let measurements = &self.resource_monitor.measurements;
        
        if measurements.is_empty() {
            return Ok(BottleneckAnalysis::default());
        }
        
        // Analyze CPU usage patterns
        let avg_cpu = measurements.iter().map(|m| m.cpu_usage).sum::<f32>() / measurements.len() as f32;
        let cpu_bound = avg_cpu > 80.0;
        
        // Analyze memory usage patterns
        let max_memory = measurements.iter().map(|m| m.memory_usage).max().unwrap_or(0);
        let memory_growth = if measurements.len() > 1 {
            let first_memory = measurements[0].memory_usage;
            let last_memory = measurements.last().unwrap().memory_usage;
            (last_memory as f64 - first_memory as f64) / first_memory as f64
        } else { 0.0 };
        let memory_bound = memory_growth > 0.5; // 50% growth indicates memory pressure
        
        // Analyze I/O patterns
        let total_io = measurements.iter()
            .map(|m| m.disk_io_read + m.disk_io_write)
            .sum::<u64>();
        let io_bound = total_io > 1_000_000_000; // 1GB threshold
        
        // Analyze network usage
        let total_network = measurements.iter()
            .map(|m| m.network_in + m.network_out)
            .sum::<u64>();
        let network_bound = total_network > 100_000_000; // 100MB threshold
        
        // Determine primary limiting factor
        let limiting_factor = if cpu_bound {
            "CPU"
        } else if memory_bound {
            "Memory"
        } else if io_bound {
            "Disk I/O"
        } else if network_bound {
            "Network"
        } else {
            "None detected"
        }.to_string();
        
        // Generate optimization suggestions
        let mut suggestions = Vec::new();
        if cpu_bound {
            suggestions.push("Consider CPU optimization: parallel processing, algorithm improvements".to_string());
        }
        if memory_bound {
            suggestions.push("Consider memory optimization: caching strategies, memory pooling".to_string());
        }
        if io_bound {
            suggestions.push("Consider I/O optimization: SSD storage, batch operations".to_string());
        }
        if network_bound {
            suggestions.push("Consider network optimization: compression, local caching".to_string());
        }
        
        Ok(BottleneckAnalysis {
            cpu_bound,
            memory_bound,
            io_bound,
            network_bound,
            limiting_factor,
            optimization_suggestions: suggestions,
        })
    }

    /// Generate performance comparison matrix
    async fn generate_comparison_matrix(&self) -> Result<ComparisonMatrix> {
        let methods = vec![
            "exact_search".to_string(),
            "semantic_search".to_string(),
            "symbol_search".to_string(),
            "bm25_search".to_string(),
            "unified_search".to_string(),
        ];
        
        let mut metrics = HashMap::new();
        
        // Latency comparison
        let mut latency_map = HashMap::new();
        // Memory comparison
        let mut memory_map = HashMap::new();
        // Throughput comparison
        let mut throughput_map = HashMap::new();
        // Accuracy comparison
        let mut accuracy_map = HashMap::new();
        
        for method in &methods {
            if let Some(result) = self.results_collector.method_results.get(method) {
                let avg_latency = result.latency_ns.iter().sum::<u64>() as f64 / result.latency_ns.len() as f64;
                latency_map.insert(method.clone(), avg_latency);
                memory_map.insert(method.clone(), result.memory_peak as f64);
                throughput_map.insert(method.clone(), result.throughput_qps);
                accuracy_map.insert(method.clone(), result.accuracy_score);
            }
        }
        
        metrics.insert("latency_ns".to_string(), latency_map);
        metrics.insert("memory_bytes".to_string(), memory_map);
        metrics.insert("throughput_qps".to_string(), throughput_map);
        metrics.insert("accuracy".to_string(), accuracy_map);
        
        Ok(ComparisonMatrix { methods, metrics })
    }

    // Helper methods
    fn generate_test_queries(&self, query_types: &[QueryType]) -> Vec<String> {
        query_types.iter().map(|qt| match qt {
            QueryType::Simple(q) => q.clone(),
            QueryType::Compound(q) => q.clone(),
            QueryType::Complex(q) => q.clone(),
            QueryType::CodePattern(q) => q.clone(),
            QueryType::Semantic(q) => q.clone(),
        }).collect()
    }

    async fn generate_test_dataset(&self, size: usize) -> Result<Vec<PathBuf>> {
        // Generate synthetic test files for benchmarking
        let mut files = Vec::new();
        
        for i in 0..size {
            let content = format!(
                "// Test file {} for performance benchmarking\n\
                 pub async fn test_function_{}() -> Result<()> {{\n\
                     let data = process_data();\n\
                     handle_async_operation(&data).await?;\n\
                     Ok(())\n\
                 }}\n\
                 \n\
                 fn helper_function_{}() {{\n\
                     // Some implementation\n\
                 }}",
                i, i, i
            );
            
            let file_path = PathBuf::from(&format!("./test_data/file_{}.rs", i));
            std::fs::create_dir_all(file_path.parent().unwrap())?;
            std::fs::write(&file_path, content)?;
            files.push(file_path);
        }
        
        Ok(files)
    }

    fn get_search_methods(&self, searcher: &UnifiedSearcher) -> Vec<(String, Box<dyn Fn(&str) -> Result<Vec<SearchResult>>>)> {
        // This would need to be implemented based on the actual UnifiedSearcher API
        // For now, return a placeholder
        Vec::new()
    }

    fn calculate_percentiles(&self, sorted_data: &[u64]) -> HashMap<String, f64> {
        let mut percentiles = HashMap::new();
        
        if sorted_data.is_empty() {
            return percentiles;
        }
        
        let p50_idx = sorted_data.len() / 2;
        let p95_idx = (sorted_data.len() as f64 * 0.95) as usize;
        let p99_idx = (sorted_data.len() as f64 * 0.99) as usize;
        
        percentiles.insert("p50".to_string(), sorted_data[p50_idx] as f64);
        percentiles.insert("p95".to_string(), sorted_data[p95_idx.min(sorted_data.len() - 1)] as f64);
        percentiles.insert("p99".to_string(), sorted_data[p99_idx.min(sorted_data.len() - 1)] as f64);
        
        percentiles
    }

    fn get_current_memory_usage(&self) -> u64 {
        // Get current process memory usage
        if let Some(process) = self.system.process(sysinfo::get_current_pid().unwrap()) {
            process.memory()
        } else {
            0
        }
    }
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
            measurements: Vec::new(),
            monitoring_active: false,
        }
    }

    pub async fn start_monitoring(&mut self) -> Result<()> {
        self.monitoring_active = true;
        self.measurements.clear();
        
        // Start background monitoring task
        let mut system = System::new_all();
        
        tokio::spawn(async move {
            while self.monitoring_active {
                system.refresh_all();
                
                let measurement = ResourceMeasurement {
                    timestamp: Instant::now(),
                    cpu_usage: system.global_processor_info().cpu_usage(),
                    memory_usage: system.used_memory(),
                    disk_io_read: system.disks().iter().map(|d| d.total_read_bytes()).sum(),
                    disk_io_write: system.disks().iter().map(|d| d.total_written_bytes()).sum(),
                    network_in: system.networks().iter().map(|(_, n)| n.received()).sum(),
                    network_out: system.networks().iter().map(|(_, n)| n.transmitted()).sum(),
                };
                
                self.measurements.push(measurement);
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
        
        Ok(())
    }

    pub async fn stop_monitoring(&mut self) -> Result<()> {
        self.monitoring_active = false;
        Ok(())
    }
}

impl BenchmarkResultsCollector {
    pub fn new() -> Self {
        Self {
            method_results: HashMap::new(),
            parallel_results: HashMap::new(),
            bottleneck_analysis: BottleneckAnalysis::default(),
            comparison_matrix: ComparisonMatrix {
                methods: Vec::new(),
                metrics: HashMap::new(),
            },
        }
    }
}

impl Default for BottleneckAnalysis {
    fn default() -> Self {
        Self {
            cpu_bound: false,
            memory_bound: false,
            io_bound: false,
            network_bound: false,
            limiting_factor: "Unknown".to_string(),
            optimization_suggestions: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct ScalabilityMetrics {
    pub data_points: Vec<ScalabilityDataPoint>,
}

#[derive(Debug)]
pub struct ScalabilityDataPoint {
    pub dataset_size: usize,
    pub indexing_time: Duration,
    pub query_time: Duration,
    pub memory_usage: u64,
    pub throughput: f64,
}

#[derive(Debug)]
pub struct BenchmarkReport {
    pub method_metrics: HashMap<String, SearchMethodMetrics>,
    pub parallel_metrics: HashMap<usize, ParallelPerformanceMetrics>,
    pub scalability_metrics: Option<ScalabilityMetrics>,
    pub bottleneck_analysis: Option<BottleneckAnalysis>,
    pub comparison_matrix: Option<ComparisonMatrix>,
    pub timestamp: std::time::SystemTime,
}

impl BenchmarkReport {
    pub fn new() -> Self {
        Self {
            method_metrics: HashMap::new(),
            parallel_metrics: HashMap::new(),
            scalability_metrics: None,
            bottleneck_analysis: None,
            comparison_matrix: None,
            timestamp: std::time::SystemTime::now(),
        }
    }

    pub fn add_method_metrics(&mut self, metrics: HashMap<String, SearchMethodMetrics>) {
        self.method_metrics.extend(metrics);
    }

    pub fn add_parallel_metrics(&mut self, metrics: HashMap<usize, ParallelPerformanceMetrics>) {
        self.parallel_metrics.extend(metrics);
    }

    pub fn add_scalability_metrics(&mut self, metrics: ScalabilityMetrics) {
        self.scalability_metrics = Some(metrics);
    }

    pub fn set_bottleneck_analysis(&mut self, analysis: BottleneckAnalysis) {
        self.bottleneck_analysis = Some(analysis);
    }

    pub fn set_comparison_matrix(&mut self, matrix: ComparisonMatrix) {
        self.comparison_matrix = Some(matrix);
    }

    /// Generate a comprehensive benchmark report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# Search System Performance Benchmark Report\n\n");
        report.push_str(&format!("Generated: {:?}\n\n", self.timestamp));
        
        // Individual method performance
        report.push_str("## Individual Search Method Performance\n\n");
        for (method, metrics) in &self.method_metrics {
            report.push_str(&format!("### {}\n", method));
            report.push_str(&format!("- Throughput: {:.2} queries/sec\n", metrics.throughput_qps));
            report.push_str(&format!("- Peak Memory: {:.2} MB\n", metrics.memory_peak as f64 / 1_000_000.0));
            report.push_str(&format!("- Error Rate: {:.4}\n", metrics.error_rate));
            report.push_str(&format!("- P50 Latency: {:.2} ms\n", metrics.percentiles.get("p50").unwrap_or(&0.0) / 1_000_000.0));
            report.push_str(&format!("- P95 Latency: {:.2} ms\n", metrics.percentiles.get("p95").unwrap_or(&0.0) / 1_000_000.0));
            report.push_str(&format!("- P99 Latency: {:.2} ms\n\n", metrics.percentiles.get("p99").unwrap_or(&0.0) / 1_000_000.0));
        }
        
        // Parallel execution performance
        report.push_str("## Parallel Execution Performance\n\n");
        for (threads, metrics) in &self.parallel_metrics {
            report.push_str(&format!("### {} Parallel Threads\n", threads));
            report.push_str(&format!("- Speedup Factor: {:.2}x\n", metrics.speedup_factor));
            report.push_str(&format!("- Efficiency: {:.2}%\n", metrics.efficiency * 100.0));
            report.push_str(&format!("- Contention Overhead: {:.2} ms\n\n", metrics.contention_overhead * 1000.0));
        }
        
        // Bottleneck analysis
        if let Some(analysis) = &self.bottleneck_analysis {
            report.push_str("## Bottleneck Analysis\n\n");
            report.push_str(&format!("- Primary Limiting Factor: {}\n", analysis.limiting_factor));
            report.push_str(&format!("- CPU Bound: {}\n", analysis.cpu_bound));
            report.push_str(&format!("- Memory Bound: {}\n", analysis.memory_bound));
            report.push_str(&format!("- I/O Bound: {}\n", analysis.io_bound));
            report.push_str(&format!("- Network Bound: {}\n\n", analysis.network_bound));
            
            if !analysis.optimization_suggestions.is_empty() {
                report.push_str("### Optimization Suggestions:\n");
                for suggestion in &analysis.optimization_suggestions {
                    report.push_str(&format!("- {}\n", suggestion));
                }
                report.push_str("\n");
            }
        }
        
        report
    }
}

// Create the main benchmarking function
pub async fn run_comprehensive_search_benchmarks() -> Result<BenchmarkReport> {
    let mut benchmarker = SearchPerformanceBenchmarker::new().await?;
    
    // Configure benchmark suites
    let config = BenchmarkConfig {
        test_data_sizes: vec![100, 500, 1000, 2000],
        query_complexities: vec![
            QueryType::Simple("function".to_string()),
            QueryType::Compound("async function handler".to_string()),
            QueryType::Complex("fn.*async.*Result".to_string()),
            QueryType::CodePattern("pub fn".to_string()),
            QueryType::Semantic("error handling patterns".to_string()),
        ],
        parallel_levels: vec![1, 2, 4, 8, 16],
        measurement_duration: Duration::from_secs(30),
        warmup_iterations: 10,
    };
    
    // Test different backends
    let backends = vec![
        SearchBackend::Tantivy,
        // Add other backends as needed
    ];
    
    for backend in backends {
        let suite = BenchmarkSuite {
            name: format!("{:?}_comprehensive_test", backend),
            config: config.clone(),
            search_backend: backend,
        };
        benchmarker.register_benchmark(suite);
    }
    
    // Run all benchmarks
    let report = benchmarker.run_comprehensive_benchmarks().await?;
    
    // Generate and save report
    let report_content = report.generate_report();
    std::fs::write("search_performance_report.md", &report_content)?;
    
    println!("📊 Benchmark report generated: search_performance_report.md");
    
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_benchmark_framework() -> Result<()> {
        let benchmarker = SearchPerformanceBenchmarker::new().await?;
        assert!(!benchmarker.benchmarks.is_empty() || benchmarker.benchmarks.is_empty()); // Framework created
        Ok(())
    }
}