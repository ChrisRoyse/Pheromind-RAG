use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use sysinfo::{System, SystemExt, ProcessExt, ProcessorExt, NetworksExt, DiskExt, ComponentExt};
use anyhow::Result;

use embed::search::{UnifiedSearcher, SearchResult};
use embed::config::{SearchBackend, Config};

/// Comprehensive bottleneck identification and profiling system
pub struct BottleneckProfiler {
    system_monitor: Arc<Mutex<System>>,
    performance_samples: Arc<RwLock<Vec<PerformanceSample>>>,
    profiling_active: bool,
    sampling_interval: Duration,
    memory_tracker: MemoryTracker,
    cpu_profiler: CPUProfiler,
    io_profiler: IOProfiler,
    network_profiler: NetworkProfiler,
}

#[derive(Debug, Clone)]
pub struct PerformanceSample {
    pub timestamp: SystemTime,
    pub cpu_metrics: CPUMetrics,
    pub memory_metrics: MemoryMetrics,
    pub io_metrics: IOMetrics,
    pub network_metrics: NetworkMetrics,
    pub search_operation_metrics: Option<SearchOperationMetrics>,
}

#[derive(Debug, Clone)]
pub struct CPUMetrics {
    pub overall_usage: f32,
    pub per_core_usage: Vec<f32>,
    pub load_average: f32,
    pub context_switches: u64,
    pub interrupts: u64,
    pub user_time: f32,
    pub system_time: f32,
    pub idle_time: f32,
    pub iowait_time: f32,
}

#[derive(Debug, Clone)]
pub struct MemoryMetrics {
    pub total_memory: u64,
    pub used_memory: u64,
    pub available_memory: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub heap_usage: u64,
    pub stack_usage: u64,
    pub process_rss: u64,
    pub process_vms: u64,
    pub gc_collections: u64,
    pub gc_time: Duration,
}

#[derive(Debug, Clone)]
pub struct IOMetrics {
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub disk_read_ops: u64,
    pub disk_write_ops: u64,
    pub disk_read_time: Duration,
    pub disk_write_time: Duration,
    pub disk_queue_depth: u32,
    pub disk_utilization: f32,
}

#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub errors_in: u64,
    pub errors_out: u64,
    pub connections_active: u32,
    pub connection_failures: u32,
}

#[derive(Debug, Clone)]
pub struct SearchOperationMetrics {
    pub operation_type: String,
    pub latency: Duration,
    pub cpu_usage_during: f32,
    pub memory_allocated: u64,
    pub disk_io_during: u64,
    pub network_io_during: u64,
    pub lock_contention_time: Duration,
}

