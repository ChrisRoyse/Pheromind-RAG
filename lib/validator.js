/**
 * Installation Validator for MCP RAG Indexer
 * Validates installation integrity and functionality
 */

const path = require('path');
const fs = require('fs').promises;
const { spawn } = require('child_process');
const chalk = require('chalk');

const { PlatformManager } = require('./platform');
const { ConfigManager } = require('./config');

/**
 * Validate the complete installation
 */
async function validateInstallation() {
  console.log(chalk.blue('🔍 Validating MCP RAG Indexer installation...'));
  console.log();
  
  let allTestsPassed = true;
  const results = [];
  
  // Test 1: Platform Support
  try {
    console.log(chalk.gray('1. Checking platform support...'));
    const platform = PlatformManager.detectPlatform();
    console.log(chalk.green(`   ✓ Platform: ${platform}`));
    results.push({ test: 'Platform Support', status: 'PASS', details: platform });
  } catch (error) {
    console.log(chalk.red(`   ✗ Unsupported platform: ${error.message}`));
    results.push({ test: 'Platform Support', status: 'FAIL', error: error.message });
    allTestsPassed = false;
  }
  
  // Test 2: Python Runtime
  try {
    console.log(chalk.gray('2. Checking Python runtime...'));
    const pythonPath = PlatformManager.getPythonExecutable();
    const isValid = await PlatformManager.validateRuntime();
    
    if (isValid) {
      console.log(chalk.green(`   ✓ Python runtime: ${pythonPath}`));
      results.push({ test: 'Python Runtime', status: 'PASS', details: pythonPath });
    } else {
      console.log(chalk.red(`   ✗ Python runtime not found: ${pythonPath}`));
      results.push({ test: 'Python Runtime', status: 'FAIL', error: 'Runtime not found' });
      allTestsPassed = false;
    }
  } catch (error) {
    console.log(chalk.red(`   ✗ Python runtime error: ${error.message}`));
    results.push({ test: 'Python Runtime', status: 'FAIL', error: error.message });
    allTestsPassed = false;
  }
  
  // Test 3: MCP Server Script
  try {
    console.log(chalk.gray('3. Checking MCP server script...'));
    const serverScript = path.join(__dirname, '..', 'python', 'mcp_rag_server.py');
    await fs.access(serverScript);
    console.log(chalk.green(`   ✓ MCP server script found`));
    results.push({ test: 'MCP Server Script', status: 'PASS', details: serverScript });
  } catch (error) {
    console.log(chalk.red(`   ✗ MCP server script not found`));
    results.push({ test: 'MCP Server Script', status: 'FAIL', error: 'Script not found' });
    allTestsPassed = false;
  }
  
  // Test 4: Python Dependencies
  try {
    console.log(chalk.gray('4. Checking Python dependencies...'));
    const dependencyTest = await testPythonDependencies();
    
    if (dependencyTest.success) {
      console.log(chalk.green(`   ✓ Python dependencies: ${dependencyTest.packages.length} packages`));
      results.push({ test: 'Python Dependencies', status: 'PASS', details: `${dependencyTest.packages.length} packages` });
    } else {
      console.log(chalk.red(`   ✗ Missing dependencies: ${dependencyTest.missing.join(', ')}`));
      results.push({ test: 'Python Dependencies', status: 'FAIL', error: `Missing: ${dependencyTest.missing.join(', ')}` });
      allTestsPassed = false;
    }
  } catch (error) {
    console.log(chalk.red(`   ✗ Dependency check failed: ${error.message}`));
    results.push({ test: 'Python Dependencies', status: 'FAIL', error: error.message });
    allTestsPassed = false;
  }
  
  // Test 5: ML Models
  try {
    console.log(chalk.gray('5. Checking ML models...'));
    const modelsDir = path.join(__dirname, '..', 'models');
    const modelPath = path.join(modelsDir, 'sentence-transformers', 'all-MiniLM-L6-v2');
    
    try {
      await fs.access(modelPath);
      console.log(chalk.green(`   ✓ Pre-bundled models found`));
      results.push({ test: 'ML Models', status: 'PASS', details: 'Pre-bundled' });
    } catch {
      console.log(chalk.yellow(`   ⚠ Models will be downloaded on first use`));
      results.push({ test: 'ML Models', status: 'WARN', details: 'Will download on demand' });
    }
  } catch (error) {
    console.log(chalk.yellow(`   ⚠ Model check failed: ${error.message}`));
    results.push({ test: 'ML Models', status: 'WARN', error: error.message });
  }
  
  // Test 6: Version Command
  try {
    console.log(chalk.gray('6. Testing version command...'));
    const versionOutput = await testVersionCommand();
    console.log(chalk.green(`   ✓ Version command: ${versionOutput.trim()}`));
    results.push({ test: 'Version Command', status: 'PASS', details: versionOutput.trim() });
  } catch (error) {
    console.log(chalk.red(`   ✗ Version command failed: ${error.message}`));
    results.push({ test: 'Version Command', status: 'FAIL', error: error.message });
    allTestsPassed = false;
  }
  
  // Test 7: Configuration
  try {
    console.log(chalk.gray('7. Checking configuration...'));
    const configManager = new ConfigManager();
    const config = await configManager.loadConfig();
    const errors = configManager.validateConfig(config);
    
    if (errors.length === 0) {
      console.log(chalk.green(`   ✓ Configuration valid`));
      results.push({ test: 'Configuration', status: 'PASS', details: 'Valid' });
    } else {
      console.log(chalk.red(`   ✗ Configuration errors: ${errors.join(', ')}`));
      results.push({ test: 'Configuration', status: 'FAIL', error: errors.join(', ') });
      allTestsPassed = false;
    }
  } catch (error) {
    console.log(chalk.red(`   ✗ Configuration check failed: ${error.message}`));
    results.push({ test: 'Configuration', status: 'FAIL', error: error.message });
    allTestsPassed = false;
  }
  
  // Test 8: Claude Configuration
  try {
    console.log(chalk.gray('8. Checking Claude configuration...'));
    const claudeConfigPath = getClaudeConfigPath();
    
    try {
      const configContent = await fs.readFile(claudeConfigPath, 'utf8');
      const config = JSON.parse(configContent);
      
      if (config.mcpServers && config.mcpServers['rag-indexer']) {
        console.log(chalk.green(`   ✓ Claude configured with MCP server`));
        results.push({ test: 'Claude Configuration', status: 'PASS', details: 'MCP server configured' });
      } else {
        console.log(chalk.yellow(`   ⚠ Claude configuration found but MCP server not configured`));
        results.push({ test: 'Claude Configuration', status: 'WARN', details: 'MCP server not configured' });
      }
    } catch {
      console.log(chalk.yellow(`   ⚠ Claude configuration not found`));
      results.push({ test: 'Claude Configuration', status: 'WARN', details: 'Configuration not found' });
    }
  } catch (error) {
    console.log(chalk.yellow(`   ⚠ Claude configuration check failed: ${error.message}`));
    results.push({ test: 'Claude Configuration', status: 'WARN', error: error.message });
  }
  
  // Summary
  console.log();
  if (allTestsPassed) {
    console.log(chalk.green.bold('✅ All critical tests passed!'));
    console.log(chalk.green('Installation is valid and ready to use.'));
  } else {
    console.log(chalk.red.bold('❌ Some tests failed'));
    console.log(chalk.red('Installation may not work correctly.'));
    console.log(chalk.yellow('Try reinstalling: npm uninstall -g mcp-rag-indexer && npm install -g mcp-rag-indexer'));
  }
  
  // Show system info
  console.log();
  console.log(chalk.blue('System Information:'));
  const systemInfo = PlatformManager.getSystemInfo();
  console.log(chalk.gray(`  Platform: ${systemInfo.platform}-${systemInfo.arch} (${systemInfo.platformId})`));
  console.log(chalk.gray(`  Node.js: ${systemInfo.nodeVersion}`));
  console.log(chalk.gray(`  Python: ${systemInfo.pythonPath}`));
  
  return allTestsPassed;
}

