# Task 003: Fix Documentation Extraction Logic

## ⏱️ Time Estimate: 10 minutes

## 🎯 Objective
Rewrite the broken documentation extraction logic to look backwards for documentation comments before code declarations, fixing the core chunking problem.

## 📋 Context for AI Model
The current indexer creates chunks starting from code declarations and never looks backwards for documentation. This breaks the documentation-code relationship:

**Current Broken Logic:**
```python
def _extract_block(self, lines, start_idx, patterns, block_type):
    # ❌ Started extraction FROM the declaration line
    block_lines = [lines[start_idx]]  # Only "pub struct MyStruct"
    
    # ❌ Never looked backwards for documentation
    # ❌ Documentation comments were completely ignored
```

**Result:** Documentation and code end up in separate chunks or documentation is missed entirely.

## 🔧 Technical Requirements

### Files to Modify
1. `python/indexer_universal.py` - Replace `_extract_block` with `_extract_block_with_docs`
2. Add backward-looking documentation detection
3. Update metadata to track documentation presence

### Core Problem to Fix
```
// Chunk 1: "/// Represents a neuromorphic memory branch"  
// Chunk 2: "pub struct NeuromorphicMemoryBranch {" <-- Detected as undocumented!
```

Should become:
```
// Single Chunk: "/// Represents a neuromorphic memory branch\npub struct NeuromorphicMemoryBranch {"
```

## 📝 Implementation Steps

### Step 1: Run Failing Tests (1 minute)

```bash
# These should fail due to broken extraction logic
npm test -- test/chunking_algorithm.test.js
```

Expected failures:
- Chunks don't include documentation  
- `line_start` points to declaration, not documentation
- `has_documentation` metadata missing

### Step 2: Replace Documentation Extraction Function (6 minutes)

**File: `python/indexer_universal.py`**

Find the `_extract_block` method and replace with:

```python
def _extract_block_with_docs(self, lines, start_idx, patterns, block_type, language):
    """
    Extract a code block WITH its preceding documentation.
    
    This is the core fix for the vector system - looks backwards for docs.
    
    Args:
        lines (list): Source code lines
        start_idx (int): Index of the declaration line (e.g. "pub struct")
        patterns (dict): Language-specific regex patterns
        block_type (str): Type of block ('function', 'struct', etc.)
        language (str): Programming language
        
    Returns:
        dict: Block with content, metadata, and documentation status
    """
    # ✅ STEP 1: Find documentation BEFORE the declaration
    doc_start_idx = start_idx
    doc_lines = []
    has_documentation = False

    if language == 'rust':
        # Look backwards for /// or //! comments
        check_idx = start_idx - 1
        while check_idx >= 0:
            line = lines[check_idx].strip()
            if not line:  # Skip empty lines
                check_idx -= 1
                continue
            elif line.startswith('///') or line.startswith('//!'):
                doc_lines.insert(0, lines[check_idx])
                has_documentation = True
                doc_start_idx = check_idx
                check_idx -= 1
            else:
                break  # Stop at first non-doc line

    elif language == 'python':
        # Look for docstrings AFTER the declaration
        # Python docstrings come after def/class, not before
        next_idx = start_idx + 1
        if next_idx < len(lines):
            next_line = lines[next_idx].strip()
            if next_line.startswith('"""') or next_line.startswith("'''"):
                has_documentation = True
                # Note: Python docstrings are handled differently
                
    elif language in ['javascript', 'typescript']:
        # Look backwards for /** JSDoc comments
        check_idx = start_idx - 1
        while check_idx >= 0:
            line = lines[check_idx].strip()
            if not line:
                check_idx -= 1
                continue
            elif line.startswith('/**') or line.startswith('*'):
                doc_lines.insert(0, lines[check_idx])
                has_documentation = True
                doc_start_idx = check_idx
                check_idx -= 1
            else:
                break

    # ✅ STEP 2: Include documentation in the chunk
    block_lines = []
    if doc_lines:
        block_lines.extend(doc_lines)  # Add documentation first
    block_lines.append(lines[start_idx])   # Then declaration

    # ✅ STEP 3: Extract rest of block normally (opening/closing braces, etc.)
    i = start_idx + 1
    brace_count = 0
    found_opening_brace = False
    
    # Count braces to find block end
    declaration_line = lines[start_idx]
    if '{' in declaration_line:
        brace_count += declaration_line.count('{')
        brace_count -= declaration_line.count('}')
        found_opening_brace = True

    while i < len(lines) and (not found_opening_brace or brace_count > 0):
        line = lines[i]
        block_lines.append(line)
        
        if not found_opening_brace and '{' in line:
            found_opening_brace = True
            
        if found_opening_brace:
            brace_count += line.count('{')
            brace_count -= line.count('}')
            
        i += 1
        
        # Safety limit
        if i - start_idx > 1000:  # Prevent infinite loops
            break

    # ✅ STEP 4: Extract name from declaration
    name = 'unknown'
    try:
        if block_type in patterns:
            match = re.search(patterns[block_type], lines[start_idx])
            if match and len(match.groups()) >= 1:
                # Get the last capture group (usually the name)
                name = match.groups()[-1]
    except Exception as e:
        print(f"Warning: Could not extract name for {block_type}: {e}")

    # ✅ STEP 5: Return block with enhanced metadata
    return {
        'content': '\n'.join(block_lines),
        'type': f'{language}_{block_type}',
        'name': name,
        'line_start': doc_start_idx,  # ✅ Include documentation in range
        'line_end': i,
        'has_documentation': has_documentation,  # ✅ Track documentation
        'metadata': {
            'language': language,
            'block_type': block_type,
            'has_documentation': has_documentation,  # ✅ Redundant tracking for safety
            'doc_lines_count': len(doc_lines),
            'total_lines': len(block_lines)
        }
    }
```

