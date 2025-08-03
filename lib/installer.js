/**
 * Post-Install Setup for MCP RAG Indexer
 * Handles Claude Code configuration and installation validation
 */

const os = require('os');
const path = require('path');
const fs = require('fs').promises;
const { spawn } = require('child_process');
const chalk = require('chalk');
const ora = require('ora');

const { PlatformManager } = require('./platform');
const { ConfigManager } = require('./config');

class Installer {
  constructor() {
    this.spinner = null;
  }
  
  /**
   * Main installation flow
   */
  async run() {
    console.log(chalk.blue.bold('🚀 Setting up MCP RAG Indexer...'));
    console.log();
    
    try {
      // 1. Validate installation
      await this.validateInstallation();
      
      // 2. Configure Claude Code
      await this.configureClaudeCode();
      
      // 3. Test functionality
      await this.runTests();
      
      console.log();
      console.log(chalk.green.bold('✅ Installation complete!'));
      this.printUsageInstructions();
      
    } catch (error) {
      console.log();
      console.error(chalk.red.bold('❌ Installation failed:'), error.message);
      
      if (error.code === 'PLATFORM_UNSUPPORTED') {
        console.log(chalk.yellow('Your platform is not currently supported.'));
        console.log(chalk.yellow('Supported platforms: Windows x64, macOS x64/ARM64, Linux x64/ARM64'));
      } else if (error.code === 'RUNTIME_MISSING') {
        console.log(chalk.yellow('The Python runtime was not properly bundled.'));
        console.log(chalk.yellow('Try reinstalling: npm uninstall -g mcp-rag-indexer && npm install -g mcp-rag-indexer'));
      }
      
      process.exit(1);
    }
  }
  
  /**
   * Validate that all required components are present
   */
  async validateInstallation() {
    this.spinner = ora('Validating installation...').start();
    
    try {
      // Check platform support
      if (!PlatformManager.isPlatformSupported()) {
        throw Object.assign(new Error('Unsupported platform'), { code: 'PLATFORM_UNSUPPORTED' });
      }
      
      // Check Python runtime
      const pythonPath = PlatformManager.getPythonExecutable();
      if (!await this.fileExists(pythonPath)) {
        throw Object.assign(new Error('Python runtime not found'), { code: 'RUNTIME_MISSING' });
      }
      
      // Check MCP server script
      const serverScript = path.join(__dirname, '..', 'python', 'mcp_rag_server.py');
      if (!await this.fileExists(serverScript)) {
        throw Object.assign(new Error('MCP server script not found'), { code: 'SCRIPT_MISSING' });
      }
      
      // Check models directory
      const modelsDir = path.join(__dirname, '..', 'models');
      if (!await this.fileExists(modelsDir)) {
        this.spinner.warn(chalk.yellow('ML models directory not found - will download on first use'));
      } else {
        // Check for specific model files
        const modelPath = path.join(modelsDir, 'sentence-transformers', 'all-MiniLM-L6-v2');
        if (await this.fileExists(modelPath)) {
          this.spinner.succeed(chalk.green('✓ Pre-bundled models found'));
        }
      }
      
      this.spinner.succeed(chalk.green('✓ Installation validated'));
      
    } catch (error) {
      this.spinner.fail(chalk.red('✗ Validation failed'));
      throw error;
    }
  }
  
  /**
   * Configure Claude Code to use the MCP server
   */
  async configureClaudeCode() {
    this.spinner = ora('Configuring Claude Code...').start();
    
    try {
      const claudeConfigPath = this.getClaudeConfigPath();
      
      // Get global npm bin path for executable
      const { execSync } = require('child_process');
      let executablePath;
      
      try {
        const npmBin = execSync('npm bin -g', { encoding: 'utf8' }).trim();
        executablePath = path.join(npmBin, 'mcp-rag-indexer');
        
        // On Windows, add .cmd extension
        if (process.platform === 'win32') {
          executablePath += '.cmd';
        }
      } catch (error) {
        // Fallback to direct command
        executablePath = 'mcp-rag-indexer';
      }
      
      // Create MCP configuration
      const mcpConfig = {
        "rag-indexer": {
          "type": "stdio",
          "command": executablePath,
          "args": ["--log-level", "info"]
        }
      };
      
      // Load existing Claude config or create new
      let config = {};
      try {
        const configContent = await fs.readFile(claudeConfigPath, 'utf8');
        config = JSON.parse(configContent);
      } catch (error) {
        // Config doesn't exist, will create new
        this.spinner.info(chalk.blue('Creating new Claude configuration file'));
      }
      
      // Add MCP servers section
      if (!config.mcpServers) {
        config.mcpServers = {};
      }
      
      config.mcpServers = { ...config.mcpServers, ...mcpConfig };
      
      // Ensure parent directory exists
      await fs.mkdir(path.dirname(claudeConfigPath), { recursive: true });
      
      // Save configuration
      await fs.writeFile(claudeConfigPath, JSON.stringify(config, null, 2));
      
      this.spinner.succeed(chalk.green('✓ Claude Code configured'));
      
      return true;
    } catch (error) {
      this.spinner.fail(chalk.red('✗ Claude configuration failed'));
      throw error;
    }
  }
  
