/**
 * Test Utilities for Vector System Testing
 * 
 * This module provides helper functions for testing the MCP RAG Indexer
 * vector system. These utilities enable comprehensive testing of:
 * - Code indexing and chunking
 * - Documentation pattern detection
 * - Vector embedding accuracy
 * - Language-specific parsing
 */

const fs = require('fs').promises;
const path = require('path');
const { spawn } = require('child_process');
const tmp = require('tmp');

/**
 * Index code content using the Python MCP server
 * This function will FAIL initially as the indexing system is broken
 * 
 * @param {string} code - Source code to index
 * @param {string} language - Programming language (rust, python, javascript)
 * @returns {Promise<Object>} Indexing result with chunks and metadata
 */
async function indexCode(code, language) {
  // Create temporary file for testing
  const tempFile = tmp.fileSync({ postfix: `.${getFileExtension(language)}` });
  await fs.writeFile(tempFile.name, code);
  
  try {
    // Call the Python indexer (this will fail until implemented)
    const result = await runPythonIndexer(tempFile.name);
    return result;
  } finally {
    // Cleanup temp file
    tempFile.removeCallback();
  }
}

/**
 * Calculate documentation coverage for a codebase
 * This will return 0% initially due to broken documentation detection
 * 
 * @param {string} projectPath - Path to project directory
 * @returns {Promise<Object>} Coverage statistics
 */
async function calculateDocumentationCoverage(projectPath) {
  try {
    const result = await runPythonCommand([
      'python/indexer_universal.py',
      'analyze-coverage',
      projectPath
    ]);
    
    // Return actual results from enhanced semantic analysis
    return {
      percentage: result.percentage || 0,
      total_chunks: result.total_chunks || 0,
      documented_chunks: result.documented_chunks || 0,
      languages: result.languages || {},
      high_quality_chunks: result.high_quality_chunks || 0,
      quality_percentage: result.quality_percentage || 0,
      analysis_metadata: result.analysis_metadata || {}
    };
  } catch (error) {
    console.error('Documentation coverage analysis failed:', error.message);
    return {
      percentage: 0,
      total_chunks: 0,
      documented_chunks: 0,
      languages: {},
      error: error.message
    };
  }
}

/**
 * Get language-specific documentation patterns
 * Calls Python function to get compiled regex patterns
 * 
 * @param {string} language - Programming language
 * @returns {Object} Regex patterns for documentation detection
 */
