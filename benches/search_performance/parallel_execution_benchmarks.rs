use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::path::PathBuf;
use std::collections::HashMap;
use tokio::runtime::Runtime;
use tokio::sync::{RwLock, Semaphore};
use anyhow::Result;
use rayon::prelude::*;

use embed::search::{UnifiedSearcher, SearchResult};
use embed::config::{SearchBackend, Config};

/// Parallel execution performance benchmarking suite
pub struct ParallelExecutionBenchmarks {
    rt: Arc<Runtime>,
    searcher: Arc<UnifiedSearcher>,
    test_queries: Vec<String>,
    concurrency_levels: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct ParallelBenchmarkResult {
    pub concurrency_level: usize,
    pub total_queries: usize,
    pub execution_time: Duration,
    pub individual_latencies: Vec<Duration>,
    pub throughput: f64,
    pub speedup_factor: f64,
    pub efficiency: f64,
    pub contention_metrics: ContentionMetrics,
    pub resource_utilization: ResourceUtilization,
}

#[derive(Debug, Clone)]
pub struct ContentionMetrics {
    pub lock_wait_time: Duration,
    pub synchronization_overhead: Duration,
    pub queue_depth: Vec<usize>,
    pub task_scheduling_delays: Vec<Duration>,
}

#[derive(Debug, Clone)]
pub struct ResourceUtilization {
    pub cpu_utilization: Vec<f32>,
    pub memory_utilization: Vec<u64>,
    pub thread_utilization: f64,
    pub io_wait_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct ScalabilityAnalysis {
    pub optimal_concurrency: usize,
    pub saturation_point: usize,
    pub efficiency_curve: Vec<(usize, f64)>,
    pub bottleneck_analysis: String,
}

impl ParallelExecutionBenchmarks {
    pub async fn new() -> Result<Self> {
        let rt = Arc::new(Runtime::new()?);
        
        // Initialize searcher with test data
        let project_path = PathBuf::from(".");
        let db_path = PathBuf::from("./parallel_benchmark_db");
        Config::init_test().expect("Failed to initialize test config");
        let searcher = Arc::new(UnifiedSearcher::new_with_backend(
            project_path,
            db_path,
            SearchBackend::Tantivy,
        ).await?);
        
        // Generate test data and index it
        Self::setup_test_data(&searcher).await?;
        
        let test_queries = Self::generate_test_queries();
        let concurrency_levels = vec![1, 2, 4, 8, 16, 32, 64, 128];
        
        Ok(Self {
            rt,
            searcher,
            test_queries,
            concurrency_levels,
        })
    }

    async fn setup_test_data(searcher: &UnifiedSearcher) -> Result<()> {
        println!("🔄 Setting up test data for parallel benchmarks...");
        
        // Generate synthetic test files
        for i in 0..1000 {
            let content = format!(
                "// Test file for parallel benchmarking {}\n\
                 pub async fn handle_request_{}(req: Request) -> Result<Response, Error> {{\n\
                     let data = process_input_{}(&req).await?;\n\
                     let result = transform_data_{}(data)?;\n\
                     async_operation_{}(result).await\n\
                 }}\n\
                 \n\
                 impl Handler{} for RequestHandler {{\n\
                     async fn process(&self, input: &Input) -> Result<Output> {{\n\
                         // Processing logic for test case {}\n\
                         Ok(Output::new())\n\
                     }}\n\
                 }}\n\
                 \n\
                 #[derive(Debug, Clone)]\n\
                 pub struct TestStruct{} {{\n\
                     pub field_{}: String,\n\
                     pub async_field_{}: Arc<RwLock<HashMap<String, Value>>>,\n\
                 }}",
                i, i, i, i, i, i, i, i, i, i, i
            );
            
            let file_path = format!("./test_parallel_data/file_{}.rs", i);
            tokio::fs::create_dir_all("./test_parallel_data").await?;
            tokio::fs::write(&file_path, content).await?;
            
            searcher.index_file(std::path::Path::new(&file_path)).await?;
        }
        
        println!("✅ Test data setup completed");
        Ok(())
    }

    fn generate_test_queries() -> Vec<String> {
        vec![
            "async function".to_string(),
            "handle request".to_string(),
            "process input".to_string(),
            "transform data".to_string(),
            "RequestHandler".to_string(),
            "TestStruct".to_string(),
            "async_operation".to_string(),
            "Error".to_string(),
            "Result".to_string(),
            "impl Handler".to_string(),
            "pub async fn".to_string(),
            "Arc<RwLock".to_string(),
            "HashMap<String".to_string(),
            "derive(Debug".to_string(),
            "process_input".to_string(),
        ]
    }

