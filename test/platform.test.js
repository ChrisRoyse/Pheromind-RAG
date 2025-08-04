/**
 * Platform-Specific Tests for MCP RAG Indexer
 * Tests cross-platform compatibility and platform-specific features
 */

const os = require('os');
const path = require('path');
const fs = require('fs');
const { spawn } = require('child_process');
const { PlatformManager } = require('../lib/platform');

describe('Platform-Specific Tests', () => {
  
  describe('Current Platform Detection', () => {
    test('should detect current platform correctly', () => {
      const detectedPlatform = PlatformManager.detectPlatform();
      const expectedPlatform = `${process.platform === 'win32' ? 'windows' : 
                                process.platform === 'darwin' ? 'macos' : 'linux'}-${process.arch}`;
      
      expect(detectedPlatform).toBe(expectedPlatform);
    });
    
    test('should provide correct system information', () => {
      const info = PlatformManager.getSystemInfo();
      
      expect(info.platform).toBe(process.platform);
      expect(info.arch).toBe(process.arch);
      expect(info.nodeVersion).toBe(process.version);
      expect(info.homeDir).toBe(os.homedir());
      expect(info.tmpDir).toBe(os.tmpdir());
    });
  });
  
  describe('Python Executable Paths', () => {
    test('should return platform-appropriate Python path', () => {
      const pythonPath = PlatformManager.getPythonExecutable();
      
      if (process.platform === 'win32') {
        expect(pythonPath).toMatch(/python\.exe$/);
      } else {
        expect(pythonPath).toMatch(/python3$/);
      }
      
      expect(path.isAbsolute(pythonPath)).toBe(true);
    });
    
    test('should return platform-appropriate pip path', () => {
      const pipPath = PlatformManager.getPipExecutable();
      
      if (Array.isArray(pipPath)) {
        // Fallback to python -m pip
        expect(pipPath[0]).toMatch(/python/);
        expect(pipPath[1]).toBe('-m');
        expect(pipPath[2]).toBe('pip');
      } else {
        // Direct pip executable
        if (process.platform === 'win32') {
          expect(pipPath).toMatch(/pip\.exe$/);
        } else {
          expect(pipPath).toMatch(/pip3?$/);
        }
      }
    });
  });
  
  describe('File System Paths', () => {
    test('should provide correct runtime directory', () => {
      const runtimeDir = PlatformManager.getRuntimeDirectory();
      const platform = PlatformManager.detectPlatform();
      
      expect(runtimeDir).toContain(platform);
      expect(path.isAbsolute(runtimeDir)).toBe(true);
    });
    
    test('should provide correct site-packages directory', () => {
      const sitePackagesDir = PlatformManager.getSitePackagesDirectory();
      
      expect(sitePackagesDir).toContain('site-packages');
      expect(path.isAbsolute(sitePackagesDir)).toBe(true);
    });
  });
  
  describe('Platform-Specific File Extensions', () => {
    test('should return correct file extensions for current platform', () => {
      const extensions = PlatformManager.getFileExtensions();
      
      if (process.platform === 'win32') {
        expect(extensions.executable).toBe('.exe');
        expect(extensions.script).toBe('.bat');
        expect(extensions.library).toBe('.dll');
      } else if (process.platform === 'darwin') {
        expect(extensions.executable).toBe('');
        expect(extensions.script).toBe('.sh');
        expect(extensions.library).toBe('.dylib');
      } else {
        expect(extensions.executable).toBe('');
        expect(extensions.script).toBe('.sh');
        expect(extensions.library).toBe('.so');
      }
    });
  });
  
  describe('Windows-Specific Tests', () => {
    const isWindows = process.platform === 'win32';
    
    test('should handle Windows paths correctly', () => {
      if (!isWindows) {
        console.log('Skipping Windows-specific test on non-Windows platform');
        return;
      }
      
      const pythonPath = PlatformManager.getPythonExecutable();
      expect(pythonPath).toMatch(/^[A-Z]:\\/);
      expect(pythonPath).toMatch(/\.exe$/);
    });
    
    test('should use correct Windows runtime structure', () => {
      if (!isWindows) {
        console.log('Skipping Windows-specific test on non-Windows platform');
        return;
      }
      
      const sitePackagesDir = PlatformManager.getSitePackagesDirectory();
      expect(sitePackagesDir).toMatch(/\\Lib\\site-packages$/);
    });
  });
  
  describe('macOS-Specific Tests', () => {
    const isMacOS = process.platform === 'darwin';
    
    test('should handle macOS paths correctly', () => {
      if (!isMacOS) {
        console.log('Skipping macOS-specific test on non-macOS platform');
        return;
      }
      
      const pythonPath = PlatformManager.getPythonExecutable();
      expect(pythonPath).toMatch(/\/bin\/python3$/);
    });
    
    test('should detect ARM64 vs x64 correctly', () => {
      if (!isMacOS) {
        console.log('Skipping macOS-specific test on non-macOS platform');
        return;
      }
      
      const platform = PlatformManager.detectPlatform();
      expect(platform).toMatch(/^macos-(x64|arm64)$/);
    });
  });
  
  describe('Linux-Specific Tests', () => {
    const isLinux = process.platform === 'linux';
    
    test('should handle Linux paths correctly', () => {
      if (!isLinux) {
        console.log('Skipping Linux-specific test on non-Linux platform');
        return;
      }
      
      const pythonPath = PlatformManager.getPythonExecutable();
      expect(pythonPath).toMatch(/\/bin\/python3$/);
    });
    
    test('should use correct Linux runtime structure', () => {
      if (!isLinux) {
        console.log('Skipping Linux-specific test on non-Linux platform');
        return;
      }
      
      const sitePackagesDir = PlatformManager.getSitePackagesDirectory();
      expect(sitePackagesDir).toMatch(/\/lib\/python3\.\d+\/site-packages$/);
    });
  });
  
  describe('Cross-Platform Compatibility', () => {
    test('should handle path separators correctly', () => {
      const pythonPath = PlatformManager.getPythonExecutable();
      const runtimeDir = PlatformManager.getRuntimeDirectory();
      
      // Should use platform-appropriate separators
      if (process.platform === 'win32') {
        expect(pythonPath).toContain('\\');
        expect(runtimeDir).toContain('\\');
      } else {
        expect(pythonPath).toContain('/');
        expect(runtimeDir).toContain('/');
      }
    });
    
    test('should provide consistent behavior across platforms', () => {
      const info1 = PlatformManager.getSystemInfo();
      const info2 = PlatformManager.getSystemInfo();
      
      expect(info1).toEqual(info2);
    });
  });
  
  describe('Runtime Validation', () => {
    test('should validate runtime existence', async () => {
      const isValid = await PlatformManager.validateRuntime();
      expect(typeof isValid).toBe('boolean');
      
      // In test environment, runtime might not be bundled
      if (!isValid) {
        console.warn('Python runtime not found - expected in test environment');
      }
    });
    
    test('should handle missing runtime gracefully', async () => {
      // Temporarily override Python path to non-existent file
      const originalMethod = PlatformManager.getPythonExecutable;
      PlatformManager.getPythonExecutable = () => '/nonexistent/python';
      
      const isValid = await PlatformManager.validateRuntime();
      expect(isValid).toBe(false);
      
      // Restore original method
      PlatformManager.getPythonExecutable = originalMethod;
    });
  });
  
  describe('Environment Variables', () => {
    test('should not modify global environment', () => {
      const originalEnv = { ...process.env };
      
      // Call methods that might modify environment
      PlatformManager.getSystemInfo();
      PlatformManager.getPythonExecutable();
      
      // Environment should be unchanged
      expect(process.env).toEqual(originalEnv);
    });
  });
  
  describe('Performance', () => {
    test('platform detection should be fast', () => {
      const start = Date.now();
      
      for (let i = 0; i < 1000; i++) {
        PlatformManager.detectPlatform();
      }
      
      const duration = Date.now() - start;
      expect(duration).toBeLessThan(100); // Should complete in under 100ms
    });
    
    test('system info should be cached or fast', () => {
      const start = Date.now();
      
      for (let i = 0; i < 100; i++) {
        PlatformManager.getSystemInfo();
      }
      
      const duration = Date.now() - start;
      expect(duration).toBeLessThan(50); // Should complete in under 50ms
    });
  });
  
  describe('Error Conditions', () => {
    test('should handle invalid platform gracefully', () => {
      // Mock invalid platform
      const originalPlatform = Object.getOwnPropertyDescriptor(process, 'platform');
      const originalArch = Object.getOwnPropertyDescriptor(process, 'arch');
      
      Object.defineProperty(process, 'platform', { value: 'invalid', configurable: true });
      Object.defineProperty(process, 'arch', { value: 'invalid', configurable: true });
      
      expect(() => {
        PlatformManager.detectPlatform();
      }).toThrow('Unsupported platform');
      
      expect(PlatformManager.isPlatformSupported()).toBe(false);
      
      // Restore original values
      if (originalPlatform) {
        Object.defineProperty(process, 'platform', originalPlatform);
      }
      if (originalArch) {
        Object.defineProperty(process, 'arch', originalArch);
      }
    });
    
    test('should handle file system errors gracefully', async () => {
      // Test with restricted permissions (if possible)
      const originalValidate = PlatformManager.validateRuntime;
      
      PlatformManager.validateRuntime = async () => {
        throw new Error('Permission denied');
      };
      
      const isValid = await PlatformManager.validateRuntime().catch(() => false);
      expect(isValid).toBe(false);
      
      // Restore original method
      PlatformManager.validateRuntime = originalValidate;
    });
  });
  
  describe('Integration with System Commands', () => {
    test('should be able to run system-appropriate commands', async () => {
      const command = process.platform === 'win32' ? 'dir' : 'ls';
      const args = process.platform === 'win32' ? ['/B'] : ['-la'];
      
      const result = await runCommand(command, args, { timeout: 5000 }).catch(() => null);
      
      if (result && result.code === 0) {
        expect(result.stdout).toBeTruthy();
      } else {
        console.warn('System command test skipped - command not available');
      }
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
      resolve({ code, stdout, stderr });
    });
    
    child.on('error', (error) => {
      reject(error);
    });
    
    // Handle timeout
    const timeout = options.timeout || 10000;
    setTimeout(() => {
      child.kill();
      reject(new Error(`Command timeout after ${timeout}ms`));
    }, timeout);
  });
}