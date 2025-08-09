//! Extended Memory Monitor for GGUF Reader Testing
//! 
//! This module extends the basic memory monitor with capabilities needed
//! for comprehensive memory safety validation of the streaming GGUF reader.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use anyhow::Result;

/// Memory allocation record
#[derive(Debug, Clone)]
pub struct AllocationRecord {
    pub size: usize,
    pub timestamp: Instant,
    pub stack_trace: String,
    pub allocation_type: AllocationType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AllocationType {
    TensorData,
    ModelWeights,
    WorkingBuffer,
    GeneralHeap,
    Unknown,
}

/// Memory allocation tracker that provides detailed monitoring
pub struct AllocationTracker {
    records: Arc<Mutex<VecDeque<AllocationRecord>>>,
    max_records: usize,
    total_allocated: AtomicU64,
    peak_allocation: AtomicU64,
    current_allocation: AtomicU64,
    violations: Arc<Mutex<Vec<String>>>,
    max_single_allocation: usize,
    enabled: AtomicBool,
}

impl AllocationTracker {
    pub fn new(max_single_allocation: usize, max_records: usize) -> Self {
        Self {
            records: Arc::new(Mutex::new(VecDeque::with_capacity(max_records))),
            max_records,
            total_allocated: AtomicU64::new(0),
            peak_allocation: AtomicU64::new(0),
            current_allocation: AtomicU64::new(0),
            violations: Arc::new(Mutex::new(Vec::new())),
            max_single_allocation,
            enabled: AtomicBool::new(true),
        }
    }
    
    /// Track a memory allocation
    pub fn track_allocation(
        &self, 
        size: usize, 
        allocation_type: AllocationType
    ) -> Result<AllocationGuard> {
        if !self.enabled.load(Ordering::SeqCst) {
            return Ok(AllocationGuard::new(self, size));
        }
        
        // CRITICAL: Check for violations
        if size > self.max_single_allocation {
            let violation = format!(
                "VIOLATION: Allocation of {} bytes exceeds limit of {} bytes (type: {:?})",
                size, self.max_single_allocation, allocation_type
            );
            
            let mut violations = self.violations.lock().unwrap();
            violations.push(violation.clone());
            
            return Err(anyhow::anyhow!("{}", violation));
        }
        
        // Update counters
        let old_total = self.total_allocated.fetch_add(size as u64, Ordering::SeqCst);
        let new_current = self.current_allocation.fetch_add(size as u64, Ordering::SeqCst) + size as u64;
        
        // Update peak
        let mut peak = self.peak_allocation.load(Ordering::SeqCst);
        while new_current > peak {
            match self.peak_allocation.compare_exchange_weak(
                peak, new_current, Ordering::SeqCst, Ordering::SeqCst
            ) {
                Ok(_) => break,
                Err(x) => peak = x,
            }
        }
        
        // Record the allocation
        let record = AllocationRecord {
            size,
            timestamp: Instant::now(),
            stack_trace: self.capture_stack_trace(),
            allocation_type,
        };
        
        {
            let mut records = self.records.lock().unwrap();
            if records.len() >= self.max_records {
                records.pop_front();
            }
            records.push_back(record);
        }
        
        Ok(AllocationGuard::new(self, size))
    }
    
    /// Track deallocation
    pub fn track_deallocation(&self, size: usize) {
        if !self.enabled.load(Ordering::SeqCst) {
            return;
        }
        
        self.current_allocation.fetch_sub(size as u64, Ordering::SeqCst);
    }
    
    /// Get allocation statistics
    pub fn get_stats(&self) -> AllocationStats {
        AllocationStats {
            total_allocated: self.total_allocated.load(Ordering::SeqCst),
            current_allocated: self.current_allocation.load(Ordering::SeqCst),
            peak_allocated: self.peak_allocation.load(Ordering::SeqCst),
            violation_count: self.violations.lock().unwrap().len(),
            record_count: self.records.lock().unwrap().len(),
        }
    }
    
    /// Get all violations
    pub fn get_violations(&self) -> Vec<String> {
        self.violations.lock().unwrap().clone()
    }
    
    /// Get recent allocation records
    pub fn get_recent_allocations(&self, count: usize) -> Vec<AllocationRecord> {
        let records = self.records.lock().unwrap();
        records.iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }
    
    /// Clear all tracking data
    pub fn clear(&self) {
        self.records.lock().unwrap().clear();
        self.violations.lock().unwrap().clear();
        self.total_allocated.store(0, Ordering::SeqCst);
        self.current_allocation.store(0, Ordering::SeqCst);
        self.peak_allocation.store(0, Ordering::SeqCst);
    }
    