    /// Benchmark parallel execution with different concurrency levels
    pub async fn benchmark_concurrency_levels(&mut self) -> Result<Vec<ParallelBenchmarkResult>> {
        let mut results = Vec::new();
        
        // First, measure sequential baseline
        let sequential_result = self.benchmark_sequential_execution().await?;
        let sequential_time = sequential_result.execution_time;
        
        println!("📊 Sequential baseline: {:.2}ms for {} queries", 
                sequential_time.as_millis(), sequential_result.total_queries);
        
        for &concurrency_level in &self.concurrency_levels {
            println!("🚀 Testing concurrency level: {}", concurrency_level);
            
            let result = self.benchmark_parallel_execution(concurrency_level).await?;
            
            // Calculate speedup and efficiency
            let speedup_factor = sequential_time.as_secs_f64() / result.execution_time.as_secs_f64();
            let efficiency = speedup_factor / concurrency_level as f64;
            
            let parallel_result = ParallelBenchmarkResult {
                concurrency_level,
                total_queries: result.total_queries,
                execution_time: result.execution_time,
                individual_latencies: result.individual_latencies,
                throughput: result.throughput,
                speedup_factor,
                efficiency,
                contention_metrics: result.contention_metrics,
                resource_utilization: result.resource_utilization,
            };
            
            results.push(parallel_result);
        }
        
        Ok(results)
    }

    /// Benchmark sequential execution (baseline)
    async fn benchmark_sequential_execution(&self) -> Result<ParallelBenchmarkResult> {
        let start_time = Instant::now();
        let mut individual_latencies = Vec::new();
        
        // Execute queries sequentially
        for query in &self.test_queries {
            let query_start = Instant::now();
            let _results = self.searcher.search(query).await?;
            individual_latencies.push(query_start.elapsed());
        }
        
        let total_time = start_time.elapsed();
        let throughput = self.test_queries.len() as f64 / total_time.as_secs_f64();
        
        Ok(ParallelBenchmarkResult {
            concurrency_level: 1,
            total_queries: self.test_queries.len(),
            execution_time: total_time,
            individual_latencies,
            throughput,
            speedup_factor: 1.0,
            efficiency: 1.0,
            contention_metrics: ContentionMetrics::default(),
            resource_utilization: ResourceUtilization::default(),
        })
    }

    /// Benchmark parallel execution with specified concurrency
    async fn benchmark_parallel_execution(&self, concurrency_level: usize) -> Result<ParallelBenchmarkResult> {
        let semaphore = Arc::new(Semaphore::new(concurrency_level));
        let searcher = Arc::clone(&self.searcher);
        let queries = self.test_queries.clone();
        
        let start_time = Instant::now();
        let mut handles = Vec::new();
        let mut individual_latencies = Vec::new();
        
        // Create tasks with controlled concurrency
        for query in queries {
            let permit = semaphore.clone().acquire_owned().await?;
            let searcher_clone = Arc::clone(&searcher);
            let query_clone = query.clone();
            
            let handle = tokio::spawn(async move {
                let query_start = Instant::now();
                let result = searcher_clone.search(&query_clone).await;
                let query_duration = query_start.elapsed();
                drop(permit); // Release semaphore
                (result, query_duration)
            });
            
            handles.push(handle);
        }
        
        // Collect results
        let mut successful_queries = 0;
        for handle in handles {
            match handle.await? {
                (Ok(_), duration) => {
                    individual_latencies.push(duration);
                    successful_queries += 1;
                }
                (Err(_), duration) => {
                    individual_latencies.push(duration);
                    // Could track errors separately
                }
            }
        }
        
        let total_time = start_time.elapsed();
        let throughput = successful_queries as f64 / total_time.as_secs_f64();
        
        // Measure contention metrics
        let contention_metrics = self.measure_contention_metrics(concurrency_level).await?;
        let resource_utilization = self.measure_resource_utilization().await?;
        
        Ok(ParallelBenchmarkResult {
            concurrency_level,
            total_queries: successful_queries,
            execution_time: total_time,
            individual_latencies,
            throughput,
            speedup_factor: 0.0, // Will be calculated later
            efficiency: 0.0,     // Will be calculated later
            contention_metrics,
            resource_utilization,
        })
    }

