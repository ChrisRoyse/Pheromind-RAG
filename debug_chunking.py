from indexer_universal import UniversalCodeIndexer

indexer = UniversalCodeIndexer()

# Test exact same code as the failing test
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

# Check the specific failing pattern
functionNames = ['function_one', 'function_two', 'function_three']

for i, (funcName, chunk) in enumerate(zip(functionNames, chunks)):
    print(f'Checking {funcName} chunk (chunk {i}):')
    chunk_content = chunk['content']
    print(f'  Contains {funcName}: {funcName in chunk_content}')
    
    # Check for other functions (this is what's failing)
    otherFunctions = [name for name in functionNames if name != funcName]
    for otherFunc in otherFunctions:
        contains_other = f'pub fn {otherFunc}(' in chunk_content
        print(f'  Contains pub fn {otherFunc}(: {contains_other}')
        if contains_other:
            print(f'    ERROR: Should not contain {otherFunc}!')
    print()

# Debug mixed documentation styles
print("=" * 50)
print("Testing mixed documentation styles:")

mixed_code = '''
/// Proper Rust documentation
pub fn documented_function() -> i32 {
    42
}

// Regular comment, not documentation
pub fn comment_function() -> i32 {
    24
}

/** 
 * Block comment style documentation 
 */
pub fn block_documented_function() -> i32 {
    12
}'''

mixed_chunks = indexer.parse_content(mixed_code, 'rust')
print(f'Generated {len(mixed_chunks)} mixed chunks')

documented_chunks = [c for c in mixed_chunks if c.get('has_documentation', False)]
undocumented_chunks = [c for c in mixed_chunks if not c.get('has_documentation', False)]

print(f'Documented chunks: {len(documented_chunks)}')
print(f'Undocumented chunks: {len(undocumented_chunks)}')

# Check specific functions
properly_documented = []
for chunk in mixed_chunks:
    if chunk.get('has_documentation') and (
        '/// Proper Rust documentation' in chunk['content'] or
        'Block comment style documentation' in chunk['content']
    ):
        properly_documented.append(chunk)

print(f'Properly documented chunks: {len(properly_documented)}')
for i, chunk in enumerate(properly_documented):
    print(f'  Chunk {i}: {repr(chunk["content"][:60])}...')