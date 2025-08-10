// Memory utilities for monitoring and management

use anyhow::Result;

#[cfg(windows)]
use winapi::um::sysinfoapi::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

#[cfg(not(windows))]
use std::fs;

#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total_memory: u64,
    pub available_memory: u64,
    pub used_memory: u64,
    pub memory_usage_percent: f32,
}

impl MemoryInfo {
    #[cfg(windows)]
    pub fn current() -> Result<Self> {
        unsafe {
            let mut mem_status = MEMORYSTATUSEX {
                dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
                dwMemoryLoad: 0,
                ullTotalPhys: 0,
                ullAvailPhys: 0,
                ullTotalPageFile: 0,
                ullAvailPageFile: 0,
                ullTotalVirtual: 0,
                ullAvailVirtual: 0,
                ullAvailExtendedVirtual: 0,
            };
            
            GlobalMemoryStatusEx(&mut mem_status);
            
            let total = mem_status.ullTotalPhys;
            let available = mem_status.ullAvailPhys;
            let used = total - available;
            let usage_percent = (used as f64 / total as f64 * 100.0) as f32;
            
            Ok(MemoryInfo {
                total_memory: total,
                available_memory: available,
                used_memory: used,
                memory_usage_percent: usage_percent,
            })
        }
    }
    
    #[cfg(not(windows))]
    pub fn current() -> Result<Self> {
        // Linux/Unix implementation using /proc/meminfo
        let meminfo = fs::read_to_string("/proc/meminfo")?;
        let mut total = 0u64;
        let mut available = 0u64;
        
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    total = parts[1].parse::<u64>().unwrap_or(0) * 1024; // Convert KB to bytes
                }
            } else if line.starts_with("MemAvailable:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    available = parts[1].parse::<u64>().unwrap_or(0) * 1024;
                }
            }
        }
        
        let used = total - available;
        let usage_percent = if total > 0 {
            (used as f64 / total as f64 * 100.0) as f32
        } else {
            0.0
        };
        
        Ok(MemoryInfo {
            total_memory: total,
            available_memory: available,
            used_memory: used,
            memory_usage_percent: usage_percent,
        })
    }
    
    pub fn is_memory_pressure(&self) -> bool {
        self.memory_usage_percent > 80.0
    }
    
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

pub fn check_memory_available(required_mb: usize) -> Result<bool> {
    let info = MemoryInfo::current()?;
    let required_bytes = (required_mb as u64) * 1024 * 1024;
    Ok(info.available_memory >= required_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_info() {
        let info = MemoryInfo::current().unwrap();
        assert!(info.total_memory > 0);
        assert!(info.memory_usage_percent >= 0.0 && info.memory_usage_percent <= 100.0);
    }
    
    #[test]
    fn test_format_bytes() {
        assert_eq!(MemoryInfo::format_bytes(512), "512.00 B");
        assert_eq!(MemoryInfo::format_bytes(1024), "1.00 KB");
        assert_eq!(MemoryInfo::format_bytes(1024 * 1024), "1.00 MB");
    }
}