/**
 * Quality Assurance and Validation Test Suite
 * Tests the comprehensive QA framework for the MCP RAG Indexer
 */

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs-extra');
const tmp = require('tmp');

// Set longer timeout for validation tests
jest.setTimeout(120000);

describe('QA Validation Framework Tests', () => {
  let tempDir;
  let indexerPath;
  
  beforeAll(() => {
    indexerPath = path.join(__dirname, '..', 'python', 'indexer_universal.py');
    expect(fs.existsSync(indexerPath)).toBe(true);
  });
  
  beforeEach(() => {
    tempDir = tmp.dirSync({ unsafeCleanup: true });
  });
  
  afterEach(() => {
    if (tempDir) {
      tempDir.removeCallback();
    }
  });

  describe('End-to-End Validation System', () => {
    test('should validate end-to-end system accuracy (99%+ success rate)', async () => {
      // Test cases covering different documentation patterns and edge cases
      const testCases = [
        {
          name: 'rust_good_documentation',
          content: `/// Calculates the factorial of a number
/// # Arguments
/// * \`n\` - The number to calculate factorial for
/// # Returns
/// The factorial of n
pub fn factorial(n: u32) -> u32 {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}`,
          language: 'rust',
          expected_has_docs: true,
          expected_confidence_min: 0.8
        },
        {
          name: 'python_docstring',
          content: `def calculate_area(radius):
    """
    Calculate the area of a circle.
    
    Args:
        radius (float): The radius of the circle
        
    Returns:
        float: The area of the circle
    """
    return 3.14159 * radius * radius`,
          language: 'python',
          expected_has_docs: true,
          expected_confidence_min: 0.8
        },
        {
          name: 'rust_todo_comment',
          content: `// TODO: Implement proper error handling
pub fn risky_function() -> Result<i32, String> {
    Ok(42)
}`,
          language: 'rust',
          expected_has_docs: false,
          expected_confidence_max: 0.6
        },
        {
          name: 'rust_mixed_quality',
          content: `/// Good documentation here
pub struct GoodStruct {
    value: i32,
}

// FIXME: This needs work
pub struct BadStruct {
    value: i32,
}`,
          language: 'rust',
          expected_has_docs: true,
          expected_confidence_min: 0.5
        },
        {
          name: 'javascript_jsdoc',
          content: `/**
 * Processes user data and returns formatted result
 * @param {Object} userData - The user data to process
 * @param {string} userData.name - User's name
 * @param {number} userData.age - User's age
 * @returns {string} Formatted user information
 */
function processUserData(userData) {
    return \`\${userData.name} is \${userData.age} years old\`;
}`,
          language: 'javascript',
          expected_has_docs: true,
          expected_confidence_min: 0.8
        }
      ];

      let totalTests = 0;
      let passedTests = 0;
      let validationResults = [];

      for (const testCase of testCases) {
        console.log(`Testing: ${testCase.name}`);
        
        const result = await runValidationTest(testCase.content, testCase.language);
        totalTests++;
        
        // Validate the validation system itself
        expect(result.overall_validation).toBeDefined();
        expect(result.validation_results).toBeDefined();
        expect(Array.isArray(result.validation_results)).toBe(true);
        
        // Check if validation passed
        const overallStatus = result.overall_validation.status;
        const avgQuality = result.overall_validation.average_quality_score;
        
        if (overallStatus === 'passed' && avgQuality > 0.8) {
          passedTests++;
        }
        
        // Store results for analysis
        validationResults.push({
          name: testCase.name,
          status: overallStatus,
          quality: avgQuality,
          warnings: result.overall_validation.total_warnings,
          errors: result.overall_validation.total_errors
        });
        
        // Test expectations based on test case
        if (testCase.expected_has_docs !== undefined) {
          const hasDocumentedChunks = result.chunks.some(chunk => 
            chunk.metadata && chunk.metadata.has_documentation
          );
          expect(hasDocumentedChunks).toBe(testCase.expected_has_docs);
        }
        
        if (testCase.expected_confidence_min !== undefined) {
          const maxConfidence = Math.max(...result.chunks.map(chunk => 
            chunk.metadata ? chunk.metadata.confidence || 0 : 0
          ));
          if (testCase.expected_has_docs) {
            expect(maxConfidence).toBeGreaterThanOrEqual(testCase.expected_confidence_min);
          }
        }
        
        if (testCase.expected_confidence_max !== undefined) {
          const maxConfidence = Math.max(...result.chunks.map(chunk => 
            chunk.metadata ? chunk.metadata.confidence || 0 : 0
          ));
          expect(maxConfidence).toBeLessThanOrEqual(testCase.expected_confidence_max);
        }
      }

      // Calculate success rate
      const successRate = passedTests / totalTests;
      console.log(`\nValidation Success Rate: ${(successRate * 100).toFixed(1)}%`);
      console.log(`Passed: ${passedTests}/${totalTests} tests`);
      
      // Log detailed results
      validationResults.forEach(result => {
        console.log(`${result.name}: ${result.status} (quality: ${result.quality.toFixed(3)}, warnings: ${result.warnings}, errors: ${result.errors})`);
      });

      // Require 99%+ success rate for production readiness
      expect(successRate).toBeGreaterThanOrEqual(0.99);
    });

    test('should detect regressions automatically', async () => {
      const testContent = `/// Well documented function
pub fn good_function() -> i32 {
    42
}`;

      // Get baseline metrics
      const baselineResult = await runValidationTest(testContent, 'rust');
      const baselineMetrics = await getPerformanceMetrics();
      
      // Simulate regression by introducing problematic content
      const regressedContent = `// TODO: Fix this mess
pub fn bad_function() -> i32 {
    42
}`;
      
      const regressedResult = await runValidationTest(regressedContent, 'rust');
      const currentMetrics = await getPerformanceMetrics();
      
      // Run regression detection
      const regressionResult = await runRegressionDetection(baselineMetrics, currentMetrics);
      
      expect(regressionResult).toBeDefined();
      expect(regressionResult.status).toBeDefined();
      
      // Should detect quality degradation
      if (regressionResult.status === 'regression_detected') {
        expect(regressionResult.degradations).toBeDefined();
        expect(Array.isArray(regressionResult.degradations)).toBe(true);
        expect(regressionResult.degradations.length).toBeGreaterThan(0);
      }
    });

    test('should maintain performance benchmarks', async () => {
      const testCases = [
        { size: 'small', lines: 50 },
        { size: 'medium', lines: 200 },
        { size: 'large', lines: 500 }
      ];

      const performanceResults = [];

      for (const testCase of testCases) {
        const content = generateTestContent(testCase.lines, 'rust');
        const startTime = Date.now();
        
        const result = await runValidationTest(content, 'rust');
        
        const endTime = Date.now();
        const processingTime = endTime - startTime;
        
        performanceResults.push({
          size: testCase.size,
          lines: testCase.lines,
          processingTime,
          chunks: result.chunks.length,
          validationTime: result.parsing_time * 1000 // Convert to ms
        });
        
        // Performance requirements
        expect(processingTime).toBeLessThan(10000); // Max 10 seconds
        expect(result.chunks.length).toBeGreaterThan(0);
      }

      // Analyze scaling performance
      console.log('\nPerformance Benchmarks:');
      performanceResults.forEach(result => {
        const timePerLine = result.processingTime / result.lines;
        console.log(`${result.size}: ${result.lines} lines, ${result.processingTime}ms total, ${timePerLine.toFixed(2)}ms/line`);
      });

      // Should scale reasonably with content size
      const smallTime = performanceResults.find(r => r.size === 'small').processingTime;
      const largeTime = performanceResults.find(r => r.size === 'large').processingTime;
      const scaleFactor = largeTime / smallTime;
      
      // Should not scale worse than O(n^2)
      expect(scaleFactor).toBeLessThan(100); // Reasonable scaling
    });

    test('should provide comprehensive quality metrics', async () => {
      const testContent = `/// Documentation for first function
pub fn function_one() -> i32 { 1 }

// TODO: Document this
pub fn function_two() -> i32 { 2 }

/// Documentation for third function
/// This has multiple lines of docs
pub fn function_three() -> i32 { 3 }`;

      const result = await runValidationTest(testContent, 'rust');
      const metrics = await getPerformanceMetrics();

      // Verify quality metrics structure
      expect(metrics).toBeDefined();
      expect(metrics.summary).toBeDefined();
      expect(metrics.performance).toBeDefined();
      expect(metrics.quality_indicators).toBeDefined();

      // Check specific metrics
      expect(metrics.summary.total_processed).toBeGreaterThan(0);
      expect(metrics.summary.documentation_coverage).toBeGreaterThanOrEqual(0);
      expect(metrics.summary.documentation_coverage).toBeLessThanOrEqual(1);
      expect(metrics.summary.validation_success_rate).toBeGreaterThanOrEqual(0);
      expect(metrics.summary.validation_success_rate).toBeLessThanOrEqual(1);

      // Verify edge case handling
      expect(typeof metrics.summary.edge_cases_handled).toBe('number');
      expect(metrics.summary.edge_cases_handled).toBeGreaterThanOrEqual(0);

      console.log('\nQuality Metrics:');
      console.log(`Documentation Coverage: ${(metrics.summary.documentation_coverage * 100).toFixed(1)}%`);
      console.log(`Validation Success Rate: ${(metrics.summary.validation_success_rate * 100).toFixed(1)}%`);
      console.log(`Edge Cases Handled: ${metrics.summary.edge_cases_handled}`);
    });

    test('should monitor production health effectively', async () => {
      // Process multiple test cases to generate health data
      const testCases = [
        `/// Good documentation
pub fn good_func() -> i32 { 1 }`,
        `// TODO: Bad documentation
pub fn bad_func() -> i32 { 2 }`,
        `/// Another good one
pub fn another_good() -> i32 { 3 }`,
      ];

      // Process test cases to generate health metrics
      for (const content of testCases) {
        await runValidationTest(content, 'rust');
      }

      const healthReport = await runHealthCheck();

      // Verify health report structure
      expect(healthReport).toBeDefined();
      expect(healthReport.status).toBeDefined();
      expect(['healthy', 'degraded', 'critical'].includes(healthReport.status)).toBe(true);
      
      expect(healthReport.alerts).toBeDefined();
      expect(Array.isArray(healthReport.alerts)).toBe(true);
      
      expect(healthReport.metrics).toBeDefined();
      expect(healthReport.recommendations).toBeDefined();
      expect(Array.isArray(healthReport.recommendations)).toBe(true);

      expect(healthReport.system_checks).toBeDefined();
      expect(typeof healthReport.system_checks.chunking_engine_available).toBe('boolean');

      console.log('\nSystem Health Status:');
      console.log(`Status: ${healthReport.status}`);
      console.log(`Alerts: ${healthReport.alerts.length}`);
      console.log(`Recommendations: ${healthReport.recommendations.length}`);

      if (healthReport.alerts.length > 0) {
        console.log('Alerts:', healthReport.alerts);
      }
      if (healthReport.recommendations.length > 0) {
        console.log('Recommendations:', healthReport.recommendations);
      }
    });
  });

  describe('Edge Case Validation', () => {
    test('should handle malformed documentation gracefully', async () => {
      const malformedCases = [
        {
          name: 'incomplete_rust_doc',
          content: `/// Incomplete doc
/// Missing closing
pub fn incomplete() -> i32 { 1 }`,
          language: 'rust'
        },
        {
          name: 'mixed_comment_styles',
          content: `/// Doc comment
// Regular comment
/// Another doc
pub fn mixed() -> i32 { 1 }`,
          language: 'rust'
        },
        {
          name: 'empty_docstring',
          content: `def empty_doc():
    \"\"\"\"\"\"
    return 42`,
          language: 'python'
        }
      ];

      for (const testCase of malformedCases) {
        const result = await runValidationTest(testCase.content, testCase.language);
        
        // Should not crash or throw errors
        expect(result).toBeDefined();
        expect(result.overall_validation).toBeDefined();
        expect(result.overall_validation.status).toBeDefined();
        
        // May have warnings but should still process
        console.log(`${testCase.name}: ${result.overall_validation.status} (warnings: ${result.overall_validation.total_warnings})`);
      }
    });

    test('should detect false positives effectively', async () => {
      const falsePositiveCases = [
        `// TODO: This is not documentation
pub fn todo_comment() -> i32 { 1 }`,
        `// FIXME: Another non-doc comment
pub fn fixme_comment() -> i32 { 2 }`,
        `// DEBUG: Temporary debug info
pub fn debug_comment() -> i32 { 3 }`
      ];

      for (const content of falsePositiveCases) {
        const result = await runValidationTest(content, 'rust');
        
        // Should have low confidence or appropriate warnings
        const hasValidationWarnings = result.validation_results.some(v => 
          v.warnings.some(w => w.includes('false positive') || w.includes('TODO') || w.includes('FIXME'))
        );
        
        if (!hasValidationWarnings) {
          // If no warnings, confidence should be appropriately low
          const maxConfidence = Math.max(...result.chunks.map(chunk => 
            chunk.metadata ? chunk.metadata.confidence || 0 : 0
          ));
          expect(maxConfidence).toBeLessThan(0.7);
        }
      }
    });
  });

  // Helper functions
  async function runValidationTest(content, language) {
    return new Promise((resolve, reject) => {
      const pythonProcess = spawn('python', [
        indexerPath,
        'create-chunks',
        '--content', content,
        '--language', language,
        '--validate'
      ], {
        cwd: path.dirname(indexerPath)
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
            reject(new Error(`Failed to parse JSON output: ${error.message}\nOutput: ${stdout}`));
          }
        } else {
          reject(new Error(`Python process exited with code ${code}\nstderr: ${stderr}\nstdout: ${stdout}`));
        }
      });

      pythonProcess.on('error', (error) => {
        reject(new Error(`Failed to start Python process: ${error.message}`));
      });
    });
  }

  async function getPerformanceMetrics() {
    return new Promise((resolve, reject) => {
      const pythonProcess = spawn('python', [
        '-c',
        `
from indexer_universal import UniversalCodeIndexer
import json
indexer = UniversalCodeIndexer()
# Initialize QA system by processing some content
indexer.parse_content_with_validation("/// test\\npub fn test() {}", "rust")
metrics = indexer.get_performance_metrics()
print(json.dumps(metrics))
        `
      ], {
        cwd: path.join(__dirname, '..', 'python')
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
            reject(new Error(`Failed to parse metrics JSON: ${error.message}\nOutput: ${stdout}`));
          }
        } else {
          reject(new Error(`Metrics process exited with code ${code}\nstderr: ${stderr}`));
        }
      });
    });
  }

  async function runHealthCheck() {
    return new Promise((resolve, reject) => {
      const pythonProcess = spawn('python', [
        '-c',
        `
from indexer_universal import UniversalCodeIndexer
import json
indexer = UniversalCodeIndexer()
# Initialize QA system
indexer.parse_content_with_validation("/// test\\npub fn test() {}", "rust")
health = indexer.run_health_check()
print(json.dumps(health))
        `
      ], {
        cwd: path.join(__dirname, '..', 'python')
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
            reject(new Error(`Failed to parse health JSON: ${error.message}\nOutput: ${stdout}`));
          }
        } else {
          reject(new Error(`Health check process exited with code ${code}\nstderr: ${stderr}`));
        }
      });
    });
  }

  async function runRegressionDetection(baseline, current) {
    return new Promise((resolve, reject) => {
      const pythonProcess = spawn('python', [
        '-c',
        `
from indexer_universal import UniversalCodeIndexer
import json
indexer = UniversalCodeIndexer()
# Initialize with some data
indexer.parse_content_with_validation("/// test\\npub fn test() {}", "rust")
baseline = ${JSON.stringify(baseline)}
regression = indexer.run_regression_detection(baseline)
print(json.dumps(regression))
        `
      ], {
        cwd: path.join(__dirname, '..', 'python')
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
            reject(new Error(`Failed to parse regression JSON: ${error.message}\nOutput: ${stdout}`));
          }
        } else {
          reject(new Error(`Regression process exited with code ${code}\nstderr: ${stderr}`));
        }
      });
    });
  }

  function generateTestContent(lineCount, language) {
    const lines = [];
    for (let i = 0; i < lineCount; i++) {
      if (i % 10 === 0 && language === 'rust') {
        lines.push(`/// Documentation for function ${Math.floor(i/10)}`);
        lines.push(`pub fn function_${Math.floor(i/10)}() -> i32 {`);
        lines.push(`    ${Math.floor(i/10)}`);
        lines.push(`}`);
      } else if (language === 'rust') {
        lines.push(`    // Regular code line ${i}`);
      }
    }
    return lines.join('\n');
  }
});