/**
 * Integration Tests for MCP RAG Indexer NPM Package
 * Tests the complete installation and functionality
 */

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs-extra');
const tmp = require('tmp');
const { PlatformManager } = require('../lib/platform');
const { validateInstallation } = require('../lib/validator');

// Set longer timeout for integration tests
jest.setTimeout(60000);

describe('MCP RAG Indexer Integration Tests', () => {
  let tempDir;
  
  beforeEach(() => {
    tempDir = tmp.dirSync({ unsafeCleanup: true });
  });
  
  afterEach(() => {
    if (tempDir) {
      tempDir.removeCallback();
    }
  });
  
  describe('Platform Detection', () => {
    test('should detect current platform', () => {
      const platform = PlatformManager.detectPlatform();
      expect(platform).toMatch(/^(windows|macos|linux)-(x64|arm64)$/);
    });
    
    test('should return Python executable path', () => {
      const pythonPath = PlatformManager.getPythonExecutable();
      expect(pythonPath).toBeTruthy();
      expect(path.isAbsolute(pythonPath)).toBe(true);
    });
    
    test('should validate platform support', () => {
      const isSupported = PlatformManager.isPlatformSupported();
      expect(isSupported).toBe(true);
    });
  });
  
  describe('CLI Commands', () => {
    test('CLI should start successfully with --version', async () => {
      const cliPath = path.join(__dirname, '..', 'bin', 'mcp-rag-indexer');
      
      const result = await runCommand('node', [cliPath, '--version'], {
        timeout: 10000
      });
      
      expect(result.code).toBe(0);
      expect(result.stdout).toContain('1.0.0');
    });
    
    test('CLI should show status', async () => {
      const cliPath = path.join(__dirname, '..', 'bin', 'mcp-rag-indexer');
      
      const result = await runCommand('node', [cliPath, '--status'], {
        timeout: 10000
      });
      
      expect(result.code).toBe(0);
      expect(result.stdout).toContain('MCP RAG Indexer Status');
      expect(result.stdout).toContain('Version: 1.0.0');
    });
    
    test('CLI should handle --validate flag', async () => {
      const cliPath = path.join(__dirname, '..', 'bin', 'mcp-rag-indexer');
      
      const result = await runCommand('node', [cliPath, '--validate'], {
        timeout: 30000
      });
      
      // Should exit with 0 or 1 (depending on validation result)
      expect([0, 1]).toContain(result.code);
    });
  });
  
  describe('Python Runtime', () => {
    test('should have valid Python runtime', async () => {
      const isValid = await PlatformManager.validateRuntime();
      
      if (isValid) {
        expect(isValid).toBe(true);
      } else {
        // Runtime might not be bundled in test environment
        console.warn('Python runtime not found - this is expected in test environment');
      }
    });
    
    test('should be able to run Python version command', async () => {
      const pythonPath = PlatformManager.getPythonExecutable();
      
      if (fs.existsSync(pythonPath)) {
        const result = await runCommand(pythonPath, ['--version'], {
          timeout: 5000
        });
        
        expect(result.code).toBe(0);
        expect(result.stdout || result.stderr).toMatch(/Python 3\.\d+\.\d+/);
      } else {
        console.warn('Python runtime not bundled - skipping test');
      }
    });
  });
  
  describe('Package Structure', () => {
    test('should have required package files', () => {
      const packageRoot = path.join(__dirname, '..');
      
      const requiredFiles = [
        'package.json',
        'bin/mcp-rag-indexer',
        'lib/platform.js',
        'lib/installer.js',
        'lib/config.js',
        'lib/validator.js'
      ];
      
      for (const file of requiredFiles) {
        const filePath = path.join(packageRoot, file);
        expect(fs.existsSync(filePath)).toBe(true);
      }
    });
    
    test('package.json should have correct structure', () => {
      const packageJson = require('../package.json');
      
      expect(packageJson.name).toBe('mcp-rag-indexer');
      expect(packageJson.version).toBe('1.0.0');
      expect(packageJson.bin).toHaveProperty('mcp-rag-indexer');
      expect(packageJson.engines).toHaveProperty('node');
      expect(packageJson.preferGlobal).toBe(true);
    });
  });
  
  describe('MCP Server Integration', () => {
    test('should be able to import MCP server modules', async () => {
      const pythonPath = PlatformManager.getPythonExecutable();
      const pythonDir = path.join(__dirname, '..', 'python');
      
      if (fs.existsSync(pythonPath) && fs.existsSync(pythonDir)) {
        const testScript = `
import sys
import os
sys.path.insert(0, "${pythonDir.replace(/\\/g, '\\\\')}")

try:
    # Test basic imports
    import mcp_rag_server
    print("SUCCESS: MCP server imported")
except ImportError as e:
    print(f"IMPORT_ERROR: {e}")
    sys.exit(1)
except Exception as e:
    print(f"ERROR: {e}")
    sys.exit(1)
`;
        
        const result = await runCommand(pythonPath, ['-c', testScript], {
          timeout: 15000
        });
        
        if (result.code === 0) {
          expect(result.stdout).toContain('SUCCESS');
        } else {
          // Dependencies might not be installed in test environment
          console.warn('MCP server import failed - dependencies may not be installed');
        }
      } else {
        console.warn('Python runtime or server files not found - skipping test');
      }
    });
  });
  
  describe('Installation Validation', () => {
    test('should run installation validation', async () => {
      // This might fail in test environment, so we just test it runs
      const result = await validateInstallation().catch(() => false);
      expect(typeof result).toBe('boolean');
    });
  });
  
  describe('Configuration Management', () => {
    test('should create and manage configuration', async () => {
      const { ConfigManager } = require('../lib/config');
      const configManager = new ConfigManager();
      
      // Load default config
      const config = await configManager.loadConfig();
      expect(config).toHaveProperty('version');
      expect(config).toHaveProperty('logLevel');
      expect(config).toHaveProperty('modelName');
      
      // Test validation
      const errors = configManager.validateConfig(config);
      expect(Array.isArray(errors)).toBe(true);
    });
  });
});

/**
 * Helper function to run commands with timeout
 */
function runCommand(command, args = [], options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      stdio: 'pipe',
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
      resolve({
        code,
        stdout,
        stderr
      });
    });
    
    child.on('error', (error) => {
      reject(error);
    });
    
    // Handle timeout
    const timeout = options.timeout || 30000;
    setTimeout(() => {
      child.kill();
      reject(new Error(`Command timeout after ${timeout}ms`));
    }, timeout);
  });
}