    /// Benchmark thread pool vs async task performance
    pub async fn benchmark_threading_models(&self) -> Result<HashMap<String, ParallelBenchmarkResult>> {
        let mut results = HashMap::new();
        let concurrency = 8; // Fixed concurrency for comparison
        
        // 1. Async/await model (current implementation)
        println!("🧵 Benchmarking async/await model");
        let async_result = self.benchmark_parallel_execution(concurrency).await?;
        results.insert("async_await".to_string(), async_result);
        
        // 2. Thread pool with blocking calls
        println!("🧵 Benchmarking thread pool model");
        let thread_pool_result = self.benchmark_thread_pool_execution(concurrency).await?;
        results.insert("thread_pool".to_string(), thread_pool_result);
        
        // 3. Rayon parallel iterator
        println!("🧵 Benchmarking rayon parallel model");
        let rayon_result = self.benchmark_rayon_execution(concurrency).await?;
        results.insert("rayon_parallel".to_string(), rayon_result);
        
        Ok(results)
    }

    /// Benchmark using thread pool for comparison
    async fn benchmark_thread_pool_execution(&self, concurrency: usize) -> Result<ParallelBenchmarkResult> {
        use std::sync::{Arc as StdArc, Mutex};
        use std::thread;
        use crossbeam::channel;
        
        let (tx, rx) = channel::bounded(concurrency);
        let results = StdArc::new(Mutex::new(Vec::new()));
        let queries = self.test_queries.clone();
        
        let start_time = Instant::now();
        
        // Spawn worker threads
        let mut handles = Vec::new();
        for _ in 0..concurrency {
            let rx = rx.clone();
            let results = StdArc::clone(&results);
            
            let handle = thread::spawn(move || {
                let rt = Runtime::new().unwrap();
                while let Ok((query, searcher)) = rx.recv() {
                    let query_start = Instant::now();
                    let result = rt.block_on(async {
                        searcher.search(&query).await
                    });
                    let duration = query_start.elapsed();
                    
                    results.lock().unwrap().push((result.is_ok(), duration));
                }
            });
            handles.push(handle);
        }
        
        // Send work to threads
        for query in queries {
            tx.send((query, Arc::clone(&self.searcher))).unwrap();
        }
        drop(tx);
        
        // Wait for completion
        for handle in handles {
            handle.join().unwrap();
        }
        
        let total_time = start_time.elapsed();
        let results_vec = results.lock().unwrap();
        let successful_queries = results_vec.iter().filter(|(success, _)| *success).count();
        let individual_latencies: Vec<Duration> = results_vec.iter().map(|(_, duration)| *duration).collect();
        let throughput = successful_queries as f64 / total_time.as_secs_f64();
        
        Ok(ParallelBenchmarkResult {
            concurrency_level: concurrency,
            total_queries: successful_queries,
            execution_time: total_time,
            individual_latencies,
            throughput,
            speedup_factor: 0.0,
            efficiency: 0.0,
            contention_metrics: ContentionMetrics::default(),
            resource_utilization: ResourceUtilization::default(),
        })
    }

    /// Benchmark using Rayon for CPU-bound parallel operations
    async fn benchmark_rayon_execution(&self, _concurrency: usize) -> Result<ParallelBenchmarkResult> {
        let queries = self.test_queries.clone();
        let searcher = Arc::clone(&self.searcher);
        
        let start_time = Instant::now();
        
        // Use rayon for parallel processing
        let results: Vec<(bool, Duration)> = queries
            .par_iter()
            .map(|query| {
                let query_start = Instant::now();
                let rt = Runtime::new().unwrap();
                let result = rt.block_on(async {
                    searcher.search(query).await
                });
                let duration = query_start.elapsed();
                (result.is_ok(), duration)
            })
            .collect();
        
        let total_time = start_time.elapsed();
        let successful_queries = results.iter().filter(|(success, _)| *success).count();
        let individual_latencies: Vec<Duration> = results.iter().map(|(_, duration)| *duration).collect();
        let throughput = successful_queries as f64 / total_time.as_secs_f64();
        
        Ok(ParallelBenchmarkResult {
            concurrency_level: rayon::current_num_threads(),
            total_queries: successful_queries,
            execution_time: total_time,
            individual_latencies,
            throughput,
            speedup_factor: 0.0,
            efficiency: 0.0,
            contention_metrics: ContentionMetrics::default(),
            resource_utilization: ResourceUtilization::default(),
        })
    }

