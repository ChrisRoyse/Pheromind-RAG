/**
 * Unit Tests for MCP RAG Indexer Components
 * Tests individual modules in isolation
 */

const { PlatformManager } = require('../lib/platform');
const { ConfigManager } = require('../lib/config');
const { MCPWrapper } = require('../lib/wrapper');

describe('Unit Tests', () => {
  
  describe('PlatformManager', () => {
    test('should detect platform correctly', () => {
      const platform = PlatformManager.detectPlatform();
      expect(platform).toMatch(/^(windows|macos|linux)-(x64|arm64)$/);
    });
    
    test('should return consistent platform info', () => {
      const platform1 = PlatformManager.detectPlatform();
      const platform2 = PlatformManager.detectPlatform();
      expect(platform1).toBe(platform2);
    });
    
    test('should provide system info', () => {
      const info = PlatformManager.getSystemInfo();
      expect(info).toHaveProperty('platform');
      expect(info).toHaveProperty('arch');
      expect(info).toHaveProperty('platformId');
      expect(info).toHaveProperty('nodeVersion');
      expect(info).toHaveProperty('pythonPath');
    });
    
    test('should get file extensions for platform', () => {
      const extensions = PlatformManager.getFileExtensions();
      expect(extensions).toHaveProperty('executable');
      expect(extensions).toHaveProperty('script');
      expect(extensions).toHaveProperty('library');
    });
    
    test('should report platform support correctly', () => {
      const isSupported = PlatformManager.isPlatformSupported();
      expect(typeof isSupported).toBe('boolean');
      expect(isSupported).toBe(true); // Current platform should be supported
    });
  });
  
  describe('ConfigManager', () => {
    let configManager;
    
    beforeEach(() => {
      configManager = new ConfigManager();
    });
    
    test('should have default configuration', async () => {
      const config = await configManager.loadConfig();
      expect(config).toHaveProperty('version');
      expect(config).toHaveProperty('logLevel');
      expect(config).toHaveProperty('modelName');
      expect(config).toHaveProperty('maxCacheSize');
      expect(config.logLevel).toBe('info');
      expect(typeof config.maxCacheSize).toBe('number');
    });
    
    test('should validate configuration correctly', () => {
      const validConfig = {
        logLevel: 'info',
        maxCacheSize: 1000,
        maxProjects: 50,
        pollInterval: 30,
        chunkSize: 1000,
        chunkOverlap: 200
      };
      
      const errors = configManager.validateConfig(validConfig);
      expect(errors).toEqual([]);
    });
    
    test('should detect invalid configuration', () => {
      const invalidConfig = {
        logLevel: 'invalid',
        maxCacheSize: -1,
        maxProjects: 0,
        pollInterval: 0,
        chunkSize: 50,
        chunkOverlap: -1
      };
      
      const errors = configManager.validateConfig(invalidConfig);
      expect(errors.length).toBeGreaterThan(0);
    });
    
    test('should provide configuration paths', () => {
      const configPath = configManager.getConfigPath();
      const configDir = configManager.getConfigDir();
      const logDir = configManager.getLogDir();
      
      expect(typeof configPath).toBe('string');
      expect(typeof configDir).toBe('string');
      expect(typeof logDir).toBe('string');
      expect(configPath.length).toBeGreaterThan(0);
    });
    
    test('should get configuration summary', async () => {
      const summary = await configManager.getConfigSummary();
      expect(summary).toHaveProperty('configFile');
      expect(summary).toHaveProperty('version');
      expect(summary).toHaveProperty('directories');
      expect(summary.directories).toHaveProperty('config');
      expect(summary.directories).toHaveProperty('logs');
    });
  });
  
  describe('MCPWrapper', () => {
    let wrapper;
    
    beforeEach(() => {
      wrapper = new MCPWrapper({
        autoRestart: false, // Disable auto-restart for tests
        processTimeout: 5000
      });
    });
    
    afterEach(async () => {
      if (wrapper) {
        await wrapper.stop().catch(() => {}); // Ignore errors in cleanup
      }
    });
    
    test('should initialize with default options', () => {
      const status = wrapper.getStatus();
      expect(status.isRunning).toBe(false);
      expect(status.isHealthy).toBe(false);
      expect(status.isStarting).toBe(false);
      expect(status.isStopping).toBe(false);
      expect(status.restartCount).toBe(0);
    });
    
    test('should provide status information', () => {
      const status = wrapper.getStatus();
      expect(status).toHaveProperty('isRunning');
      expect(status).toHaveProperty('isHealthy');
      expect(status).toHaveProperty('isStarting');
      expect(status).toHaveProperty('isStopping');
      expect(status).toHaveProperty('pid');
      expect(status).toHaveProperty('restartCount');
      expect(status).toHaveProperty('uptime');
    });
    
    test('should emit events', (done) => {
      const events = [];
      
      wrapper.on('started', () => events.push('started'));
      wrapper.on('stopped', () => events.push('stopped'));
      wrapper.on('error', () => events.push('error'));
      
      // Trigger an error condition
      wrapper.emit('error', new Error('test'));
      
      setTimeout(() => {
        expect(events).toContain('error');
        done();
      }, 100);
    });
    
    test('should handle process states correctly', () => {
      expect(wrapper.getStatus().isRunning).toBe(false);
      
      // Simulate state changes
      wrapper.isStarting = true;
      expect(wrapper.getStatus().isStarting).toBe(true);
      
      wrapper.isStarting = false;
      wrapper.isStopping = true;
      expect(wrapper.getStatus().isStopping).toBe(true);
    });
  });
  
  describe('Package Structure Validation', () => {
    const path = require('path');
    const fs = require('fs');
    
    test('should have all required directories', () => {
      const packageRoot = path.join(__dirname, '..');
      const requiredDirs = ['bin', 'lib', 'python', 'scripts', 'test'];
      
      for (const dir of requiredDirs) {
        const dirPath = path.join(packageRoot, dir);
        expect(fs.existsSync(dirPath)).toBe(true);
      }
    });
    
    test('should have correct package.json structure', () => {
      const packageJson = require('../package.json');
      
      // Basic package info
      expect(packageJson.name).toBe('mcp-rag-indexer');
      expect(packageJson.version).toBeDefined();
      expect(packageJson.description).toBeDefined();
      expect(packageJson.license).toBe('MIT');
      
      // NPM configuration
      expect(packageJson.preferGlobal).toBe(true);
      expect(packageJson.bin).toHaveProperty('mcp-rag-indexer');
      expect(packageJson.engines).toHaveProperty('node');
      
      // Scripts
      expect(packageJson.scripts).toHaveProperty('postinstall');
      expect(packageJson.scripts).toHaveProperty('preuninstall');
      expect(packageJson.scripts).toHaveProperty('build');
      expect(packageJson.scripts).toHaveProperty('test');
      
      // Dependencies
      expect(packageJson.dependencies).toBeDefined();
      expect(typeof packageJson.dependencies).toBe('object');
      
      // Files to include in package
      expect(packageJson.files).toContain('bin/');
      expect(packageJson.files).toContain('lib/');
      expect(packageJson.files).toContain('python/');
    });
    
    test('should have required Python files', () => {
      const pythonDir = path.join(__dirname, '..', 'python');
      const requiredFiles = ['requirements.txt'];
      
      for (const file of requiredFiles) {
        const filePath = path.join(pythonDir, file);
        expect(fs.existsSync(filePath)).toBe(true);
      }
    });
    
    test('should have required scripts', () => {
      const scriptsDir = path.join(__dirname, '..', 'scripts');
      const requiredScripts = ['build.js', 'download-runtimes.js', 'download-models.js'];
      
      for (const script of requiredScripts) {
        const scriptPath = path.join(scriptsDir, script);
        expect(fs.existsSync(scriptPath)).toBe(true);
      }
    });
  });
  
  describe('Error Handling', () => {
    test('PlatformManager should handle unsupported platforms gracefully', () => {
      // Mock process.platform and process.arch temporarily
      const originalPlatform = process.platform;
      const originalArch = process.arch;
      
      Object.defineProperty(process, 'platform', { value: 'unsupported' });
      Object.defineProperty(process, 'arch', { value: 'unsupported' });
      
      expect(() => {
        PlatformManager.detectPlatform();
      }).toThrow('Unsupported platform');
      
      // Restore original values
      Object.defineProperty(process, 'platform', { value: originalPlatform });
      Object.defineProperty(process, 'arch', { value: originalArch });
    });
    
    test('ConfigManager should handle missing config gracefully', async () => {
      const configManager = new ConfigManager();
      
      // Try to get a non-existent config value
      const value = await configManager.get('nonexistent', 'default');
      expect(value).toBe('default');
    });
    
    test('MCPWrapper should handle invalid options gracefully', () => {
      expect(() => {
        new MCPWrapper({
          maxRestarts: -1,
          restartDelay: -1000,
          healthCheckInterval: 0
        });
      }).not.toThrow();
    });
  });
  
  describe('Environment Integration', () => {
    test('should respect environment variables', async () => {
      const originalEnv = process.env.MCP_RAG_LOG_LEVEL;
      
      process.env.MCP_RAG_LOG_LEVEL = 'debug';
      
      const configManager = new ConfigManager();
      const config = await configManager.getEnvironmentConfig();
      
      expect(config.logLevel).toBe('debug');
      
      // Restore original environment
      if (originalEnv !== undefined) {
        process.env.MCP_RAG_LOG_LEVEL = originalEnv;
      } else {
        delete process.env.MCP_RAG_LOG_LEVEL;
      }
    });
    
    test('should provide reasonable defaults', async () => {
      const configManager = new ConfigManager();
      const config = await configManager.loadConfig();
      
      expect(config.logLevel).toBe('info');
      expect(config.maxCacheSize).toBeGreaterThan(0);
      expect(config.maxProjects).toBeGreaterThan(0);
      expect(config.chunkSize).toBeGreaterThan(0);
    });
  });
});