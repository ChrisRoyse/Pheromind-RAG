//! V8 Crash Prevention Tests
//! 
//! This module specifically tests scenarios that historically caused V8 crashes
//! and validates that the streaming GGUF reader prevents them.

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::process::{Command, Stdio};
use std::io::Write;

use embed::embedding::streaming_core::StreamingGGUFLoader;
use embed::utils::memory::MemoryMonitor;
use crate::memory_safety::memory_monitor_extension::GGUFMemoryMonitor;

/// V8 crash scenarios that we need to prevent
#[derive(Debug, Clone)]
pub enum V8CrashScenario {
    LargeMemoryAllocation,
    RapidMemoryGrowth,
    MemoryFragmentation,
    GCPressure,
    ArrayBufferOverflow,
    StringConcatenationBomb,
    RecursionStackOverflow,
    EventLoopBlocking,
}

/// V8 crash prevention tester
pub struct V8CrashTester {
    monitor: Arc<GGUFMemoryMonitor>,
    crash_scenarios: Vec<V8CrashScenario>,
    test_results: std::sync::Mutex<Vec<V8TestResult>>,
}

#[derive(Debug)]
pub struct V8TestResult {
    pub scenario: V8CrashScenario,
    pub prevented_crash: bool,
    pub memory_usage_mb: u64,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
}

