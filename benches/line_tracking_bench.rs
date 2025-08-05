use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use embed_search::chunking::{SimpleRegexChunker, LineValidator};
use std::fs;
use std::path::Path;

fn benchmark_chunking_performance(c: &mut Criterion) {
    let chunker = SimpleRegexChunker::new();
    
    // Synthetic content of various sizes
    let test_cases = vec![
        ("small", generate_code_content(100)),
        ("medium", generate_code_content(1000)),
        ("large", generate_code_content(5000)),
        ("xlarge", generate_code_content(20000)),
    ];
    
    let mut group = c.benchmark_group("chunking_performance");
    
    for (name, content) in test_cases {
        group.bench_with_input(
            BenchmarkId::new("chunk_file", name),
            &content,
            |b, content| {
                b.iter(|| {
                    black_box(chunker.chunk_file(black_box(content)))
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_line_validation(c: &mut Criterion) {
    let chunker = SimpleRegexChunker::new();
    
    // Prepare test data
    let content = generate_code_content(1000);
    let lines: Vec<&str> = content.lines().collect();
    let chunks = chunker.chunk_file(&content);
    
    c.bench_function("line_validation", |b| {
        b.iter(|| {
            black_box(LineValidator::validate_coverage(
                black_box(&chunks),
                black_box(lines.len())
            ).unwrap());
            
            black_box(LineValidator::validate_content(
                black_box(&chunks),
                black_box(&lines)
            ).unwrap());
        })
    });
}

fn benchmark_real_files(c: &mut Criterion) {
    let chunker = SimpleRegexChunker::new();
    
    // Test with actual files from vectortest if available
    let test_files = vec![
        "vectortest/auth_service.py",
        "vectortest/user_controller.js",
        "vectortest/OrderService.java",
        "vectortest/analytics_dashboard.go",
        "vectortest/memory_cache.rs",
    ];
    
    let mut group = c.benchmark_group("real_files");
    
    for file_path in test_files {
        if Path::new(file_path).exists() {
            let content = fs::read_to_string(file_path).unwrap();
            let file_name = Path::new(file_path).file_name().unwrap().to_str().unwrap();
            
            group.bench_with_input(
                BenchmarkId::new("chunk_real_file", file_name),
                &content,
                |b, content| {
                    b.iter(|| {
                        black_box(chunker.chunk_file(black_box(content)))
                    })
                },
            );
        }
    }
    
    group.finish();
}

fn generate_code_content(lines: usize) -> String {
    let mut content = String::new();
    let mut line_count = 0;
    
    // Generate realistic code patterns
    for i in 0..(lines / 10) {
        content.push_str(&format!("class TestClass{} {{\n", i));
        line_count += 1;
        
        for j in 0..8 {
            if line_count >= lines { break; }
            content.push_str(&format!("    fn method{}() {{\n", j));
            content.push_str("        // Implementation\n");
            content.push_str("        println!(\"Hello, world!\");\n");
            content.push_str("    }\n");
            content.push_str("\n");
            line_count += 5;
        }
        
        if line_count >= lines { break; }
        content.push_str("}\n\n");
        line_count += 2;
    }
    
    // Fill remaining with simple lines
    while line_count < lines {
        content.push_str(&format!("// Comment line {}\n", line_count));
        line_count += 1;
    }
    
    content
}

criterion_group!(
    benches,
    benchmark_chunking_performance,
    benchmark_line_validation,
    benchmark_real_files
);
criterion_main!(benches);