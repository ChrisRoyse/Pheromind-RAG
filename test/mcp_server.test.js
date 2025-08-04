/**
 * TDD Tests for MCP RAG Server Core Functionality
 * These tests MUST FAIL initially - we're implementing the functionality to make them pass
 */

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');
const { PlatformManager } = require('../lib/platform');

// Longer timeout for Python processes
jest.setTimeout(30000);

describe('MCP RAG Server - Core Functionality (TDD)', () => {
  let serverProcess;
  let pythonPath;
  let serverScript;
  
  beforeAll(() => {
    // Try bundled Python first, fallback to system Python
    pythonPath = PlatformManager.getPythonExecutable();
    if (!fs.existsSync(pythonPath)) {
      // Fallback to system Python
      pythonPath = 'python';
    }
    serverScript = path.join(__dirname, '..', 'python', 'mcp_rag_server.py');
  });
  
  afterEach(async () => {
    if (serverProcess && !serverProcess.killed) {
      serverProcess.kill('SIGTERM');
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  });

  describe('Server Existence and Basic CLI', () => {
    test('MUST FAIL: mcp_rag_server.py should exist', () => {
      expect(fs.existsSync(serverScript)).toBe(true);
    });

    test('MUST FAIL: server should respond to --version flag', async () => {
      const result = await runPythonCommand([serverScript, '--version']);
      expect(result.code).toBe(0);
      expect(result.stdout).toMatch(/MCP RAG Indexer/i);
      expect(result.stdout).toMatch(/\d+\.\d+\.\d+/); // Version number
    });

    test('MUST FAIL: server should respond to --help flag', async () => {
      const result = await runPythonCommand([serverScript, '--help']);
      expect(result.code).toBe(0);
      expect(result.stdout).toContain('usage');
      expect(result.stdout).toContain('MCP RAG Indexer');
    });
  });

  describe('MCP Protocol Compliance', () => {
    test('MUST FAIL: server should start and accept MCP initialize message', async () => {
      serverProcess = spawn(pythonPath, [serverScript], {
        stdio: ['pipe', 'pipe', 'pipe']
      });

      // Send MCP initialize message
      const initMessage = {
        jsonrpc: '2.0',
        id: 1,
        method: 'initialize',
        params: {
          protocolVersion: '2024-11-05',
          capabilities: {},
          clientInfo: {
            name: 'test-client',
            version: '1.0.0'
          }
        }
      };

      serverProcess.stdin.write(JSON.stringify(initMessage) + '\n');

      const response = await waitForMessage(serverProcess, 5000);
      expect(response).toBeDefined();
      
      const parsed = JSON.parse(response);
      expect(parsed.id).toBe(1);
      expect(parsed.result).toBeDefined();
      expect(parsed.result.capabilities).toBeDefined();
    });

    test('MUST FAIL: server should list available tools', async () => {
      serverProcess = spawn(pythonPath, [serverScript], {
        stdio: ['pipe', 'pipe', 'pipe']
      });

      await initializeServer(serverProcess);

      const toolsMessage = {
        jsonrpc: '2.0',
        id: 2,
        method: 'tools/list'
      };

      serverProcess.stdin.write(JSON.stringify(toolsMessage) + '\n');

      const response = await waitForMessage(serverProcess, 5000);
      const parsed = JSON.parse(response);
      
      expect(parsed.result).toBeDefined();
      expect(parsed.result.tools).toBeDefined();
      expect(Array.isArray(parsed.result.tools)).toBe(true);
      
      // Should have at least index_project and search_code tools
      const toolNames = parsed.result.tools.map(tool => tool.name);
      expect(toolNames).toContain('index_project');
      expect(toolNames).toContain('search_code');
    });
  });

  describe('RAG Functionality', () => {
    test('MUST FAIL: should be able to index a simple project', async () => {
      // Create a temporary test project
      const testProject = path.join(__dirname, 'temp_test_project');
      await fs.promises.mkdir(testProject, { recursive: true });
      await fs.promises.writeFile(
        path.join(testProject, 'test.py'),
        'def hello_world():\n    return "Hello, World!"'
      );

      serverProcess = spawn(pythonPath, [serverScript], {
        stdio: ['pipe', 'pipe', 'pipe']
      });

      await initializeServer(serverProcess);

      const indexMessage = {
        jsonrpc: '2.0',
        id: 3,
        method: 'tools/call',
        params: {
          name: 'index_project',
          arguments: {
            path: testProject
          }
        }
      };

      serverProcess.stdin.write(JSON.stringify(indexMessage) + '\n');

      const response = await waitForMessage(serverProcess, 10000);
      const parsed = JSON.parse(response);

      expect(parsed.result).toBeDefined();
      expect(parsed.result.content).toBeDefined();
      expect(parsed.result.content[0].text).toContain('indexed');

      // Cleanup
      await fs.promises.rm(testProject, { recursive: true, force: true });
    });

    test('MUST FAIL: should be able to search indexed code', async () => {
      serverProcess = spawn(pythonPath, [serverScript], {
        stdio: ['pipe', 'pipe', 'pipe']
      });

      await initializeServer(serverProcess);

      const searchMessage = {
        jsonrpc: '2.0',
        id: 4,
        method: 'tools/call',
        params: {
          name: 'search_code',
          arguments: {
            query: 'hello world function'
          }
        }
      };

      serverProcess.stdin.write(JSON.stringify(searchMessage) + '\n');

      const response = await waitForMessage(serverProcess, 5000);
      const parsed = JSON.parse(response);

      expect(parsed.result).toBeDefined();
      expect(parsed.result.content).toBeDefined();
      expect(Array.isArray(parsed.result.content)).toBe(true);
    });
  });

  describe('Error Handling', () => {
    test('MUST FAIL: should handle invalid JSON gracefully', async () => {
      serverProcess = spawn(pythonPath, [serverScript], {
        stdio: ['pipe', 'pipe', 'pipe']
      });

      serverProcess.stdin.write('invalid json\n');

      const response = await waitForMessage(serverProcess, 3000);
      expect(response).toBeDefined();
      
      const parsed = JSON.parse(response);
      expect(parsed.error).toBeDefined();
      expect(parsed.error.code).toBe(-32700); // Parse error
    });

    test('MUST FAIL: should handle unknown methods gracefully', async () => {
      serverProcess = spawn(pythonPath, [serverScript], {
        stdio: ['pipe', 'pipe', 'pipe']
      });

      await initializeServer(serverProcess);

      const unknownMessage = {
        jsonrpc: '2.0',
        id: 5,
        method: 'unknown/method'
      };

      serverProcess.stdin.write(JSON.stringify(unknownMessage) + '\n');

      const response = await waitForMessage(serverProcess, 3000);
      const parsed = JSON.parse(response);

      expect(parsed.error).toBeDefined();
      expect(parsed.error.code).toBe(-32601); // Method not found
    });
  });

  describe('Resource Management', () => {
    test('MUST FAIL: should properly cleanup on shutdown', async () => {
      serverProcess = spawn(pythonPath, [serverScript], {
        stdio: ['pipe', 'pipe', 'pipe']
      });

      await initializeServer(serverProcess);

      // Send shutdown notification
      const shutdownMessage = {
        jsonrpc: '2.0',
        method: 'shutdown'
      };

      serverProcess.stdin.write(JSON.stringify(shutdownMessage) + '\n');

      // Wait for graceful shutdown
      const exitCode = await new Promise((resolve) => {
        serverProcess.on('exit', resolve);
        setTimeout(() => resolve(-1), 5000); // Timeout
      });

      expect(exitCode).toBe(0); // Should exit cleanly
    });
  });
});

// Helper Functions
async function runPythonCommand(args, options = {}) {
  return new Promise((resolve, reject) => {
    // Get the Python path
    let pythonExe = PlatformManager.getPythonExecutable();
    if (!fs.existsSync(pythonExe)) {
      pythonExe = 'python';
    }
    
    const child = spawn(pythonExe, args, {
      stdio: 'pipe',
      timeout: 10000,
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
      reject(new Error('Command timeout'));
    }, 10000);
  });
}

async function waitForMessage(process, timeout = 5000) {
  return new Promise((resolve, reject) => {
    let buffer = '';

    const onData = (data) => {
      buffer += data.toString();
      const lines = buffer.split('\n');
      
      // Keep incomplete line in buffer
      buffer = lines.pop() || '';
      
      // Process complete lines
      for (const line of lines) {
        if (line.trim()) {
          process.stdout.removeListener('data', onData);
          resolve(line.trim());
          return;
        }
      }
    };

    process.stdout.on('data', onData);

    setTimeout(() => {
      process.stdout.removeListener('data', onData);
      reject(new Error(`Message timeout after ${timeout}ms`));
    }, timeout);
  });
}

async function initializeServer(process) {
  const initMessage = {
    jsonrpc: '2.0',
    id: 1,
    method: 'initialize',
    params: {
      protocolVersion: '2024-11-05',
      capabilities: {},
      clientInfo: {
        name: 'test-client',
        version: '1.0.0'
      }
    }
  };

  process.stdin.write(JSON.stringify(initMessage) + '\n');
  await waitForMessage(process, 5000);
}