/// Bottleneck analysis results
#[derive(Debug, Clone)]
pub struct BottleneckAnalysis {
    pub primary_bottleneck: BottleneckType,
    pub bottleneck_confidence: f64,
    pub contributing_factors: Vec<BottleneckFactor>,
    pub performance_impact: PerformanceImpact,
    pub optimization_recommendations: Vec<OptimizationRecommendation>,
    pub trending_analysis: TrendingAnalysis,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BottleneckType {
    CPU,
    Memory,
    DiskIO,
    NetworkIO,
    Lock,
    Algorithm,
    None,
}

#[derive(Debug, Clone)]
pub struct BottleneckFactor {
    pub factor_type: String,
    pub severity: f64,        // 0.0 to 1.0
    pub description: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PerformanceImpact {
    pub throughput_reduction: f64,     // Percentage reduction
    pub latency_increase: f64,         // Percentage increase
    pub resource_waste: f64,           // Percentage of wasted resources
    pub projected_improvement: f64,    // Potential improvement if fixed
}

#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    pub category: String,
    pub priority: Priority,
    pub description: String,
    pub expected_improvement: f64,
    pub implementation_difficulty: Difficulty,
    pub code_changes_required: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

#[derive(Debug, Clone)]
pub struct TrendingAnalysis {
    pub performance_trend: Trend,
    pub resource_growth_rate: f64,
    pub degradation_rate: f64,
    pub stability_score: f64,
}

#[derive(Debug, Clone)]
pub enum Trend {
    Improving,
    Stable,
    Degrading,
    Volatile,
}

/// Memory tracker for leak detection
pub struct MemoryTracker {
    samples: VecDeque<MemorySnapshot>,
    leak_threshold: f64,      // Growth rate threshold
    monitoring_window: Duration,
}

#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub timestamp: SystemTime,
    pub heap_size: u64,
    pub used_memory: u64,
    pub allocated_objects: u64,
    pub gc_pressure: f64,
}

/// CPU profiler for performance analysis
pub struct CPUProfiler {
    samples: VecDeque<CPUSample>,
    high_usage_threshold: f32,
    profiling_depth: usize,
}

#[derive(Debug, Clone)]
pub struct CPUSample {
    pub timestamp: SystemTime,
    pub usage_percentage: f32,
    pub active_threads: u32,
    pub runnable_tasks: u32,
    pub context_switch_rate: f64,
}

/// I/O profiler for disk operations
pub struct IOProfiler {
    samples: VecDeque<IOSample>,
    io_wait_threshold: f32,
    bandwidth_threshold: u64,
}

#[derive(Debug, Clone)]
pub struct IOSample {
    pub timestamp: SystemTime,
    pub read_bandwidth: u64,
    pub write_bandwidth: u64,
    pub iops: u32,
    pub avg_latency: Duration,
    pub queue_depth: u32,
}

/// Network profiler
pub struct NetworkProfiler {
    samples: VecDeque<NetworkSample>,
    bandwidth_threshold: u64,
    error_rate_threshold: f64,
}

#[derive(Debug, Clone)]
pub struct NetworkSample {
    pub timestamp: SystemTime,
    pub throughput: u64,
    pub packet_loss: f64,
    pub latency: Duration,
    pub connection_count: u32,
}

impl BottleneckProfiler {
    pub fn new() -> Self {
        Self {
            system_monitor: Arc::new(Mutex::new(System::new_all())),
            performance_samples: Arc::new(RwLock::new(Vec::new())),
            profiling_active: false,
            sampling_interval: Duration::from_millis(100),
            memory_tracker: MemoryTracker::new(),
            cpu_profiler: CPUProfiler::new(),
            io_profiler: IOProfiler::new(),
            network_profiler: NetworkProfiler::new(),
        }
    }

    /// Start comprehensive profiling
    pub async fn start_profiling(&mut self) -> Result<()> {
        println!("🔍 Starting comprehensive bottleneck profiling...");
        
        self.profiling_active = true;
        
        // Start background monitoring task
        let system_monitor = Arc::clone(&self.system_monitor);
        let performance_samples = Arc::clone(&self.performance_samples);
        let sampling_interval = self.sampling_interval;
        let profiling_active = self.profiling_active;
        
        tokio::spawn(async move {
            while profiling_active {
                let sample = Self::collect_performance_sample(&system_monitor).await;
                
                let mut samples = performance_samples.write().await;
                samples.push(sample);
                
                // Keep only last 10,000 samples to prevent memory issues
                if samples.len() > 10_000 {
                    samples.drain(0..1_000);
                }
                
                tokio::time::sleep(sampling_interval).await;
            }
        });
        
        Ok(())
    }

    /// Stop profiling
    pub async fn stop_profiling(&mut self) {
        self.profiling_active = false;
        println!("⏹️ Profiling stopped");
    }

