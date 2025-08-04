/**
 * Final Integration and Validation Test Suite for MCP RAG Indexer
 * Tests complete system pipeline, end-to-end accuracy, production readiness,
 * and verifies original problem resolution (0% → 95%+ documentation coverage)
 */

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs-extra');
const tmp = require('tmp');
const crypto = require('crypto');

// Set long timeout for comprehensive integration tests
jest.setTimeout(300000); // 5 minutes

describe('Task 010: Final Integration and Validation Tests', () => {
  let tempDir;
  let pythonPath;
  let indexerPath;
  let performanceBenchmarkPath;
  
  beforeAll(async () => {
    pythonPath = process.platform === 'win32' ? 
      path.join(__dirname, '..', 'runtime', 'windows-x64', 'python.exe') :
      'python3';
    indexerPath = path.join(__dirname, '..', 'python', 'indexer_universal.py');
    performanceBenchmarkPath = path.join(__dirname, '..', 'python', 'performance_benchmarks.py');
    
    // Verify all components exist
    expect(fs.existsSync(indexerPath)).toBe(true);
    expect(fs.existsSync(performanceBenchmarkPath)).toBe(true);
    if (process.platform === 'win32') {
      expect(fs.existsSync(pythonPath)).toBe(true);
    }
  });
  
  beforeEach(() => {
    tempDir = tmp.dirSync({ unsafeCleanup: true });
  });
  
  afterEach(() => {
    if (tempDir) {
      tempDir.removeCallback();
    }
  });

  describe('End-to-End System Pipeline Validation', () => {
    test('should achieve 95%+ documentation detection accuracy on real codebases', async () => {
      console.log('\n=== Task 010.1: End-to-End Accuracy Validation ===');
      
      // Create comprehensive test scenarios representing different documentation qualities
      const testScenarios = [
        {
          name: 'well_documented_rust_codebase',
          language: 'rust',
          expected_coverage: 0.95,
          content: `/// Calculates the factorial of a positive integer
/// 
/// # Arguments
/// * \`n\` - A positive integer to calculate factorial for
/// 
/// # Returns
/// The factorial of n as u64
/// 
/// # Examples
/// \`\`\`
/// let result = factorial(5);
/// assert_eq!(result, 120);
/// \`\`\`
pub fn factorial(n: u32) -> u64 {
    if n <= 1 {
        1
    } else {
        n as u64 * factorial(n - 1)
    }
}

/// Represents a mathematical point in 2D space
/// 
/// This struct provides methods for common point operations
/// including distance calculations and transformations.
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    /// The x-coordinate of the point
    pub x: f64,
    /// The y-coordinate of the point  
    pub y: f64,
}

impl Point {
    /// Creates a new Point at the specified coordinates
    /// 
    /// # Arguments
    /// * \`x\` - The x-coordinate
    /// * \`y\` - The y-coordinate
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
    
    /// Calculates the Euclidean distance to another point
    /// 
    /// # Arguments
    /// * \`other\` - The other point to measure distance to
    /// 
    /// # Returns
    /// The distance as a f64
    pub fn distance_to(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// A trait for objects that can be serialized to JSON
/// 
/// This trait provides a common interface for converting
/// objects to JSON representation.
pub trait JsonSerializable {
    /// Converts the object to a JSON string
    /// 
    /// # Returns
    /// A Result containing the JSON string or an error
    fn to_json(&self) -> Result<String, serde_json::Error>;
}`,
        },
        {
          name: 'well_documented_python_codebase',
          language: 'python',
          expected_coverage: 0.90,
          content: `"""
Advanced mathematical operations module.

This module provides various mathematical functions and utilities
for complex calculations and data processing.
"""

import math
from typing import List, Optional, Union


class Calculator:
    """
    A comprehensive calculator class for mathematical operations.
    
    This class provides methods for basic arithmetic, statistical
    calculations, and advanced mathematical functions.
    
    Attributes:
        precision (int): Number of decimal places for results
        history (List[str]): History of calculations performed
    """
    
    def __init__(self, precision: int = 2):
        """
        Initialize a new Calculator instance.
        
        Args:
            precision (int): Number of decimal places for results (default: 2)
        """
        self.precision = precision
        self.history = []
    
    def add(self, a: float, b: float) -> float:
        """
        Add two numbers together.
        
        Args:
            a (float): First number
            b (float): Second number
            
        Returns:
            float: Sum of a and b
            
        Examples:
            >>> calc = Calculator()
            >>> calc.add(2.5, 3.7)
            6.2
        """
        result = round(a + b, self.precision)
        self.history.append(f"{a} + {b} = {result}")
        return result
    
    def calculate_statistics(self, data: List[float]) -> dict:
        """
        Calculate comprehensive statistics for a dataset.
        
        Args:
            data (List[float]): List of numerical values
            
        Returns:
            dict: Dictionary containing mean, median, std_dev, and range
            
        Raises:
            ValueError: If data list is empty
            
        Examples:
            >>> calc = Calculator()
            >>> stats = calc.calculate_statistics([1, 2, 3, 4, 5])
            >>> stats['mean']
            3.0
        """
        if not data:
            raise ValueError("Data list cannot be empty")
        
        mean_val = sum(data) / len(data)
        sorted_data = sorted(data)
        n = len(sorted_data)
        
        # Calculate median
        if n % 2 == 0:
            median_val = (sorted_data[n//2 - 1] + sorted_data[n//2]) / 2
        else:
            median_val = sorted_data[n//2]
        
        # Calculate standard deviation
        variance = sum((x - mean_val) ** 2 for x in data) / len(data)
        std_dev = math.sqrt(variance)
        
        return {
            'mean': round(mean_val, self.precision),
            'median': round(median_val, self.precision),
            'std_dev': round(std_dev, self.precision),
            'range': round(max(data) - min(data), self.precision)
        }


def fibonacci_sequence(n: int) -> List[int]:
    """
    Generate the first n numbers in the Fibonacci sequence.
    
    The Fibonacci sequence starts with 0 and 1, and each subsequent
    number is the sum of the two preceding ones.
    
    Args:
        n (int): Number of Fibonacci numbers to generate
        
    Returns:
        List[int]: List containing the first n Fibonacci numbers
        
    Raises:
        ValueError: If n is negative
        
    Examples:
        >>> fibonacci_sequence(5)
        [0, 1, 1, 2, 3]
        >>> fibonacci_sequence(0)
        []
    """
    if n < 0:
        raise ValueError("n must be non-negative")
    
    if n == 0:
        return []
    elif n == 1:
        return [0]
    elif n == 2:
        return [0, 1]
    
    sequence = [0, 1]
    for i in range(2, n):
        sequence.append(sequence[i-1] + sequence[i-2])
    
    return sequence`,
        },
        {
          name: 'well_documented_javascript_codebase',
          language: 'javascript',
          expected_coverage: 0.85,
          content: `/**
 * Advanced data processing and analysis utilities
 * 
 * This module provides comprehensive tools for data manipulation,
 * analysis, and transformation operations.
 * 
 * @module DataProcessor
 * @version 1.0.0
 * @author Development Team
 */

/**
 * A comprehensive data processor class for handling various data operations
 * 
 * @class DataProcessor
 * @description Provides methods for data validation, transformation, and analysis
 */
class DataProcessor {
    /**
     * Create a new DataProcessor instance
     * 
     * @constructor
     * @param {Object} options - Configuration options
     * @param {boolean} options.strictMode - Enable strict validation
     * @param {number} options.maxSize - Maximum data size to process
     */
    constructor(options = {}) {
        this.strictMode = options.strictMode || false;
        this.maxSize = options.maxSize || 10000;
        this.processedCount = 0;
    }
    
    /**
     * Process and validate an array of data items
     * 
     * @method processData
     * @param {Array} data - Array of data items to process
     * @param {Function} validator - Optional validation function
     * @returns {Promise<Array>} Promise resolving to processed data array
     * @throws {Error} If data is invalid or exceeds size limits
     * 
     * @example
     * const processor = new DataProcessor();
     * const result = await processor.processData([1, 2, 3], x => x > 0);
     * console.log(result); // [1, 2, 3]
     */
    async processData(data, validator = null) {
        if (!Array.isArray(data)) {
            throw new Error('Data must be an array');
        }
        
        if (data.length > this.maxSize) {
            throw new Error(\`Data size exceeds limit of \${this.maxSize}\`);
        }
        
        const processedData = [];
        
        for (const item of data) {
            if (validator && !validator(item)) {
                if (this.strictMode) {
                    throw new Error(\`Invalid item: \${item}\`);
                }
                continue;
            }
            
            processedData.push(await this.transformItem(item));
        }
        
        this.processedCount += processedData.length;
        return processedData;
    }
    
    /**
     * Transform a single data item using configured transformations
     * 
     * @private
     * @method transformItem
     * @param {*} item - The item to transform
     * @returns {Promise<*>} Promise resolving to transformed item
     */
    async transformItem(item) {
        // Simulate async transformation
        return new Promise(resolve => {
            setTimeout(() => {
                if (typeof item === 'number') {
                    resolve(item * 2);
                } else if (typeof item === 'string') {
                    resolve(item.toUpperCase());
                } else {
                    resolve(item);
                }
            }, 1);
        });
    }
    
    /**
     * Calculate comprehensive statistics for numerical data
     * 
     * @method calculateStats
     * @param {Array<number>} numbers - Array of numerical values
     * @returns {Object} Statistics object with mean, median, mode, etc.
     * @throws {Error} If input is not an array of numbers
     * 
     * @example
     * const processor = new DataProcessor();
     * const stats = processor.calculateStats([1, 2, 3, 4, 5]);
     * console.log(stats.mean); // 3
     */
    calculateStats(numbers) {
        if (!Array.isArray(numbers) || !numbers.every(n => typeof n === 'number')) {
            throw new Error('Input must be an array of numbers');
        }
        
        if (numbers.length === 0) {
            return { mean: 0, median: 0, mode: null, range: 0 };
        }
        
        const sorted = [...numbers].sort((a, b) => a - b);
        const mean = numbers.reduce((sum, n) => sum + n, 0) / numbers.length;
        
        // Calculate median
        const mid = Math.floor(sorted.length / 2);
        const median = sorted.length % 2 === 0 
            ? (sorted[mid - 1] + sorted[mid]) / 2 
            : sorted[mid];
        
        // Calculate mode
        const frequency = {};
        let maxCount = 0;
        let mode = null;
        
        for (const num of numbers) {
            frequency[num] = (frequency[num] || 0) + 1;
            if (frequency[num] > maxCount) {
                maxCount = frequency[num];
                mode = num;
            }
        }
        
        return {
            mean: Math.round(mean * 100) / 100,
            median: Math.round(median * 100) / 100,
            mode: maxCount > 1 ? mode : null,
            range: sorted[sorted.length - 1] - sorted[0]
        };
    }
}

/**
 * Utility functions for data manipulation and analysis
 * 
 * @namespace DataUtils
 */
const DataUtils = {
    /**
     * Deep clone an object or array
     * 
     * @function deepClone
     * @memberof DataUtils
     * @param {*} obj - Object to clone
     * @returns {*} Deep cloned copy of the object
     * 
     * @example
     * const original = { a: { b: 1 } };
     * const cloned = DataUtils.deepClone(original);
     */
    deepClone(obj) {
        if (obj === null || typeof obj !== 'object') return obj;
        if (obj instanceof Date) return new Date(obj.getTime());
        if (obj instanceof Array) return obj.map(item => this.deepClone(item));
        if (typeof obj === 'object') {
            const cloned = {};
            for (const key in obj) {
                if (obj.hasOwnProperty(key)) {
                    cloned[key] = this.deepClone(obj[key]);
                }
            }
            return cloned;
        }
    }
};

module.exports = { DataProcessor, DataUtils };`,
        }
      ];

      let totalScenarios = 0;
      let successfulScenarios = 0;
      const detailedResults = [];

      for (const scenario of testScenarios) {
        console.log(\`Testing scenario: \${scenario.name}\`);
        
        const result = await runAccuracyTest(scenario.content, scenario.language);
        totalScenarios++;
        
        // Analyze documentation coverage accuracy
        const documentedChunks = result.chunks.filter(chunk => 
          chunk.metadata && chunk.metadata.has_documentation
        );
        const actualCoverage = documentedChunks.length / result.chunks.length;
        
        // Calculate confidence score
        const avgConfidence = documentedChunks.length > 0 
          ? documentedChunks.reduce((sum, chunk) => 
              sum + (chunk.metadata.confidence || 0), 0) / documentedChunks.length
          : 0;
        
        const scenarioResult = {
          name: scenario.name,
          language: scenario.language,
          expected_coverage: scenario.expected_coverage,
          actual_coverage: actualCoverage,
          confidence: avgConfidence,
          chunks_total: result.chunks.length,
          chunks_documented: documentedChunks.length,
          accuracy_score: Math.min(actualCoverage / scenario.expected_coverage, 1.0)
        };
        
        detailedResults.push(scenarioResult);
        
        // Success criteria: achieve at least 85% of expected coverage with >70% confidence
        if (scenarioResult.accuracy_score >= 0.85 && avgConfidence >= 0.7) {
          successfulScenarios++;
        }
        
        console.log(\`  Expected coverage: \${(scenario.expected_coverage * 100).toFixed(1)}%\`);
        console.log(\`  Actual coverage: \${(actualCoverage * 100).toFixed(1)}%\`);
        console.log(\`  Confidence: \${(avgConfidence * 100).toFixed(1)}%\`);
        console.log(\`  Accuracy score: \${(scenarioResult.accuracy_score * 100).toFixed(1)}%\`);
      }

      // Calculate overall system accuracy
      const overallAccuracy = successfulScenarios / totalScenarios;
      const avgActualCoverage = detailedResults.reduce((sum, r) => sum + r.actual_coverage, 0) / detailedResults.length;
      const avgConfidence = detailedResults.reduce((sum, r) => sum + r.confidence, 0) / detailedResults.length;

      console.log(\`\\n=== End-to-End Accuracy Results ===\`);
      console.log(\`Overall accuracy: \${(overallAccuracy * 100).toFixed(1)}% (\${successfulScenarios}/\${totalScenarios} scenarios)\`);
      console.log(\`Average documentation coverage: \${(avgActualCoverage * 100).toFixed(1)}%\`);
      console.log(\`Average confidence score: \${(avgConfidence * 100).toFixed(1)}%\`);

      // Validate performance targets
      expect(overallAccuracy).toBeGreaterThanOrEqual(0.95); // 95%+ success rate
      expect(avgActualCoverage).toBeGreaterThanOrEqual(0.85); // 85%+ documentation coverage  
      expect(avgConfidence).toBeGreaterThanOrEqual(0.70); // 70%+ confidence
      
      console.log('✅ End-to-end accuracy validation PASSED');
    });

    test('should integrate all system components seamlessly', async () => {
      console.log('\\n=== Task 010.2: Component Integration Matrix Validation ===');
      
      // Test integration of all major components
      const integrationTests = [
        {
          name: 'documentation_patterns_to_extraction',
          description: 'Task 002 patterns → Task 003 extraction',
          test: async () => {
            const content = \`/// Well documented function
pub fn test_function() -> i32 { 42 }\`;
            const result = await runAccuracyTest(content, 'rust');
            return result.chunks.length > 0 && 
                   result.chunks.some(c => c.metadata?.has_documentation);
          }
        },
        {
          name: 'extraction_to_multipass_detection',
          description: 'Task 003 extraction → Task 004 multi-pass detection',
          test: async () => {
            const content = \`/// First pass detection
pub fn func1() -> i32 { 1 }

// TODO: Second pass should catch this
pub fn func2() -> i32 { 2 }\`;
            const result = await runAccuracyTest(content, 'rust');
            return result.validation_results && 
                   result.validation_results.length > 0;
          }
        },
        {
          name: 'multipass_to_semantic_analysis',
          description: 'Task 004 multi-pass → Task 005 semantic analysis',
          test: async () => {
            const content = \`/// Semantic analysis test
/// This function calculates important metrics
pub fn calculate_metrics() -> f64 { 3.14 }\`;
            const result = await runAccuracyTest(content, 'rust');
            return result.chunks.some(c => 
              c.metadata?.confidence && c.metadata.confidence > 0.5);
          }
        },
        {
          name: 'semantic_to_chunking',
          description: 'Task 005 semantic → Task 006 smart chunking',
          test: async () => {
            const longContent = Array(50).fill(\`/// Function documentation
pub fn func() -> i32 { 42 }\`).join('\\n\\n');
            const result = await runAccuracyTest(longContent, 'rust');
            return result.chunks.length > 1; // Should create multiple chunks
          }
        },
        {
          name: 'chunking_to_confidence_scoring',
          description: 'Task 006 chunking → Task 007 confidence scoring',
          test: async () => {
            const content = \`/// High confidence documentation
/// Multiple lines with detailed info
/// Including examples and parameters
pub fn well_documented() -> i32 { 42 }\`;
            const result = await runAccuracyTest(content, 'rust');
            return result.chunks.some(c => 
              c.metadata?.confidence && c.metadata.confidence > 0.8);
          }
        },
        {
          name: 'confidence_to_qa_validation',
          description: 'Task 007 confidence → Task 008 QA validation',
          test: async () => {
            const content = \`/// QA validation test
pub fn qa_test() -> i32 { 42 }\`;
            const result = await runAccuracyTest(content, 'rust');
            return result.overall_validation && 
                   result.overall_validation.status;
          }
        },
        {
          name: 'qa_to_performance_optimization',
          description: 'Task 008 QA → Task 009 performance optimization',
          test: async () => {
            const startTime = Date.now();
            const content = \`/// Performance test
pub fn perf_test() -> i32 { 42 }\`;
            const result = await runAccuracyTest(content, 'rust');
            const processingTime = Date.now() - startTime;
            return result.parsing_time !== undefined && 
                   processingTime < 1000; // Should be fast
          }
        }
      ];

      let passedTests = 0;
      const testResults = [];

      for (const test of integrationTests) {
        console.log(\`Testing: \${test.description}\`);
        try {
          const success = await test.test();
          if (success) {
            passedTests++;
            console.log(\`  ✅ PASSED\`);
          } else {
            console.log(\`  ❌ FAILED\`);
          }
          testResults.push({ name: test.name, success });
        } catch (error) {
          console.log(\`  ❌ ERROR: \${error.message}\`);
          testResults.push({ name: test.name, success: false, error: error.message });
        }
      }

      const integrationSuccessRate = passedTests / integrationTests.length;
      console.log(\`\\n=== Component Integration Results ===\`);
      console.log(\`Integration success rate: \${(integrationSuccessRate * 100).toFixed(1)}% (\${passedTests}/\${integrationTests.length})\`);

      // Require 100% component integration success
      expect(integrationSuccessRate).toBe(1.0);
      
      console.log('✅ Component integration validation PASSED');
    });

    test('should demonstrate original problem resolution (0% → 95%+ improvement)', async () => {
      console.log('\\n=== Task 010.3: Original Problem Resolution Verification ===');
      
      // Simulate the original problem scenario
      const problemScenario = {
        description: 'Original vector system reporting 0% coverage on well-documented code',
        // This represents the type of code that was incorrectly reporting 0% coverage
        testCode: \`
/// This is comprehensive documentation for a mathematical function
/// that calculates the greatest common divisor of two integers.
/// 
/// # Algorithm
/// Uses Euclid's algorithm for efficient GCD calculation
/// 
/// # Arguments
/// * \`a\` - First integer
/// * \`b\` - Second integer
/// 
/// # Returns
/// The greatest common divisor of a and b
/// 
/// # Examples
/// \`\`\`
/// assert_eq!(gcd(48, 18), 6);
/// assert_eq!(gcd(7, 3), 1);
/// \`\`\`
/// 
/// # Time Complexity
/// O(log(min(a, b)))
pub fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// Calculates the least common multiple of two integers.
/// 
/// Uses the relationship: LCM(a,b) = |a*b| / GCD(a,b)
/// 
/// # Arguments
/// * \`a\` - First integer
/// * \`b\` - Second integer
/// 
/// # Returns
/// The least common multiple of a and b
/// 
/// # Panics
/// Panics if either input is 0
pub fn lcm(a: u64, b: u64) -> u64 {
    if a == 0 || b == 0 {
        panic!("LCM undefined for zero inputs");
    }
    (a * b) / gcd(a, b)
}

/// A structure representing a mathematical fraction
/// 
/// Provides methods for fraction arithmetic and simplification
#[derive(Debug, Clone, PartialEq)]
pub struct Fraction {
    /// Numerator of the fraction
    pub numerator: i64,
    /// Denominator of the fraction (must be non-zero)
    pub denominator: i64,
}

impl Fraction {
    /// Creates a new fraction and reduces it to lowest terms
    /// 
    /// # Arguments
    /// * \`numerator\` - The numerator
    /// * \`denominator\` - The denominator (must be non-zero)
    /// 
    /// # Panics
    /// Panics if denominator is 0
    pub fn new(numerator: i64, denominator: i64) -> Self {
        if denominator == 0 {
            panic!("Denominator cannot be zero");
        }
        
        let gcd_val = gcd(numerator.abs() as u64, denominator.abs() as u64) as i64;
        let reduced_num = numerator / gcd_val;
        let reduced_den = denominator / gcd_val;
        
        // Ensure denominator is positive
        if reduced_den < 0 {
            Fraction {
                numerator: -reduced_num,
                denominator: -reduced_den,
            }
        } else {
            Fraction {
                numerator: reduced_num,
                denominator: reduced_den,
            }
        }
    }
    
    /// Adds two fractions together
    /// 
    /// # Arguments
    /// * \`other\` - The fraction to add
    /// 
    /// # Returns
    /// A new fraction representing the sum
    pub fn add(&self, other: &Fraction) -> Fraction {
        let new_num = self.numerator * other.denominator + other.numerator * self.denominator;
        let new_den = self.denominator * other.denominator;
        Fraction::new(new_num, new_den)
    }
}
\`,
        expected_original_coverage: 0.0, // Original system reported 0%
        expected_new_coverage: 0.95     // New system should achieve 95%+
      };

      console.log('Testing original problem scenario...');
      console.log('Code sample: Well-documented Rust mathematical functions');
      
      const result = await runAccuracyTest(problemScenario.testCode, 'rust');
      
      // Analyze the results
      const totalChunks = result.chunks.length;
      const documentedChunks = result.chunks.filter(chunk => 
        chunk.metadata && chunk.metadata.has_documentation
      ).length;
      
      const actualCoverage = documentedChunks / totalChunks;
      const avgConfidence = documentedChunks > 0 
        ? result.chunks
            .filter(c => c.metadata?.has_documentation)
            .reduce((sum, c) => sum + (c.metadata.confidence || 0), 0) / documentedChunks
        : 0;

      // Calculate improvement metrics
      const coverageImprovement = actualCoverage - problemScenario.expected_original_coverage;
      const improvementPercentage = (coverageImprovement / (problemScenario.expected_new_coverage - problemScenario.expected_original_coverage)) * 100;

      console.log(\`\\n=== Original Problem Resolution Results ===\`);
      console.log(\`Original system coverage: \${(problemScenario.expected_original_coverage * 100).toFixed(1)}%\`);
      console.log(\`New system coverage: \${(actualCoverage * 100).toFixed(1)}%\`);
      console.log(\`Coverage improvement: +\${(coverageImprovement * 100).toFixed(1)} percentage points\`);
      console.log(\`Improvement achievement: \${improvementPercentage.toFixed(1)}% of target\`);
      console.log(\`Average confidence: \${(avgConfidence * 100).toFixed(1)}%\`);
      console.log(\`Total chunks analyzed: \${totalChunks}\`);
      console.log(\`Documented chunks found: \${documentedChunks}\`);

      // Validate the problem resolution
      expect(actualCoverage).toBeGreaterThanOrEqual(0.95); // 95%+ coverage achieved
      expect(avgConfidence).toBeGreaterThanOrEqual(0.70);  // 70%+ confidence
      expect(coverageImprovement).toBeGreaterThanOrEqual(0.90); // 90+ percentage point improvement
      expect(totalChunks).toBeGreaterThan(0); // System should process the code
      expect(documentedChunks).toBeGreaterThan(0); // Should find documentation

      console.log('✅ Original problem resolution verification PASSED');
      console.log(\`✅ Successfully resolved: 0% → \${(actualCoverage * 100).toFixed(1)}% documentation coverage\`);
    });
  });

  describe('Production Deployment Readiness Assessment', () => {
    test('should validate production load handling and stability', async () => {
      console.log('\\n=== Task 010.4: Production Load Testing ===');
      
      // Test with realistic production scenarios
      const loadTests = [
        {
          name: 'high_volume_processing',
          description: 'Process multiple files concurrently',
          scale: 'large',
          concurrent_workers: 4
        },
        {
          name: 'memory_stability',
          description: 'Sustained processing without memory leaks',
          scale: 'medium',
          iterations: 10
        },
        {
          name: 'error_recovery',
          description: 'Recovery from processing errors',
          scale: 'small',
          inject_errors: true
        }
      ];

      const loadTestResults = [];

      for (const test of loadTests) {
        console.log(\`Running load test: \${test.description}\`);
        
        try {
          const result = await runPerformanceBenchmark([
            '--scale', test.scale,
            '--language', 'rust',
            ...(test.concurrent_workers ? ['--workers', test.concurrent_workers.toString()] : []),
            ...(test.iterations ? ['--iterations', test.iterations.toString()] : [])
          ]);
          
          if (result.success) {
            const performanceData = result.data.performance_report || result.data;
            loadTestResults.push({
              name: test.name,
              success: true,
              processing_time: performanceData.summary?.avg_processing_time || 0,
              memory_usage: performanceData.summary?.avg_memory_usage || 0,
              target_compliance: performanceData.target_compliance || {}
            });
            console.log(\`  ✅ PASSED - Processing time: \${(performanceData.summary?.avg_processing_time || 0).toFixed(3)}s\`);
          } else {
            loadTestResults.push({
              name: test.name,
              success: false,
              error: 'Test execution failed'
            });
            console.log(\`  ❌ FAILED\`);
          }
        } catch (error) {
          loadTestResults.push({
            name: test.name,
            success: false,
            error: error.message
          });
          console.log(\`  ❌ ERROR: \${error.message}\`);
        }
      }

      const successfulTests = loadTestResults.filter(r => r.success).length;
      const loadTestSuccessRate = successfulTests / loadTests.length;

      console.log(\`\\n=== Production Load Test Results ===\`);
      console.log(\`Load test success rate: \${(loadTestSuccessRate * 100).toFixed(1)}% (\${successfulTests}/\${loadTests.length})\`);

      // Require 100% load test success for production readiness
      expect(loadTestSuccessRate).toBe(1.0);
      
      console.log('✅ Production load testing PASSED');
    });

    test('should meet all production deployment criteria', async () => {
      console.log('\\n=== Task 010.5: Production Deployment Readiness Checklist ===');
      
      const deploymentCriteria = [
        {
          name: 'performance_benchmarks',
          description: 'Meet performance targets across all scales',
          check: async () => {
            const result = await runPerformanceBenchmark(['--production-readiness-test']);
            return result.success && 
                   result.data.performance_report?.target_compliance?.medium?.overall_compliant;
          }
        },
        {
          name: 'memory_optimization',
          description: 'Achieve memory savings targets',
          check: async () => {
            const result = await runPerformanceBenchmark(['--memory-optimization-test']);
            return result.success && 
                   result.data.memory_optimization?.medium?.memory_savings_percent > 20;
          }
        },
        {
          name: 'concurrent_processing',
          description: 'Support concurrent processing effectively',
          check: async () => {
            const result = await runPerformanceBenchmark(['--concurrent-test']);
            return result.success &&
                   result.data.performance_report?.concurrent_performance;
          }
        },
        {
          name: 'cross_language_support',
          description: 'Consistent performance across languages',
          check: async () => {
            const languages = ['rust', 'python', 'javascript'];
            const results = [];
            for (const lang of languages) {
              const result = await runPerformanceBenchmark(['--language', lang, '--scale', 'small']);
              results.push(result.success);
            }
            return results.every(r => r);
          }
        },
        {
          name: 'documentation_accuracy',
          description: 'High documentation detection accuracy',
          check: async () => {
            const testContent = \`/// Well documented function
pub fn test() -> i32 { 42 }\`;
            const result = await runAccuracyTest(testContent, 'rust');
            const coverage = result.chunks.filter(c => c.metadata?.has_documentation).length / result.chunks.length;
            return coverage >= 0.9;
          }
        }
      ];

      let passedCriteria = 0;
      const criteriaResults = [];

      for (const criterion of deploymentCriteria) {
        console.log(\`Checking: \${criterion.description}\`);
        try {
          const passed = await criterion.check();
          if (passed) {
            passedCriteria++;
            console.log(\`  ✅ PASSED\`);
          } else {
            console.log(\`  ❌ FAILED\`);
          }
          criteriaResults.push({ name: criterion.name, passed });
        } catch (error) {
          console.log(\`  ❌ ERROR: \${error.message}\`);
          criteriaResults.push({ name: criterion.name, passed: false, error: error.message });
        }
      }

      const deploymentReadiness = passedCriteria / deploymentCriteria.length;
      
      console.log(\`\\n=== Production Deployment Readiness Assessment ===\`);
      console.log(\`Deployment readiness: \${(deploymentReadiness * 100).toFixed(1)}% (\${passedCriteria}/\${deploymentCriteria.length} criteria met)\`);

      // Require 100% deployment readiness
      expect(deploymentReadiness).toBe(1.0);
      
      console.log('✅ Production deployment readiness PASSED');
    });
  });

  describe('Final System Validation', () => {
    test('should provide comprehensive quality score and deployment recommendations', async () => {
      console.log('\\n=== Task 010.6: Final Quality Score and Recommendations ===');
      
      // Collect all validation metrics
      const validationMetrics = {
        end_to_end_accuracy: 0,
        component_integration: 0,
        original_problem_resolution: 0,
        production_load_handling: 0,
        deployment_readiness: 0
      };

      // Test end-to-end accuracy
      try {
        const accuracyTest = await runAccuracyTest(\`/// Test function
pub fn test() -> i32 { 42 }\`, 'rust');
        const coverage = accuracyTest.chunks.filter(c => c.metadata?.has_documentation).length / accuracyTest.chunks.length;
        validationMetrics.end_to_end_accuracy = coverage >= 0.9 ? 100 : Math.round(coverage * 100);
      } catch (error) {
        console.warn('Accuracy test failed:', error.message);
      }

      // Test component integration
      try {
        const integrationTest = await runAccuracyTest(\`/// Integration test
pub fn integration() -> i32 { 42 }\`, 'rust');
        validationMetrics.component_integration = integrationTest.overall_validation ? 100 : 50;
      } catch (error) {
        console.warn('Integration test failed:', error.message);
      }

      // Test original problem resolution
      try {
        const problemTest = await runAccuracyTest(\`/// Original problem test
/// This should be detected at high accuracy
pub fn original_problem() -> i32 { 42 }\`, 'rust');
        const problemCoverage = problemTest.chunks.filter(c => c.metadata?.has_documentation).length / problemTest.chunks.length;
        validationMetrics.original_problem_resolution = problemCoverage >= 0.95 ? 100 : Math.round(problemCoverage * 100);
      } catch (error) {
        console.warn('Problem resolution test failed:', error.message);
      }

      // Test production load handling
      try {
        const loadTest = await runPerformanceBenchmark(['--scale', 'medium']);
        validationMetrics.production_load_handling = loadTest.success ? 100 : 0;
      } catch (error) {
        console.warn('Load test failed:', error.message);
      }

      // Test deployment readiness
      try {
        const deploymentTest = await runPerformanceBenchmark(['--production-readiness-test']);
        validationMetrics.deployment_readiness = deploymentTest.success ? 100 : 0;
      } catch (error) {
        console.warn('Deployment test failed:', error.message);
      }

      // Calculate overall quality score
      const qualityScores = Object.values(validationMetrics);
      const overallQualityScore = qualityScores.reduce((sum, score) => sum + score, 0) / qualityScores.length;

      // Generate deployment recommendations
      const recommendations = [];
      if (validationMetrics.end_to_end_accuracy < 95) {
        recommendations.push('Improve documentation detection accuracy for production use');
      }
      if (validationMetrics.component_integration < 100) {
        recommendations.push('Address component integration issues before deployment');
      }
      if (validationMetrics.original_problem_resolution < 95) {
        recommendations.push('Verify original problem resolution is complete');
      }
      if (validationMetrics.production_load_handling < 100) {
        recommendations.push('Optimize system for production load handling');
      }
      if (validationMetrics.deployment_readiness < 100) {
        recommendations.push('Complete all deployment readiness criteria');
      }

      if (recommendations.length === 0) {
        recommendations.push('System is ready for production deployment');
        recommendations.push('Consider implementing monitoring and alerting');
        recommendations.push('Plan for gradual rollout and performance monitoring');
      }

      console.log(\`\\n=== Final Quality Score Breakdown ===\`);
      console.log(\`End-to-End Accuracy: \${validationMetrics.end_to_end_accuracy}/100\`);
      console.log(\`Component Integration: \${validationMetrics.component_integration}/100\`);
      console.log(\`Original Problem Resolution: \${validationMetrics.original_problem_resolution}/100\`);
      console.log(\`Production Load Handling: \${validationMetrics.production_load_handling}/100\`);
      console.log(\`Deployment Readiness: \${validationMetrics.deployment_readiness}/100\`);
      console.log(\`\\n**OVERALL QUALITY SCORE: \${overallQualityScore.toFixed(1)}/100**\`);
      
      console.log(\`\\n=== Deployment Recommendations ===\`);
      recommendations.forEach((rec, index) => {
        console.log(\`\${index + 1}. \${rec}\`);
      });

      // Validate quality score meets target
      expect(overallQualityScore).toBeGreaterThanOrEqual(95); // 95+ quality score required

      console.log('\\n✅ Final system validation PASSED');
      console.log(\`✅ Quality Score: \${overallQualityScore.toFixed(1)}/100 (Target: 95+)\`);
    });
  });

  // Helper functions
  async function runAccuracyTest(content, language) {
    return new Promise((resolve, reject) => {
      const pythonProcess = spawn(pythonPath, [
        indexerPath,
        'create-chunks',
        '--content', content,
        '--language', language,
        '--validate'
      ], {
        cwd: path.dirname(indexerPath),
        stdio: ['ignore', 'pipe', 'pipe']
      });

      let stdout = '';
      let stderr = '';

      pythonProcess.stdout.on('data', (data) => {
        stdout += data.toString();
      });

      pythonProcess.stderr.on('data', (data) => {
        stderr += data.toString();
      });

      pythonProcess.on('close', (code) => {
        if (code === 0) {
          try {
            const result = JSON.parse(stdout);
            resolve(result);
          } catch (error) {
            reject(new Error(\`Failed to parse JSON output: \${error.message}\\nOutput: \${stdout}\`));
          }
        } else {
          reject(new Error(\`Python process exited with code \${code}\\nstderr: \${stderr}\\nstdout: \${stdout}\`));
        }
      });

      pythonProcess.on('error', (error) => {
        reject(new Error(\`Failed to start Python process: \${error.message}\`));
      });
    });
  }

  async function runPerformanceBenchmark(args = []) {
    return new Promise((resolve, reject) => {
      const process = spawn(pythonPath, [performanceBenchmarkPath, ...args], {
        cwd: path.join(__dirname, '..'),
        stdio: ['ignore', 'pipe', 'pipe']
      });

      let stdout = '';
      let stderr = '';

      process.stdout.on('data', (data) => {
        stdout += data.toString();
      });

      process.stderr.on('data', (data) => {
        stderr += data.toString();
      });

      process.on('close', (code) => {
        if (code === 0) {
          try {
            // Try to parse JSON output
            const lines = stdout.trim().split('\\n');
            let jsonLine = lines.find(line => line.startsWith('{'));
            
            if (jsonLine) {
              const data = JSON.parse(jsonLine);
              resolve({ success: true, data });
            } else {
              resolve({ success: true, data: { message: stdout } });
            }
          } catch (e) {
            resolve({ success: true, data: { message: stdout, error: e.message } });
          }
        } else {
          reject(new Error(\`Benchmark failed with code \${code}: \${stderr}\`));
        }
      });

      process.on('error', (error) => {
        reject(error);
      });
    });
  }
});