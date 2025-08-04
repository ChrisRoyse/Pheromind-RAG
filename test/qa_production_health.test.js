/**
 * Production Health Monitoring Test Suite
 * Tests the core QA and monitoring functionality
 */

const { spawn } = require('child_process');
const path = require('path');

// Set timeout for health tests
jest.setTimeout(60000);

describe('Production Health Monitoring', () => {
  let indexerPath;
  
  beforeAll(() => {
    indexerPath = path.join(__dirname, '..', 'python', 'indexer_universal.py');
  });

  describe('Core QA System Health', () => {
    test('should validate system initialization', async () => {
      const testContent = `/// Test function
pub fn test() -> i32 { 42 }`;

      const result = await runValidationTest(testContent, 'rust');
      
      // Verify basic validation structure
      expect(result).toBeDefined();
      expect(result.chunks).toBeDefined();
      expect(Array.isArray(result.chunks)).toBe(true);
      expect(result.parsing_time).toBeDefined();
      expect(typeof result.parsing_time).toBe('number');
      
      // Verify validation results structure
      expect(result.validation_results).toBeDefined();
      expect(Array.isArray(result.validation_results)).toBe(true);
      expect(result.overall_validation).toBeDefined();
      expect(result.overall_validation.status).toBeDefined();
      expect(result.overall_validation.success_rate).toBeDefined();
      
      console.log(`✓ System initialized with ${result.chunks.length} chunks`);
      console.log(`✓ Validation status: ${result.overall_validation.status}`);
      console.log(`✓ Success rate: ${(result.overall_validation.success_rate * 100).toFixed(1)}%`);
    });

    test('should handle various code quality scenarios', async () => {
      const testCases = [
        {
          name: 'Well documented',
          content: `/// Calculates something important
pub fn calculate() -> i32 { 42 }`,
          language: 'rust'
        },
        {
          name: 'Poorly documented',
          content: `// TODO: fix this later
pub fn broken() -> i32 { 0 }`,
          language: 'rust'
        },
        {
          name: 'Python docstring',
          content: `def example():
    """This is a proper docstring."""
    return 42`,
          language: 'python'
        }
      ];

      let totalTests = 0;
      let successfulTests = 0;

      for (const testCase of testCases) {
        try {
          const result = await runValidationTest(testCase.content, testCase.language);
          totalTests++;
          
          if (result.overall_validation.status === 'passed') {
            successfulTests++;
          }
          
          console.log(`${testCase.name}: ${result.overall_validation.status} (${result.chunks.length} chunks)`);
        } catch (error) {
          console.error(`Failed ${testCase.name}: ${error.message}`);
          totalTests++;
        }
      }

      const successRate = successfulTests / totalTests;
      console.log(`\nOverall success rate: ${(successRate * 100).toFixed(1)}%`);
      
      // Should handle at least 80% of test cases successfully
      expect(successRate).toBeGreaterThanOrEqual(0.8);
    });

    test('should provide performance metrics', async () => {
      // Process some content to generate metrics data
      const testContent = `/// First function
pub fn first() -> i32 { 1 }

/// Second function  
pub fn second() -> i32 { 2 }`;

      const result = await runValidationTest(testContent, 'rust');
      
      // Verify basic performance data is available
      expect(result.parsing_time).toBeDefined();
      expect(result.parsing_time).toBeGreaterThan(0);
      expect(result.parsing_time).toBeLessThan(10); // Should be fast
      
      // Verify chunks were created
      expect(result.chunks.length).toBeGreaterThan(0);
      
      console.log(`✓ Processing time: ${(result.parsing_time * 1000).toFixed(2)}ms`);
      console.log(`✓ Chunks created: ${result.chunks.length}`);
      console.log(`✓ Validation checks: ${result.validation_results.length}`);
    });

    test('should maintain system stability under load', async () => {
      const testCases = Array.from({ length: 5 }, (_, i) => ({
        content: `/// Function ${i}
pub fn func_${i}() -> i32 { ${i} }`,
        language: 'rust'
      }));

      const results = [];
      
      // Process multiple test cases
      for (const testCase of testCases) {
        const result = await runValidationTest(testCase.content, testCase.language);
        results.push(result);
      }

      // Verify all tests completed successfully
      expect(results.length).toBe(testCases.length);
      
      // Verify consistent performance
      const processingTimes = results.map(r => r.parsing_time);
      const avgTime = processingTimes.reduce((a, b) => a + b, 0) / processingTimes.length;
      const maxTime = Math.max(...processingTimes);
      
      console.log(`✓ Average processing time: ${(avgTime * 1000).toFixed(2)}ms`);
      console.log(`✓ Max processing time: ${(maxTime * 1000).toFixed(2)}ms`);
      
      // Performance should be reasonable and consistent
      expect(avgTime).toBeLessThan(1.0); // Less than 1 second average
      expect(maxTime).toBeLessThan(2.0); // Less than 2 seconds max
    });

    test('should detect edge cases appropriately', async () => {
      const edgeCases = [
        {
          name: 'Empty content',
          content: '',
          language: 'rust',
          shouldSucceed: true // Should handle gracefully
        },
        {
          name: 'Only comments',
          content: '// Just a comment\n// Another comment',
          language: 'rust',
          shouldSucceed: true
        },
        {
          name: 'Mixed content',
          content: `/// Good docs
pub fn good() {}
// TODO: bad docs
pub fn bad() {}`,
          language: 'rust',
          shouldSucceed: true
        }
      ];

      let edgeCasesHandled = 0;

      for (const edgeCase of edgeCases) {
        try {
          const result = await runValidationTest(edgeCase.content, edgeCase.language);
          
          if (edgeCase.shouldSucceed) {
            expect(result).toBeDefined();
            expect(result.overall_validation).toBeDefined();
            edgeCasesHandled++;
            console.log(`✓ ${edgeCase.name}: handled successfully`);
          }
        } catch (error) {
          if (!edgeCase.shouldSucceed) {
            console.log(`✓ ${edgeCase.name}: appropriately rejected`);
            edgeCasesHandled++;
          } else {
            console.error(`✗ ${edgeCase.name}: unexpected failure - ${error.message}`);
          }
        }
      }

      // Should handle all edge cases appropriately
      expect(edgeCasesHandled).toBe(edgeCases.length);
    });
  });

  describe('Quality Assurance Validation', () => {
    test('should validate quality metrics structure', async () => {
      const testContent = `/// Test function
pub fn test() -> i32 { 42 }`;

      const result = await runValidationTest(testContent, 'rust');
      
      // Verify validation results have proper structure
      expect(result.validation_results).toBeDefined();
      expect(Array.isArray(result.validation_results)).toBe(true);
      
      if (result.validation_results.length > 0) {
        const validation = result.validation_results[0];
        expect(validation.passed).toBeDefined();
        expect(typeof validation.passed).toBe('boolean');
        expect(validation.quality_score).toBeDefined();
        expect(typeof validation.quality_score).toBe('number');
        expect(validation.quality_score).toBeGreaterThanOrEqual(0);
        expect(validation.quality_score).toBeLessThanOrEqual(1);
      }
      
      // Verify overall validation
      expect(result.overall_validation.status).toMatch(/^(passed|failed|no_validation)$/);
      expect(result.overall_validation.success_rate).toBeGreaterThanOrEqual(0);
      expect(result.overall_validation.success_rate).toBeLessThanOrEqual(1);
      
      console.log(`✓ Validation structure verified`);
      console.log(`✓ Quality metrics: ${JSON.stringify(result.overall_validation, null, 2)}`);
    });

    test('should provide actionable recommendations', async () => {
      // Test with content that should generate warnings/recommendations
      const problematicContent = `// TODO: This needs documentation
pub fn undocumented() -> i32 { 42 }

// FIXME: Another problematic function  
pub fn another() -> i32 { 0 }`;

      const result = await runValidationTest(problematicContent, 'rust');
      
      expect(result.overall_validation).toBeDefined();
      expect(result.overall_validation.recommendations).toBeDefined();
      expect(Array.isArray(result.overall_validation.recommendations)).toBe(true);
      
      console.log(`✓ Recommendations provided: ${result.overall_validation.recommendations.length}`);
      if (result.overall_validation.recommendations.length > 0) {
        console.log(`  Recommendations: ${result.overall_validation.recommendations.join(', ')}`);
      }
    });

    test('should track validation performance', async () => {
      const testContent = `/// Performance test function
pub fn performance_test() -> i32 { 42 }`;

      const result = await runValidationTest(testContent, 'rust');
      
      // Should track timing information
      expect(result.parsing_time).toBeDefined();
      expect(typeof result.parsing_time).toBe('number');
      expect(result.parsing_time).toBeGreaterThan(0);
      
      // Validation should be fast
      const validationTime = result.parsing_time;
      expect(validationTime).toBeLessThan(5.0); // Should complete in under 5 seconds
      
      console.log(`✓ Validation performance: ${(validationTime * 1000).toFixed(2)}ms`);
    });
  });

  // Helper function
  async function runValidationTest(content, language) {
    return new Promise((resolve, reject) => {
      const pythonProcess = spawn('python', [
        indexerPath,
        'create-chunks',
        '--content', content,
        '--language', language,
        '--validate'
      ], {
        cwd: path.dirname(indexerPath),
        stdio: ['pipe', 'pipe', 'pipe']
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
            reject(new Error(`Failed to parse JSON output: ${error.message}\nOutput: ${stdout.substring(0, 500)}...`));
          }
        } else {
          reject(new Error(`Python process exited with code ${code}\nstderr: ${stderr.substring(0, 500)}...\nstdout: ${stdout.substring(0, 500)}...`));
        }
      });

      pythonProcess.on('error', (error) => {
        reject(new Error(`Failed to start Python process: ${error.message}`));
      });

      // Set a timeout for the process
      setTimeout(() => {
        pythonProcess.kill();
        reject(new Error('Process timeout'));
      }, 30000);
    });
  }
});