    /// Collect a single performance sample
    async fn collect_performance_sample(system_monitor: &Arc<Mutex<System>>) -> PerformanceSample {
        let mut system = system_monitor.lock().unwrap();
        system.refresh_all();
        
        let cpu_metrics = CPUMetrics {
            overall_usage: system.global_processor_info().cpu_usage(),
            per_core_usage: system.processors().iter().map(|p| p.cpu_usage()).collect(),
            load_average: system.load_average().one,
            context_switches: 0, // Would need OS-specific implementation
            interrupts: 0,      // Would need OS-specific implementation
            user_time: 0.0,     // Would need detailed CPU stats
            system_time: 0.0,   // Would need detailed CPU stats
            idle_time: 100.0 - system.global_processor_info().cpu_usage(),
            iowait_time: 0.0,   // Would need OS-specific implementation
        };
        
        let memory_metrics = MemoryMetrics {
            total_memory: system.total_memory(),
            used_memory: system.used_memory(),
            available_memory: system.available_memory(),
            swap_total: system.total_swap(),
            swap_used: system.used_swap(),
            heap_usage: 0,      // Would need runtime-specific implementation
            stack_usage: 0,     // Would need runtime-specific implementation
            process_rss: system.processes().values().map(|p| p.memory()).sum(),
            process_vms: system.processes().values().map(|p| p.virtual_memory()).sum(),
            gc_collections: 0,  // Would need GC-specific implementation
            gc_time: Duration::ZERO,
        };
        
        let io_metrics = IOMetrics {
            disk_read_bytes: system.disks().iter().map(|d| d.total_read_bytes()).sum(),
            disk_write_bytes: system.disks().iter().map(|d| d.total_written_bytes()).sum(),
            disk_read_ops: 0,   // Would need OS-specific implementation
            disk_write_ops: 0,  // Would need OS-specific implementation
            disk_read_time: Duration::ZERO,
            disk_write_time: Duration::ZERO,
            disk_queue_depth: 0,
            disk_utilization: 0.0,
        };
        
        let network_metrics = NetworkMetrics {
            bytes_sent: system.networks().iter().map(|(_, n)| n.transmitted()).sum(),
            bytes_received: system.networks().iter().map(|(_, n)| n.received()).sum(),
            packets_sent: system.networks().iter().map(|(_, n)| n.packets_transmitted()).sum(),
            packets_received: system.networks().iter().map(|(_, n)| n.packets_received()).sum(),
            errors_in: system.networks().iter().map(|(_, n)| n.errors_on_received()).sum(),
            errors_out: system.networks().iter().map(|(_, n)| n.errors_on_transmitted()).sum(),
            connections_active: 0,    // Would need network stack access
            connection_failures: 0,   // Would need network stack access
        };
        
        PerformanceSample {
            timestamp: SystemTime::now(),
            cpu_metrics,
            memory_metrics,
            io_metrics,
            network_metrics,
            search_operation_metrics: None,
        }
    }

    /// Profile specific search operations
    pub async fn profile_search_operation<F, R>(&self, operation_name: &str, operation: F) -> Result<(R, SearchOperationMetrics)>
    where
        F: std::future::Future<Output = Result<R>>,
    {
        let start_time = Instant::now();
        let start_sample = Self::collect_performance_sample(&self.system_monitor).await;
        
        // Execute the operation
        let result = operation.await?;
        
        let end_time = Instant::now();
        let end_sample = Self::collect_performance_sample(&self.system_monitor).await;
        
        // Calculate operation-specific metrics
        let metrics = SearchOperationMetrics {
            operation_type: operation_name.to_string(),
            latency: end_time - start_time,
            cpu_usage_during: end_sample.cpu_metrics.overall_usage - start_sample.cpu_metrics.overall_usage,
            memory_allocated: end_sample.memory_metrics.used_memory.saturating_sub(start_sample.memory_metrics.used_memory),
            disk_io_during: (end_sample.io_metrics.disk_read_bytes + end_sample.io_metrics.disk_write_bytes)
                .saturating_sub(start_sample.io_metrics.disk_read_bytes + start_sample.io_metrics.disk_write_bytes),
            network_io_during: (end_sample.network_metrics.bytes_sent + end_sample.network_metrics.bytes_received)
                .saturating_sub(start_sample.network_metrics.bytes_sent + start_sample.network_metrics.bytes_received),
            lock_contention_time: Duration::ZERO, // Would need instrumentation
        };
        
        Ok((result, metrics))
    }

