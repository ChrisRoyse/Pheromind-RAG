# Task 001: Setup Test Infrastructure for Vector System Repair

## ⏱️ Time Estimate: 10 minutes

## 🎯 Objective
Create comprehensive test infrastructure to validate embedding system fixes using Test-Driven Development (TDD) approach.

## 📋 Context for AI Model
You are working on an MCP RAG Indexer project that has a broken vector embedding system. The current system has 0% documentation detection accuracy when it should be ~65%. You need to create tests that will initially FAIL and then guide the implementation of fixes.

The project structure:
- `python/` - Contains the Python MCP server components
- `test/` - Contains JavaScript test files
- `docs/` - Task files (current directory)

## 🔧 Technical Requirements

### Test Files to Create

1. **Create `test/vector_system.test.js`**
   - Integration tests for the complete vector system
   - Tests for Rust documentation detection (currently failing)
   - Tests for chunking accuracy
   - Performance benchmarks

2. **Create `test/documentation_patterns.test.js`** 
   - Unit tests for language-specific documentation patterns
   - Tests for Rust `///` and `//!` patterns
   - Tests for Python docstrings
   - Tests for JavaScript JSDoc

3. **Create `test/chunking_algorithm.test.js`**
   - Tests for smart chunking that preserves doc-code relationships
   - Tests for backward-looking documentation detection
   - Tests for chunk boundary correctness

## 📝 Implementation Steps

### Step 1: Create Vector System Integration Tests (4 minutes)

```javascript
// test/vector_system.test.js
const path = require('path');
const fs = require('fs').promises;
const { spawn } = require('child_process');

describe('Vector System Integration Tests (TDD)', () => {
  describe('Documentation Detection Accuracy', () => {
    test('MUST FAIL: Should detect Rust struct documentation', async () => {
      // Create test Rust file with /// documentation
      const rustCode = `
/// A spiking cortical column with TTFS dynamics.
/// This struct represents a biologically-inspired cortical column.
pub struct SpikingCorticalColumn {
    activation_level: f64,
}`;
      
      // Index the code
      const result = await indexCode(rustCode, 'rust');
      
      // Should find documentation (WILL FAIL INITIALLY)
      expect(result.chunks[0].has_documentation).toBe(true);
      expect(result.chunks[0].content).toContain('/// A spiking cortical column');
    });

    test('MUST FAIL: Should achieve 65%+ documentation coverage on real codebase', async () => {
      // Test on actual Rust codebase
      const coverage = await calculateDocumentationCoverage('./test_rust_project');
      expect(coverage.percentage).toBeGreaterThan(65);
    });
  });
});
```

### Step 2: Create Documentation Pattern Tests (3 minutes)

```javascript
// test/documentation_patterns.test.js
describe('Documentation Pattern Detection (TDD)', () => {
  test('MUST FAIL: Should detect Rust outer documentation (///)', () => {
    const patterns = getLanguagePatterns('rust');
    const testLine = '/// This is outer documentation';
    
    expect(patterns.outer_doc.test(testLine)).toBe(true);
  });

  test('MUST FAIL: Should detect Rust inner documentation (//!)', () => {
    const patterns = getLanguagePatterns('rust');
    const testLine = '//! This is inner documentation';
    
    expect(patterns.inner_doc.test(testLine)).toBe(true);
  });
});
```

### Step 3: Create Chunking Algorithm Tests (3 minutes)

```javascript
// test/chunking_algorithm.test.js
describe('Smart Chunking Algorithm (TDD)', () => {
  test('MUST FAIL: Should include documentation in code chunks', async () => {
    const code = `
/// Documentation for function
pub fn example() -> i32 {
    42
}`;
    
    const chunks = await createChunks(code, 'rust');
    const chunk = chunks[0];
    
    // Chunk should start with documentation, not declaration
    expect(chunk.content.startsWith('/// Documentation')).toBe(true);
    expect(chunk.line_start).toBe(2); // Start from doc line, not declaration
  });
});
```

## ✅ Success Criteria

1. **All tests are created and initially FAIL**
   - Vector system tests fail with 0% documentation detection
   - Pattern tests fail due to missing Rust patterns
   - Chunking tests fail due to broken boundary detection

2. **Test infrastructure is complete**
   - Helper functions for code indexing
   - Test data creation utilities
   - Performance measurement tools

3. **Tests provide clear feedback**
   - Specific error messages about what's broken
   - Expected vs actual results clearly shown
   - Guidance for what needs to be implemented

## 🔍 Validation Commands

```bash
# Run the new tests (should all fail initially)
npm test -- test/vector_system.test.js
npm test -- test/documentation_patterns.test.js  
npm test -- test/chunking_algorithm.test.js

# Check test coverage
npm run test:coverage
```

## 📊 Expected Results

- **Before Implementation**: 0/15 tests passing (0%)
- **After Task Completion**: 0/15 tests passing (0%) - but with proper test infrastructure
- **After Full Fix Implementation**: 15/15 tests passing (100%)

## 🚨 TDD Principle Compliance

- ✅ Write tests first (Red phase)
- ⏸️ Implementation comes in later tasks (Green phase)
- ⏸️ Refactoring comes in final tasks (Refactor phase)

## 📁 Files Created

1. `test/vector_system.test.js` - Integration tests
2. `test/documentation_patterns.test.js` - Pattern unit tests  
3. `test/chunking_algorithm.test.js` - Chunking tests
4. `test/helpers/test_utils.js` - Test utilities

## ➡️ Next Task
Task 002: Implement Multi-Language Documentation Patterns