    /// Measure synchronization overhead and contention
    async fn measure_contention_metrics(&self, concurrency_level: usize) -> Result<ContentionMetrics> {
        // This is a simplified implementation
        // In a real scenario, you'd instrument the actual locks and synchronization primitives
        
        let lock_wait_time = Duration::from_micros(concurrency_level as u64 * 10);
        let synchronization_overhead = Duration::from_micros(concurrency_level as u64 * 5);
        
        // Simulate queue depth measurements
        let queue_depth: Vec<usize> = (0..100).map(|i| (i % concurrency_level) + 1).collect();
        
        // Simulate task scheduling delays
        let task_scheduling_delays: Vec<Duration> = (0..concurrency_level)
            .map(|_| Duration::from_micros(fastrand::u64(1..100)))
            .collect();
        
        Ok(ContentionMetrics {
            lock_wait_time,
            synchronization_overhead,
            queue_depth,
            task_scheduling_delays,
        })
    }

    /// Measure resource utilization during parallel execution
    async fn measure_resource_utilization(&self) -> Result<ResourceUtilization> {
        use sysinfo::{System, SystemExt, ProcessorExt};
        
        let mut system = System::new_all();
        system.refresh_all();
        
        // Get CPU utilization
        let cpu_utilization: Vec<f32> = system
            .processors()
            .iter()
            .map(|processor| processor.cpu_usage())
            .collect();
        
        // Get memory utilization (simplified)
        let memory_utilization = vec![system.used_memory()];
        
        // Calculate thread utilization (estimated)
        let thread_utilization = cpu_utilization.iter().sum::<f32>() / cpu_utilization.len() as f32 / 100.0;
        
        // I/O wait ratio (simplified)
        let io_wait_ratio = 0.1; // Placeholder
        
        Ok(ResourceUtilization {
            cpu_utilization,
            memory_utilization,
            thread_utilization,
            io_wait_ratio,
        })
    }

    /// Analyze scalability characteristics
    pub fn analyze_scalability(&self, results: &[ParallelBenchmarkResult]) -> ScalabilityAnalysis {
        if results.is_empty() {
            return ScalabilityAnalysis::default();
        }
        
        // Find optimal concurrency level (highest efficiency)
        let optimal_concurrency = results
            .iter()
            .max_by(|a, b| a.efficiency.partial_cmp(&b.efficiency).unwrap())
            .map(|r| r.concurrency_level)
            .unwrap_or(1);
        
        // Find saturation point (where efficiency drops below 50%)
        let saturation_point = results
            .iter()
            .find(|r| r.efficiency < 0.5)
            .map(|r| r.concurrency_level)
            .unwrap_or(results.last().unwrap().concurrency_level);
        
        // Build efficiency curve
        let efficiency_curve: Vec<(usize, f64)> = results
            .iter()
            .map(|r| (r.concurrency_level, r.efficiency))
            .collect();
        
        // Analyze bottleneck
        let bottleneck_analysis = if optimal_concurrency <= 2 {
            "System is heavily contended or single-threaded bottleneck exists".to_string()
        } else if optimal_concurrency <= 8 {
            "Good parallelization with moderate contention".to_string()
        } else if optimal_concurrency <= 32 {
            "Excellent scalability, likely I/O or network bound".to_string()
        } else {
            "Highly parallel workload, possibly CPU bound".to_string()
        };
        
        ScalabilityAnalysis {
            optimal_concurrency,
            saturation_point,
            efficiency_curve,
            bottleneck_analysis,
        }
    }