async function getLanguagePatterns(language) {
  const pythonScript = path.join(__dirname, '..', '..', 'python', 'indexer_universal.py');
  
  return new Promise((resolve, reject) => {
    const child = spawn('python', ['-c', `
import sys
sys.path.append('${path.dirname(pythonScript).replace(/\\/g, '\\\\')}')
from indexer_universal import get_language_patterns
import json

try:
    patterns = get_language_patterns('${language}')
    # Convert regex objects to testable format
    result = {}
    for name, pattern in patterns.items():
        if pattern and hasattr(pattern, 'pattern'):
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

/**
 * Create chunks from source code
 * This will fail initially due to broken chunking algorithm
 * 
 * @param {string} code - Source code to chunk
 * @param {string} language - Programming language
 * @returns {Promise<Array>} Array of code chunks
 */
async function createChunks(code, language) {
  try {
    const result = await runPythonCommand([
      'python/indexer_universal.py',
      'create-chunks',
      '--language', language,
      '--content', code
    ]);
    
    // Transform result to expected format
    return result.chunks || [];
  } catch (error) {
    // Expected to fail - chunking algorithm is broken
    throw new Error(`Chunking failed: ${error.message}`);
  }
}

/**
 * Create test Rust project with documentation
 * Generates sample Rust code for testing documentation detection
 * 
 * @returns {Promise<string>} Path to temporary test project
 */
async function createTestRustProject() {
  const tempDir = tmp.dirSync({ unsafeCleanup: true });
  const projectPath = tempDir.name;
  
  // Create sample Rust files with various documentation patterns
  const rustFiles = {
    'lib.rs': `
//! This is a module-level documentation comment.
//! It describes the entire module and its purpose.
//! This crate implements neural network components for spiking networks.

/// A spiking cortical column with TTFS dynamics.
/// This struct represents a biologically-inspired cortical column
/// that processes temporal information using time-to-first-spike encoding.
pub struct SpikingCorticalColumn {
    /// The current activation level of the column
    activation_level: f64,
    /// Threshold for spike generation
    spike_threshold: f64,
}

/// Implementation of core functionality for SpikingCorticalColumn
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
    
    /// Updates the activation level based on input stimulus.
    /// This method processes input and updates internal state.
    pub fn update(&mut self, input: f64) {
        self.activation_level += input;
    }
    
    /// Gets the current activation level.
    /// Returns the current activation state of the column.
    pub fn get_activation(&self) -> f64 {
        self.activation_level
    }
}`,
    
    'utils.rs': `
/// Utility functions for neural processing and spike calculations
pub mod neural_utils {
    /// Calculate the time-to-first-spike for a given input stimulus.
    /// 
    /// This function computes when a neuron will fire based on current input
    /// and threshold values using standard TTFS encoding.
    pub fn calculate_ttfs(input: f64, threshold: f64) -> Option<f64> {
        if input >= threshold {
            Some(threshold / input)
        } else {
            None
        }
    }
}

/// Helper function for utility operations.
/// Provides basic functionality for testing and debugging.
pub fn helper_function() -> i32 {
    42
}

/// Mathematical constants used in neural calculations
pub mod constants {
    /// Standard spike threshold for cortical neurons
    pub const DEFAULT_THRESHOLD: f64 = 1.0;
    
    /// Time constant for membrane decay
    pub const TAU_MEMBRANE: f64 = 10.0;
}`,
    
    'main.rs': `
//! Main entry point for the spiking neural network demo.
//! This demonstrates the usage of cortical columns in neural processing.

use crate::SpikingCorticalColumn;

/// Main function that demonstrates cortical column usage.
/// This creates a column, processes input, and displays results.
fn main() {
    let mut column = SpikingCorticalColumn::new();
    column.update(0.5);
    println!("Column activation: {}", column.get_activation());
}`
  };
  
  // Write all files
  for (const [filename, content] of Object.entries(rustFiles)) {
    await fs.writeFile(path.join(projectPath, filename), content);
  }
  
  return projectPath;
}

/**
 * Run Python indexer command
 * This will fail initially as the Python components are not properly integrated
 * 
 * @param {string} filePath - Path to file to index
 * @returns {Promise<Object>} Indexing result
 */
async function runPythonIndexer(filePath) {
  return new Promise((resolve, reject) => {
    const pythonPath = path.join(process.cwd(), 'python', 'indexer_universal.py');
    const child = spawn('python', [pythonPath, 'index', filePath], {
      stdio: ['pipe', 'pipe', 'pipe'],
      cwd: process.cwd()
    });
    
    let stdout = '';
    let stderr = '';
    
    child.stdout.on('data', (data) => {
      stdout += data.toString();
    });
    
    child.stderr.on('data', (data) => {
      stderr += data.toString();
    });
    
    child.on('close', (code) => {
      if (code !== 0) {
        reject(new Error(`Python indexer failed: ${stderr}`));
      } else {
        try {
          const result = JSON.parse(stdout);
          resolve(result);
        } catch (parseError) {
          reject(new Error(`Failed to parse indexer output: ${parseError.message}`));
        }
      }
    });
    
    child.on('error', (error) => {
      reject(new Error(`Failed to spawn Python process: ${error.message}`));
    });
  });
}

/**
 * Run generic Python command
 * 
 * @param {Array<string>} args - Command line arguments
 * @returns {Promise<Object>} Command result
 */
async function runPythonCommand(args) {
  return new Promise((resolve, reject) => {
    const child = spawn('python', args, {
      stdio: ['pipe', 'pipe', 'pipe'],
      cwd: process.cwd()
    });
    
    let stdout = '';
    let stderr = '';
    
    child.stdout.on('data', (data) => {
      stdout += data.toString();
    });
    
    child.stderr.on('data', (data) => {
      stderr += data.toString();
    });
    
    child.on('close', (code) => {
      if (code !== 0) {
        reject(new Error(`Python command failed: ${stderr}`));
      } else {
        try {
          const result = JSON.parse(stdout || '{}');
          resolve(result);
        } catch (parseError) {
          // If not JSON, return raw output
          resolve({ output: stdout });
        }
      }
    });
    
    child.on('error', (error) => {
      reject(new Error(`Failed to spawn Python process: ${error.message}`));
    });
  });
}

/**
 * Get file extension for programming language
 * 
 * @param {string} language - Programming language
 * @returns {string} File extension
 */
function getFileExtension(language) {
  const extensions = {
    rust: 'rs',
    python: 'py',
    javascript: 'js',
    typescript: 'ts',
    java: 'java',
    cpp: 'cpp',
    c: 'c'
  };
  
  return extensions[language] || 'txt';
}

/**
 * Performance measurement utility
 * 
 * @param {Function} fn - Function to measure
 * @returns {Promise<Object>} Result with timing information
 */
async function measurePerformance(fn) {
  const startTime = process.hrtime.bigint();
  const result = await fn();
  const endTime = process.hrtime.bigint();
  
  return {
    result,
    duration_ms: Number(endTime - startTime) / 1000000,
    duration_ns: Number(endTime - startTime)
  };
}

/**
 * Assert that chunks contain expected documentation
 * This helper will make test failures more readable
 * 
 * @param {Array} chunks - Chunks to validate
 * @param {Array} expectedDocs - Expected documentation patterns
 */
function assertDocumentationPresent(chunks, expectedDocs) {
  const documentedChunks = chunks.filter(chunk => chunk.has_documentation);
  
  if (documentedChunks.length === 0) {
    throw new Error('No documented chunks found - documentation detection is broken');
  }
  
  for (const expectedDoc of expectedDocs) {
    const found = documentedChunks.some(chunk => 
      chunk.content.includes(expectedDoc)
    );
    
    if (!found) {
      throw new Error(`Expected documentation pattern not found: "${expectedDoc}"`);
    }
  }
}

module.exports = {
  // Core functionality
  indexCode,
  calculateDocumentationCoverage,
  getLanguagePatterns,
  createChunks,
  
  // Test data creation
  createTestRustProject,
  
  // Utilities
  measurePerformance,
  assertDocumentationPresent,
  getFileExtension,
  
  // Internal functions (exported for testing)
  runPythonIndexer,
  runPythonCommand
};