/// Memory monitoring utilities for detecting and preventing OOM conditions
/// 
/// This module provides utilities to monitor memory usage and prevent
/// out-of-memory crashes, especially important when running in constrained
/// environments like Node.js/V8.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use anyhow::Result;

/// Memory usage monitor with configurable thresholds
pub struct MemoryMonitor {
    /// Maximum allowed memory in bytes
    max_memory_bytes: u64,
    /// Current memory usage in bytes
    current_usage: Arc<AtomicU64>,
    /// Warning threshold as percentage (0-100)
    warning_threshold_percent: u8,
}

impl MemoryMonitor {
    /// Create a new memory monitor with specified limits
    pub fn new(max_memory_mb: u64, warning_threshold_percent: u8) -> Self {
        Self {
            max_memory_bytes: max_memory_mb * 1024 * 1024,
            current_usage: Arc::new(AtomicU64::new(0)),
            warning_threshold_percent: warning_threshold_percent.min(100),
        }
    }
    
    /// Create a monitor suitable for Node.js environments
    /// Default: 2GB max, warn at 80%
    pub fn for_nodejs() -> Self {
        Self::new(2048, 80)
    }
    
    /// Check if allocation would exceed limits
    pub fn can_allocate(&self, bytes: usize) -> bool {
        let current = self.current_usage.load(Ordering::Relaxed);
        let new_total = current + bytes as u64;
        new_total <= self.max_memory_bytes
    }
    
    /// Try to allocate memory, returning error if would exceed limits
    pub fn try_allocate(&self, bytes: usize) -> Result<MemoryAllocation> {
        let current = self.current_usage.load(Ordering::Relaxed);
        let new_total = current + bytes as u64;
        
        if new_total > self.max_memory_bytes {
            anyhow::bail!(
                "Memory allocation would exceed limit: {} MB requested, {} MB available",
                bytes / 1_048_576,
                (self.max_memory_bytes - current) / 1_048_576
            );
        }
        
        // Check if we should warn
        let usage_percent = (new_total as f64 / self.max_memory_bytes as f64) * 100.0;
        if usage_percent >= self.warning_threshold_percent as f64 {
            eprintln!(
                "⚠️  Memory usage warning: {:.1}% of limit ({} MB / {} MB)",
                usage_percent,
                new_total / 1_048_576,
                self.max_memory_bytes / 1_048_576
            );
        }
        
        // Update usage
        self.current_usage.fetch_add(bytes as u64, Ordering::SeqCst);
        
        Ok(MemoryAllocation {
            monitor: self.current_usage.clone(),
            bytes: bytes as u64,
        })
    }
    
    /// Get current memory usage in bytes
    pub fn current_usage_bytes(&self) -> u64 {
        self.current_usage.load(Ordering::Relaxed)
    }
    
    /// Get current memory usage in MB
    pub fn current_usage_mb(&self) -> u64 {
        self.current_usage_bytes() / 1_048_576
    }
    
    /// Get memory limit in MB
    pub fn limit_mb(&self) -> u64 {
        self.max_memory_bytes / 1_048_576
    }
    
    /// Get percentage of memory used
    pub fn usage_percent(&self) -> f64 {
        (self.current_usage_bytes() as f64 / self.max_memory_bytes as f64) * 100.0
    }
    
    /// Check if memory usage is critical (>90%)
    pub fn is_critical(&self) -> bool {
        self.usage_percent() > 90.0
    }
}

/// RAII guard for memory allocation tracking
pub struct MemoryAllocation {
    monitor: Arc<AtomicU64>,
    bytes: u64,
}

impl Drop for MemoryAllocation {
    fn drop(&mut self) {
        // Release memory when allocation is dropped
        self.monitor.fetch_sub(self.bytes, Ordering::SeqCst);
    }
}

/// Get system memory info (best effort, may not work on all platforms)
pub fn get_system_memory_info() -> Option<SystemMemoryInfo> {
    #[cfg(target_os = "windows")]
    {
        use winapi::um::sysinfoapi::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
        use std::mem;
        
        unsafe {
            let mut status: MEMORYSTATUSEX = mem::zeroed();
            status.dwLength = mem::size_of::<MEMORYSTATUSEX>() as u32;
            
            if GlobalMemoryStatusEx(&mut status) != 0 {
                return Some(SystemMemoryInfo {
                    total_mb: status.ullTotalPhys / 1_048_576,
                    available_mb: status.ullAvailPhys / 1_048_576,
                    used_percent: status.dwMemoryLoad as f64,
                });
            }
        }
    }
    
    #[cfg(unix)]
    {
        // Try to read from /proc/meminfo on Linux
        if let Ok(contents) = std::fs::read_to_string("/proc/meminfo") {
            let mut total_kb = 0u64;
            let mut available_kb = 0u64;
            
            for line in contents.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(kb) = parse_meminfo_line(line) {
                        total_kb = kb;
                    }
                } else if line.starts_with("MemAvailable:") {
                    if let Some(kb) = parse_meminfo_line(line) {
                        available_kb = kb;
                    }
                }
            }
            
            if total_kb > 0 && available_kb > 0 {
                let used_kb = total_kb - available_kb;
                return Some(SystemMemoryInfo {
                    total_mb: total_kb / 1024,
                    available_mb: available_kb / 1024,
                    used_percent: (used_kb as f64 / total_kb as f64) * 100.0,
                });
            }
        }
    }
    
    None
}

#[cfg(unix)]
fn parse_meminfo_line(line: &str) -> Option<u64> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        parts[1].parse().ok()
    } else {
        None
    }
}

/// System memory information
#[derive(Debug, Clone)]
pub struct SystemMemoryInfo {
    /// Total system memory in MB
    pub total_mb: u64,
    /// Available system memory in MB
    pub available_mb: u64,
    /// Percentage of memory used
    pub used_percent: f64,
}

impl SystemMemoryInfo {
    /// Check if system memory is low (<500MB available)
    pub fn is_low_memory(&self) -> bool {
        self.available_mb < 500
    }
    
    /// Check if system memory is critical (<200MB available)
    pub fn is_critical_memory(&self) -> bool {
        self.available_mb < 200
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_monitor() {
        let monitor = MemoryMonitor::new(100, 80);
        
        // Should allow small allocations
        assert!(monitor.can_allocate(10_000_000)); // 10MB
        
        // Should track allocations
        let _alloc1 = monitor.try_allocate(10_000_000).unwrap();
        assert_eq!(monitor.current_usage_mb(), 9); // ~10MB
        
        // Should prevent exceeding limit
        assert!(!monitor.can_allocate(100_000_000)); // 100MB
        
        // Should release memory when allocation dropped
        drop(_alloc1);
        assert_eq!(monitor.current_usage_mb(), 0);
    }
}