    /// Analyze bottlenecks from collected samples
    pub async fn analyze_bottlenecks(&self) -> Result<BottleneckAnalysis> {
        let samples = self.performance_samples.read().await;
        
        if samples.len() < 10 {
            return Ok(BottleneckAnalysis::default());
        }
        
        // Analyze CPU bottlenecks
        let cpu_analysis = self.analyze_cpu_bottlenecks(&samples);
        
        // Analyze memory bottlenecks
        let memory_analysis = self.analyze_memory_bottlenecks(&samples);
        
        // Analyze I/O bottlenecks
        let io_analysis = self.analyze_io_bottlenecks(&samples);
        
        // Analyze network bottlenecks
        let network_analysis = self.analyze_network_bottlenecks(&samples);
        
        // Determine primary bottleneck
        let primary_bottleneck = self.determine_primary_bottleneck(&cpu_analysis, &memory_analysis, &io_analysis, &network_analysis);
        
        // Calculate performance impact
        let performance_impact = self.calculate_performance_impact(&samples, &primary_bottleneck);
        
        // Generate optimization recommendations
        let recommendations = self.generate_optimization_recommendations(&primary_bottleneck, &cpu_analysis, &memory_analysis, &io_analysis, &network_analysis);
        
        // Trending analysis
        let trending_analysis = self.analyze_trends(&samples);
        
        Ok(BottleneckAnalysis {
            primary_bottleneck,
            bottleneck_confidence: 0.85, // Would calculate based on evidence strength
            contributing_factors: vec![cpu_analysis, memory_analysis, io_analysis, network_analysis],
            performance_impact,
            optimization_recommendations: recommendations,
            trending_analysis,
        })
    }

    /// Analyze CPU performance patterns
    fn analyze_cpu_bottlenecks(&self, samples: &[PerformanceSample]) -> BottleneckFactor {
        let avg_cpu_usage = samples.iter()
            .map(|s| s.cpu_metrics.overall_usage)
            .sum::<f32>() / samples.len() as f32;
        
        let high_cpu_periods = samples.iter()
            .filter(|s| s.cpu_metrics.overall_usage > 80.0)
            .count();
        
        let severity = if avg_cpu_usage > 90.0 {
            1.0
        } else if avg_cpu_usage > 70.0 {
            0.7
        } else if avg_cpu_usage > 50.0 {
            0.4
        } else {
            0.1
        };
        
        let mut evidence = Vec::new();
        evidence.push(format!("Average CPU usage: {:.1}%", avg_cpu_usage));
        evidence.push(format!("High CPU periods: {} out of {}", high_cpu_periods, samples.len()));
        
        if high_cpu_periods > samples.len() / 4 {
            evidence.push("Sustained high CPU usage detected".to_string());
        }
        
        // Analyze per-core usage for imbalance
        if let Some(sample) = samples.last() {
            let core_variance = self.calculate_variance(&sample.cpu_metrics.per_core_usage);
            if core_variance > 400.0 { // High variance indicates poor load balancing
                evidence.push("CPU core load imbalance detected".to_string());
            }
        }
        
        BottleneckFactor {
            factor_type: "CPU".to_string(),
            severity,
            description: if severity > 0.7 {
                "High CPU usage is limiting performance".to_string()
            } else if severity > 0.4 {
                "Moderate CPU usage may cause occasional slowdowns".to_string()
            } else {
                "CPU usage is within acceptable limits".to_string()
            },
            evidence,
        }
    }