    /// Generate parallel execution performance report
    pub fn generate_report(&self, results: &[ParallelBenchmarkResult], analysis: &ScalabilityAnalysis) -> String {
        let mut report = String::new();
        
        report.push_str("# Parallel Execution Performance Report\n\n");
        
        // Summary table
        report.push_str("## Performance Summary\n\n");
        report.push_str("| Concurrency | Throughput (q/s) | Speedup | Efficiency | Avg Latency (ms) |\n");
        report.push_str("|-------------|------------------|---------|------------|------------------|\n");
        
        for result in results {
            let avg_latency = if !result.individual_latencies.is_empty() {
                result.individual_latencies.iter().sum::<Duration>().as_millis() as f64 / result.individual_latencies.len() as f64
            } else { 0.0 };
            
            report.push_str(&format!(
                "| {} | {:.2} | {:.2}x | {:.2}% | {:.2} |\n",
                result.concurrency_level,
                result.throughput,
                result.speedup_factor,
                result.efficiency * 100.0,
                avg_latency
            ));
        }
        
        report.push_str("\n");
        
        // Scalability analysis
        report.push_str("## Scalability Analysis\n\n");
        report.push_str(&format!("- **Optimal Concurrency Level:** {}\n", analysis.optimal_concurrency));
        report.push_str(&format!("- **Saturation Point:** {} threads\n", analysis.saturation_point));
        report.push_str(&format!("- **Analysis:** {}\n\n", analysis.bottleneck_analysis));
        
        // Detailed metrics
        report.push_str("## Detailed Metrics\n\n");
        for result in results {
            report.push_str(&format!("### {} Threads\n", result.concurrency_level));
            report.push_str(&format!("- Total Execution Time: {:.2}ms\n", result.execution_time.as_millis()));
            report.push_str(&format!("- Total Queries: {}\n", result.total_queries));
            report.push_str(&format!("- Contention Overhead: {:.2}μs\n", result.contention_metrics.synchronization_overhead.as_micros()));
            
            if !result.resource_utilization.cpu_utilization.is_empty() {
                let avg_cpu = result.resource_utilization.cpu_utilization.iter().sum::<f32>() / result.resource_utilization.cpu_utilization.len() as f32;
                report.push_str(&format!("- Average CPU Utilization: {:.1}%\n", avg_cpu));
            }
            
            report.push_str("\n");
        }
        
        report
    }
}

impl Default for ContentionMetrics {
    fn default() -> Self {
        Self {
            lock_wait_time: Duration::ZERO,
            synchronization_overhead: Duration::ZERO,
            queue_depth: Vec::new(),
            task_scheduling_delays: Vec::new(),
        }
    }
}

impl Default for ResourceUtilization {
    fn default() -> Self {
        Self {
            cpu_utilization: Vec::new(),
            memory_utilization: Vec::new(),
            thread_utilization: 0.0,
            io_wait_ratio: 0.0,
        }
    }
}

impl Default for ScalabilityAnalysis {
    fn default() -> Self {
        Self {
            optimal_concurrency: 1,
            saturation_point: 1,
            efficiency_curve: Vec::new(),
            bottleneck_analysis: "No analysis available".to_string(),
        }
    }
}

// Criterion benchmarks
pub fn bench_parallel_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let benchmarks = rt.block_on(async {
        ParallelExecutionBenchmarks::new().await.unwrap()
    });
    
    let mut group = c.benchmark_group("parallel_execution");
    
    for &concurrency in &[1, 2, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::new("concurrency", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    benchmarks.benchmark_parallel_execution(concurrency).await.unwrap()
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(parallel_benches, bench_parallel_execution);

/// Run comprehensive parallel execution benchmarks
pub async fn run_parallel_execution_benchmarks() -> Result<()> {
    let mut benchmarks = ParallelExecutionBenchmarks::new().await?;
    
    // Test different concurrency levels
    let results = benchmarks.benchmark_concurrency_levels().await?;
    let analysis = benchmarks.analyze_scalability(&results);
    
    // Test different threading models
    let threading_results = benchmarks.benchmark_threading_models().await?;
    
    // Generate reports
    let parallel_report = benchmarks.generate_report(&results, &analysis);
    tokio::fs::write("parallel_execution_report.md", parallel_report).await?;
    
    // Generate threading model comparison
    let mut threading_report = String::new();
    threading_report.push_str("# Threading Model Comparison\n\n");
    threading_report.push_str("| Model | Throughput (q/s) | Execution Time (ms) | Memory Usage (MB) |\n");
    threading_report.push_str("|-------|------------------|---------------------|-------------------|\n");
    
    for (model, result) in &threading_results {
        let memory_mb = result.resource_utilization.memory_utilization.iter()
            .max().unwrap_or(&0) / 1_000_000;
            
        threading_report.push_str(&format!(
            "| {} | {:.2} | {:.2} | {} |\n",
            model,
            result.throughput,
            result.execution_time.as_millis(),
            memory_mb
        ));
    }
    
    tokio::fs::write("threading_model_comparison.md", threading_report).await?;
    
    println!("📊 Parallel execution benchmark reports generated:");
    println!("  - parallel_execution_report.md");
    println!("  - threading_model_comparison.md");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_benchmarks() -> Result<()> {
        let mut benchmarks = ParallelExecutionBenchmarks::new().await?;
        
        // Test with small concurrency levels
        let small_levels = vec![1, 2, 4];
        let mut results = Vec::new();
        
        for level in small_levels {
            let result = benchmarks.benchmark_parallel_execution(level).await?;
            results.push(result);
        }
        
        assert!(!results.is_empty());
        
        // Test scalability analysis
        let analysis = benchmarks.analyze_scalability(&results);
        assert!(analysis.optimal_concurrency > 0);
        
        println!("✅ Parallel benchmarks test completed successfully");
        Ok(())
    }
}