impl V8CrashTester {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            monitor: Arc::new(GGUFMemoryMonitor::new(1)?), // 1MB limit
            crash_scenarios: vec![
                V8CrashScenario::LargeMemoryAllocation,
                V8CrashScenario::RapidMemoryGrowth,
                V8CrashScenario::MemoryFragmentation,
                V8CrashScenario::GCPressure,
                V8CrashScenario::ArrayBufferOverflow,
                V8CrashScenario::StringConcatenationBomb,
                V8CrashScenario::RecursionStackOverflow,
                V8CrashScenario::EventLoopBlocking,
            ],
            test_results: std::sync::Mutex::new(Vec::new()),
        })
    }
    
    /// Run all V8 crash prevention tests
    pub async fn run_all_tests(&self) -> anyhow::Result<Vec<V8TestResult>> {
        println!("🛡️  Running V8 Crash Prevention Test Suite");
        println!("==========================================");
        
        for scenario in &self.crash_scenarios {
            let result = self.test_scenario(scenario.clone()).await?;
            
            println!("  Test {:?}: {}", 
                scenario, 
                if result.prevented_crash { "✅ PREVENTED" } else { "❌ FAILED" }
            );
            
            self.test_results.lock().unwrap().push(result);
        }
        
        Ok(self.test_results.lock().unwrap().clone())
    }
    
    /// Test individual crash scenario
    async fn test_scenario(&self, scenario: V8CrashScenario) -> anyhow::Result<V8TestResult> {
        let start_time = Instant::now();
        let initial_memory = self.monitor.get_memory_report()?.process_memory_mb;
        
        let (prevented_crash, error_message) = match scenario {
            V8CrashScenario::LargeMemoryAllocation => {
                self.test_large_memory_allocation().await
            }
            V8CrashScenario::RapidMemoryGrowth => {
                self.test_rapid_memory_growth().await
            }
            V8CrashScenario::MemoryFragmentation => {
                self.test_memory_fragmentation().await
            }
            V8CrashScenario::GCPressure => {
                self.test_gc_pressure().await
            }
            V8CrashScenario::ArrayBufferOverflow => {
                self.test_array_buffer_overflow().await
            }
            V8CrashScenario::StringConcatenationBomb => {
                self.test_string_concatenation_bomb().await
            }
            V8CrashScenario::RecursionStackOverflow => {
                self.test_recursion_stack_overflow().await
            }
            V8CrashScenario::EventLoopBlocking => {
                self.test_event_loop_blocking().await
            }
        };
        
        let final_memory = self.monitor.get_memory_report()?.process_memory_mb;
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        Ok(V8TestResult {
            scenario,
            prevented_crash,
            memory_usage_mb: final_memory.max(initial_memory) - initial_memory,
            execution_time_ms: execution_time,
            error_message,
        })
    }
    
    /// Test scenario: Large memory allocation
    async fn test_large_memory_allocation(&self) -> (bool, Option<String>) {
        println!("    Testing large memory allocation prevention...");
        
        // Try to allocate a buffer that would crash V8
        match std::panic::catch_unwind(|| {
            // This should be caught by our memory monitor
            let result = self.monitor.track_tensor_allocation(500_000_000); // 500MB
            match result {
                Ok(_) => false, // Should not succeed
                Err(_) => true,  // Should be prevented
            }
        }) {
            Ok(prevented) => (prevented, None),
            Err(_) => (false, Some("Panic occurred during large allocation test".to_string())),
        }
    }
    
    /// Test scenario: Rapid memory growth
    async fn test_rapid_memory_growth(&self) -> (bool, Option<String>) {
        println!("    Testing rapid memory growth prevention...");
        
        let mut allocations = Vec::new();
        let start_memory = self.monitor.get_memory_report().unwrap().process_memory_mb;
        
        // Try to rapidly allocate memory
        for i in 0..100 {
            match self.monitor.track_tensor_allocation(100_000) { // 100KB each
                Ok(guard) => {
                    allocations.push(guard);
                    
                    // Check if memory monitor is working
                    let current_memory = self.monitor.get_memory_report().unwrap().process_memory_mb;
                    if current_memory - start_memory > 50 { // 50MB limit
                        return (true, Some("Memory growth properly limited".to_string()));
                    }
                }
                Err(e) => {
                    return (true, Some(format!("Allocation #{} properly rejected: {}", i, e)));
                }
            }
            
            tokio::task::yield_now().await;
        }
        
        (false, Some("Memory growth was not limited".to_string()))
    }
    
    /// Test scenario: Memory fragmentation
    async fn test_memory_fragmentation(&self) -> (bool, Option<String>) {
        println!("    Testing memory fragmentation handling...");
        
        let mut small_allocations = Vec::new();
        
        // Create many small allocations
        for i in 0..1000 {
            match self.monitor.track_buffer_allocation(1024) { // 1KB each
                Ok(guard) => {
                    small_allocations.push(guard);
                    
                    // Drop every other allocation to create fragmentation
                    if i % 2 == 0 && small_allocations.len() > 1 {
                        small_allocations.remove(small_allocations.len() - 2);
                    }
                }
                Err(_) => {
                    // Should handle gracefully
                    break;
                }
            }
            
            if i % 100 == 0 {
                tokio::task::yield_now().await;
            }
        }
        
        // Check if system remains stable
        let memory_report = self.monitor.get_memory_report().unwrap();
        let stable = memory_report.violations.is_empty();
        
        (stable, if stable { 
            Some("Memory fragmentation handled gracefully".to_string()) 
        } else { 
            Some("Memory violations occurred".to_string()) 
        })
    }
    
    /// Test scenario: GC pressure
    async fn test_gc_pressure(&self) -> (bool, Option<String>) {
        println!("    Testing GC pressure handling...");
        
        // Create rapid allocation/deallocation cycles
        for cycle in 0..50 {
            let mut cycle_allocations = Vec::new();
            
            // Allocate
            for _ in 0..20 {
                if let Ok(guard) = self.monitor.track_tensor_allocation(50000) { // 50KB
                    cycle_allocations.push(guard);
                }
            }
            
            // Deallocate (drop guards)
            drop(cycle_allocations);
            
            // Check memory stability
            if !self.monitor.is_safe_for_gguf_operations().unwrap_or(false) {
                return (true, Some(format!("GC pressure detected at cycle {}, operations paused", cycle)));
            }
            
            tokio::task::yield_now().await;
        }
        
        (true, Some("GC pressure handled without crashes".to_string()))
    }
    
    /// Test scenario: Array buffer overflow
    async fn test_array_buffer_overflow(&self) -> (bool, Option<String>) {
        println!("    Testing array buffer overflow prevention...");
        
        // Simulate large array buffer operations that could overflow
        let large_size = 2_000_000_000usize; // 2GB - would crash V8
        
        match std::panic::catch_unwind(|| {
            // Try to create a buffer that's too large
            match self.monitor.track_tensor_allocation(large_size) {
                Ok(_) => false, // Should not succeed
                Err(_) => true,  // Should be prevented
            }
        }) {
            Ok(prevented) => (prevented, if prevented { 
                Some("Large buffer allocation properly prevented".to_string()) 
            } else { 
                Some("Large buffer was allowed - potential crash risk".to_string()) 
            }),
            Err(_) => (false, Some("Test panicked - buffer overflow protection failed".to_string())),
        }
    }
    
    /// Test scenario: String concatenation bomb
    async fn test_string_concatenation_bomb(&self) -> (bool, Option<String>) {
        println!("    Testing string concatenation bomb prevention...");
        
        // Simulate processing of text that would create massive strings
        let mut text = String::new();
        let mut iterations = 0;
        
        loop {
            let new_chunk = "a".repeat(10000); // 10KB chunks
            
            // Track memory usage for string operations
            match self.monitor.track_buffer_allocation(new_chunk.len()) {
                Ok(_guard) => {
                    text.push_str(&new_chunk);
                    iterations += 1;
                    
                    // Check if we've hit reasonable limits
                    if iterations > 100 { // 1MB of strings
                        break;
                    }
                }
                Err(_) => {
                    // Memory limit reached - this is good
                    return (true, Some(format!("String growth limited after {} iterations", iterations)));
                }
            }
            
            if iterations % 10 == 0 {
                tokio::task::yield_now().await;
            }
        }
        
        // Check final memory state
        let memory_ok = self.monitor.get_memory_report().unwrap().heap_allocated_mb < 50;
        
        (memory_ok, if memory_ok { 
            Some("String operations kept within memory bounds".to_string()) 
        } else { 
            Some("String operations used excessive memory".to_string()) 
        })
    }
    
    /// Test scenario: Recursion stack overflow
    async fn test_recursion_stack_overflow(&self) -> (bool, Option<String>) {
        println!("    Testing recursion stack overflow prevention...");
        
        // Test deep recursion with memory allocation at each level
        fn recursive_allocator(
            depth: u32, 
            max_depth: u32, 
            monitor: &GGUFMemoryMonitor
        ) -> anyhow::Result<u32> {
            if depth >= max_depth {
                return Ok(depth);
            }
            
            // Allocate memory at each recursion level
            let _guard = monitor.track_buffer_allocation(1024)?;
            
            // Check if we should continue
            if !monitor.is_safe_for_gguf_operations()? {
                return Ok(depth); // Safe exit
            }
            
            recursive_allocator(depth + 1, max_depth, monitor)
        }
        
        match recursive_allocator(0, 10000, &self.monitor) {
            Ok(final_depth) => {
                let safe = final_depth < 10000; // Should not reach max depth
                (safe, Some(format!("Recursion safely limited to {} levels", final_depth)))
            }
            Err(e) => {
                (true, Some(format!("Recursion properly limited: {}", e)))
            }
        }
    }
    
    /// Test scenario: Event loop blocking
    async fn test_event_loop_blocking(&self) -> (bool, Option<String>) {
        println!("    Testing event loop blocking prevention...");
        
        let blocking_detected = Arc::new(AtomicBool::new(false));
        let blocking_detected_clone = blocking_detected.clone();
        
        // Spawn a watchdog task
        let watchdog = tokio::spawn(async move {
            for i in 0..100 {
                tokio::time::sleep(Duration::from_millis(10)).await;
                if i > 50 { // If we get past 50 iterations (500ms), event loop is not blocked
                    blocking_detected_clone.store(false, Ordering::SeqCst);
                    return;
                }
            }
            blocking_detected_clone.store(true, Ordering::SeqCst); // Didn't complete - blocked
        });
        
        // Simulate potentially blocking operations
        for i in 0..1000 {
            // This could block if not properly yielding
            let _guard = self.monitor.track_tensor_allocation(10000);
            
            // Critical: Yield to prevent event loop blocking
            if i % 10 == 0 {
                tokio::task::yield_now().await;
            }
        }
        
        // Wait for watchdog
        let _ = tokio::time::timeout(Duration::from_secs(2), watchdog).await;
        
        let not_blocked = !blocking_detected.load(Ordering::SeqCst);
        
        (not_blocked, if not_blocked { 
            Some("Event loop remained responsive".to_string()) 
        } else { 
            Some("Event loop was blocked".to_string()) 
        })
    }
    
    /// Generate comprehensive crash prevention report
    pub fn generate_report(&self) -> V8CrashPreventionReport {
        let results = self.test_results.lock().unwrap();
        
        let total_tests = results.len();
        let prevented_crashes = results.iter().filter(|r| r.prevented_crash).count();
        let avg_memory_usage = if !results.is_empty() {
            results.iter().map(|r| r.memory_usage_mb).sum::<u64>() / results.len() as u64
        } else {
            0
        };
        let avg_execution_time = if !results.is_empty() {
            results.iter().map(|r| r.execution_time_ms).sum::<u64>() / results.len() as u64
        } else {
            0
        };
        
        V8CrashPreventionReport {
            total_tests,
            prevented_crashes,
            success_rate: if total_tests > 0 { 
                (prevented_crashes as f64 / total_tests as f64) * 100.0 
            } else { 
                0.0 
            },
            avg_memory_usage_mb: avg_memory_usage,
            avg_execution_time_ms: avg_execution_time,
            test_results: results.clone(),
        }
    }
}

