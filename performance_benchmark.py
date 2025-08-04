import sys
import time
sys.path.append('python')
from indexer_universal import UniversalCodeIndexer

def benchmark_smart_chunking():
    """Benchmark the smart chunking performance."""
    indexer = UniversalCodeIndexer()
    
    # Create test cases of different sizes
    test_cases = [
        # Small code (single function)
        '''
/// Small function documentation
/// This function returns a constant value
pub fn small_function() -> i32 {
    42
}''',
        
        # Medium code (multiple functions)
        '\n'.join([f'''
/// Documentation for function {i}
/// This function performs operation number {i}
pub fn function_{i}() -> i32 {{
    let result = {i} * 2;
    result + 1
}}''' for i in range(20)]),
        
        # Large code (many functions)
        '\n'.join([f'''
/// Documentation for function {i}
/// This function performs operation number {i}
/// 
/// # Examples
/// 
/// ```
/// let result = function_{i}();
/// assert_eq!(result, {i * 2 + 1});
/// ```
pub fn function_{i}() -> i32 {{
    // Implementation details
    let intermediate = {i} * 2;
    let result = intermediate + 1;
    
    // Additional processing
    if result > 100 {{
        result - 50
    }} else {{
        result
    }}
}}''' for i in range(100)])
    ]
    
    test_names = ['Small (1 function)', 'Medium (20 functions)', 'Large (100 functions)']
    
    print("=== Smart Chunking Performance Benchmark ===\n")
    
    total_time = 0
    for name, code in zip(test_names, test_cases):
        print(f"Testing {name}:")
        
        # Warm up
        indexer.parse_content(code, 'rust')
        
        # Benchmark
        start_time = time.time()
        chunks = indexer.parse_content(code, 'rust')
        end_time = time.time()
        
        duration_ms = (end_time - start_time) * 1000
        total_time += duration_ms
        
        # Calculate metrics
        metrics = indexer.calculate_chunking_metrics(chunks)
        
        print(f"  Duration: {duration_ms:.1f}ms")
        print(f"  Chunks created: {len(chunks)}")
        print(f"  Documentation coverage: {metrics['documentation_coverage']:.1%}")
        print(f"  Average chunk size: {metrics['avg_chunk_size']} chars")
        print(f"  Average confidence: {metrics['avg_confidence']:.3f}")
        print()
    
    print(f"Total benchmark time: {total_time:.1f}ms")
    print(f"Performance target (<500ms): {'PASSED' if total_time < 500 else 'FAILED'}")
    
    return total_time < 500

def test_documentation_coverage():
    """Test documentation detection accuracy."""
    indexer = UniversalCodeIndexer()
    
    # Test with various documentation styles
    rust_code = '''
//! Module-level documentation
//! This module provides advanced functionality

/// Standard function documentation
/// This follows Rust conventions
pub fn standard_function() -> i32 {
    42
}

/**
 * Block comment documentation
 * This is also valid documentation
 */
pub fn block_function() -> i32 {
    24
}

// Regular comment, not documentation
pub fn undocumented_function() -> i32 {
    12
}

/// Another documented function
/// With multiple lines of documentation
/// And examples
pub fn another_function() -> String {
    "hello".to_string()
}'''
    
    chunks = indexer.parse_content(rust_code, 'rust')
    metrics = indexer.calculate_chunking_metrics(chunks)
    
    print("=== Documentation Coverage Test ===\n")
    print(f"Total chunks: {len(chunks)}")
    print(f"Documented chunks: {metrics['documented_chunks']}")
    print(f"Documentation coverage: {metrics['documentation_coverage']:.1%}")
    print(f"Average confidence: {metrics['avg_confidence']:.3f}")
    
    # Analyze by function
    functions = ['standard_function', 'block_function', 'undocumented_function', 'another_function']
    documented_functions = []
    
    for func_name in functions:
        matching_chunks = [c for c in chunks if f'pub fn {func_name}(' in c['content']]
        if matching_chunks:
            chunk = matching_chunks[0]
            has_docs = chunk.get('has_documentation', False)
            confidence = chunk.get('confidence', 0)
            print(f"  {func_name}: documented={has_docs}, confidence={confidence:.3f}")
            if has_docs:
                documented_functions.append(func_name)
    
    expected_documented = ['standard_function', 'block_function', 'another_function']
    actual_documented = documented_functions
    
    accuracy = len(set(expected_documented) & set(actual_documented)) / len(expected_documented)
    print(f"\nDocumentation detection accuracy: {accuracy:.1%}")
    print(f"Target accuracy (97%+): {'PASSED' if accuracy >= 0.97 else 'FAILED'}")
    
    return accuracy >= 0.97

if __name__ == "__main__":
    print("MCP RAG Indexer - Smart Chunking Algorithm Validation\n")
    
    # Run benchmarks
    performance_ok = benchmark_smart_chunking()
    print()
    coverage_ok = test_documentation_coverage()
    
    print("\n" + "="*50)
    print("FINAL RESULTS:")
    print(f"Performance test: {'PASSED' if performance_ok else 'FAILED'}")
    print(f"Coverage test: {'PASSED' if coverage_ok else 'FAILED'}")
    
    if performance_ok and coverage_ok:
        print("\nSmart Chunking Algorithm implementation successful!")
        print("Ready for Task 007: Implement Confidence Scoring System")
    else:
        print("\nSome tests failed - requires additional optimization")