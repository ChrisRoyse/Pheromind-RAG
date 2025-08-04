#!/usr/bin/env node
/**
 * Runtime Download Script for MCP RAG Indexer
 * Downloads and extracts Python runtimes for all supported platforms
 */

const fs = require('fs-extra');
const path = require('path');
const tar = require('tar');
const chalk = require('chalk');
const ora = require('ora');
const { spawn } = require('child_process');

// Async fetch function to handle ES module import
async function getFetch() {
  if (!global._fetch) {
    const fetchModule = await import('node-fetch');
    global._fetch = fetchModule.default;
  }
  return global._fetch;
}

const PACKAGE_ROOT = path.join(__dirname, '..');

// Python runtime configurations using Astral's python-build-standalone
// Updated URLs for latest release with Python 3.11.13 (2025)
const PYTHON_RUNTIMES = {
  'windows-x64': {
    url: 'https://github.com/astral-sh/python-build-standalone/releases/download/20250723/cpython-3.11.13+20250723-x86_64-pc-windows-msvc-install_only.tar.gz',
    extract: 'tar.gz',
    pythonExe: 'python.exe',
    size: '~45MB'
  },
  'windows-arm64': {
    url: 'https://github.com/astral-sh/python-build-standalone/releases/download/20250723/cpython-3.11.13+20250723-aarch64-pc-windows-msvc-install_only.tar.gz',
    extract: 'tar.gz',
    pythonExe: 'python.exe',
    size: '~45MB'
  },
  'macos-x64': {
    url: 'https://github.com/astral-sh/python-build-standalone/releases/download/20250723/cpython-3.11.13+20250723-x86_64-apple-darwin-install_only.tar.gz',
    extract: 'tar.gz',
    pythonExe: 'bin/python3',
    size: '~45MB'
  },
  'macos-arm64': {
    url: 'https://github.com/astral-sh/python-build-standalone/releases/download/20250723/cpython-3.11.13+20250723-aarch64-apple-darwin-install_only.tar.gz',
    extract: 'tar.gz',
    pythonExe: 'bin/python3',
    size: '~45MB'
  },
  'linux-x64': {
    url: 'https://github.com/astral-sh/python-build-standalone/releases/download/20250723/cpython-3.11.13+20250723-x86_64-unknown-linux-gnu-install_only.tar.gz',
    extract: 'tar.gz',
    pythonExe: 'bin/python3',
    size: '~50MB'
  },
  'linux-arm64': {
    url: 'https://github.com/astral-sh/python-build-standalone/releases/download/20250723/cpython-3.11.13+20250723-aarch64-unknown-linux-gnu-install_only.tar.gz',
    extract: 'tar.gz',
    pythonExe: 'bin/python3',
    size: '~50MB'
  }
};

class RuntimeDownloader {
  constructor() {
    this.spinner = null;
  }
  
  async downloadAll() {
    console.log(chalk.blue.bold('📥 Python Runtime Downloader'));
    console.log(chalk.gray('Downloads embedded Python runtimes for all supported platforms'));
    console.log();
    
    try {
      // Clean runtime directory
      await this.cleanRuntimeDir();
      
      // Download all runtimes
      for (const [platform, config] of Object.entries(PYTHON_RUNTIMES)) {
        await this.downloadRuntime(platform, config);
      }
      
      // Validate downloads
      await this.validateDownloads();
      
      console.log();
      console.log(chalk.green.bold('✅ All Python runtimes downloaded successfully!'));
      
    } catch (error) {
      console.log();
      console.error(chalk.red.bold('❌ Runtime download failed:'), error.message);
      process.exit(1);
    }
  }
  
  async downloadCurrent() {
    const currentPlatform = this.detectCurrentPlatform();
    console.log(chalk.blue.bold(`📥 Downloading Python runtime for ${currentPlatform}...`));
    console.log();
    
    try {
      const config = PYTHON_RUNTIMES[currentPlatform];
      if (!config) {
        throw new Error(`Unsupported platform: ${currentPlatform}`);
      }
      
      // Ensure runtime directory exists
      const runtimeDir = path.join(PACKAGE_ROOT, 'runtime');
      await fs.ensureDir(runtimeDir);
      
      // Download current platform runtime
      await this.downloadRuntime(currentPlatform, config);
      
      // Install Python packages
      await this.installPythonPackages(currentPlatform);
      
      console.log();
      console.log(chalk.green.bold(`✅ Python runtime for ${currentPlatform} ready!`));
      
    } catch (error) {
      console.log();
      console.error(chalk.red.bold('❌ Runtime setup failed:'), error.message);
      process.exit(1);
    }
  }
  
