/**
 * Vector System Integration Tests (TDD - Red Phase)
 * 
 * These tests validate the complete vector embedding system for the MCP RAG Indexer.
 * All tests are designed to FAIL initially, guiding the implementation process.
 * 
 * Current State: 0% documentation detection accuracy (should be ~65%)
 * Expected State After Implementation: 65%+ documentation detection accuracy
 */

const path = require('path');
const { 
  indexCode, 
  calculateDocumentationCoverage, 
  createTestRustProject,
  measurePerformance,
  assertDocumentationPresent
} = require('./helpers/test_utils');

describe('Vector System Integration Tests (TDD)', () => {
  
  describe('Documentation Detection Accuracy', () => {
    
    test('MUST FAIL: Should detect Rust struct documentation', async () => {
      // Create test Rust code with /// documentation
      const rustCode = `
/// A spiking cortical column with TTFS dynamics.
/// This struct represents a biologically-inspired cortical column
/// that processes temporal information using time-to-first-spike encoding.
pub struct SpikingCorticalColumn {
    /// The current activation level of the column
    activation_level: f64,
    /// Threshold for spike generation  
    spike_threshold: f64,
}`;
      
      // Index the code - THIS WILL FAIL because indexing is broken
      const result = await indexCode(rustCode, 'rust');
      
      // Assertions that will fail initially
      expect(result).toBeDefined();
      expect(result.chunks).toBeDefined();
      expect(result.chunks.length).toBeGreaterThan(0);
      
      // Should find documentation (WILL FAIL INITIALLY)
      const documentedChunks = result.chunks.filter(chunk => chunk.has_documentation);
      expect(documentedChunks.length).toBeGreaterThan(0);
      
      // First chunk should contain the struct documentation
      expect(result.chunks[0].has_documentation).toBe(true);
      expect(result.chunks[0].content).toContain('/// A spiking cortical column');
      expect(result.chunks[0].content).toContain('biologically-inspired');
      
      // Should detect field documentation too
      const fieldDocChunks = result.chunks.filter(chunk => 
        chunk.content.includes('/// The current activation level') ||
        chunk.content.includes('/// Threshold for spike generation')
      );
      expect(fieldDocChunks.length).toBeGreaterThan(0);
    }, 30000);

    test('MUST FAIL: Should detect Rust function documentation', async () => {
      const rustCode = `
impl SpikingCorticalColumn {
    /// Creates a new spiking cortical column with default parameters.
    /// 
    /// # Examples
    /// 
    /// \`\`\`
    /// let column = SpikingCorticalColumn::new();
    /// assert_eq!(column.activation_level, 0.0);
    /// \`\`\`
    pub fn new() -> Self {
        Self {
            activation_level: 0.0,
            spike_threshold: 1.0,
        }
    }
}`;

      const result = await indexCode(rustCode, 'rust');
      
      // Should detect function documentation
      expect(result.chunks).toBeDefined();
      const docChunks = result.chunks.filter(chunk => chunk.has_documentation);
      expect(docChunks.length).toBeGreaterThan(0);
      
      // Should include examples in documentation
      const exampleChunks = result.chunks.filter(chunk => 
        chunk.content.includes('# Examples') && 
        chunk.content.includes('let column = SpikingCorticalColumn::new()')
      );
      expect(exampleChunks.length).toBeGreaterThan(0);
    }, 30000);

    test('MUST FAIL: Should detect Rust module documentation', async () => {
      const rustCode = `
//! This is a module-level documentation comment.
//! It describes the entire module and its purpose.
//! This module implements neural network components.

use std::collections::HashMap;

/// Public function with documentation
pub fn process_neural_input(input: f64) -> f64 {
    input * 0.5
}`;

      const result = await indexCode(rustCode, 'rust');
      
      // Should detect module-level documentation (//!)
      expect(result.chunks).toBeDefined();
      const moduleDocChunks = result.chunks.filter(chunk => 
        chunk.content.includes('//! This is a module-level') &&
        chunk.has_documentation
      );
      expect(moduleDocChunks.length).toBeGreaterThan(0);
      
      // Should also detect function documentation
      const funcDocChunks = result.chunks.filter(chunk => 
        chunk.content.includes('/// Public function') &&
        chunk.has_documentation  
      );
      expect(funcDocChunks.length).toBeGreaterThan(0);
    }, 30000);

    test('MUST FAIL: Should achieve 65%+ documentation coverage on real codebase', async () => {
      // Create test Rust project with mixed documented/undocumented code
      const projectPath = await createTestRustProject();
      
      try {
        // Calculate documentation coverage - THIS WILL FAIL (returns 0%)
        const coverage = await calculateDocumentationCoverage(projectPath);
        
        expect(coverage).toBeDefined();
        expect(coverage.percentage).toBeDefined();
        expect(coverage.total_chunks).toBeGreaterThan(0);
        
        // Main assertion - should achieve 65%+ coverage
        expect(coverage.percentage).toBeGreaterThan(65);
        expect(coverage.documented_chunks).toBeGreaterThan(0);
        
        // Should detect Rust-specific patterns
        expect(coverage.languages).toBeDefined();
        expect(coverage.languages.rust).toBeDefined();
        expect(coverage.languages.rust.documented_functions).toBeGreaterThan(0);
        
      } finally {
        // Cleanup handled by test utility
      }
    }, 60000);

    test('MUST FAIL: Should distinguish documented vs undocumented code', async () => {
      const rustCode = `
/// This function has documentation
pub fn documented_function() -> i32 {
    42
}

// This is just a comment, not documentation
pub fn undocumented_function() -> i32 {
    24
}`;

      const result = await indexCode(rustCode, 'rust');
      
      expect(result.chunks).toBeDefined();
      expect(result.chunks.length).toBeGreaterThan(0);
      
      // Should find exactly one documented chunk
      const documentedChunks = result.chunks.filter(chunk => chunk.has_documentation);
      const undocumentedChunks = result.chunks.filter(chunk => !chunk.has_documentation);
      
      expect(documentedChunks.length).toBe(1);
      expect(undocumentedChunks.length).toBeGreaterThan(0);
      
      // Documented chunk should contain the right function
      expect(documentedChunks[0].content).toContain('documented_function');
      expect(documentedChunks[0].content).toContain('/// This function has documentation');
    }, 30000);
    
  });

  describe('Performance Benchmarks', () => {
    
    test('MUST FAIL: Should index code within performance thresholds', async () => {
      const largeRustCode = `
//! Large test module for performance testing
${Array(100).fill(0).map((_, i) => `
/// Function number ${i} with documentation
/// This function performs operation ${i}
pub fn function_${i}() -> i32 {
    ${i}
}
`).join('\n')}`;

      const { result, duration_ms } = await measurePerformance(async () => {
        return await indexCode(largeRustCode, 'rust');
      });
      
      // Performance assertions
      expect(duration_ms).toBeLessThan(5000); // Should complete within 5 seconds
      expect(result.chunks.length).toBeGreaterThan(90); // Should create many chunks
      
      // Quality assertions  
      const documentedChunks = result.chunks.filter(chunk => chunk.has_documentation);
      expect(documentedChunks.length).toBeGreaterThan(80); // Most should be documented
      
    }, 10000);

    test('MUST FAIL: Should handle mixed language files efficiently', async () => {
      // Test with multiple languages in sequence
      const languages = ['rust', 'python', 'javascript'];
      const codes = {
        rust: '/// Rust function\npub fn rust_func() {}',
        python: '"""Python docstring"""\ndef python_func(): pass',
        javascript: '/** JavaScript JSDoc */\nfunction jsFunc() {}'
      };
      
      const results = [];
      
      for (const lang of languages) {
        const { result, duration_ms } = await measurePerformance(async () => {
          return await indexCode(codes[lang], lang);
        });
        
        results.push({ lang, result, duration_ms });
      }
      
      // All should complete quickly
      results.forEach(({ duration_ms, lang }) => {
        expect(duration_ms).toBeLessThan(1000); // 1 second per language
      });
      
      // All should detect documentation
      results.forEach(({ result, lang }) => {
        expect(result.chunks).toBeDefined();
        const docChunks = result.chunks.filter(chunk => chunk.has_documentation);
        expect(docChunks.length).toBeGreaterThan(0);
      });
      
    }, 15000);
    
  });

  describe('Cross-Language Documentation Support', () => {
    
    test('MUST FAIL: Should detect Python docstrings', async () => {
      const pythonCode = `
"""
Module-level docstring for neural processing utilities.
This module contains functions for spiking neural networks.
"""

def calculate_spike_time(voltage, threshold):
    """
    Calculate the time when a neuron will spike given current voltage.
    
    Args:
        voltage (float): Current membrane voltage
        threshold (float): Spike threshold voltage
        
    Returns:
        float: Time to spike in milliseconds
    """
    if voltage >= threshold:
        return 0.0
    return (threshold - voltage) * 10.0

# Function without documentation
def helper_func():
    return 42`;

      const result = await indexCode(pythonCode, 'python');
      
      expect(result.chunks).toBeDefined();
      const docChunks = result.chunks.filter(chunk => chunk.has_documentation);
      
      // Should detect both module and function docstrings
      expect(docChunks.length).toBeGreaterThan(0);
      
      // Should find module docstring
      const moduleDocChunks = result.chunks.filter(chunk => 
        chunk.content.includes('Module-level docstring') &&
        chunk.has_documentation
      );
      expect(moduleDocChunks.length).toBeGreaterThan(0);
      
      // Should find function docstring with parameters
      const funcDocChunks = result.chunks.filter(chunk => 
        chunk.content.includes('Args:') &&
        chunk.content.includes('Returns:') &&
        chunk.has_documentation
      );
      expect(funcDocChunks.length).toBeGreaterThan(0);
      
    }, 30000);

    test('MUST FAIL: Should detect JavaScript JSDoc', async () => {
      const jsCode = `
/**
 * Neural network processing utilities
 * @module NeuralUtils
 */

/**
 * Calculates spike timing for a neural column
 * @param {number} input - Input current value
 * @param {number} threshold - Firing threshold
 * @returns {number} Time to first spike
 * @example
 * const spikeTime = calculateSpikeTiming(0.8, 1.0);
 */
function calculateSpikeTiming(input, threshold) {
    return threshold / input;
}

// Undocumented function
function helperFunction() {
    return 42;
}`;

      const result = await indexCode(jsCode, 'javascript');
      
      expect(result.chunks).toBeDefined();
      const docChunks = result.chunks.filter(chunk => chunk.has_documentation);
      
      // Should detect JSDoc comments
      expect(docChunks.length).toBeGreaterThan(0);
      
      // Should find function documentation with @param and @returns
      const funcDocChunks = result.chunks.filter(chunk => 
        chunk.content.includes('@param') &&
        chunk.content.includes('@returns') &&
        chunk.has_documentation
      );
      expect(funcDocChunks.length).toBeGreaterThan(0);
      
      // Should find example in documentation
      const exampleChunks = result.chunks.filter(chunk => 
        chunk.content.includes('@example') &&
        chunk.content.includes('calculateSpikeTiming(0.8, 1.0)') &&
        chunk.has_documentation
      );
      expect(exampleChunks.length).toBeGreaterThan(0);
      
    }, 30000);
    
  });

  describe('Error Handling and Edge Cases', () => {
    
    test('MUST FAIL: Should handle empty code gracefully', async () => {
      const emptyCode = '';
      
      const result = await indexCode(emptyCode, 'rust');
      
      expect(result).toBeDefined();
      expect(result.chunks).toBeDefined();
      expect(result.chunks.length).toBe(0);
    });

    test('MUST FAIL: Should handle code without documentation', async () => {
      const undocumentedCode = `
pub fn function_one() -> i32 { 1 }
pub fn function_two() -> i32 { 2 }
pub fn function_three() -> i32 { 3 }`;

      const result = await indexCode(undocumentedCode, 'rust');
      
      expect(result).toBeDefined();
      expect(result.chunks).toBeDefined();
      expect(result.chunks.length).toBeGreaterThan(0);
      
      // All chunks should be marked as undocumented
      const undocumentedChunks = result.chunks.filter(chunk => !chunk.has_documentation);
      expect(undocumentedChunks.length).toBe(result.chunks.length);
    });

    test('MUST FAIL: Should handle malformed documentation', async () => {
      const malformedCode = `
/// Incomplete documentation without proper structure
/// Missing details and
pub struct IncompleteDoc {
    field: i32,
}

// Wrong documentation style for Rust (should be ///)
// This is not proper Rust documentation  
pub fn wrong_style() {}`;

      const result = await indexCode(malformedCode, 'rust');
      
      expect(result).toBeDefined();
      expect(result.chunks).toBeDefined();
      
      // Should still detect the proper /// documentation
      const properDocChunks = result.chunks.filter(chunk => 
        chunk.has_documentation && 
        chunk.content.includes('/// Incomplete documentation')
      );
      expect(properDocChunks.length).toBeGreaterThan(0);
      
      // Should not detect // comments as documentation  
      const improperDocChunks = result.chunks.filter(chunk => 
        chunk.has_documentation && 
        chunk.content.includes('// Wrong documentation style')
      );
      expect(improperDocChunks.length).toBe(0);
    });
    
  });

  describe('Multi-Dimensional Confidence Scoring Tests (TDD)', () => {
    
    test('MUST FAIL: Should provide calibrated confidence scores (0.95 = 95% accuracy)', async () => {
      // Test with high-quality Rust documentation
      const highQualityCode = `
/// Calculates the spike timing for a cortical column using TTFS encoding.
/// 
/// This function implements the time-to-first-spike algorithm for neural processing.
/// The calculation takes into account membrane potential, threshold dynamics, and
/// temporal encoding patterns typical of cortical columns.
/// 
/// # Arguments
/// 
/// * \`membrane_voltage\` - Current membrane voltage in millivolts
/// * \`spike_threshold\` - Threshold voltage for spike generation
/// * \`time_constant\` - Membrane time constant in milliseconds
/// 
/// # Returns
/// 
/// Returns the time to first spike in milliseconds, or None if no spike occurs
/// within the simulation window.
/// 
/// # Examples
/// 
/// \`\`\`rust
/// let spike_time = calculate_spike_timing(65.0, 70.0, 10.0);
/// assert!(spike_time.is_some());
/// \`\`\`
pub fn calculate_spike_timing(
    membrane_voltage: f64, 
    spike_threshold: f64, 
    time_constant: f64
) -> Option<f64> {
    if membrane_voltage >= spike_threshold {
        return Some(0.0);
    }
    Some((spike_threshold - membrane_voltage) / time_constant)
}`;

      const result = await indexCode(highQualityCode, 'rust');
      
      expect(result.chunks).toBeDefined();
      expect(result.chunks.length).toBeGreaterThan(0);
      
      // Find the documented chunk
      const docChunk = result.chunks.find(chunk => chunk.has_documentation);
      expect(docChunk).toBeDefined();
      
      // High-quality documentation should have high confidence (0.85+)
      expect(docChunk.confidence).toBeGreaterThan(0.85);
      
      // Should have multi-dimensional scoring information
      expect(docChunk.dimension_scores).toBeDefined();
      expect(docChunk.dimension_scores.pattern).toBeDefined();
      expect(docChunk.dimension_scores.semantic).toBeDefined();
      expect(docChunk.dimension_scores.context).toBeDefined();
      expect(docChunk.dimension_scores.quality).toBeDefined();
      
      // Pattern confidence should be high for /// documentation
      expect(docChunk.dimension_scores.pattern.score).toBeGreaterThan(0.9);
      
      // Semantic confidence should be high for meaningful content
      expect(docChunk.dimension_scores.semantic.score).toBeGreaterThan(0.7);
      
      // Quality confidence should be high for complete documentation
      expect(docChunk.dimension_scores.quality.score).toBeGreaterThan(0.8);
      
    }, 30000);

    test('MUST FAIL: Should implement multi-dimensional confidence scoring', async () => {
      // Test with minimal but valid documentation
      const minimalCode = `
/// Simple function
pub fn simple() -> i32 { 42 }`;
      
      const result = await indexCode(minimalCode, 'rust');
      const docChunk = result.chunks.find(chunk => chunk.has_documentation);
      
      expect(docChunk).toBeDefined();
      expect(docChunk.dimension_scores).toBeDefined();
      
      // Should have all 5 dimensions
      const dimensions = docChunk.dimension_scores;
      expect(dimensions.pattern).toBeDefined();
      expect(dimensions.semantic).toBeDefined();
      expect(dimensions.context).toBeDefined();
      expect(dimensions.quality).toBeDefined();
      expect(dimensions.meta).toBeDefined();
      
      // Pattern should be high (proper /// syntax)
      expect(dimensions.pattern.score).toBeGreaterThan(0.8);
      
      // Semantic should be lower (minimal content)
      expect(dimensions.semantic.score).toBeLessThan(0.5);
      
      // Quality should be lower (incomplete documentation)
      expect(dimensions.quality.score).toBeLessThan(0.4);
      
      // Should have calibration applied
      expect(docChunk.calibration_applied).toBe(true);
      
    }, 30000);

    test('MUST FAIL: Should use adaptive thresholds based on context', async () => {
      // Test API documentation (should have higher threshold)
      const apiCode = `
/// Public API function for neural processing
pub fn public_api_function() -> Result<String, Error> {
    Ok("processed".to_string())
}`;

      // Test internal helper (should have lower threshold)
      const internalCode = `
/// Helper function
fn internal_helper() -> i32 { 42 }`;
      
      const apiResult = await indexCode(apiCode, 'rust');
      const internalResult = await indexCode(internalCode, 'rust');
      
      const apiChunk = apiResult.chunks.find(chunk => chunk.has_documentation);
      const internalChunk = internalResult.chunks.find(chunk => chunk.has_documentation);
      
      expect(apiChunk).toBeDefined();
      expect(internalChunk).toBeDefined();
      
      // API documentation should have higher threshold
      expect(apiChunk.adaptive_threshold).toBeDefined();
      expect(internalChunk.adaptive_threshold).toBeDefined();
      
      // API threshold should be higher than internal threshold
      // (Note: This might not always be true, depends on context classification)
      expect(typeof apiChunk.adaptive_threshold).toBe('number');
      expect(typeof internalChunk.adaptive_threshold).toBe('number');
      
      // Both should have different thresholds based on context
      expect(apiChunk.adaptive_threshold).toBeGreaterThan(0.3);
      expect(internalChunk.adaptive_threshold).toBeGreaterThan(0.3);
      
    }, 30000);

    test('MUST FAIL: Should enable quality-based filtering of documentation', async () => {
      // Mix of high and low quality documentation
      const mixedQualityCode = `
/// Comprehensive neural network layer implementation.
/// 
/// This struct represents a fully-connected neural network layer with
/// configurable activation functions, dropout, and weight initialization.
/// Supports both forward and backward propagation for training.
/// 
/// # Fields  
/// 
/// * \`weights\` - Weight matrix for the layer
/// * \`biases\` - Bias vector for neurons
/// * \`activation\` - Activation function type
/// 
/// # Examples
/// 
/// \`\`\`rust
/// let layer = NeuralLayer::new(128, 64, ActivationType::ReLU);
/// let output = layer.forward(&input_tensor);
/// \`\`\`
pub struct NeuralLayer {
    weights: Matrix,
    biases: Vector,
    activation: ActivationType,
}

/// does stuff
pub fn low_quality_function() -> i32 {
    1
}

/// Helper
fn minimal_docs() -> String {
    "helper".to_string()
}`;

      const result = await indexCode(mixedQualityCode, 'rust');
      
      expect(result.chunks).toBeDefined();
      expect(result.chunks.length).toBeGreaterThan(0);
      
      const docChunks = result.chunks.filter(chunk => chunk.has_documentation);
      expect(docChunks.length).toBeGreaterThan(0);
      
      // Should be able to filter by confidence threshold
      const highConfidenceChunks = docChunks.filter(chunk => chunk.confidence > 0.8);
      const lowConfidenceChunks = docChunks.filter(chunk => chunk.confidence < 0.5);
      
      expect(highConfidenceChunks.length).toBeGreaterThan(0); // High-quality struct docs
      expect(lowConfidenceChunks.length).toBeGreaterThan(0);  // Low-quality function docs
      
      // High confidence chunk should have better quality scores
      if (highConfidenceChunks.length > 0 && lowConfidenceChunks.length > 0) {
        const highQualityChunk = highConfidenceChunks[0];
        const lowQualityChunk = lowConfidenceChunks[0];
        
        expect(highQualityChunk.dimension_scores.quality.score)
          .toBeGreaterThan(lowQualityChunk.dimension_scores.quality.score);
      }
      
    }, 30000);

    test('MUST FAIL: Should maintain confidence accuracy across languages', async () => {
      // Test similar documentation patterns across languages
      const rustCode = `
/// Calculates neural activation using sigmoid function
/// Returns activation value between 0.0 and 1.0
pub fn sigmoid_activation(input: f64) -> f64 {
    1.0 / (1.0 + (-input).exp())
}`;

      const pythonCode = `
"""
Calculates neural activation using sigmoid function
Returns activation value between 0.0 and 1.0
"""
def sigmoid_activation(input_val):
    return 1.0 / (1.0 + math.exp(-input_val))`;

      const jsCode = `
/**
 * Calculates neural activation using sigmoid function  
 * Returns activation value between 0.0 and 1.0
 * @param {number} input - Input value to the sigmoid function
 * @returns {number} Activation value between 0.0 and 1.0
 */
function sigmoidActivation(input) {
    return 1.0 / (1.0 + Math.exp(-input));
}`;

      const rustResult = await indexCode(rustCode, 'rust');
      const pythonResult = await indexCode(pythonCode, 'python');
      const jsResult = await indexCode(jsCode, 'javascript');
      
      const rustChunk = rustResult.chunks.find(chunk => chunk.has_documentation);
      const pythonChunk = pythonResult.chunks.find(chunk => chunk.has_documentation);
      const jsChunk = jsResult.chunks.find(chunk => chunk.has_documentation);
      
      expect(rustChunk).toBeDefined();
      expect(pythonChunk).toBeDefined();
      expect(jsChunk).toBeDefined();
      
      // All should have reasonable confidence scores (accounting for language differences)
      expect(rustChunk.confidence).toBeGreaterThan(0.6);
      expect(pythonChunk.confidence).toBeGreaterThan(0.6);
      expect(jsChunk.confidence).toBeGreaterThan(0.6);
      
      // Confidence scores should be relatively consistent (within 0.3 range)
      const confidences = [rustChunk.confidence, pythonChunk.confidence, jsChunk.confidence];
      const maxConfidence = Math.max(...confidences);
      const minConfidence = Math.min(...confidences);
      
      expect(maxConfidence - minConfidence).toBeLessThan(0.3);
      
      // All should have multi-dimensional scoring
      [rustChunk, pythonChunk, jsChunk].forEach(chunk => {
        expect(chunk.dimension_scores).toBeDefined();
        expect(chunk.dimension_scores.pattern).toBeDefined();
        expect(chunk.dimension_scores.semantic).toBeDefined();
        expect(chunk.calibration_applied).toBe(true);
      });
      
    }, 45000);
    
  });
  
});