    /// Analyze memory usage patterns and detect leaks
    fn analyze_memory_bottlenecks(&self, samples: &[PerformanceSample]) -> BottleneckFactor {
        let memory_usage_trend = self.calculate_memory_trend(samples);
        let avg_memory_usage = samples.iter()
            .map(|s| s.memory_metrics.used_memory as f64 / s.memory_metrics.total_memory as f64)
            .sum::<f64>() / samples.len() as f64;
        
        let severity = if avg_memory_usage > 0.9 {
            1.0
        } else if avg_memory_usage > 0.8 {
            0.8
        } else if memory_usage_trend > 0.1 { // Growing at >10% per measurement window
            0.6
        } else if avg_memory_usage > 0.7 {
            0.4
        } else {
            0.1
        };
        
        let mut evidence = Vec::new();
        evidence.push(format!("Average memory usage: {:.1}%", avg_memory_usage * 100.0));
        evidence.push(format!("Memory growth trend: {:.3}/period", memory_usage_trend));
        
        if memory_usage_trend > 0.05 {
            evidence.push("Potential memory leak detected".to_string());
        }
        
        // Check swap usage
        if let Some(sample) = samples.last() {
            if sample.memory_metrics.swap_used > 0 {
                evidence.push(format!("Swap usage detected: {} bytes", sample.memory_metrics.swap_used));
            }
        }
        
        BottleneckFactor {
            factor_type: "Memory".to_string(),
            severity,
            description: if severity > 0.8 {
                "High memory usage is causing performance degradation".to_string()
            } else if memory_usage_trend > 0.1 {
                "Memory usage is growing, indicating potential leak".to_string()
            } else {
                "Memory usage is within acceptable limits".to_string()
            },
            evidence,
        }
    }

    /// Analyze I/O performance patterns
    fn analyze_io_bottlenecks(&self, samples: &[PerformanceSample]) -> BottleneckFactor {
        let total_io = samples.iter()
            .map(|s| s.io_metrics.disk_read_bytes + s.io_metrics.disk_write_bytes)
            .sum::<u64>();
        
        let avg_io_per_second = if samples.len() > 1 {
            total_io / (samples.len() as u64 * (self.sampling_interval.as_millis() as u64 / 1000))
        } else {
            0
        };
        
        // Estimate severity based on I/O rate
        let severity = if avg_io_per_second > 100_000_000 { // 100MB/s
            0.8
        } else if avg_io_per_second > 50_000_000 { // 50MB/s
            0.5
        } else if avg_io_per_second > 10_000_000 { // 10MB/s
            0.3
        } else {
            0.1
        };
        
        let mut evidence = Vec::new();
        evidence.push(format!("Average I/O rate: {:.2} MB/s", avg_io_per_second as f64 / 1_000_000.0));
        evidence.push(format!("Total I/O during profiling: {:.2} MB", total_io as f64 / 1_000_000.0));
        
        BottleneckFactor {
            factor_type: "Disk I/O".to_string(),
            severity,
            description: if severity > 0.7 {
                "High disk I/O activity may be limiting performance".to_string()
            } else {
                "Disk I/O is within expected ranges".to_string()
            },
            evidence,
        }
    }

    /// Analyze network performance patterns
    fn analyze_network_bottlenecks(&self, samples: &[PerformanceSample]) -> BottleneckFactor {
        let total_network = samples.iter()
            .map(|s| s.network_metrics.bytes_sent + s.network_metrics.bytes_received)
            .sum::<u64>();
        
        let avg_network_per_second = if samples.len() > 1 {
            total_network / (samples.len() as u64 * (self.sampling_interval.as_millis() as u64 / 1000))
        } else {
            0
        };
        
        let total_errors = samples.iter()
            .map(|s| s.network_metrics.errors_in + s.network_metrics.errors_out)
            .sum::<u64>();
        
        let error_rate = if total_network > 0 {
            total_errors as f64 / total_network as f64
        } else {
            0.0
        };
        
        let severity = if avg_network_per_second > 10_000_000 { // 10MB/s
            0.5
        } else if error_rate > 0.01 { // 1% error rate
            0.6
        } else {
            0.1
        };
        
        let mut evidence = Vec::new();
        evidence.push(format!("Average network rate: {:.2} MB/s", avg_network_per_second as f64 / 1_000_000.0));
        evidence.push(format!("Network error rate: {:.4}%", error_rate * 100.0));
        
        BottleneckFactor {
            factor_type: "Network".to_string(),
            severity,
            description: if error_rate > 0.01 {
                "High network error rate detected".to_string()
            } else if severity > 0.4 {
                "Moderate network activity".to_string()
            } else {
                "Network usage is minimal".to_string()
            },
            evidence,
        }
    }

