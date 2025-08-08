use std::fs::{File, metadata};
use std::io::{Read, Seek, SeekFrom, BufReader};
use std::path::Path;

/// GGUF magic bytes: "GGUF" in ASCII
const GGUF_MAGIC: &[u8] = b"GGUF";

/// Expected file size in bytes
const EXPECTED_SIZE: u64 = 87_431_936;

#[derive(Debug)]
struct FileIntegrityReport {
    file_path: String,
    exists: bool,
    actual_size: Option<u64>,
    size_matches: bool,
    header_bytes: Option<Vec<u8>>,
    has_gguf_magic: bool,
    can_memory_map: bool,
    error_details: Option<String>,
}

fn test_file_integrity(file_path: &str) -> FileIntegrityReport {
    let path = Path::new(file_path);
    
    // Check if file exists
    if !path.exists() {
        return FileIntegrityReport {
            file_path: file_path.to_string(),
            exists: false,
            actual_size: None,
            size_matches: false,
            header_bytes: None,
            has_gguf_magic: false,
            can_memory_map: false,
            error_details: Some("File does not exist".to_string()),
        };
    }

    // Get file size
    let actual_size = match metadata(path) {
        Ok(meta) => meta.len(),
        Err(e) => {
            return FileIntegrityReport {
                file_path: file_path.to_string(),
                exists: true,
                actual_size: None,
                size_matches: false,
                header_bytes: None,
                has_gguf_magic: false,
                can_memory_map: false,
                error_details: Some(format!("Cannot read metadata: {}", e)),
            };
        }
    };

    let size_matches = actual_size == EXPECTED_SIZE;

    // Try to read header bytes
    let mut header_result = None;
    let mut has_magic = false;
    let mut can_mmap = false;
    let mut error_msg = None;

    match File::open(path) {
        Ok(mut file) => {
            // Try to read first 16 bytes for header inspection
            let mut header = vec![0u8; 16];
            match file.read_exact(&mut header) {
                Ok(_) => {
                    header_result = Some(header.clone());
                    has_magic = header.starts_with(GGUF_MAGIC);
                    
                    // Test memory mapping capability by seeking to end and back
                    match file.seek(SeekFrom::End(0)) {
                        Ok(_) => {
                            match file.seek(SeekFrom::Start(0)) {
                                Ok(_) => can_mmap = true,
                                Err(e) => error_msg = Some(format!("Seek error: {}", e)),
                            }
                        }
                        Err(e) => error_msg = Some(format!("Cannot seek to end: {}", e)),
                    }
                }
                Err(e) => {
                    error_msg = Some(format!("Cannot read header: {}", e));
                }
            }
        }
        Err(e) => {
            error_msg = Some(format!("Cannot open file: {}", e));
        }
    }

    FileIntegrityReport {
        file_path: file_path.to_string(),
        exists: true,
        actual_size: Some(actual_size),
        size_matches,
        header_bytes: header_result,
        has_gguf_magic: has_magic,
        can_memory_map: can_mmap,
        error_details: error_msg,
    }
}

fn print_hex_bytes(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn verify_model_file_integrity() {
    println!("=== MODEL FILE INTEGRITY VERIFICATION ===");
    println!("Expected file size: {} bytes", EXPECTED_SIZE);
    println!("Expected GGUF magic: {:?} ({:02X?})", 
             String::from_utf8_lossy(GGUF_MAGIC), GGUF_MAGIC);
    println!();

    let file_paths = vec![
        "C:\\code\\embed\\models\\nomic-ai_nomic-embed-code-Q4_K_M.gguf",
        "C:\\code\\embed\\src\\embedding\\nomic-ai_nomic-embed-code-Q4_K_M.gguf",
    ];

    let mut all_reports = Vec::new();
    let mut verification_passed = true;

    for (index, file_path) in file_paths.iter().enumerate() {
        println!("--- FILE {} REPORT ---", index + 1);
        let report = test_file_integrity(file_path);
        
        println!("File: {}", report.file_path);
        println!("Exists: {}", report.exists);
        
        if !report.exists {
            verification_passed = false;
            println!("CRITICAL: File does not exist!");
        }
        
        if let Some(size) = report.actual_size {
            println!("Actual size: {} bytes", size);
            println!("Size matches expected: {}", report.size_matches);
            if !report.size_matches {
                verification_passed = false;
                let diff = if size > EXPECTED_SIZE {
                    format!("+{} bytes", size - EXPECTED_SIZE)
                } else {
                    format!("-{} bytes", EXPECTED_SIZE - size)
                };
                println!("SIZE MISMATCH: {}", diff);
            }
        }
        
        if let Some(ref header) = report.header_bytes {
            println!("Header bytes (16): {}", print_hex_bytes(header));
            println!("First 4 bytes as ASCII: {:?}", 
                     String::from_utf8_lossy(&header[0..4.min(header.len())]));
        }
        
        println!("Has GGUF magic: {}", report.has_gguf_magic);
        if !report.has_gguf_magic {
            verification_passed = false;
            println!("CRITICAL: Missing GGUF magic bytes!");
        }
        
        println!("Can memory-map: {}", report.can_memory_map);
        if !report.can_memory_map {
            verification_passed = false;
            println!("CRITICAL: Cannot memory-map file!");
        }
        
        if let Some(ref error) = report.error_details {
            verification_passed = false;
            println!("ERROR: {}", error);
        }
        
        all_reports.push(report);
        println!();
    }

    // Compare files if both exist
    if all_reports.len() == 2 && all_reports.iter().all(|r| r.exists) {
        println!("--- COMPARISON ANALYSIS ---");
        let report1 = &all_reports[0];
        let report2 = &all_reports[1];
        
        match (report1.actual_size, report2.actual_size) {
            (Some(size1), Some(size2)) => {
                if size1 == size2 {
                    println!("Both files have identical sizes: {} bytes", size1);
                } else {
                    verification_passed = false;
                    println!("CRITICAL SIZE MISMATCH:");
                    println!("  File 1: {} bytes", size1);
                    println!("  File 2: {} bytes", size2);
                    println!("  Difference: {} bytes", (size1 as i64 - size2 as i64).abs());
                }
            }
            _ => {
                verification_passed = false;
                println!("CRITICAL: Cannot compare sizes due to read errors");
            }
        }
        
        match (&report1.header_bytes, &report2.header_bytes) {
            (Some(h1), Some(h2)) => {
                if h1 == h2 {
                    println!("Headers are identical");
                } else {
                    verification_passed = false;
                    println!("CRITICAL HEADER MISMATCH:");
                    println!("  File 1 header: {}", print_hex_bytes(h1));
                    println!("  File 2 header: {}", print_hex_bytes(h2));
                }
            }
            _ => {
                verification_passed = false;
                println!("CRITICAL: Cannot compare headers due to read errors");
            }
        }
    }

    println!("=== VERIFICATION COMPLETE ===");
    
    if verification_passed {
        println!("RESULT: ALL CHECKS PASSED");
    } else {
        println!("RESULT: VERIFICATION FAILED - SEE ERRORS ABOVE");
        panic!("Model file integrity verification failed!");
    }
}