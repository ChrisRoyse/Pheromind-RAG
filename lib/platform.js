/**
 * Platform Management for MCP RAG Indexer
 * Handles cross-platform Python runtime detection and management
 */

const os = require('os');
const path = require('path');
const fs = require('fs');
const { promisify } = require('util');

const exists = promisify(fs.exists);
const access = promisify(fs.access);

class PlatformManager {
  /**
   * Detect current platform and architecture
   * @returns {string} Platform identifier (e.g., 'windows-x64', 'macos-arm64')
   */
  static detectPlatform() {
    const platform = process.platform;
    const arch = process.arch;
    
    const platformMap = {
      'win32-x64': 'windows-x64',
      'win32-arm64': 'windows-arm64',
      'darwin-x64': 'macos-x64', 
      'darwin-arm64': 'macos-arm64',
      'linux-x64': 'linux-x64',
      'linux-arm64': 'linux-arm64'
    };
    
    const platformId = platformMap[`${platform}-${arch}`];
    
    if (!platformId) {
      throw new Error(`Unsupported platform: ${platform}-${arch}`);
    }
    
    return platformId;
  }
  
  /**
   * Get the path to the embedded Python executable
   * @returns {string} Path to Python executable
   */
  static getPythonExecutable() {
    const platform = this.detectPlatform();
    const packageRoot = path.join(__dirname, '..');
    const runtimePath = path.join(packageRoot, 'runtime', platform);
    
    switch (process.platform) {
      case 'win32':
        // Windows standalone Python build
        return path.join(runtimePath, 'python.exe');
        
      case 'darwin':
      case 'linux':
        // Unix-like systems
        return path.join(runtimePath, 'bin', 'python3');
        
      default:
        throw new Error(`Unsupported platform: ${process.platform}`);
    }
  }
  
  /**
   * Get the path to pip executable
   * @returns {string} Path to pip executable
   */
  static getPipExecutable() {
    const platform = this.detectPlatform();
    const packageRoot = path.join(__dirname, '..');
    const runtimePath = path.join(packageRoot, 'runtime', platform);
    
    switch (process.platform) {
      case 'win32':
        // Check for standalone pip
        const standalonePip = path.join(runtimePath, 'Scripts', 'pip.exe');
        if (fs.existsSync(standalonePip)) {
          return standalonePip;
        }
        // Fallback to python -m pip
        return [this.getPythonExecutable(), '-m', 'pip'];
        
      case 'darwin':
      case 'linux':
        const unixPip = path.join(runtimePath, 'bin', 'pip3');
        if (fs.existsSync(unixPip)) {
          return unixPip;
        }
        // Fallback to python -m pip
        return [this.getPythonExecutable(), '-m', 'pip'];
        
      default:
        throw new Error(`Unsupported platform: ${process.platform}`);
    }
  }
  
  /**
   * Validate that the Python runtime exists and is functional
   * @returns {Promise<boolean>} True if runtime is valid
   */
  static async validateRuntime() {
    try {
      const pythonPath = this.getPythonExecutable();
      
      // Check if file exists
      if (!fs.existsSync(pythonPath)) {
        return false;
      }
      
      // Check if file is executable
      try {
        await access(pythonPath, fs.constants.F_OK | fs.constants.X_OK);
      } catch (error) {
        return false;
      }
      
      return true;
    } catch (error) {
      return false;
    }
  }
  
  /**
   * Get runtime directory for current platform
   * @returns {string} Runtime directory path
   */
  static getRuntimeDirectory() {
    const platform = this.detectPlatform();
    const packageRoot = path.join(__dirname, '..');
    return path.join(packageRoot, 'runtime', platform);
  }
  
  /**
   * Get site-packages directory for current platform
   * @returns {string} Site-packages directory path
   */
  static getSitePackagesDirectory() {
    const runtimeDir = this.getRuntimeDirectory();
    
    switch (process.platform) {
      case 'win32':
        return path.join(runtimeDir, 'Lib', 'site-packages');
        
      case 'darwin':
      case 'linux':
        // Find Python version directory
        const libDir = path.join(runtimeDir, 'lib');
        if (fs.existsSync(libDir)) {
          const pythonDirs = fs.readdirSync(libDir)
            .filter(dir => dir.startsWith('python'))
            .sort()
            .reverse(); // Get latest version first
            
          if (pythonDirs.length > 0) {
            return path.join(libDir, pythonDirs[0], 'site-packages');
          }
        }
        
        // Fallback for Python 3.11 (our target version)
        return path.join(runtimeDir, 'lib', 'python3.11', 'site-packages');
        
      default:
        throw new Error(`Unsupported platform: ${process.platform}`);
    }
  }
  
  /**
   * Get system information for debugging
   * @returns {object} System information
   */
  static getSystemInfo() {
    return {
      platform: process.platform,
      arch: process.arch,
      platformId: this.detectPlatform(),
      nodeVersion: process.version,
      pythonPath: this.getPythonExecutable(),
      runtimeDir: this.getRuntimeDirectory(),
      sitePackages: this.getSitePackagesDirectory(),
      homeDir: os.homedir(),
      tmpDir: os.tmpdir()
    };
  }
  
  /**
   * Check if current platform is supported
   * @returns {boolean} True if platform is supported
   */
  static isPlatformSupported() {
    try {
      this.detectPlatform();
      return true;
    } catch (error) {
      return false;
    }
  }
  
  /**
   * Get platform-specific file extensions
   * @returns {object} File extensions for current platform
   */
  static getFileExtensions() {
    switch (process.platform) {
      case 'win32':
        return {
          executable: '.exe',
          script: '.bat',
          library: '.dll'
        };
        
      case 'darwin':
        return {
          executable: '',
          script: '.sh',
          library: '.dylib'
        };
        
      case 'linux':
        return {
          executable: '',
          script: '.sh',
          library: '.so'
        };
        
      default:
        return {
          executable: '',
          script: '.sh',
          library: '.so'
        };
    }
  }
}

module.exports = { PlatformManager };