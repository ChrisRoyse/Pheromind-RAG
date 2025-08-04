import sys
sys.path.append('python')
from indexer_universal import UniversalCodeIndexer

indexer = UniversalCodeIndexer()

# Test the boundary correctness issue
rust_code = '''
/// First function documentation
pub fn function_one() -> i32 {
    let x = 42;
    x + 1
}

/// Second function documentation  
pub fn function_two() -> String {
    "hello".to_string()
}

/// Third function documentation
pub fn function_three() -> bool {
    true
}'''

chunks = indexer.parse_content(rust_code, 'rust')
print(f'Generated {len(chunks)} chunks')

# Check the test logic - each chunk should contain only one function
function_names = ['function_one', 'function_two', 'function_three']

for i, (func_name, chunk) in enumerate(zip(function_names, chunks)):
    print(f'\nChunk {i+1} should contain only {func_name}:')
    content = chunk['content']
    
    # Check if this chunk contains the expected function
    contains_expected = f'pub fn {func_name}(' in content
    print(f'  Contains expected {func_name}: {contains_expected}')
    
    # Check if this chunk contains other functions (should not)
    other_functions = [name for name in function_names if name != func_name]
    problematic_others = []
    
    for other_func in other_functions:
        if f'pub fn {other_func}(' in content:
            problematic_others.append(other_func)
    
    if problematic_others:
        print(f'  ERROR: Also contains {problematic_others}')
        print(f'  Full content: {repr(content)}')
    else:
        print(f'  OK: Does not contain other functions')