/**
 * TDD Tests for Code Indexing Functionality
 * These tests MUST FAIL initially - we're implementing indexer_universal.py to make them pass
 */

const path = require('path');
const fs = require('fs').promises;
const { spawn } = require('child_process');
const tmp = require('tmp');

// Set longer timeout for indexing operations
jest.setTimeout(60000);

describe('Code Indexing Functionality (TDD)', () => {
  let tempProjectDir;
  let indexerScript;
  
  beforeAll(() => {
    indexerScript = path.join(__dirname, '..', 'python', 'indexer_universal.py');
  });
  
  beforeEach(async () => {
    // Create temporary test project
    tempProjectDir = tmp.dirSync({ unsafeCleanup: true });
    
    // Create test files
    await createTestProject(tempProjectDir.name);
  });
  
  afterEach(() => {
    if (tempProjectDir) {
      tempProjectDir.removeCallback();
    }
  });

  describe('Indexer Module Existence', () => {
    test('MUST FAIL: indexer_universal.py should exist', async () => {
      const exists = await fileExists(indexerScript);
      expect(exists).toBe(true);
    });

    test('MUST FAIL: indexer should have CLI interface', async () => {
      const result = await runPythonScript([indexerScript, '--help']);
      expect(result.code).toBe(0);
      expect(result.stdout).toContain('usage');
      expect(result.stdout).toContain('indexer');
    });
  });

  describe('File Discovery', () => {
    test('MUST FAIL: should discover all code files in project', async () => {
      const result = await runPythonScript([
        indexerScript, 
        '--discover', 
        tempProjectDir.name
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.files).toBeDefined();
      expect(Array.isArray(output.files)).toBe(true);
      expect(output.files.length).toBeGreaterThan(0);
      
      // Should find our test files
      const filenames = output.files.map(f => path.basename(f));
      expect(filenames).toContain('main.py');
      expect(filenames).toContain('utils.js');
      expect(filenames).toContain('config.json');
    });

    test('MUST FAIL: should filter files by extension', async () => {
      const result = await runPythonScript([
        indexerScript, 
        '--discover', 
        tempProjectDir.name,
        '--extensions', 'py,js'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.files).toBeDefined();
      
      // Should only include .py and .js files
      const extensions = output.files.map(f => path.extname(f));
      expect(extensions.every(ext => ['.py', '.js'].includes(ext))).toBe(true);
    });

    test('MUST FAIL: should exclude common non-code directories', async () => {
      // Create some directories that should be excluded
      await fs.mkdir(path.join(tempProjectDir.name, 'node_modules'), { recursive: true });
      await fs.mkdir(path.join(tempProjectDir.name, '.git'), { recursive: true });
      await fs.mkdir(path.join(tempProjectDir.name, '__pycache__'), { recursive: true });
      
      await fs.writeFile(path.join(tempProjectDir.name, 'node_modules', 'package.js'), 'module.exports = {}');
      await fs.writeFile(path.join(tempProjectDir.name, '.git', 'config'), '[core]');
      
      const result = await runPythonScript([
        indexerScript, 
        '--discover', 
        tempProjectDir.name
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      
      // Should not include files from excluded directories
      const filePaths = output.files.join(' ');
      expect(filePaths).not.toContain('node_modules');
      expect(filePaths).not.toContain('.git');
      expect(filePaths).not.toContain('__pycache__');
    });
  });

  describe('Code Parsing and Chunking', () => {
    test('MUST FAIL: should parse Python file and extract functions', async () => {
      const result = await runPythonScript([
        indexerScript,
        '--parse',
        path.join(tempProjectDir.name, 'main.py')
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.chunks).toBeDefined();
      expect(Array.isArray(output.chunks)).toBe(true);
      
      // Should find the functions we created
      const chunkTexts = output.chunks.map(c => c.content).join(' ');
      expect(chunkTexts).toContain('def main');
      expect(chunkTexts).toContain('def process_data');
    });

    test('MUST FAIL: should parse JavaScript file and extract functions', async () => {
      const result = await runPythonScript([
        indexerScript,
        '--parse',
        path.join(tempProjectDir.name, 'utils.js')
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.chunks).toBeDefined();
      
      // Should find the functions we created
      const chunkTexts = output.chunks.map(c => c.content).join(' ');
      expect(chunkTexts).toContain('function formatData');
      expect(chunkTexts).toContain('class DataProcessor');
    });

    test('MUST FAIL: should create meaningful chunks with metadata', async () => {
      const result = await runPythonScript([
        indexerScript,
        '--parse',
        path.join(tempProjectDir.name, 'main.py')
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.chunks.length).toBeGreaterThan(0);
      
      const chunk = output.chunks[0];
      expect(chunk).toHaveProperty('content');
      expect(chunk).toHaveProperty('file_path');
      expect(chunk).toHaveProperty('start_line');
      expect(chunk).toHaveProperty('end_line');
      expect(chunk).toHaveProperty('chunk_type'); // function, class, etc.
      
      expect(typeof chunk.content).toBe('string');
      expect(chunk.content.length).toBeGreaterThan(0);
      expect(chunk.start_line).toBeGreaterThan(0);
    });
  });

  describe('Full Project Indexing', () => {
    test('MUST FAIL: should index entire project and create database', async () => {
      const indexPath = path.join(tempProjectDir.name, '.mcp_index');
      
      const result = await runPythonScript([
        indexerScript,
        '--index',
        tempProjectDir.name,
        '--output', indexPath
      ]);
      
      expect(result.code).toBe(0);
      
      // Should create index directory
      const indexExists = await fileExists(indexPath);
      expect(indexExists).toBe(true);
      
      // Should contain database files
      const dbExists = await fileExists(path.join(indexPath, 'code_index.db'));
      expect(dbExists).toBe(true);
      
      const output = JSON.parse(result.stdout);
      expect(output.indexed_files).toBeGreaterThan(0);
      expect(output.total_chunks).toBeGreaterThan(0);
    });

    test('MUST FAIL: should generate embeddings for code chunks', async () => {
      const indexPath = path.join(tempProjectDir.name, '.mcp_index');
      
      const result = await runPythonScript([
        indexerScript,
        '--index',
        tempProjectDir.name,
        '--output', indexPath,
        '--embeddings'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.embeddings_generated).toBeGreaterThan(0);
      
      // Should create embeddings file
      const embeddingsExists = await fileExists(path.join(indexPath, 'embeddings.npy'));
      expect(embeddingsExists).toBe(true);
    });

    test('MUST FAIL: should handle incremental indexing', async () => {
      const indexPath = path.join(tempProjectDir.name, '.mcp_index');
      
      // Initial indexing
      const result1 = await runPythonScript([
        indexerScript,
        '--index',
        tempProjectDir.name,
        '--output', indexPath
      ]);
      expect(result1.code).toBe(0);
      
      const output1 = JSON.parse(result1.stdout);
      const initialCount = output1.indexed_files;
      
      // Add a new file
      await fs.writeFile(
        path.join(tempProjectDir.name, 'new_file.py'),
        'def new_function():\n    return "new"'
      );
      
      // Incremental indexing
      const result2 = await runPythonScript([
        indexerScript,
        '--index',
        tempProjectDir.name,
        '--output', indexPath,
        '--incremental'
      ]);
      expect(result2.code).toBe(0);
      
      const output2 = JSON.parse(result2.stdout);
      expect(output2.indexed_files).toBeGreaterThan(initialCount);
      expect(output2.new_files).toBe(1);
    });
  });

  describe('Error Handling', () => {
    test('MUST FAIL: should handle non-existent directory gracefully', async () => {
      const result = await runPythonScript([
        indexerScript,
        '--index',
        '/non/existent/path'
      ]);
      
      expect(result.code).toBe(1);
      expect(result.stderr).toContain('directory does not exist');
    });

    test('MUST FAIL: should handle corrupted files gracefully', async () => {
      // Create a file with invalid encoding
      const corruptedFile = path.join(tempProjectDir.name, 'corrupted.py');
      await fs.writeFile(corruptedFile, Buffer.from([0xFF, 0xFE, 0x00, 0x00]));
      
      const result = await runPythonScript([
        indexerScript,
        '--parse',
        corruptedFile
      ]);
      
      // Should not crash, but may return empty results
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.chunks).toBeDefined();
      expect(Array.isArray(output.chunks)).toBe(true);
    });
  });

  describe('Performance and Scalability', () => {
    test('MUST FAIL: should handle large files efficiently', async () => {
      // Create a large file
      const largeContent = 'def function_' + Array.from({length: 1000}, (_, i) => 
        `${i}():\n    return ${i}\n\n`
      ).join('');
      
      const largeFile = path.join(tempProjectDir.name, 'large.py');
      await fs.writeFile(largeFile, largeContent);
      
      const startTime = Date.now();
      
      const result = await runPythonScript([
        indexerScript,
        '--parse',
        largeFile
      ]);
      
      const duration = Date.now() - startTime;
      
      expect(result.code).toBe(0);
      expect(duration).toBeLessThan(30000); // Should complete within 30 seconds
      
      const output = JSON.parse(result.stdout);
      expect(output.chunks.length).toBeGreaterThan(100);
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
  
  // Create README
  await fs.writeFile(
    path.join(projectPath, 'README.md'),
    `# Test Project

This is a test project for the MCP RAG indexer.

## Features

- Python main module
- JavaScript utilities
- JSON configuration
`
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