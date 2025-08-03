#!/usr/bin/env node
/**
 * Build Script for MCP RAG Indexer NPM Package
 * Downloads Python runtimes, models, and dependencies for all platforms
 */

const fs = require('fs-extra');
const path = require('path');
const { execSync } = require('child_process');
const fetch = require('node-fetch');
const tar = require('tar');
const chalk = require('chalk');
const ora = require('ora');

const PACKAGE_ROOT = path.join(__dirname, '..');

// Python runtime configurations
const PYTHON_RUNTIMES = {
  'windows-x64': {
    url: 'https://www.python.org/ftp/python/3.11.7/python-3.11.7-embed-amd64.zip',
    extract: 'zip',
    pythonExe: 'python.exe'
  },
  'windows-arm64': {
    url: 'https://www.python.org/ftp/python/3.11.7/python-3.11.7-embed-arm64.zip',
    extract: 'zip',
    pythonExe: 'python.exe'
  },
  'macos-x64': {
    url: 'https://github.com/indygreg/python-build-standalone/releases/download/20231002/cpython-3.11.6+20231002-x86_64-apple-darwin-install_only.tar.gz',
    extract: 'tar.gz',
    pythonExe: 'bin/python3'
  },
  'macos-arm64': {
    url: 'https://github.com/indygreg/python-build-standalone/releases/download/20231002/cpython-3.11.6+20231002-aarch64-apple-darwin-install_only.tar.gz',
    extract: 'tar.gz',
    pythonExe: 'bin/python3'
  },
  'linux-x64': {
    url: 'https://github.com/indygreg/python-build-standalone/releases/download/20231002/cpython-3.11.6+20231002-x86_64-unknown-linux-gnu-install_only.tar.gz',
    extract: 'tar.gz',
    pythonExe: 'bin/python3'
  },
  'linux-arm64': {
    url: 'https://github.com/indygreg/python-build-standalone/releases/download/20231002/cpython-3.11.6+20231002-aarch64-unknown-linux-gnu-install_only.tar.gz',
    extract: 'tar.gz',
    pythonExe: 'bin/python3'
  }
};

// ML Model configurations
const ML_MODELS = {
  'sentence-transformers/all-MiniLM-L6-v2': {
    files: [
      'config.json',
      'pytorch_model.bin',
      'sentence_bert_config.json',
      'tokenizer.json',
      'tokenizer_config.json',
      'vocab.txt',
      'modules.json',
      '1_Pooling/config.json'
    ],
    baseUrl: 'https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/'
  }
};

class PackageBuilder {
  constructor() {
    this.spinner = null;
  }
  
  async build() {
    console.log(chalk.blue.bold('🔨 Building MCP RAG Indexer NPM Package'));
    console.log();
    
    try {
      // 1. Clean build directories
      await this.clean();
      
      // 2. Copy Python source files
      await this.copyPythonFiles();
      
      // 3. Download Python runtimes
      await this.downloadRuntimes();
      
      // 4. Install Python dependencies
      await this.installPythonDependencies();
      
      // 5. Download ML models
      await this.downloadModels();
      
      // 6. Validate package
      await this.validatePackage();
      
      console.log();
      console.log(chalk.green.bold('✅ Package build complete!'));
      console.log(chalk.gray('Package is ready for npm publish'));
      
    } catch (error) {
      console.log();
      console.error(chalk.red.bold('❌ Build failed:'), error.message);
      process.exit(1);
    }
  }
  
  async clean() {
    this.spinner = ora('Cleaning build directories...').start();
    
    try {
      const dirsToClean = ['runtime', 'models', 'python'];
      
      for (const dir of dirsToClean) {
        const dirPath = path.join(PACKAGE_ROOT, dir);
        if (await fs.pathExists(dirPath)) {
          await fs.remove(dirPath);
        }
      }
      
      this.spinner.succeed(chalk.green('✓ Build directories cleaned'));
    } catch (error) {
      this.spinner.fail(chalk.red('✗ Failed to clean directories'));
      throw error;
    }
  }
  
