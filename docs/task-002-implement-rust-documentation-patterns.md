# Task 002: Implement Rust Documentation Patterns

## ⏱️ Time Estimate: 10 minutes

## 🎯 Objective
Add missing Rust documentation patterns to the indexer to fix the 0% documentation detection rate for Rust code.

## 📋 Context for AI Model
The vector system currently fails to detect Rust documentation because the language patterns in `python/indexer_universal.py` are missing Rust-specific documentation patterns (`///` and `//!`). This task implements the missing patterns following TDD principles.

**Current Problem:**
```python
'rust': {
    'extensions': ['.rs'],
    'function': r'^\s*(pub\s+)?(async\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)',
    'struct': r'^\s*(pub\s+)?struct\s+([a-zA-Z_][a-zA-Z0-9_]*)',
    'comment': r'^\s*//.*|^\s*/\*',  # Generic comments only!
    # ❌ NO DOCUMENTATION PATTERNS!
}
```

## 🔧 Technical Requirements

### Files to Modify
1. `python/indexer_universal.py` - Add Rust documentation patterns
2. `test/documentation_patterns.test.js` - Verify patterns work

### Rust Documentation Types
- `///` - Outer documentation (documents the following item)
- `//!` - Inner documentation (documents the enclosing item)
- `/** */` - Block documentation (less common)

## 📝 Implementation Steps

### Step 1: Run Failing Tests (1 minute)

```bash
# These should fail initially
npm test -- test/documentation_patterns.test.js
```

Expected failures:
- `getLanguagePatterns is not defined`
- `outer_doc pattern missing`
- `inner_doc pattern missing`

### Step 2: Add Rust Documentation Patterns (5 minutes)

**File: `python/indexer_universal.py`**

Find the `LANGUAGE_PATTERNS` dictionary and update the Rust entry:

```python
'rust': {
    'extensions': ['.rs'],
    'function': r'^\s*(pub\s+)?(async\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)',
    'struct': r'^\s*(pub\s+)?struct\s+([a-zA-Z_][a-zA-Z0-9_]*)',
    'enum': r'^\s*(pub\s+)?enum\s+([a-zA-Z_][a-zA-Z0-9_]*)',
    'impl': r'^\s*impl\s+',
    'trait': r'^\s*(pub\s+)?trait\s+([a-zA-Z_][a-zA-Z0-9_]*)',
    'comment': r'^\s*//.*|^\s*/\*',
    # ✅ FIXED: Add Rust documentation patterns
    'doc_comment': r'^\s*///.*|^\s*//!.*|^\s*/\*\*.*\*/',
    'outer_doc': r'^\s*///.*',      # Documents following item
    'inner_doc': r'^\s*//!.*',      # Documents enclosing item
    'block_doc': r'^\s*/\*\*.*\*/', # Block documentation
},
```

### Step 3: Create Pattern Testing Function (2 minutes)

**File: `python/indexer_universal.py`**

Add a function to expose patterns for testing:

```python
def get_language_patterns(language):
    """
    Get compiled regex patterns for a specific language.
    Used by test suite to validate pattern detection.
    
    Args:
        language (str): Language name (e.g., 'rust', 'python')
        
    Returns:
        dict: Compiled regex patterns for the language
    """
    if language not in LANGUAGE_PATTERNS:
        raise ValueError(f"Unsupported language: {language}")
    
    patterns = LANGUAGE_PATTERNS[language]
    compiled_patterns = {}
    
    for pattern_name, pattern_regex in patterns.items():
        if isinstance(pattern_regex, str):
            try:
                compiled_patterns[pattern_name] = re.compile(pattern_regex)
            except re.error as e:
                print(f"Warning: Invalid regex pattern '{pattern_name}' for {language}: {e}")
                compiled_patterns[pattern_name] = None
        else:
            compiled_patterns[pattern_name] = pattern_regex
    
    return compiled_patterns
```

### Step 4: Update Test Helper Functions (2 minutes)

**File: `test/helpers/test_utils.js`** (create if doesn't exist)

```javascript
const { spawn } = require('child_process');
const path = require('path');

/**
 * Get language patterns by calling Python function
 */
async function getLanguagePatterns(language) {
  const pythonScript = path.join(__dirname, '..', '..', 'python', 'indexer_universal.py');
  
  return new Promise((resolve, reject) => {
    const child = spawn('python', ['-c', `
import sys
sys.path.append('${path.dirname(pythonScript)}')
from indexer_universal import get_language_patterns
import json

try:
    patterns = get_language_patterns('${language}')
    # Convert regex objects to testable format
    result = {}
    for name, pattern in patterns.items():
        if pattern:
            result[name] = {'pattern': pattern.pattern}
        else:
            result[name] = None
    print(json.dumps(result))
except Exception as e:
    print(json.dumps({'error': str(e)}))
`]);

    let stdout = '';
    let stderr = '';
    
    child.stdout.on('data', (data) => stdout += data.toString());
    child.stderr.on('data', (data) => stderr += data.toString());
    
    child.on('exit', (code) => {
      if (code === 0) {
        try {
          const result = JSON.parse(stdout);
          if (result.error) {
            reject(new Error(result.error));
          } else {
            // Convert back to testable format
            const testablePatterns = {};
            for (const [name, data] of Object.entries(result)) {
              if (data && data.pattern) {
                testablePatterns[name] = {
                  test: (str) => new RegExp(data.pattern).test(str)
                };
              }
            }
            resolve(testablePatterns);
          }
        } catch (e) {
          reject(new Error(`Failed to parse Python output: ${stdout}`));
        }
      } else {
        reject(new Error(`Python script failed: ${stderr}`));
      }
    });
  });
}

module.exports = {
  getLanguagePatterns
};
```

## ✅ Success Criteria

1. **Rust documentation patterns are added**
   - `outer_doc` pattern matches `/// comments`
   - `inner_doc` pattern matches `//! comments`
   - `block_doc` pattern matches `/** comments */`

2. **Pattern testing function works**
   - `get_language_patterns('rust')` returns compiled patterns
   - Patterns can be tested from JavaScript tests

3. **Tests now pass**
   - Documentation pattern tests pass
   - No regression in existing functionality

## 🔍 Validation Commands

```bash
# Test the new patterns
npm test -- test/documentation_patterns.test.js

# Test Python function directly
cd python && python -c "
from indexer_universal import get_language_patterns
patterns = get_language_patterns('rust')
print('Outer doc test:', patterns['outer_doc'].match('/// Test doc'))
print('Inner doc test:', patterns['inner_doc'].match('//! Test doc'))
"
```

## 📊 Expected Results

- **Before**: 0/3 documentation pattern tests pass
- **After**: 3/3 documentation pattern tests pass  
- **Impact**: Enables documentation detection (fixes 0% → ~30% accuracy)

## 🚨 Quality Assurance (10 iterations)

Run these validation checks:

1. **Pattern Correctness**: Do patterns match intended documentation?
2. **No False Positives**: Do patterns avoid matching regular comments?
3. **Performance**: Are patterns efficient (no catastrophic backtracking)?
4. **Cross-platform**: Do patterns work on Windows/Linux/macOS?
5. **Edge Cases**: Handle empty docs, nested comments, etc.?

## 📁 Files Modified

1. `python/indexer_universal.py` - Added Rust doc patterns
2. `test/helpers/test_utils.js` - Added pattern testing utilities

## ➡️ Next Task
Task 003: Fix Documentation Extraction Logic