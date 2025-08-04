/**
 * Smart Chunking Algorithm Tests (TDD - Red Phase)
 * 
 * These tests validate the intelligent code chunking system that preserves
 * documentation-code relationships. All tests are designed to FAIL initially
 * because the chunking algorithm is broken.
 * 
 * Key Requirements:
 * - Documentation should be included with associated code
 * - Chunks should start from documentation, not code declarations
 * - Backward-looking documentation detection should work
 * - Chunk boundaries should be semantically meaningful
 */

const { createChunks, indexCode } = require('./helpers/test_utils');

describe('Smart Chunking Algorithm (TDD)', () => {

  describe('Documentation-Code Relationship Preservation', () => {

    test('MUST FAIL: Should include documentation in code chunks', async () => {
      const code = `
/// Documentation for function
/// This function performs important calculations
pub fn example() -> i32 {
    42
}`;
      
      // This will fail because chunking is broken
      const chunks = await createChunks(code, 'rust');
      
      expect(chunks).toBeDefined();
      expect(chunks.length).toBeGreaterThan(0);
      
      const chunk = chunks[0];
      expect(chunk).toBeDefined();
      expect(chunk.content).toBeDefined();
      
      // Chunk should start with documentation, not declaration
      expect(chunk.content.startsWith('/// Documentation for function')).toBe(true);
      expect(chunk.content).toContain('pub fn example()');
      
      // Line numbers should reflect documentation start
      expect(chunk.line_start).toBe(2); // Start from doc line, not declaration
      expect(chunk.line_end).toBeGreaterThan(chunk.line_start);
    });

    test('MUST FAIL: Should handle multi-line documentation blocks', async () => {
      const code = `
/// Primary documentation line
/// Secondary documentation line  
/// Detailed explanation with examples
/// 
/// # Examples
/// 
/// \`\`\`
/// let result = complex_function(42);
/// assert_eq!(result, 84);
/// \`\`\`
pub fn complex_function(input: i32) -> i32 {
    input * 2
}`;

      const chunks = await createChunks(code, 'rust');
      
      expect(chunks).toBeDefined();
      expect(chunks.length).toBeGreaterThan(0);
      
      const chunk = chunks[0];
      
      // Should include entire documentation block
      expect(chunk.content).toContain('/// Primary documentation line');
      expect(chunk.content).toContain('/// # Examples');
      expect(chunk.content).toContain('let result = complex_function(42)');
      expect(chunk.content).toContain('pub fn complex_function');
      
      // Should start from first documentation line
      expect(chunk.line_start).toBe(2);
    });

    test('MUST FAIL: Should detect backward-looking documentation', async () => {
      const code = `
pub struct MyStruct {
    /// Field documentation comes after declaration
    field1: i32,
    
    /// Another field with documentation
    field2: String,
}

impl MyStruct {
    pub fn new() -> Self {
        Self { 
            field1: 0, 
            field2: String::new() 
        }
    }
    
    /// Method documentation  
    pub fn get_field1(&self) -> i32 {
        self.field1
    }
}`;

      const chunks = await createChunks(code, 'rust');
      
      expect(chunks).toBeDefined();
      expect(chunks.length).toBeGreaterThan(0);
      
      // Should find documentation for fields
      const fieldDocChunks = chunks.filter(chunk => 
        chunk.content.includes('/// Field documentation') ||
        chunk.content.includes('/// Another field with documentation')
      );
      expect(fieldDocChunks.length).toBeGreaterThan(0);
      
      // Should find method documentation
      const methodDocChunks = chunks.filter(chunk =>
        chunk.content.includes('/// Method documentation') &&
        chunk.content.includes('pub fn get_field1')
      );
      expect(methodDocChunks.length).toBeGreaterThan(0);
    });

    test('MUST FAIL: Should handle module-level documentation', async () => {
      const code = `
//! This module contains neural network utilities
//! 
//! It provides functions for:
//! - Spike timing calculations
//! - Neural column processing
//! - Time-to-first-spike encoding

use std::collections::HashMap;

/// First function with its own documentation
pub fn neural_function() -> f64 {
    0.5
}`;

      const chunks = await createChunks(code, 'rust');
      
      expect(chunks).toBeDefined();
      expect(chunks.length).toBeGreaterThan(0);
      
      // Should have a chunk that includes module documentation
      const moduleDocChunks = chunks.filter(chunk =>
        chunk.content.includes('//! This module contains') &&
        chunk.content.includes('Spike timing calculations')
      );
      expect(moduleDocChunks.length).toBeGreaterThan(0);
      
      // Should also have separate chunks for functions
      const funcDocChunks = chunks.filter(chunk =>
        chunk.content.includes('/// First function') &&
        chunk.content.includes('pub fn neural_function')
      );
      expect(funcDocChunks.length).toBeGreaterThan(0);
    });

  });

  describe('Chunk Boundary Correctness', () => {

    test('MUST FAIL: Should create semantically meaningful boundaries', async () => {
      const code = `
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
}`;

      const chunks = await createChunks(code, 'rust');
      
      expect(chunks).toBeDefined();
      expect(chunks.length).toBe(3); // Should create exactly 3 chunks
      
      // Each chunk should contain one function and its documentation
      const functionNames = ['function_one', 'function_two', 'function_three'];
      
      functionNames.forEach((funcName, index) => {
        const chunk = chunks[index];
        // Check that chunk contains the proper documentation for this function
        if (funcName === 'function_one') {
          expect(chunk.content).toContain('/// First function documentation');
        } else if (funcName === 'function_two') {
          expect(chunk.content).toContain('/// Second function documentation');
        } else if (funcName === 'function_three') {
          expect(chunk.content).toContain('/// Third function documentation');
        }
        expect(chunk.content).toContain(`pub fn ${funcName}()`);
        
        // Should not contain other functions
        const otherFunctions = functionNames.filter(name => name !== funcName);
        otherFunctions.forEach(otherFunc => {
          expect(chunk.content).not.toContain(`pub fn ${otherFunc}(`);
        });
      });
    });

    test('MUST FAIL: Should handle nested code structures correctly', async () => {
      const code = `
/// Outer struct documentation
pub struct OuterStruct {
    /// Field documentation
    field: InnerStruct,
}

/// Inner struct documentation  
pub struct InnerStruct {
    /// Inner field documentation
    value: i32,
}

impl OuterStruct {
    /// Constructor documentation
    pub fn new() -> Self {
        Self {
            field: InnerStruct::new()
        }
    }
}

impl InnerStruct {
    /// Inner constructor documentation
    pub fn new() -> Self {
        Self { value: 0 }
    }
}`;

      const chunks = await createChunks(code, 'rust');
      
      expect(chunks).toBeDefined();
      expect(chunks.length).toBeGreaterThan(0);
      
      // Should have chunks for each struct and impl block
      const outerStructChunks = chunks.filter(chunk =>
        chunk.content.includes('/// Outer struct documentation') &&
        chunk.content.includes('pub struct OuterStruct')
      );
      expect(outerStructChunks.length).toBeGreaterThan(0);
      
      const innerStructChunks = chunks.filter(chunk =>
        chunk.content.includes('/// Inner struct documentation') &&
        chunk.content.includes('pub struct InnerStruct')
      );
      expect(innerStructChunks.length).toBeGreaterThan(0);
      
      // Should have separate chunks for impl blocks
      const outerImplChunks = chunks.filter(chunk =>
        chunk.content.includes('impl OuterStruct') &&
        chunk.content.includes('/// Constructor documentation')
      );
      expect(outerImplChunks.length).toBeGreaterThan(0);
    });

    test('MUST FAIL: Should respect maximum chunk size limits', async () => {
      // Create a very large function that should be split
      const largeFunction = `
/// Documentation for large function
/// This function has a lot of code that should be chunked appropriately
pub fn large_function() -> Vec<i32> {
    let mut result = Vec::new();
    ${Array(200).fill(0).map((_, i) => `    result.push(${i});`).join('\n')}
    result
}`;

      const chunks = await createChunks(largeFunction, 'rust');
      
      expect(chunks).toBeDefined();
      expect(chunks.length).toBeGreaterThan(0);
      
      // Check that no chunk exceeds reasonable size limits
      chunks.forEach((chunk, index) => {
        expect(chunk.content.length).toBeLessThan(8000); // Max ~8KB per chunk
        expect(chunk.line_end - chunk.line_start).toBeLessThan(300); // Max ~300 lines
        
        // First chunk should include documentation
        if (index === 0) {
          expect(chunk.content).toContain('/// Documentation for large function');
        }
      });
    });

  });

  describe('Language-Specific Chunking Behavior', () => {

    test('MUST FAIL: Should handle Python class and method documentation', async () => {
      const pythonCode = `
"""
Module-level docstring explaining the neural network implementation.
This module provides classes for spiking neural networks.
"""

class SpikeProcessor:
    """
    A class for processing neural spikes using TTFS encoding.
    
    This class implements time-to-first-spike processing algorithms
    for spiking cortical columns.
    """
    
    def __init__(self, threshold=1.0):
        """
        Initialize the spike processor.
        
        Args:
            threshold (float): Firing threshold for spike detection
        """
        self.threshold = threshold
        
    def process_spike(self, voltage):
        """
        Process incoming voltage to determine spike timing.
        
        Args:
            voltage (float): Input voltage level
            
        Returns:
            float: Time to first spike in milliseconds
        """
        if voltage >= self.threshold:
            return 0.0
        return (self.threshold - voltage) * 10.0`;

      const chunks = await createChunks(pythonCode, 'python');
      
      expect(chunks).toBeDefined();
      expect(chunks.length).toBeGreaterThan(0);
      
      // Should have module-level documentation chunk
      const moduleChunks = chunks.filter(chunk =>
        chunk.content.includes('Module-level docstring') &&
        chunk.content.includes('neural network implementation')
      );
      expect(moduleChunks.length).toBeGreaterThan(0);
      
      // Should have class documentation chunk
      const classChunks = chunks.filter(chunk =>
        chunk.content.includes('A class for processing neural spikes') &&
        chunk.content.includes('class SpikeProcessor')
      );
      expect(classChunks.length).toBeGreaterThan(0);
      
      // Should have method documentation chunks
      const methodChunks = chunks.filter(chunk =>
        (chunk.content.includes('Initialize the spike processor') ||
         chunk.content.includes('Process incoming voltage')) &&
        chunk.content.includes('def ')
      );
      expect(methodChunks.length).toBeGreaterThan(0);
    });

    test('MUST FAIL: Should handle JavaScript JSDoc documentation', async () => {
      const jsCode = `
/**
 * Neural network utilities module
 * @module NeuralUtils
 */

/**
 * Represents a spiking cortical column
 * @class
 */
class CorticalColumn {
    
    /**
     * Create a cortical column
     * @param {number} threshold - Firing threshold
     * @param {number} decayRate - Activation decay rate
     */
    constructor(threshold = 1.0, decayRate = 0.1) {
        this.threshold = threshold;
        this.decayRate = decayRate;
        this.activation = 0.0;
    }
    
    /**
     * Process input and update activation
     * @param {number} input - Input current value
     * @returns {boolean} True if spike occurred
     * @example
     * const column = new CorticalColumn();
     * const spiked = column.processInput(1.5);
     */
    processInput(input) {
        this.activation += input;
        if (this.activation >= this.threshold) {
            this.activation = 0.0;
            return true;
        }
        this.activation *= (1 - this.decayRate);
        return false;
    }
}`;

      const chunks = await createChunks(jsCode, 'javascript');
      
      expect(chunks).toBeDefined();
      expect(chunks.length).toBeGreaterThan(0);
      
      // Should have module documentation chunk
      const moduleChunks = chunks.filter(chunk =>
        chunk.content.includes('@module NeuralUtils')
      );
      expect(moduleChunks.length).toBeGreaterThan(0);
      
      // Should have class documentation chunk
      const classChunks = chunks.filter(chunk =>
        chunk.content.includes('Represents a spiking cortical column') &&
        chunk.content.includes('class CorticalColumn')
      );
      expect(classChunks.length).toBeGreaterThan(0);
      
      // Should have method documentation chunks with JSDoc tags
      const methodChunks = chunks.filter(chunk =>
        chunk.content.includes('@param') &&
        chunk.content.includes('@returns')
      );
      expect(methodChunks.length).toBeGreaterThan(0);
      
      // Should include examples in chunks
      const exampleChunks = chunks.filter(chunk =>
        chunk.content.includes('@example') &&
        chunk.content.includes('const column = new CorticalColumn()')
      );
      expect(exampleChunks.length).toBeGreaterThan(0);
    });

  });

  describe('Edge Cases and Error Handling', () => {

    test('MUST FAIL: Should handle code with mixed documentation styles', async () => {
      const mixedCode = `
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
}`;

      const chunks = await createChunks(mixedCode, 'rust');
      
      expect(chunks).toBeDefined();
      expect(chunks.length).toBeGreaterThan(0);
      
      // Should properly categorize which chunks have documentation
      const documentedChunks = chunks.filter(chunk => chunk.has_documentation);
      const undocumentedChunks = chunks.filter(chunk => !chunk.has_documentation);
      
      expect(documentedChunks.length).toBeGreaterThan(0);
      expect(undocumentedChunks.length).toBeGreaterThan(0);
      
      // Should find the properly documented functions
      const properlyDocumented = chunks.filter(chunk =>
        chunk.has_documentation && (
          chunk.content.includes('/// Proper Rust documentation') ||
          chunk.content.includes('Block comment style documentation')
        )
      );
      expect(properlyDocumented.length).toBe(2);
    });

    test('MUST FAIL: Should handle empty lines and whitespace correctly', async () => {
      const codeWithWhitespace = `

/// Function documentation with surrounding whitespace


pub fn function_with_whitespace() -> i32 {

    let x = 42;
    
    
    x + 1

}


/// Another function
pub fn another_function() -> String {
    "test".to_string()
}

`;

      const chunks = await createChunks(codeWithWhitespace, 'rust');
      
      expect(chunks).toBeDefined();
      expect(chunks.length).toBeGreaterThan(0);
      
      // Should handle whitespace without creating empty chunks
      chunks.forEach(chunk => {
        expect(chunk.content.trim().length).toBeGreaterThan(0);
        expect(chunk.line_start).toBeLessThanOrEqual(chunk.line_end);
      });
      
      // Should still detect documentation despite whitespace
      const documentedChunks = chunks.filter(chunk => chunk.has_documentation);
      expect(documentedChunks.length).toBe(2);
    });

    test('MUST FAIL: Should handle files with only documentation', async () => {
      const onlyDocs = `
//! This file contains only documentation
//! 
//! It serves as a reference for understanding
//! the overall architecture of the system

/// This is a function signature that might be implemented elsewhere
/// 
/// # Arguments
/// 
/// * \`input\` - The input value to process

/// Another documentation block
/// This one describes a different concept`;

      const chunks = await createChunks(onlyDocs, 'rust');
      
      expect(chunks).toBeDefined();
      expect(chunks.length).toBeGreaterThan(0);
      
      // All chunks should be marked as having documentation
      chunks.forEach(chunk => {
        expect(chunk.has_documentation).toBe(true);
      });
      
      // Should find module documentation
      const moduleDocChunks = chunks.filter(chunk =>
        chunk.content.includes('//! This file contains only documentation')
      );
      expect(moduleDocChunks.length).toBeGreaterThan(0);
    });

    test('MUST FAIL: Should handle malformed or incomplete code gracefully', async () => {
      const malformedCode = `
/// This documentation is for an incomplete function
pub fn incomplete_function(

/// This documentation has no following code

struct MissingFields {
    /// Field documentation but no field

/// Orphaned documentation at end of file`;

      const chunks = await createChunks(malformedCode, 'rust');
      
      // Should not throw errors, even with malformed code
      expect(chunks).toBeDefined();
      
      // Should still attempt to create meaningful chunks
      if (chunks.length > 0) {
        chunks.forEach(chunk => {
          expect(chunk.content).toBeDefined();
          expect(typeof chunk.line_start).toBe('number');
          expect(typeof chunk.line_end).toBe('number');
        });
      }
    });

  });

  describe('Performance and Scalability', () => {

    test('MUST FAIL: Should chunk large files efficiently', async () => {
      // Generate large file with many functions
      const largeFunctions = Array(100).fill(0).map((_, i) => `
/// Documentation for function ${i}
/// This function performs operation number ${i}
pub fn function_${i}() -> i32 {
    // Implementation details
    let result = ${i} * 2;
    result + 1
}`).join('\n\n');

      const startTime = process.hrtime.bigint();
      const chunks = await createChunks(largeFunctions, 'rust');
      const endTime = process.hrtime.bigint();
      
      const durationMs = Number(endTime - startTime) / 1000000;
      
      // Should complete within reasonable time
      expect(durationMs).toBeLessThan(5000); // 5 seconds max
      
      // Should create appropriate number of chunks
      expect(chunks).toBeDefined();
      expect(chunks.length).toBeGreaterThan(50); // Should create many chunks
      
      // Most chunks should have documentation
      const documentedChunks = chunks.filter(chunk => chunk.has_documentation);
      expect(documentedChunks.length).toBeGreaterThan(80); // Most should be documented
    });

    test('MUST FAIL: Should maintain chunk quality with large documentation blocks', async () => {
      const largeDocBlock = `
/// ${'A'.repeat(5000)}
/// 
/// This is a very large documentation block that tests the system's
/// ability to handle substantial amounts of documentation text.
/// 
/// ${'B'.repeat(3000)}
/// 
/// # Examples
/// 
/// ${'C'.repeat(2000)}
pub fn function_with_large_docs() -> String {
    "test".to_string()
}`;

      const chunks = await createChunks(largeDocBlock, 'rust');
      
      expect(chunks).toBeDefined();
      expect(chunks.length).toBeGreaterThan(0);
      
      // Should handle large documentation without corruption
      const chunk = chunks[0];
      expect(chunk.content).toContain('A'.repeat(100)); // Should have large content
      expect(chunk.content).toContain('pub fn function_with_large_docs');
      expect(chunk.has_documentation).toBe(true);
    });

  });

});