    /// Enable/disable tracking
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);
    }
    
    /// Check if any violations occurred
    pub fn has_violations(&self) -> bool {
        !self.violations.lock().unwrap().is_empty()
    }
    
    /// Capture stack trace (simplified for testing)
    fn capture_stack_trace(&self) -> String {
        // In a real implementation, you'd capture actual stack traces
        format!("allocation_at_{:?}", Instant::now())
    }
}

/// RAII guard for tracking allocation lifetime
pub struct AllocationGuard<'a> {
    tracker: &'a AllocationTracker,
    size: usize,
}

impl<'a> AllocationGuard<'a> {
    fn new(tracker: &'a AllocationTracker, size: usize) -> Self {
        Self { tracker, size }
    }
}

impl<'a> Drop for AllocationGuard<'a> {
    fn drop(&mut self) {
        self.tracker.track_deallocation(self.size);
    }
}

/// Allocation statistics
#[derive(Debug, Clone)]
pub struct AllocationStats {
    pub total_allocated: u64,
    pub current_allocated: u64,
    pub peak_allocated: u64,
    pub violation_count: usize,
    pub record_count: usize,
}

/// Enhanced memory monitor for GGUF validation
pub struct GGUFMemoryMonitor {
    allocation_tracker: Arc<AllocationTracker>,
    system_monitor: embed::utils::memory::MemoryMonitor,
    monitoring_interval: Duration,
    last_check: Arc<Mutex<Instant>>,
    memory_samples: Arc<Mutex<VecDeque<MemorySample>>>,
    max_samples: usize,
}

#[derive(Debug, Clone)]
pub struct MemorySample {
    pub timestamp: Instant,
    pub system_memory_mb: u64,
    pub process_memory_mb: u64,
    pub heap_allocated_mb: u64,
    pub memory_pressure: String,
}

impl GGUFMemoryMonitor {
    /// Create new enhanced memory monitor
    pub fn new(max_single_allocation_mb: usize) -> Result<Self> {
        let allocation_tracker = Arc::new(AllocationTracker::new(
            max_single_allocation_mb * 1_048_576, // Convert MB to bytes
            10000 // Max allocation records
        ));
        
        let system_monitor = embed::utils::memory::MemoryMonitor::new()?;
        
        Ok(Self {
            allocation_tracker,
            system_monitor,
            monitoring_interval: Duration::from_millis(100),
            last_check: Arc::new(Mutex::new(Instant::now())),
            memory_samples: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            max_samples: 1000,
        })
    }
    
    /// Start continuous monitoring
    pub fn start_monitoring(&self) -> Arc<AtomicBool> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();
        
        let monitor = self.system_monitor.clone();
        let samples = self.memory_samples.clone();
        let tracker = self.allocation_tracker.clone();
        let interval = self.monitoring_interval;
        let max_samples = self.max_samples;
        
        tokio::spawn(async move {
            while !stop_flag_clone.load(Ordering::SeqCst) {
                if let Ok(usage) = monitor.get_memory_usage() {
                    let stats = tracker.get_stats();
                    
                    let sample = MemorySample {
                        timestamp: Instant::now(),
                        system_memory_mb: usage.total_memory / 1_048_576,
                        process_memory_mb: usage.process_memory / 1_048_576,
                        heap_allocated_mb: stats.current_allocated / 1_048_576,
                        memory_pressure: format!("{:?}", usage.memory_pressure),
                    };
                    
                    let mut samples_guard = samples.lock().unwrap();
                    if samples_guard.len() >= max_samples {
                        samples_guard.pop_front();
                    }
                    samples_guard.push_back(sample);
                }
                
                tokio::time::sleep(interval).await;
            }
        });
        