    /// Determine the primary bottleneck from all factors
    fn determine_primary_bottleneck(&self, cpu: &BottleneckFactor, memory: &BottleneckFactor, io: &BottleneckFactor, network: &BottleneckFactor) -> BottleneckType {
        let max_severity = [cpu.severity, memory.severity, io.severity, network.severity]
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        
        if *max_severity < 0.3 {
            return BottleneckType::None;
        }
        
        if cpu.severity == *max_severity {
            BottleneckType::CPU
        } else if memory.severity == *max_severity {
            BottleneckType::Memory
        } else if io.severity == *max_severity {
            BottleneckType::DiskIO
        } else if network.severity == *max_severity {
            BottleneckType::NetworkIO
        } else {
            BottleneckType::None
        }
    }

    /// Calculate performance impact metrics
    fn calculate_performance_impact(&self, samples: &[PerformanceSample], bottleneck: &BottleneckType) -> PerformanceImpact {
        // This is a simplified calculation - in practice would be more sophisticated
        let impact_multiplier = match bottleneck {
            BottleneckType::CPU => 0.8,
            BottleneckType::Memory => 0.6,
            BottleneckType::DiskIO => 0.7,
            BottleneckType::NetworkIO => 0.4,
            BottleneckType::Lock => 0.9,
            BottleneckType::Algorithm => 0.95,
            BottleneckType::None => 0.0,
        };
        
        PerformanceImpact {
            throughput_reduction: impact_multiplier * 30.0, // Up to 30% reduction
            latency_increase: impact_multiplier * 50.0,     // Up to 50% increase
            resource_waste: impact_multiplier * 25.0,       // Up to 25% waste
            projected_improvement: impact_multiplier * 40.0, // Up to 40% improvement if fixed
        }
    }

    /// Generate optimization recommendations
    fn generate_optimization_recommendations(&self, primary_bottleneck: &BottleneckType, _cpu: &BottleneckFactor, memory: &BottleneckFactor, _io: &BottleneckFactor, _network: &BottleneckFactor) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        
        match primary_bottleneck {
            BottleneckType::CPU => {
                recommendations.push(OptimizationRecommendation {
                    category: "CPU Optimization".to_string(),
                    priority: Priority::High,
                    description: "Implement parallel processing for CPU-intensive operations".to_string(),
                    expected_improvement: 25.0,
                    implementation_difficulty: Difficulty::Medium,
                    code_changes_required: vec![
                        "Add rayon or tokio parallelization".to_string(),
                        "Optimize algorithm complexity".to_string(),
                        "Use SIMD instructions where applicable".to_string(),
                    ],
                });
            }
            BottleneckType::Memory => {
                let leak_detected = memory.evidence.iter().any(|e| e.contains("leak"));
                
                recommendations.push(OptimizationRecommendation {
                    category: "Memory Optimization".to_string(),
                    priority: if leak_detected { Priority::Critical } else { Priority::High },
                    description: if leak_detected {
                        "Fix memory leak to prevent system degradation".to_string()
                    } else {
                        "Optimize memory usage to reduce pressure".to_string()
                    },
                    expected_improvement: 30.0,
                    implementation_difficulty: if leak_detected { Difficulty::Hard } else { Difficulty::Medium },
                    code_changes_required: vec![
                        "Implement object pooling".to_string(),
                        "Use weak references where appropriate".to_string(),
                        "Optimize data structures".to_string(),
                    ],
                });
            }
            BottleneckType::DiskIO => {
                recommendations.push(OptimizationRecommendation {
                    category: "I/O Optimization".to_string(),
                    priority: Priority::High,
                    description: "Optimize disk I/O operations".to_string(),
                    expected_improvement: 35.0,
                    implementation_difficulty: Difficulty::Medium,
                    code_changes_required: vec![
                        "Implement I/O batching".to_string(),
                        "Add caching layer".to_string(),
                        "Use async I/O operations".to_string(),
                    ],
                });
            }
            BottleneckType::NetworkIO => {
                recommendations.push(OptimizationRecommendation {
                    category: "Network Optimization".to_string(),
                    priority: Priority::Medium,
                    description: "Optimize network operations".to_string(),
                    expected_improvement: 20.0,
                    implementation_difficulty: Difficulty::Medium,
                    code_changes_required: vec![
                        "Implement connection pooling".to_string(),
                        "Add request batching".to_string(),
                        "Use compression".to_string(),
                    ],
                });
            }
            _ => {}
        }
        