  /**
   * Get the path to Claude configuration file
   */
  getClaudeConfigPath() {
    const homeDir = os.homedir();
    
    // Try different possible locations
    const possiblePaths = [
      path.join(homeDir, '.claude.json'),
      path.join(homeDir, '.config', 'claude', 'config.json'),
      path.join(homeDir, 'AppData', 'Roaming', 'claude', 'config.json'), // Windows
      path.join(homeDir, 'Library', 'Application Support', 'claude', 'config.json') // macOS
    ];
    
    // Return the first existing path, or default to ~/.claude.json
    for (const configPath of possiblePaths) {
      if (fs.existsSync && fs.existsSync(configPath)) {
        return configPath;
      }
    }
    
    return possiblePaths[0]; // Default to ~/.claude.json
  }
  
  /**
   * Run basic functionality tests
   */
  async runTests() {
    this.spinner = ora('Testing functionality...').start();
    
    try {
      // Test Python import
      await this.testPythonImport();
      
      // Test version command
      await this.testVersionCommand();
      
      this.spinner.succeed(chalk.green('✓ Functionality tests passed'));
      
    } catch (error) {
      this.spinner.fail(chalk.red('✗ Functionality tests failed'));
      throw error;
    }
  }
  
  /**
   * Test that Python can import our modules
   */
  async testPythonImport() {
    const pythonPath = PlatformManager.getPythonExecutable();
    const testScript = `
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'python'))

try:
    from mcp_rag_server import RAGIndexerServer
    server = RAGIndexerServer()
    print("SUCCESS: MCP server can be imported and initialized")
    server.shutdown()
except Exception as e:
    print(f"ERROR: {e}")
    sys.exit(1)
`;
    
    const testFile = path.join(__dirname, '..', 'test_import.py');
    await fs.writeFile(testFile, testScript);
    
    try {
      await this.runCommand([pythonPath, testFile]);
      await fs.unlink(testFile); // Clean up
    } catch (error) {
      await fs.unlink(testFile).catch(() => {}); // Clean up on error
      throw error;
    }
  }
  
  /**
   * Test version command
   */
  async testVersionCommand() {
    const pythonPath = PlatformManager.getPythonExecutable();
    const serverScript = path.join(__dirname, '..', 'python', 'mcp_rag_server.py');
    
    await this.runCommand([pythonPath, serverScript, '--version']);
  }
  
  /**
   * Run a command and return result
   */
  runCommand(args, options = {}) {
    return new Promise((resolve, reject) => {
      const child = spawn(args[0], args.slice(1), {
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
        if (code === 0) {
          resolve(stdout);
        } else {
          reject(new Error(`Command failed with code ${code}: ${stderr || stdout}`));
        }
      });
      
      child.on('error', (error) => {
        reject(error);
      });
      
      // Handle timeout
      setTimeout(() => {
        child.kill();
        reject(new Error('Command timeout'));
      }, 10000);
    });
  }
  
  /**
   * Check if file exists
   */
  async fileExists(filePath) {
    try {
      await fs.access(filePath);
      return true;
    } catch {
      return false;
    }
  }
  
  /**
   * Print usage instructions
   */
  printUsageInstructions() {
    console.log(chalk.cyan(`
🎉 MCP RAG Indexer v${require('../package.json').version} installed successfully!

${chalk.bold('Next steps:')}
1. ${chalk.green('Restart Claude Code completely')}
2. ${chalk.green('Check MCP connection:')} Type '/mcp' in Claude  
3. ${chalk.green('Index your first project:')} "Index C:/your/project/path"
4. ${chalk.green('Search your code:')} "Find authentication functions"

${chalk.bold('CLI Commands:')}
• mcp-rag-indexer --version          Show version information
• mcp-rag-indexer --status           Show installation status  
• mcp-rag-indexer --configure        Reconfigure Claude integration
• mcp-rag-indexer --validate         Validate installation

${chalk.bold('Troubleshooting:')}
• ${chalk.yellow('Logs:')} ~/.mcp-rag-indexer/server.log
• ${chalk.yellow('Test:')} mcp-rag-indexer --validate
• ${chalk.yellow('Reconfigure:')} mcp-rag-indexer --configure

${chalk.bold('Documentation:')} https://github.com/your-org/mcp-rag-indexer
    `));
  }
}

/**
 * Main function for standalone execution
 */
async function main() {
  const installer = new Installer();
  await installer.run();
}

/**
 * Export for programmatic usage
 */
async function configureClaudeCode() {
  const installer = new Installer();
  return await installer.configureClaudeCode();
}

// Run installer if called directly
if (require.main === module) {
  main().catch((error) => {
    console.error(chalk.red('Installation failed:'), error);
    process.exit(1);
  });
}

module.exports = { Installer, configureClaudeCode };