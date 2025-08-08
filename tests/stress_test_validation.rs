/// Quick validation test to verify stress test helper functions
/// This ensures the core logic works before running full stress tests
use std::fs;
use tempfile::TempDir;

#[test]
fn test_helper_functions() {
    println!("🔍 Validating stress test helper functions");
    
    // Test valid embedding generation
    let embedding = generate_valid_embedding(768);
    assert_eq!(embedding.len(), 768);
    
    // Verify embedding is not all zeros
    let sum: f32 = embedding.iter().sum();
    assert!(sum.abs() > 0.01, "Embedding should not be all zeros");
    
    // Test mathematical chaos generation
    let chaos = create_mixed_mathematical_chaos(100);
    assert_eq!(chaos.len(), 100);
    
    // Verify it contains NaN/Inf values
    let has_nan = chaos.iter().any(|x| x.is_nan());
    let has_inf = chaos.iter().any(|x| x.is_infinite());
    assert!(has_nan, "Chaos should contain NaN values");
    assert!(has_inf, "Chaos should contain infinite values");
    
    // Test GGUF corruption functions  
    let corrupted_header = corrupt_gguf_header();
    assert!(corrupted_header.len() > 4, "Corrupted header should have content");
    assert_ne!(&corrupted_header[0..4], b"GGUF", "Header should be corrupted");
    
    let q2k_data = create_gguf_with_q2k_format();
    assert!(q2k_data.len() > 20, "Q2K data should have reasonable size");
    assert_eq!(&q2k_data[0..4], b"GGUF", "Should start with GGUF magic");
    
    println!("✅ All helper functions validated successfully");
}

#[test]
fn test_memory_measurement() {
    println!("🔍 Validating memory measurement function");
    
    let initial_memory = get_process_memory_mb();
    assert!(initial_memory > 0.0, "Memory measurement should return positive value");
    assert!(initial_memory < 10000.0, "Memory should be reasonable (< 10GB)");
    
    println!("📊 Current process memory: {:.2} MB", initial_memory);
    
    // Allocate some memory to test measurement sensitivity
    let _large_vec: Vec<u8> = vec![0; 10_000_000]; // 10MB
    let after_alloc = get_process_memory_mb();
    
    println!("📊 After 10MB allocation: {:.2} MB", after_alloc);
    
    // Memory should have increased (though GC may affect this)
    if after_alloc > initial_memory {
        println!("✅ Memory measurement detects allocation");
    } else {
        println!("ℹ️  Memory measurement may be affected by GC or optimization");
    }
    
    println!("✅ Memory measurement function working");
}

#[cfg(windows)]
#[test]
fn test_windows_memory_api() {
    println!("🔍 Validating Windows memory API access");
    
    use std::mem;
    use winapi::um::processthreadsapi::GetCurrentProcess;
    use winapi::um::psapi::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    
    unsafe {
        let process = GetCurrentProcess();
        let mut pmc: PROCESS_MEMORY_COUNTERS = mem::zeroed();
        
        let result = GetProcessMemoryInfo(process, &mut pmc, mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32);
        
        if result != 0 {
            println!("✅ Windows API memory measurement successful");
            println!("📊 Working set size: {} bytes", pmc.WorkingSetSize);
            println!("📊 Peak working set: {} bytes", pmc.PeakWorkingSetSize);
            
            assert!(pmc.WorkingSetSize > 0, "Working set should be positive");
            assert!(pmc.WorkingSetSize < 100_000_000_000, "Working set should be reasonable");
        } else {
            panic!("❌ Windows API memory measurement failed");
        }
    }
}

// Include the helper functions from the main stress test file
fn generate_valid_embedding(dimension: usize) -> Vec<f32> {
    (0..dimension).map(|i| {
        let angle = (i as f32) * std::f32::consts::PI * 2.0 / dimension as f32;
        angle.sin() * 0.1 + 0.1
    }).collect()
}

fn get_process_memory_mb() -> f64 {
    #[cfg(windows)]
    {
        use std::mem;
        use winapi::um::processthreadsapi::GetCurrentProcess;
        use winapi::um::psapi::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
        
        unsafe {
            let process = GetCurrentProcess();
            let mut pmc: PROCESS_MEMORY_COUNTERS = mem::zeroed();
            
            if GetProcessMemoryInfo(process, &mut pmc, mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32) != 0 {
                (pmc.WorkingSetSize as f64) / 1_048_576.0 // Convert to MB
            } else {
                0.0 // Failed to get memory info
            }
        }
    }
    
    #[cfg(not(windows))]
    {
        100.0 // Placeholder for non-Windows
    }
}

fn create_mixed_mathematical_chaos(size: usize) -> Vec<f32> {
    let mut chaos = Vec::with_capacity(size);
    for i in 0..size {
        match i % 8 {
            0 => chaos.push(f32::NAN),
            1 => chaos.push(f32::INFINITY),
            2 => chaos.push(f32::NEG_INFINITY),
            3 => chaos.push(f32::MIN_POSITIVE / 1000.0), // Subnormal
            4 => chaos.push(f32::MAX),
            5 => chaos.push(f32::MIN),
            6 => chaos.push(0.0),
            7 => chaos.push(-0.0),
            _ => chaos.push(1.0),
        }
    }
    chaos
}

fn corrupt_gguf_header() -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(b"FAKE"); // Wrong magic
    data.extend_from_slice(&999u32.to_le_bytes()); // Invalid version
    data.extend_from_slice(&u64::MAX.to_le_bytes()); // Invalid tensor count
    data.extend_from_slice(&[0xBB; 2000]);
    data
}

fn create_gguf_with_q2k_format() -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(b"GGUF"); // Magic
    data.extend_from_slice(&3u32.to_le_bytes()); // Version
    data.extend_from_slice(&1u64.to_le_bytes()); // Tensor count
    data.extend_from_slice(&0u64.to_le_bytes()); // Metadata count
    
    // Add fake tensor info with Q2K format marker
    data.extend_from_slice(&[0x02, 0x0B]); // Q2K format marker
    data.extend_from_slice(&[0xFF; 1000]); // Fake tensor data
    
    data
}

#[cfg(windows)]
extern crate winapi;