  async copyPythonFiles() {
    this.spinner = ora('Copying Python source files...').start();
    
    try {
      const pythonDir = path.join(PACKAGE_ROOT, 'python');
      await fs.ensureDir(pythonDir);
      
      // Copy Python files from current directory
      const pythonFiles = [
        'mcp_rag_server.py',
        'indexer_universal.py',
        'query_universal.py',
        'git_tracker.py',
        'cache_manager.py',
        'requirements.txt'
      ];
      
      for (const file of pythonFiles) {
        const srcPath = path.join(PACKAGE_ROOT, file);
        const destPath = path.join(pythonDir, file);
        
        if (await fs.pathExists(srcPath)) {
          await fs.copy(srcPath, destPath);
        } else {
          this.spinner.warn(chalk.yellow(`Python file not found: ${file}`));
        }
      }
      
      this.spinner.succeed(chalk.green('✓ Python source files copied'));
    } catch (error) {
      this.spinner.fail(chalk.red('✗ Failed to copy Python files'));
      throw error;
    }
  }
  
  async downloadRuntimes() {
    console.log(chalk.blue('📥 Downloading Python runtimes...'));
    
    for (const [platform, config] of Object.entries(PYTHON_RUNTIMES)) {
      await this.downloadRuntime(platform, config);
    }
  }
  
  async downloadRuntime(platform, config) {
    this.spinner = ora(`Downloading ${platform} runtime...`).start();
    
    try {
      const runtimeDir = path.join(PACKAGE_ROOT, 'runtime', platform);
      await fs.ensureDir(runtimeDir);
      
      this.spinner.text = `Downloading ${platform} runtime from ${config.url}`;
      
      const response = await fetch(config.url);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      
      const buffer = await response.buffer();
      
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
      
      this.spinner.succeed(chalk.green(`✓ ${platform} runtime downloaded`));
      
    } catch (error) {
      this.spinner.fail(chalk.red(`✗ Failed to download ${platform} runtime`));
      throw error;
    }
  }
  
  async extractZip(buffer, targetDir) {
    // For ZIP files, we'll use a simple extraction method
    const AdmZip = require('adm-zip');
    const zip = new AdmZip(buffer);
    zip.extractAllTo(targetDir, true);
  }
  