### Step 3: Update Main Extraction Caller (2 minutes)

Find where `_extract_block` is called and update:

```python
# In the main parsing loop, replace:
# block = self._extract_block(lines, i, patterns, block_type)

# With:
block = self._extract_block_with_docs(lines, i, patterns, block_type, language)
```

### Step 4: Update Chunk Metadata (1 minute)

Find the chunk creation code and ensure metadata includes documentation status:

```python
chunk = {
    'content': context + block['content'],
    'type': block['type'],
    'name': block.get('name', 'unknown'),
    'metadata': {
        'language': language,
        'has_imports': len(relevant_imports) > 0,
        'has_documentation': block.get('has_documentation', False),  # ✅ FIXED
        'line_start': block.get('line_start', 0),
        'line_end': block.get('line_end', 0),
        **block.get('metadata', {})
    }
}
```

## ✅ Success Criteria

1. **Documentation is included in chunks**
   - Rust `///` comments appear before `pub struct` in same chunk
   - Python docstrings are properly detected
   - JavaScript JSDoc comments are included

2. **Metadata is accurate**
   - `has_documentation` correctly set to `true`/`false`
   - `line_start` points to documentation start, not declaration
   - `doc_lines_count` reflects actual documentation lines

3. **Tests pass**
   - Chunking algorithm tests pass
   - No regression in existing behavior

## 🔍 Validation Commands

```bash
# Test the new extraction logic
npm test -- test/chunking_algorithm.test.js

# Test Python function directly with Rust code
cd python && python -c "
from indexer_universal import UniversalCodeIndexer
indexer = UniversalCodeIndexer()

rust_code = '''
/// A spiking cortical column with TTFS dynamics.
/// This represents a biologically-inspired column.
pub struct SpikingCorticalColumn {
    activation_level: f64,
}
'''

chunks = indexer.parse_content(rust_code, 'rust')
for chunk in chunks:
    print(f'Has docs: {chunk.get(\"metadata\", {}).get(\"has_documentation\", False)}')
    print(f'Content preview: {chunk[\"content\"][:100]}...')
"
```

## 📊 Expected Results

- **Before**: Chunks start with declarations, documentation missed
- **After**: Chunks include documentation, accurate metadata
- **Accuracy Boost**: ~30% improvement in documentation detection

## 🚨 Quality Assurance (10 iterations)

Validate these aspects:

1. **Backward Search Works**: Documentation found before declarations?
2. **Language Support**: Works for Rust, Python, JavaScript?
3. **Edge Cases**: Empty docs, nested comments, malformed code?
4. **Performance**: No infinite loops or excessive memory usage?
5. **Metadata Accuracy**: All fields correctly populated?

## 📁 Files Modified

1. `python/indexer_universal.py` - New `_extract_block_with_docs` method
2. Updated chunk metadata structure
3. Enhanced language-specific documentation handling

## ➡️ Next Task
Task 004: Implement Multi-Pass Detection System