/**
 * Test Python dependencies
 */
async function testPythonDependencies() {
  const pythonPath = PlatformManager.getPythonExecutable();
  
  const requiredPackages = [
    'mcp',
    'langchain',
    'langchain_community',
    'langchain_huggingface',
    'chromadb',
    'sentence_transformers',
    'numpy',
    'torch'
  ];
  
  const testScript = `
import sys
packages = ${JSON.stringify(requiredPackages)}
missing = []
found = []

for package in packages:
    try:
        __import__(package)
        found.append(package)
    except ImportError:
        missing.append(package)

print(f"FOUND:{','.join(found)}")
if missing:
    print(f"MISSING:{','.join(missing)}")
    sys.exit(1)
`;
  
  try {
    const output = await runCommand([pythonPath, '-c', testScript]);
    const lines = output.split('\n');
    const foundLine = lines.find(line => line.startsWith('FOUND:'));
    const missingLine = lines.find(line => line.startsWith('MISSING:'));
    
    const found = foundLine ? foundLine.replace('FOUND:', '').split(',').filter(p => p) : [];
    const missing = missingLine ? missingLine.replace('MISSING:', '').split(',').filter(p => p) : [];
    
    return {
      success: missing.length === 0,
      packages: found,
      missing: missing
    };
  } catch (error) {
    return {
      success: false,
      packages: [],
      missing: requiredPackages,
      error: error.message
    };
  }
}

/**
 * Test version command
 */
async function testVersionCommand() {
  const pythonPath = PlatformManager.getPythonExecutable();
  const serverScript = path.join(__dirname, '..', 'python', 'mcp_rag_server.py');
  
  return await runCommand([pythonPath, serverScript, '--version']);
}

/**
 * Get Claude configuration path
 */
function getClaudeConfigPath() {
  const os = require('os');
  return path.join(os.homedir(), '.claude.json');
}

/**
 * Run a command and return output
 */
function runCommand(args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(args[0], args.slice(1), {
      stdio: 'pipe',
      timeout: 15000,
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
    }, 15000);
  });
}

// Export for CLI usage
if (require.main === module) {
  validateInstallation().then((success) => {
    process.exit(success ? 0 : 1);
  }).catch((error) => {
    console.error(chalk.red('Validation failed:'), error);
    process.exit(1);
  });
}

module.exports = { validateInstallation };