  detectCurrentPlatform() {
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
  
  async downloadSpecific(platforms) {
    console.log(chalk.blue.bold(`📥 Downloading runtimes for: ${platforms.join(', ')}`));
    console.log();
    
    try {
      for (const platform of platforms) {
        if (!PYTHON_RUNTIMES[platform]) {
          throw new Error(`Unsupported platform: ${platform}`);
        }
        
        await this.downloadRuntime(platform, PYTHON_RUNTIMES[platform]);
      }
      
      console.log();
      console.log(chalk.green.bold('✅ Selected runtimes downloaded successfully!'));
      
    } catch (error) {
      console.log();
      console.error(chalk.red.bold('❌ Runtime download failed:'), error.message);
      process.exit(1);
    }
  }
  
  async cleanRuntimeDir() {
    this.spinner = ora('Cleaning runtime directory...').start();
    
    try {
      const runtimeDir = path.join(PACKAGE_ROOT, 'runtime');
      if (await fs.pathExists(runtimeDir)) {
        await fs.remove(runtimeDir);
      }
      await fs.ensureDir(runtimeDir);
      
      this.spinner.succeed(chalk.green('✓ Runtime directory cleaned'));
    } catch (error) {
      this.spinner.fail(chalk.red('✗ Failed to clean runtime directory'));
      throw error;
    }
  }
  
  async downloadRuntime(platform, config) {
    this.spinner = ora(`Downloading ${platform} runtime (${config.size})...`).start();
    
    try {
      const runtimeDir = path.join(PACKAGE_ROOT, 'runtime', platform);
      await fs.ensureDir(runtimeDir);
      
      this.spinner.text = `Fetching ${platform} from ${config.url}`;
      
      const fetch = await getFetch();
      const response = await fetch(config.url, {
        timeout: 300000, // 5 minute timeout
        headers: {
          'User-Agent': 'mcp-rag-indexer-build'
        }
      });
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      
      const totalSize = parseInt(response.headers.get('content-length') || '0');
      let downloadedSize = 0;
      
      // Stream download with progress
      const chunks = [];
      response.body.on('data', (chunk) => {
        chunks.push(chunk);
        downloadedSize += chunk.length;
        
        if (totalSize > 0) {
          const percent = Math.round((downloadedSize / totalSize) * 100);
          this.spinner.text = `Downloading ${platform} runtime... ${percent}%`;
        }
      });
      
      const buffer = await new Promise((resolve, reject) => {
        response.body.on('end', () => resolve(Buffer.concat(chunks)));
        response.body.on('error', reject);
      });
      
      this.spinner.text = `Extracting ${platform} runtime...`;
      
      if (config.extract === 'zip') {
        await this.extractZip(buffer, runtimeDir);
      } else if (config.extract === 'tar.gz') {
        await this.extractTarGz(buffer, runtimeDir);
      }
      
      // Verify Python executable exists
      const pythonPath = path.join(runtimeDir, config.pythonExe);
      if (!await fs.pathExists(pythonPath)) {
        throw new Error(`Python executable not found: ${pythonPath}`);
      }
      
      // Make executable (Unix systems)
      if (process.platform !== 'win32') {
        await fs.chmod(pythonPath, 0o755);
      }
      
      
      this.spinner.succeed(chalk.green(`✓ ${platform} runtime downloaded and extracted`));
      
    } catch (error) {
      this.spinner.fail(chalk.red(`✗ Failed to download ${platform} runtime`));
      throw error;
    }
  }
  
  async extractZip(buffer, targetDir) {
    // For ZIP files, we need adm-zip
    const AdmZip = require('adm-zip');
    const zip = new AdmZip(buffer);
    zip.extractAllTo(targetDir, true);
  }
  
  async extractTarGz(buffer, targetDir) {
    const tempFile = path.join(require('os').tmpdir(), `python-${Date.now()}.tar.gz`);
    
    try {
      await fs.writeFile(tempFile, buffer);
      
      await tar.extract({
        file: tempFile,
        cwd: targetDir,
        strip: 1 // Remove top-level directory
      });
      
    } finally {
      // Clean up temp file
      await fs.remove(tempFile).catch(() => {});
    }
  }
  
  async installPythonPackages(platform) {
    this.spinner = ora('Installing Python packages...').start();
    
    try {
      const config = PYTHON_RUNTIMES[platform];
      const runtimeDir = path.join(PACKAGE_ROOT, 'runtime', platform);
      const pythonPath = path.join(runtimeDir, config.pythonExe);
      const requirementsPath = path.join(PACKAGE_ROOT, 'python', 'requirements.txt');
      
      // Check if requirements.txt exists
      if (!await fs.pathExists(requirementsPath)) {
        this.spinner.warn(chalk.yellow('No requirements.txt found, skipping package installation'));
        return;
      }
      
      // Get pip command for the platform
      let pipCommand;
      if (platform.startsWith('windows')) {
        // Try direct pip.exe first
        const pipExe = path.join(runtimeDir, 'Scripts', 'pip.exe');
        if (await fs.pathExists(pipExe)) {
          pipCommand = [pipExe];
        } else {
          // Fallback to python -m pip
          pipCommand = [pythonPath, '-m', 'pip'];
        }
      } else {
        // Unix systems
        pipCommand = [pythonPath, '-m', 'pip'];
      }
      
      // Install packages using pip
      const pipArgs = [...pipCommand, 'install', '-r', requirementsPath, '--no-cache-dir'];
      
      this.spinner.text = `Installing packages: ${pipArgs.join(' ')}`;
      
      await this.runCommand(pipArgs, {
        cwd: PACKAGE_ROOT,
        env: {
          ...process.env,
          PYTHONPATH: path.join(PACKAGE_ROOT, 'python')
        }
      });
      
      this.spinner.succeed(chalk.green('✓ Python packages installed successfully'));
      
    } catch (error) {
      this.spinner.fail(chalk.red('✗ Failed to install Python packages'));
      throw error;
    }
  }
  
  async fixWindowsPythonConfig(runtimeDir) {
    // Fix Windows embedded Python configuration to enable site packages
    try {
      const pthFile = path.join(runtimeDir, 'python311._pth');
      
      if (await fs.pathExists(pthFile)) {
        let content = await fs.readFile(pthFile, 'utf8');
        
        // Enable site.main() if not already enabled
        if (content.includes('#import site')) {
          content = content.replace('#import site', 'import site');
          await fs.writeFile(pthFile, content);
          this.spinner.text = 'Fixed Python site configuration';
        }
      }
    } catch (error) {
      console.warn(chalk.yellow('Warning: Could not fix Python configuration:', error.message));
      // Continue anyway
    }
  }
  
  runCommand(args, options = {}) {
    return new Promise((resolve, reject) => {
      console.log(chalk.gray(`Executing: ${args.join(' ')}`));
      
      const child = spawn(args[0], args.slice(1), {
        stdio: 'pipe',
        timeout: 300000, // 5 minute timeout for package installation
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
        if (code === 0) {
          resolve(stdout);
        } else {
          console.log(chalk.red(`Command stdout: ${stdout}`));
          console.log(chalk.red(`Command stderr: ${stderr}`));
          reject(new Error(`Command failed with code ${code}: ${stderr || stdout}`));
        }
      });
      
      child.on('error', (error) => {
        console.log(chalk.red(`Command error: ${error.message}`));
        reject(error);
      });
      
      // Handle timeout
      const timeout = setTimeout(() => {
        child.kill();
        reject(new Error('Command timeout'));
      }, options.timeout || 300000);
      
      child.on('exit', () => clearTimeout(timeout));
    });
  }
  
  async validateDownloads() {
    this.spinner = ora('Validating downloaded runtimes...').start();
    
    try {
      const runtimeDir = path.join(PACKAGE_ROOT, 'runtime');
      const platforms = await fs.readdir(runtimeDir);
      
      if (platforms.length === 0) {
        throw new Error('No runtimes were downloaded');
      }
      
      let validCount = 0;
      for (const platform of platforms) {
        const config = PYTHON_RUNTIMES[platform];
        if (config) {
          const pythonPath = path.join(runtimeDir, platform, config.pythonExe);
          if (await fs.pathExists(pythonPath)) {
            validCount++;
          }
        }
      }
      
      this.spinner.succeed(chalk.green(`✓ ${validCount} runtime(s) validated`));
      
      if (validCount === 0) {
        throw new Error('No valid runtimes found');
      }
      
    } catch (error) {
      this.spinner.fail(chalk.red('✗ Runtime validation failed'));
      throw error;
    }
  }
  
  async listAvailablePlatforms() {
    console.log(chalk.blue.bold('Available Platforms:'));
    console.log();
    
    for (const [platform, config] of Object.entries(PYTHON_RUNTIMES)) {
      console.log(`${chalk.cyan(platform.padEnd(15))} ${chalk.gray(config.size)}`);
      console.log(`  ${chalk.gray(config.url)}`);
      console.log();
    }
  }
  
  async getDownloadSummary() {
    const runtimeDir = path.join(PACKAGE_ROOT, 'runtime');
    
    if (!await fs.pathExists(runtimeDir)) {
      return { total: 0, platforms: [] };
    }
    
    const platforms = await fs.readdir(runtimeDir);
    const validPlatforms = [];
    
    for (const platform of platforms) {
      const config = PYTHON_RUNTIMES[platform];
      if (config) {
        const pythonPath = path.join(runtimeDir, platform, config.pythonExe);
        if (await fs.pathExists(pythonPath)) {
          validPlatforms.push(platform);
        }
      }
    }
    
    return {
      total: validPlatforms.length,
      platforms: validPlatforms
    };
  }
}

// CLI Interface
async function main() {
  const args = process.argv.slice(2);
  const downloader = new RuntimeDownloader();
  
  if (args.includes('--help') || args.includes('-h')) {
    console.log(`
${chalk.blue.bold('Python Runtime Downloader')}

Usage:
  node download-runtimes.js [options] [platforms...]

Options:
  --help, -h        Show this help message
  --current         Download runtime for current platform only
  --list            List available platforms
  --clean           Clean runtime directory only
  --summary         Show download summary

Platforms:
  windows-x64       Windows 64-bit
  windows-arm64     Windows ARM64
  macos-x64         macOS Intel
  macos-arm64       macOS Apple Silicon
  linux-x64         Linux 64-bit
  linux-arm64       Linux ARM64

Examples:
  node download-runtimes.js                    # Download all platforms
  node download-runtimes.js --current          # Download current platform only
  node download-runtimes.js windows-x64 linux-x64  # Download specific platforms
  node download-runtimes.js --list             # List available platforms
  node download-runtimes.js --summary          # Show what's downloaded
    `);
    return;
  }
  
  if (args.includes('--list')) {
    await downloader.listAvailablePlatforms();
    return;
  }
  
  if (args.includes('--clean')) {
    await downloader.cleanRuntimeDir();
    return;
  }
  
  if (args.includes('--summary')) {
    const summary = await downloader.getDownloadSummary();
    console.log(chalk.blue.bold('Download Summary:'));
    console.log(`Total runtimes: ${summary.total}`);
    console.log(`Platforms: ${summary.platforms.join(', ') || 'None'}`);
    return;
  }
  
  if (args.includes('--current')) {
    await downloader.downloadCurrent();
    return;
  }
  
  // Filter platform arguments
  const platformArgs = args.filter(arg => !arg.startsWith('--'));
  
  if (platformArgs.length > 0) {
    await downloader.downloadSpecific(platformArgs);
  } else {
    await downloader.downloadAll();
  }
}

if (require.main === module) {
  main().catch((error) => {
    console.error(chalk.red('Fatal error:'), error);
    process.exit(1);
  });
}

module.exports = { RuntimeDownloader, PYTHON_RUNTIMES };