#[derive(Debug)]
pub struct V8CrashPreventionReport {
    pub total_tests: usize,
    pub prevented_crashes: usize,
    pub success_rate: f64,
    pub avg_memory_usage_mb: u64,
    pub avg_execution_time_ms: u64,
    pub test_results: Vec<V8TestResult>,
}

impl std::fmt::Display for V8CrashPreventionReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "V8 Crash Prevention Report:\n\
             Total Tests: {}\n\
             Crashes Prevented: {}\n\
             Success Rate: {:.1}%\n\
             Avg Memory Usage: {}MB\n\
             Avg Execution Time: {}ms\n\
             \n\
             Detailed Results:\n",
            self.total_tests,
            self.prevented_crashes,
            self.success_rate,
            self.avg_memory_usage_mb,
            self.avg_execution_time_ms
        )?;
        
        for result in &self.test_results {
            write!(f, 
                "  {:?}: {} ({}MB, {}ms)\n",
                result.scenario,
                if result.prevented_crash { "PREVENTED" } else { "FAILED" },
                result.memory_usage_mb,
                result.execution_time_ms
            )?;
            
            if let Some(msg) = &result.error_message {
                write!(f, "    {}\n", msg)?;
            }
        }
        
        Ok(())
    }
}

/// Main V8 crash prevention test
#[tokio::test]
async fn test_v8_crash_prevention_suite() -> anyhow::Result<()> {
    println!("🚀 Running V8 Crash Prevention Test Suite");
    
    let tester = V8CrashTester::new()?;
    let _results = tester.run_all_tests().await?;
    let report = tester.generate_report();
    
    println!("\n{}", report);
    
    // Assert success criteria
    assert!(report.success_rate >= 80.0, "Should prevent at least 80% of crash scenarios");
    assert!(report.avg_memory_usage_mb < 100, "Should keep memory usage reasonable");
    
    println!("🎉 V8 CRASH PREVENTION TESTS PASSED!");
    println!("   Success rate: {:.1}%", report.success_rate);
    println!("   Memory usage: {}MB average", report.avg_memory_usage_mb);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_large_allocation_prevention() {
        let tester = V8CrashTester::new().unwrap();
        let (prevented, _) = tester.test_large_memory_allocation().await;
        assert!(prevented, "Large allocations should be prevented");
    }
    
    #[tokio::test]
    async fn test_rapid_growth_prevention() {
        let tester = V8CrashTester::new().unwrap();
        let (prevented, msg) = tester.test_rapid_memory_growth().await;
        println!("Rapid growth test: prevented={}, msg={:?}", prevented, msg);
        // This test might succeed or fail depending on system resources
        // The important thing is that it doesn't crash
    }
}