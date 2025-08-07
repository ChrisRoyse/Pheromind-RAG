use embed_search::chunking::{SimpleRegexChunker, Chunk};

#[test]
fn test_empty_file() {
    let chunker = SimpleRegexChunker::new().expect("Chunker creation must succeed in test");
    let content = "";
    let chunks = chunker.chunk_file(content);
    
    assert!(chunks.is_empty(), "Empty file should produce no chunks");
}

#[test]
fn test_single_line_file() {
    let chunker = SimpleRegexChunker::new().expect("Chunker creation must succeed in test");
    let content = "single line content";
    let chunks = chunker.chunk_file(content);
    
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].content, "single line content");
    assert_eq!(chunks[0].start_line, 0);
    assert_eq!(chunks[0].end_line, 0);
}

#[test]
fn test_whitespace_only_file() {
    let chunker = SimpleRegexChunker::new().expect("Chunker creation must succeed in test");
    let content = "   \n\n   \n";
    let chunks = chunker.chunk_file(content);
    
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].start_line, 0);
    assert_eq!(chunks[0].end_line, 2);
}

#[test]
fn test_no_chunk_boundaries() {
    let chunker = SimpleRegexChunker::new().expect("Chunker creation must succeed in test");
    let mut content = String::new();
    for i in 0..50 {
        content.push_str(&format!("// Comment line {}\n", i));
    }
    
    let chunks = chunker.chunk_file(&content);
    assert_eq!(chunks.len(), 1, "Without boundaries, should create single chunk under size limit");
    assert_eq!(chunks[0].start_line, 0);
    assert_eq!(chunks[0].end_line, 49);
}

#[test]
fn test_line_continuity_validation() {
    let chunker = SimpleRegexChunker::new().expect("Chunker creation must succeed in test");
    let content = r#"line0
fn func1() {
    line2
}
line4
fn func2() {
    line6
}
line8"#;
    
    let chunks = chunker.chunk_file(content);
    
    // Validate no gaps in line coverage
    let mut covered_lines = vec![false; content.lines().count()];
    for chunk in &chunks {
        for line_num in chunk.start_line..=chunk.end_line {
            assert!(!covered_lines[line_num], "Line {} covered multiple times", line_num);
            covered_lines[line_num] = true;
        }
    }
    
    for (i, covered) in covered_lines.iter().enumerate() {
        assert!(covered, "Line {} not covered by any chunk", i);
    }
}

#[test]
fn test_chunk_content_matches_lines() {
    let chunker = SimpleRegexChunker::new().expect("Chunker creation must succeed in test");
    let content = r#"header
fn test() {
    body1
    body2
}
footer"#;
    
    let lines: Vec<&str> = content.lines().collect();
    let chunks = chunker.chunk_file(content);
    
    for chunk in &chunks {
        let chunk_lines: Vec<&str> = chunk.content.lines().collect();
        let expected_lines = &lines[chunk.start_line..=chunk.end_line];
        
        assert_eq!(chunk_lines.len(), expected_lines.len(), 
                   "Chunk line count mismatch");
        
        for (i, (actual, expected)) in chunk_lines.iter().zip(expected_lines.iter()).enumerate() {
            assert_eq!(actual, expected, 
                      "Line {} in chunk doesn't match source", i);
        }
    }
}

#[test]
fn test_boundary_at_file_start() {
    let chunker = SimpleRegexChunker::new().expect("Chunker creation must succeed in test");
    let content = "fn start() {\n    content\n}\nmore content";
    let chunks = chunker.chunk_file(content);
    
    assert!(!chunks.is_empty());
    assert_eq!(chunks[0].start_line, 0);
    assert!(chunks[0].content.contains("fn start"));
}

#[test]
fn test_boundary_at_file_end() {
    let chunker = SimpleRegexChunker::new().expect("Chunker creation must succeed in test");
    let content = "some content\nfn end() {\n    final\n}";
    let chunks = chunker.chunk_file(content);
    
    assert!(chunks.len() >= 2);
    let last_chunk = chunks.last().unwrap();
    assert!(last_chunk.content.contains("fn end"));
    assert_eq!(last_chunk.end_line, 3);
}

#[test]
fn test_consecutive_boundaries() {
    let chunker = SimpleRegexChunker::new().expect("Chunker creation must succeed in test");
    let content = r#"fn func1() {}
fn func2() {}
fn func3() {}"#;
    
    let chunks = chunker.chunk_file(content);
    assert_eq!(chunks.len(), 3, "Each function should be its own chunk");
    
    for (i, chunk) in chunks.iter().enumerate() {
        assert!(chunk.content.contains(&format!("fn func{}", i + 1)));
    }
}

#[test]
fn test_mixed_line_endings() {
    let chunker = SimpleRegexChunker::new().expect("Chunker creation must succeed in test");
    // Mix of LF and CRLF
    let content = "line1\nfn test() {\r\n    body\r\n}\nline5";
    let chunks = chunker.chunk_file(content);
    
    // Should handle mixed line endings correctly
    assert!(!chunks.is_empty());
    let total_lines = content.lines().count();
    assert_eq!(chunks.last().unwrap().end_line, total_lines - 1);
}

// Utility function to validate chunk line tracking
fn validate_chunks_cover_all_lines(chunks: &[Chunk], total_lines: usize) -> Result<(), String> {
    if chunks.is_empty() && total_lines > 0 {
        return Err("No chunks for non-empty content".to_string());
    }
    
    // Check first chunk starts at 0
    if !chunks.is_empty() && chunks[0].start_line != 0 {
        return Err(format!("First chunk doesn't start at line 0, starts at {}", 
                          chunks[0].start_line));
    }
    
    // Check last chunk ends at last line
    if !chunks.is_empty() && chunks.last().unwrap().end_line != total_lines - 1 {
        return Err(format!("Last chunk doesn't end at last line {}, ends at {}", 
                          total_lines - 1, chunks.last().unwrap().end_line));
    }
    
    // Check no gaps or overlaps
    for i in 1..chunks.len() {
        let prev_end = chunks[i-1].end_line;
        let curr_start = chunks[i].start_line;
        
        if curr_start <= prev_end {
            return Err(format!("Chunks {} and {} overlap: {} -> {}", 
                              i-1, i, prev_end, curr_start));
        }
        
        if curr_start > prev_end + 1 {
            return Err(format!("Gap between chunks {} and {}: {} -> {}", 
                              i-1, i, prev_end, curr_start));
        }
    }
    
    Ok(())
}

#[test]
fn test_chunk_validation_utility() {
    let chunker = SimpleRegexChunker::new().expect("Chunker creation must succeed in test");
    
    // Test various scenarios
    let test_cases = vec![
        ("", 0),
        ("single line", 1),
        ("line1\nline2\nline3", 3),
        ("fn test() {\n    body\n}\nfn test2() {\n    body2\n}", 6),  // 6 lines, not 5
    ];
    
    for (content, expected_lines) in test_cases {
        let chunks = chunker.chunk_file(content);
        match validate_chunks_cover_all_lines(&chunks, expected_lines) {
            Ok(()) => {},
            Err(e) => panic!("Validation failed for '{}': {}", 
                            content.replace('\n', "\\n"), e),
        }
    }
}