        recommendations
    }

    /// Analyze performance trends
    fn analyze_trends(&self, samples: &[PerformanceSample]) -> TrendingAnalysis {
        if samples.len() < 5 {
            return TrendingAnalysis::default();
        }
        
        // Calculate memory growth rate
        let memory_growth_rate = self.calculate_memory_trend(samples);
        
        // Calculate performance stability
        let cpu_variance = self.calculate_cpu_variance(samples);
        let stability_score = 1.0 / (1.0 + cpu_variance / 100.0); // Higher variance = lower stability
        
        let performance_trend = if memory_growth_rate > 0.1 {
            Trend::Degrading
        } else if cpu_variance > 20.0 {
            Trend::Volatile
        } else if memory_growth_rate < -0.05 {
            Trend::Improving
        } else {
            Trend::Stable
        };
        
        TrendingAnalysis {
            performance_trend,
            resource_growth_rate: memory_growth_rate,
            degradation_rate: memory_growth_rate.max(0.0),
            stability_score,
        }
    }

    // Helper methods
    fn calculate_variance(&self, values: &[f32]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        
        let mean = values.iter().sum::<f32>() / values.len() as f32;
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f32>() / values.len() as f32;
        
        variance as f64
    }

    fn calculate_memory_trend(&self, samples: &[PerformanceSample]) -> f64 {
        if samples.len() < 2 {
            return 0.0;
        }
        
        let first = samples[0].memory_metrics.used_memory as f64;
        let last = samples.last().unwrap().memory_metrics.used_memory as f64;
        
        (last - first) / first
    }

    fn calculate_cpu_variance(&self, samples: &[PerformanceSample]) -> f64 {
        let cpu_values: Vec<f32> = samples.iter().map(|s| s.cpu_metrics.overall_usage).collect();
        self.calculate_variance(&cpu_values)
    }

    /// Generate comprehensive bottleneck report
    pub fn generate_report(&self, analysis: &BottleneckAnalysis) -> String {
        let mut report = String::new();
        
        report.push_str("# Bottleneck Analysis Report\n\n");
        
        // Executive summary
        report.push_str("## Executive Summary\n\n");
        report.push_str(&format!("**Primary Bottleneck:** {:?}\n", analysis.primary_bottleneck));
        report.push_str(&format!("**Confidence Level:** {:.1}%\n", analysis.bottleneck_confidence * 100.0));
        report.push_str(&format!("**Performance Impact:** {:.1}% throughput reduction, {:.1}% latency increase\n\n",
            analysis.performance_impact.throughput_reduction,
            analysis.performance_impact.latency_increase));
        
        // Detailed analysis
        report.push_str("## Detailed Analysis\n\n");
        for factor in &analysis.contributing_factors {
            report.push_str(&format!("### {} Analysis\n", factor.factor_type));
            report.push_str(&format!("**Severity:** {:.1}/10\n", factor.severity * 10.0));
            report.push_str(&format!("**Description:** {}\n\n", factor.description));
            
            report.push_str("**Evidence:**\n");
            for evidence in &factor.evidence {
                report.push_str(&format!("- {}\n", evidence));
            }
            report.push_str("\n");
        }
        
        // Optimization recommendations
        report.push_str("## Optimization Recommendations\n\n");
        for (i, rec) in analysis.optimization_recommendations.iter().enumerate() {
            report.push_str(&format!("### {}. {} ({:?} Priority)\n", i + 1, rec.category, rec.priority));
            report.push_str(&format!("**Expected Improvement:** {:.1}%\n", rec.expected_improvement));
            report.push_str(&format!("**Implementation Difficulty:** {:?}\n", rec.implementation_difficulty));
            report.push_str(&format!("**Description:** {}\n\n", rec.description));
            
            report.push_str("**Required Changes:**\n");
            for change in &rec.code_changes_required {
                report.push_str(&format!("- {}\n", change));
            }
            report.push_str("\n");
        }
        
        // Trending analysis
        report.push_str("## Performance Trends\n\n");
        report.push_str(&format!("**Overall Trend:** {:?}\n", analysis.trending_analysis.performance_trend));
        report.push_str(&format!("**Stability Score:** {:.2}/1.0\n", analysis.trending_analysis.stability_score));
        report.push_str(&format!("**Resource Growth Rate:** {:.2}%/period\n", analysis.trending_analysis.resource_growth_rate * 100.0));
        
        report
    }
}

