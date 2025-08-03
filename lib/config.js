/**
 * Configuration Management for MCP RAG Indexer
 * Handles application configuration and settings
 */

const os = require('os');
const path = require('path');
const fs = require('fs').promises;

class ConfigManager {
  constructor() {
    this.configDir = path.join(os.homedir(), '.mcp-rag-indexer');
    this.configFile = path.join(this.configDir, 'config.json');
    this.defaultConfig = {
      version: '1.0.0',
      logLevel: 'info',
      modelName: 'sentence-transformers/all-MiniLM-L6-v2',
      maxCacheSize: 1000,
      maxProjects: 50,
      pollInterval: 30,
      chunkSize: 1000,
      chunkOverlap: 200,
      offlineMode: true,
      autoUpdate: true,
      telemetry: false
    };
  }
  
  /**
   * Ensure configuration directory exists
   */
  async ensureConfigDir() {
    try {
      await fs.mkdir(this.configDir, { recursive: true });
    } catch (error) {
      // Directory might already exist
    }
  }
  
  /**
   * Load configuration from file
   */
  async loadConfig() {
    try {
      await this.ensureConfigDir();
      const configData = await fs.readFile(this.configFile, 'utf8');
      const config = JSON.parse(configData);
      
      // Merge with defaults to handle new config options
      return { ...this.defaultConfig, ...config };
    } catch (error) {
      // Config doesn't exist or is invalid, return defaults
      return { ...this.defaultConfig };
    }
  }
  
  /**
   * Save configuration to file
   */
  async saveConfig(config) {
    try {
      await this.ensureConfigDir();
      const configData = JSON.stringify(config, null, 2);
      await fs.writeFile(this.configFile, configData, 'utf8');
      return true;
    } catch (error) {
      console.error('Failed to save configuration:', error);
      return false;
    }
  }
  
  /**
   * Get a specific configuration value
   */
  async get(key, defaultValue = null) {
    const config = await this.loadConfig();
    return config[key] !== undefined ? config[key] : defaultValue;
  }
  
  /**
   * Set a specific configuration value
   */
  async set(key, value) {
    const config = await this.loadConfig();
    config[key] = value;
    return await this.saveConfig(config);
  }
  
  /**
   * Update multiple configuration values
   */
  async update(updates) {
    const config = await this.loadConfig();
    const newConfig = { ...config, ...updates };
    return await this.saveConfig(newConfig);
  }
  
  /**
   * Reset configuration to defaults
   */
  async reset() {
    return await this.saveConfig({ ...this.defaultConfig });
  }
  
  /**
   * Get configuration file path
   */
  getConfigPath() {
    return this.configFile;
  }
  
  /**
   * Get configuration directory path
   */
  getConfigDir() {
    return this.configDir;
  }
  
  /**
   * Get log directory path
   */
  getLogDir() {
    return path.join(this.configDir, 'logs');
  }
  
  /**
   * Get database directory path
   */
  getDatabaseDir() {
    return path.join(this.configDir, 'databases');
  }
  
  /**
   * Get cache directory path
   */
  getCacheDir() {
    return path.join(this.configDir, 'cache');
  }
  
  /**
   * Initialize all required directories
   */
  async initializeDirectories() {
    const dirs = [
      this.configDir,
      this.getLogDir(),
      this.getDatabaseDir(),
      this.getCacheDir()
    ];
    
    for (const dir of dirs) {
      try {
        await fs.mkdir(dir, { recursive: true });
      } catch (error) {
        // Directory might already exist
      }
    }
  }
  
  /**
   * Get environment-specific configuration
   */
  async getEnvironmentConfig() {
    const config = await this.loadConfig();
    
    // Override with environment variables if present
    return {
      ...config,
      logLevel: process.env.MCP_RAG_LOG_LEVEL || config.logLevel,
      modelName: process.env.MCP_RAG_MODEL_NAME || config.modelName,
      offlineMode: process.env.MCP_RAG_OFFLINE === 'true' || config.offlineMode,
      telemetry: process.env.MCP_RAG_TELEMETRY === 'true' || config.telemetry
    };
  }
  
  /**
   * Validate configuration values
   */
  validateConfig(config) {
    const errors = [];
    
    // Validate log level
    const validLogLevels = ['debug', 'info', 'warning', 'error'];
    if (!validLogLevels.includes(config.logLevel)) {
      errors.push(`Invalid log level: ${config.logLevel}`);
    }
    
    // Validate numeric values
    if (typeof config.maxCacheSize !== 'number' || config.maxCacheSize < 0) {
      errors.push('maxCacheSize must be a non-negative number');
    }
    
    if (typeof config.maxProjects !== 'number' || config.maxProjects < 1) {
      errors.push('maxProjects must be a positive number');
    }
    
    if (typeof config.pollInterval !== 'number' || config.pollInterval < 1) {
      errors.push('pollInterval must be a positive number');
    }
    
    if (typeof config.chunkSize !== 'number' || config.chunkSize < 100) {
      errors.push('chunkSize must be at least 100');
    }
    
    if (typeof config.chunkOverlap !== 'number' || config.chunkOverlap < 0) {
      errors.push('chunkOverlap must be non-negative');
    }
    
    return errors;
  }
  
  /**
   * Get configuration summary for debugging
   */
  async getConfigSummary() {
    const config = await this.getEnvironmentConfig();
    
    return {
      configFile: this.configFile,
      configDir: this.configDir,
      version: config.version,
      logLevel: config.logLevel,
      modelName: config.modelName,
      offlineMode: config.offlineMode,
      directories: {
        config: this.configDir,
        logs: this.getLogDir(),
        databases: this.getDatabaseDir(),
        cache: this.getCacheDir()
      }
    };
  }
}

module.exports = { ConfigManager };