  async extractTarGz(buffer, targetDir) {
    // Extract tar.gz to target directory
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
  
  async installPythonDependencies() {
    console.log(chalk.blue('📦 Installing Python dependencies...'));
    
    for (const [platform, config] of Object.entries(PYTHON_RUNTIMES)) {
      await this.installDependenciesForPlatform(platform, config);
    }
  }
  
  async installDependenciesForPlatform(platform, config) {
    this.spinner = ora(`Installing dependencies for ${platform}...`).start();
    
    try {
      const runtimeDir = path.join(PACKAGE_ROOT, 'runtime', platform);
      const pythonPath = path.join(runtimeDir, config.pythonExe);
      const requirementsPath = path.join(PACKAGE_ROOT, 'python', 'requirements.txt');
      
      // Ensure requirements file exists
      if (!await fs.pathExists(requirementsPath)) {
        throw new Error('requirements.txt not found');
      }
      
      // Create site-packages directory
      let sitePackagesDir;
      if (platform.startsWith('windows')) {
        sitePackagesDir = path.join(runtimeDir, 'Lib', 'site-packages');
      } else {
        sitePackagesDir = path.join(runtimeDir, 'lib', 'python3.11', 'site-packages');
      }
      
      await fs.ensureDir(sitePackagesDir);
      
      // Install dependencies
      const pipArgs = [
        '-m', 'pip', 'install',
        '-r', requirementsPath,
        '--target', sitePackagesDir,
        '--no-deps', // Don't install dependencies of dependencies to keep size down
        '--quiet'
      ];
      
      execSync(`"${pythonPath}" ${pipArgs.join(' ')}`, {
        cwd: PACKAGE_ROOT,
        stdio: 'pipe'
      });
      
      this.spinner.succeed(chalk.green(`✓ Dependencies installed for ${platform}`));
      
    } catch (error) {
      this.spinner.fail(chalk.red(`✗ Failed to install dependencies for ${platform}`));
      // Don't throw - some platforms might fail but we want to continue
      console.error(chalk.yellow(`Warning: ${error.message}`));
    }
  }
  
  async downloadModels() {
    console.log(chalk.blue('🧠 Downloading ML models...'));
    
    for (const [modelName, config] of Object.entries(ML_MODELS)) {
      await this.downloadModel(modelName, config);
    }
  }
  
  async downloadModel(modelName, config) {
    this.spinner = ora(`Downloading ${modelName}...`).start();
    
    try {
      const modelDir = path.join(PACKAGE_ROOT, 'models', modelName);
      await fs.ensureDir(modelDir);
      
      for (const file of config.files) {
        this.spinner.text = `Downloading ${modelName}/${file}...`;
        
        const url = `${config.baseUrl}${file}`;
        const response = await fetch(url);
        
        if (!response.ok) {
          // Skip optional files
          if (file.includes('/')) {
            // Ensure subdirectory exists
            await fs.ensureDir(path.join(modelDir, path.dirname(file)));
          }
          continue;
        }
        
        const buffer = await response.buffer();
        const filePath = path.join(modelDir, file);
        
        // Ensure directory exists
        await fs.ensureDir(path.dirname(filePath));
        await fs.writeFile(filePath, buffer);
      }
      
      this.spinner.succeed(chalk.green(`✓ ${modelName} downloaded`));
      
    } catch (error) {
      this.spinner.fail(chalk.red(`✗ Failed to download ${modelName}`));
      // Don't throw - models can be downloaded on demand
      console.error(chalk.yellow(`Warning: ${error.message}`));
    }
  }
  
  async validatePackage() {
    this.spinner = ora('Validating package...').start();
    
    try {
      // Check that at least one runtime exists
      const runtimeDir = path.join(PACKAGE_ROOT, 'runtime');
      const platforms = await fs.readdir(runtimeDir);
      
      if (platforms.length === 0) {
        throw new Error('No Python runtimes found');
      }
      
      // Check that Python files exist
      const pythonDir = path.join(PACKAGE_ROOT, 'python');
      const requiredFiles = ['mcp_rag_server.py', 'requirements.txt'];
      
      for (const file of requiredFiles) {
        const filePath = path.join(pythonDir, file);
        if (!await fs.pathExists(filePath)) {
          throw new Error(`Required Python file missing: ${file}`);
        }
      }
      
      // Calculate package size
      const packageSize = await this.calculatePackageSize();
      
      this.spinner.succeed(chalk.green(`✓ Package validated (${this.formatSize(packageSize)})`));
      
      if (packageSize > 500 * 1024 * 1024) {
        console.log(chalk.yellow(`⚠ Package size is large: ${this.formatSize(packageSize)}`));
      }
      
    } catch (error) {
      this.spinner.fail(chalk.red('✗ Package validation failed'));
      throw error;
    }
  }
  
  async calculatePackageSize() {
    const { execSync } = require('child_process');
    
    try {
      // Use du command if available (Unix)
      const output = execSync('du -sb .', { cwd: PACKAGE_ROOT, encoding: 'utf8' });
      return parseInt(output.split('\t')[0]);
    } catch {
      // Fallback: recursively calculate size
      return await this.recursiveSize(PACKAGE_ROOT);
    }
  }
  
  async recursiveSize(dir) {
    let size = 0;
    const files = await fs.readdir(dir);
    
    for (const file of files) {
      const filePath = path.join(dir, file);
      const stat = await fs.stat(filePath);
      
      if (stat.isDirectory()) {
        size += await this.recursiveSize(filePath);
      } else {
        size += stat.size;
      }
    }
    
    return size;
  }
  
  formatSize(bytes) {
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;
    
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    
    return `${size.toFixed(1)} ${units[unitIndex]}`;
  }
}

// Add missing dependency
try {
  require('adm-zip');
} catch {
  console.log(chalk.yellow('Installing build dependency: adm-zip'));
  execSync('npm install adm-zip', { stdio: 'inherit' });
}

// Main execution
async function main() {
  const builder = new PackageBuilder();
  await builder.build();
}

if (require.main === module) {
  main().catch((error) => {
    console.error(chalk.red('Build failed:'), error);
    process.exit(1);
  });
}

module.exports = { PackageBuilder };