// Implementation of helper structs
impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::new(),
            leak_threshold: 0.1,
            monitoring_window: Duration::from_secs(300),
        }
    }
}

impl CPUProfiler {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::new(),
            high_usage_threshold: 80.0,
            profiling_depth: 1000,
        }
    }
}

impl IOProfiler {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::new(),
            io_wait_threshold: 30.0,
            bandwidth_threshold: 100_000_000,
        }
    }
}

impl NetworkProfiler {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::new(),
            bandwidth_threshold: 10_000_000,
            error_rate_threshold: 0.01,
        }
    }
}

impl Default for BottleneckAnalysis {
    fn default() -> Self {
        Self {
            primary_bottleneck: BottleneckType::None,
            bottleneck_confidence: 0.0,
            contributing_factors: Vec::new(),
            performance_impact: PerformanceImpact::default(),
            optimization_recommendations: Vec::new(),
            trending_analysis: TrendingAnalysis::default(),
        }
    }
}

impl Default for PerformanceImpact {
    fn default() -> Self {
        Self {
            throughput_reduction: 0.0,
            latency_increase: 0.0,
            resource_waste: 0.0,
            projected_improvement: 0.0,
        }
    }
}

impl Default for TrendingAnalysis {
    fn default() -> Self {
        Self {
            performance_trend: Trend::Stable,
            resource_growth_rate: 0.0,
            degradation_rate: 0.0,
            stability_score: 1.0,
        }
    }
}

/// Run comprehensive bottleneck profiling
pub async fn run_bottleneck_profiling() -> Result<()> {
    let mut profiler = BottleneckProfiler::new();
    
    // Start profiling
    profiler.start_profiling().await?;
    
    // Create test searcher for profiling
    let project_path = PathBuf::from(".");
    let db_path = PathBuf::from("./bottleneck_test_db");
    Config::init_test().expect("Failed to initialize test config");
    let searcher = UnifiedSearcher::new_with_backend(
        project_path,
        db_path,
        SearchBackend::Tantivy,
    ).await?;
    
    // Profile search operations
    println!("🔍 Profiling search operations...");
    
    let test_queries = ["function", "async", "error", "handler"];
    
    for query in &test_queries {
        let (_result, metrics) = profiler.profile_search_operation(
            &format!("search_{}", query),
            searcher.search(query)
        ).await?;
        
        println!("  Query '{}': {:.2}ms latency, {:.2}% CPU", 
                 query, metrics.latency.as_secs_f64() * 1000.0, metrics.cpu_usage_during);
    }
    
    // Let profiling run for a bit
    tokio::time::sleep(Duration::from_secs(30)).await;
    
    // Stop profiling and analyze
    profiler.stop_profiling().await;
    
    let analysis = profiler.analyze_bottlenecks().await?;
    let report = profiler.generate_report(&analysis);
    
    // Save report
    tokio::fs::write("bottleneck_analysis_report.md", &report).await?;
    
    println!("📊 Bottleneck analysis report generated: bottleneck_analysis_report.md");
    println!("Primary bottleneck: {:?}", analysis.primary_bottleneck);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bottleneck_profiler() -> Result<()> {
        let mut profiler = BottleneckProfiler::new();
        
        // Test profiling start/stop
        profiler.start_profiling().await?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        profiler.stop_profiling().await;
        
        // Test analysis with minimal data
        let analysis = profiler.analyze_bottlenecks().await?;
        println!("✅ Bottleneck profiler test completed: {:?}", analysis.primary_bottleneck);
        
        Ok(())
    }
}