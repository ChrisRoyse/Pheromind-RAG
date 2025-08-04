import sys
sys.path.append('python')
from indexer_universal import UniversalCodeIndexer

indexer = UniversalCodeIndexer()

# Test mixed documentation styles
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

chunks = indexer.parse_content(mixed_code, 'rust')
print(f'Generated {len(chunks)} mixed chunks')

for i, chunk in enumerate(chunks):
    print(f'Chunk {i+1}:')
    print(f'  Has docs: {chunk.get("has_documentation", False)}')
    content_preview = chunk['content'].replace('\n', '\\n')[:100]
    print(f'  Content preview: {content_preview}...')
    print()

# Check for properly documented functions (should be 2)
properly_documented = []
for chunk in chunks:
    if chunk.get('has_documentation') and (
        '/// Proper Rust documentation' in chunk['content'] or
        'Block comment style documentation' in chunk['content']
    ):
        properly_documented.append(chunk)

print(f'Properly documented chunks: {len(properly_documented)} (expected 2)')

# Debug individual function detection
print("\nDetailed analysis:")
all_functions = ['documented_function', 'comment_function', 'block_documented_function']
for func_name in all_functions:
    matching_chunks = [c for c in chunks if f'pub fn {func_name}(' in c['content']]
    if matching_chunks:
        chunk = matching_chunks[0]
        has_docs = chunk.get('has_documentation', False)
        print(f'{func_name}: has_documentation = {has_docs}')
    else:
        print(f'{func_name}: not found in any chunk')