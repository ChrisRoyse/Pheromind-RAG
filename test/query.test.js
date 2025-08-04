/**
 * TDD Tests for Code Query Functionality
 * These tests MUST FAIL initially - we're implementing query_universal.py to make them pass
 */

const path = require('path');
const fs = require('fs').promises;
const { spawn } = require('child_process');
const tmp = require('tmp');

// Set longer timeout for query operations
jest.setTimeout(60000);

describe('Code Query Functionality (TDD)', () => {
  let tempProjectDir;
  let indexerScript;
  let queryScript;
  let indexPath;
  
  beforeAll(() => {
    indexerScript = path.join(__dirname, '..', 'python', 'indexer_universal.py');
    queryScript = path.join(__dirname, '..', 'python', 'query_universal.py');
  });
  
  beforeEach(async () => {
    // Create temporary test project
    tempProjectDir = tmp.dirSync({ unsafeCleanup: true });
    indexPath = path.join(tempProjectDir.name, '.mcp_index');
    
    // Create test files
    await createTestProject(tempProjectDir.name);
    
    // Index the project first
    const indexResult = await runPythonScript([
      indexerScript,
      '--index',
      tempProjectDir.name,
      '--output', indexPath
    ]);
    expect(indexResult.code).toBe(0);
  });
  
  afterEach(() => {
    if (tempProjectDir) {
      tempProjectDir.removeCallback();
    }
  });

  describe('Query Module Existence', () => {
    test('MUST FAIL: query_universal.py should exist', async () => {
      const exists = await fileExists(queryScript);
      expect(exists).toBe(true);
    });

    test('MUST FAIL: query should have CLI interface', async () => {
      const result = await runPythonScript([queryScript, '--help']);
      expect(result.code).toBe(0);
      expect(result.stdout).toContain('usage');
      expect(result.stdout).toContain('query');
    });
  });

  describe('Semantic Code Search', () => {
    test('MUST FAIL: should find functions by semantic description', async () => {
      const result = await runPythonScript([
        queryScript,
        '--search',
        'function that processes data',
        '--index', indexPath
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.results).toBeDefined();
      expect(Array.isArray(output.results)).toBe(true);
      expect(output.results.length).toBeGreaterThan(0);
      
      // Should find the process_data function
      const resultTexts = output.results.map(r => r.content).join(' ');
      expect(resultTexts).toContain('process_data');
    });

    test('MUST FAIL: should find classes by description', async () => {
      const result = await runPythonScript([
        queryScript,
        '--search',
        'class for processing data',
        '--index', indexPath
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.results.length).toBeGreaterThan(0);
      
      // Should find the DataProcessor class
      const resultTexts = output.results.map(r => r.content).join(' ');
      expect(resultTexts).toContain('DataProcessor');
    });

    test('MUST FAIL: should rank results by relevance', async () => {
      const result = await runPythonScript([
        queryScript,
        '--search',
        'main entry point',
        '--index', indexPath,
        '--limit', '5'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.results.length).toBeGreaterThan(0);
      
      // Results should have scores
      for (const result of output.results) {
        expect(result).toHaveProperty('score');
        expect(typeof result.score).toBe('number');
        expect(result.score).toBeGreaterThan(0);
        expect(result.score).toBeLessThanOrEqual(1);
      }
      
      // Results should be ordered by score (descending)
      for (let i = 1; i < output.results.length; i++) {
        expect(output.results[i-1].score).toBeGreaterThanOrEqual(output.results[i].score);
      }
    });
  });

  describe('Keyword-based Search', () => {
    test('MUST FAIL: should support exact keyword matching', async () => {
      const result = await runPythonScript([
        queryScript,
        '--keyword',
        'def main',
        '--index', indexPath
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.results.length).toBeGreaterThan(0);
      
      // Should find exact matches
      const found = output.results.some(r => r.content.includes('def main'));
      expect(found).toBe(true);
    });

    test('MUST FAIL: should support regex patterns', async () => {
      const result = await runPythonScript([
        queryScript,
        '--regex',
        'def \\w+\\(',
        '--index', indexPath
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.results.length).toBeGreaterThan(0);
      
      // Should find function definitions
      const found = output.results.some(r => /def \w+\(/.test(r.content));
      expect(found).toBe(true);
    });
  });

  describe('Filter and Scope Options', () => {
    test('MUST FAIL: should filter by file type', async () => {
      const result = await runPythonScript([
        queryScript,
        '--search',
        'function',
        '--index', indexPath,
        '--file-type', 'py'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.results.length).toBeGreaterThan(0);
      
      // All results should be from Python files
      for (const result of output.results) {
        expect(result.file_path.endsWith('.py')).toBe(true);
      }
    });

    test('MUST FAIL: should filter by chunk type', async () => {
      const result = await runPythonScript([
        queryScript,
        '--search',
        'code',
        '--index', indexPath,
        '--chunk-type', 'function'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.results.length).toBeGreaterThan(0);
      
      // All results should be functions
      for (const result of output.results) {
        expect(result.chunk_type).toBe('function');
      }
    });

    test('MUST FAIL: should limit result count', async () => {
      const result = await runPythonScript([
        queryScript,
        '--search',
        'code',
        '--index', indexPath,
        '--limit', '3'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.results.length).toBeLessThanOrEqual(3);
    });
  });

  describe('Advanced Query Features', () => {
    test('MUST FAIL: should support combining multiple search terms', async () => {
      const result = await runPythonScript([
        queryScript,
        '--search',
        'data processing function',
        '--index', indexPath
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.results.length).toBeGreaterThan(0);
      
      // Should find relevant results
      const resultTexts = output.results.map(r => r.content.toLowerCase()).join(' ');
      expect(resultTexts).toMatch(/(data|process)/);
    });

    test('MUST FAIL: should provide context around matches', async () => {
      const result = await runPythonScript([
        queryScript,
        '--keyword',
        'def process_data',
        '--index', indexPath,
        '--context', '2'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.results.length).toBeGreaterThan(0);
      
      const result0 = output.results[0];
      expect(result0.content.split('\n').length).toBeGreaterThan(1);
    });

    test('MUST FAIL: should highlight matching terms', async () => {
      const result = await runPythonScript([
        queryScript,
        '--keyword',
        'main',
        '--index', indexPath,
        '--highlight'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.results.length).toBeGreaterThan(0);
      
      // Should have highlighting information
      const result0 = output.results[0];
      expect(result0).toHaveProperty('highlights');
      expect(Array.isArray(result0.highlights)).toBe(true);
    });
  });

  describe('Performance and Optimization', () => {
    test('MUST FAIL: should cache embeddings for faster queries', async () => {
      // First query (should generate embeddings)
      const start1 = Date.now();
      const result1 = await runPythonScript([
        queryScript,
        '--search',
        'test query',
        '--index', indexPath
      ]);
      const time1 = Date.now() - start1;
      
      expect(result1.code).toBe(0);
      
      // Second query (should use cached embeddings)
      const start2 = Date.now();
      const result2 = await runPythonScript([
        queryScript,
        '--search',
        'another test query',
        '--index', indexPath
      ]);
      const time2 = Date.now() - start2;
      
      expect(result2.code).toBe(0);
      
      // Second query should be faster (allowing for variance)
      expect(time2).toBeLessThan(time1 + 1000); // Within 1 second difference
    });

    test('MUST FAIL: should handle large result sets efficiently', async () => {
      const result = await runPythonScript([
        queryScript,
        '--search',
        'function',
        '--index', indexPath,
        '--limit', '100'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.query_time).toBeDefined();
      expect(typeof output.query_time).toBe('number');
      expect(output.query_time).toBeLessThan(10); // Should complete within 10 seconds
    });
  });

  describe('Error Handling', () => {
    test('MUST FAIL: should handle missing index gracefully', async () => {
      const result = await runPythonScript([
        queryScript,
        '--search',
        'test',
        '--index', '/non/existent/path'
      ]);
      
      expect(result.code).toBe(1);
      expect(result.stderr).toContain('index not found');
    });

    test('MUST FAIL: should handle empty queries gracefully', async () => {
      const result = await runPythonScript([
        queryScript,
        '--search',
        '',
        '--index', indexPath
      ]);
      
      expect(result.code).toBe(1);
      expect(result.stderr).toContain('query cannot be empty');
    });

    test('MUST FAIL: should handle corrupted index gracefully', async () => {
      // Corrupt the database
      await fs.writeFile(path.join(indexPath, 'code_index.db'), 'corrupted data');
      
      const result = await runPythonScript([
        queryScript,
        '--search',
        'test',
        '--index', indexPath
      ]);
      
      expect(result.code).toBe(1);
      expect(result.stderr).toContain('database');
    });
  });

  describe('Output Formats', () => {
    test('MUST FAIL: should support JSON output format', async () => {
      const result = await runPythonScript([
        queryScript,
        '--search',
        'function',
        '--index', indexPath,
        '--format', 'json'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.results).toBeDefined();
      expect(output.query).toBeDefined();
      expect(output.total_results).toBeDefined();
    });

    test('MUST FAIL: should support plain text output format', async () => {
      const result = await runPythonScript([
        queryScript,
        '--search',
        'function',
        '--index', indexPath,
        '--format', 'text'
      ]);
      
      expect(result.code).toBe(0);
      
      // Should be plain text (not JSON)
      expect(() => JSON.parse(result.stdout)).toThrow();
      expect(result.stdout).toContain('Results:');
    });
  });
});

// Helper Functions
async function createTestProject(projectPath) {
  // Create Python file
  await fs.writeFile(
    path.join(projectPath, 'main.py'),
    `#!/usr/bin/env python3
"""
Main module for test project
"""

import json
from pathlib import Path

def main():
    """Main entry point"""
    data = load_config()
    result = process_data(data)
    return result

def process_data(data):
    """Process the configuration data"""
    processed = {}
    for key, value in data.items():
        if isinstance(value, str):
            processed[key] = value.upper()
        else:
            processed[key] = value
    return processed

def load_config():
    """Load configuration from file"""
    config_path = Path(__file__).parent / 'config.json'
    with open(config_path) as f:
        return json.load(f)

if __name__ == '__main__':
    main()
`
  );
  
  // Create JavaScript file
  await fs.writeFile(
    path.join(projectPath, 'utils.js'),
    `/**
 * Utility functions for data processing
 */

function formatData(data) {
    if (typeof data === 'string') {
        return data.trim().toLowerCase();
    }
    return data;
}

class DataProcessor {
    constructor(options = {}) {
        this.options = options;
    }
    
    process(input) {
        return formatData(input);
    }
    
    validate(data) {
        return data !== null && data !== undefined;
    }
}

module.exports = {
    formatData,
    DataProcessor
};
`
  );
  
  // Create configuration file
  await fs.writeFile(
    path.join(projectPath, 'config.json'),
    JSON.stringify({
      name: "test-project",
      version: "1.0.0",
      settings: {
        debug: true,
        timeout: 5000
      }
    }, null, 2)
  );
}

async function fileExists(filePath) {
  try {
    await fs.access(filePath);
    return true;
  } catch {
    return false;
  }
}

async function runPythonScript(args, options = {}) {
  return new Promise((resolve, reject) => {
    // Use system Python for testing
    const pythonExe = 'python';
    
    const child = spawn(pythonExe, args, {
      stdio: 'pipe',
      timeout: 30000,
      ...options
    });

    let stdout = '';
    let stderr = '';

    child.stdout.on('data', (data) => {
      stdout += data.toString();
    });

    child.stderr.on('data', (data) => {
      stderr += data.toString();
    });

    child.on('exit', (code) => {
      resolve({ code, stdout, stderr });
    });

    child.on('error', reject);

    setTimeout(() => {
      child.kill();
      reject(new Error('Script timeout'));
    }, 30000);
  });
}