        stop_flag
    }
    
    /// Track tensor allocation
    pub fn track_tensor_allocation(&self, size: usize) -> Result<AllocationGuard> {
        self.allocation_tracker.track_allocation(size, AllocationType::TensorData)
    }
    
    /// Track model weight allocation
    pub fn track_model_allocation(&self, size: usize) -> Result<AllocationGuard> {
        self.allocation_tracker.track_allocation(size, AllocationType::ModelWeights)
    }
    
    /// Track working buffer allocation
    pub fn track_buffer_allocation(&self, size: usize) -> Result<AllocationGuard> {
        self.allocation_tracker.track_allocation(size, AllocationType::WorkingBuffer)
    }
    
    /// Get comprehensive memory report
    pub fn get_memory_report(&self) -> Result<MemoryReport> {
        let system_usage = self.system_monitor.get_memory_usage()?;
        let allocation_stats = self.allocation_tracker.get_stats();
        let violations = self.allocation_tracker.get_violations();
        
        let samples = self.memory_samples.lock().unwrap();
        let memory_trend = if samples.len() > 1 {
            let first = &samples[0];
            let last = &samples[samples.len() - 1];
            let mb_change = last.process_memory_mb as i64 - first.process_memory_mb as i64;
            format!("{:+}MB over {} samples", mb_change, samples.len())
        } else {
            "Insufficient data".to_string()
        };
        
        Ok(MemoryReport {
            system_memory_mb: system_usage.total_memory / 1_048_576,
            process_memory_mb: system_usage.process_memory / 1_048_576,
            heap_allocated_mb: allocation_stats.current_allocated / 1_048_576,
            peak_allocated_mb: allocation_stats.peak_allocated / 1_048_576,
            memory_pressure: format!("{:?}", system_usage.memory_pressure),
            violations,
            memory_trend,
            allocation_count: allocation_stats.record_count,
            sample_count: samples.len(),
        })
    }
    
    /// Check if memory usage is safe for GGUF operations
    pub fn is_safe_for_gguf_operations(&self) -> Result<bool> {
        let system_usage = self.system_monitor.get_memory_usage()?;
        let stats = self.allocation_tracker.get_stats();
        
        // Multiple safety checks
        let system_safe = matches!(
            system_usage.memory_pressure, 
            embed::utils::memory::MemoryPressure::Low | 
            embed::utils::memory::MemoryPressure::Medium
        );
        
        let heap_safe = stats.current_allocated < 50_000_000; // 50MB limit
        let no_violations = !self.allocation_tracker.has_violations();
        
        Ok(system_safe && heap_safe && no_violations)
    }
    
    /// Force garbage collection and measurement
    pub fn force_gc_and_measure(&self) -> Result<(u64, u64)> {
        let before = self.system_monitor.get_memory_usage()?.process_memory;
        
        // Note: Rust doesn't have explicit GC, but we can suggest cleanup
        // In a real implementation, you might trigger cleanup of internal caches
        
        std::thread::sleep(Duration::from_millis(100)); // Allow cleanup time
        
        let after = self.system_monitor.get_memory_usage()?.process_memory;
        Ok((before / 1_048_576, after / 1_048_576))
    }
    
    /// Get memory samples for analysis
    pub fn get_memory_samples(&self) -> Vec<MemorySample> {
        self.memory_samples.lock().unwrap().iter().cloned().collect()
    }
    
    /// Clear all monitoring data
    pub fn clear_monitoring_data(&self) {
        self.allocation_tracker.clear();
        self.memory_samples.lock().unwrap().clear();
    }
}

/// Comprehensive memory report
#[derive(Debug)]
pub struct MemoryReport {
    pub system_memory_mb: u64,
    pub process_memory_mb: u64,
    pub heap_allocated_mb: u64,
    pub peak_allocated_mb: u64,
    pub memory_pressure: String,
    pub violations: Vec<String>,
    pub memory_trend: String,
    pub allocation_count: usize,
    pub sample_count: usize,
}

impl std::fmt::Display for MemoryReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "Memory Report:\n\
             System Memory: {}MB\n\
             Process Memory: {}MB\n\
             Heap Allocated: {}MB\n\
             Peak Allocated: {}MB\n\
             Memory Pressure: {}\n\
             Memory Trend: {}\n\
             Allocation Records: {}\n\
             Monitoring Samples: {}\n\
             Violations: {}",
            self.system_memory_mb,
            self.process_memory_mb,
            self.heap_allocated_mb,
            self.peak_allocated_mb,
            self.memory_pressure,
            self.memory_trend,
            self.allocation_count,
            self.sample_count,
            if self.violations.is_empty() { 
                "None".to_string() 
            } else { 
                format!("{} violations", self.violations.len()) 
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_allocation_tracker() {
        let tracker = AllocationTracker::new(1_048_576, 100); // 1MB limit
        
        // Normal allocation should work
        let guard = tracker.track_allocation(65536, AllocationType::WorkingBuffer).unwrap();
        let stats = tracker.get_stats();
        assert_eq!(stats.current_allocated, 65536);
        
        drop(guard);
        let stats = tracker.get_stats();
        assert_eq!(stats.current_allocated, 0);
        
        // Large allocation should fail
        let result = tracker.track_allocation(2_000_000, AllocationType::TensorData);
        assert!(result.is_err());
        assert!(tracker.has_violations());
    }
    
    #[tokio::test]
    async fn test_memory_monitor() {
        let monitor = GGUFMemoryMonitor::new(1).unwrap(); // 1MB limit
        
        // Test normal allocation
        let guard = monitor.track_buffer_allocation(65536).unwrap();
        let report = monitor.get_memory_report().unwrap();
        assert!(report.heap_allocated_mb < 1);
        
        drop(guard);
        
        // Test violation
        let result = monitor.track_tensor_allocation(2_000_000);
        assert!(result.is_err());
        
        let report = monitor.get_memory_report().unwrap();
        assert!(!report.violations.is_empty());
    }
}