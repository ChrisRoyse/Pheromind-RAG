/**
 * Uninstaller for MCP RAG Indexer
 * Cleans up configuration and data when package is uninstalled
 */

const os = require('os');
const path = require('path');
const fs = require('fs').promises;
const chalk = require('chalk');

class Uninstaller {
  constructor() {
    this.dataDir = path.join(os.homedir(), '.mcp-rag-indexer');
    this.claudeConfigPath = path.join(os.homedir(), '.claude.json');
  }
  
  async run() {
    console.log(chalk.blue('🗑️  Uninstalling MCP RAG Indexer...'));
    console.log();
    
    try {
      let actionsPerformed = 0;
      
      // 1. Remove Claude configuration
      const claudeRemoved = await this.removeClaudeConfig();
      if (claudeRemoved) actionsPerformed++;
      
      // 2. Ask about user data
      const removeData = await this.askRemoveUserData();
      if (removeData) {
        const dataRemoved = await this.removeUserData();
        if (dataRemoved) actionsPerformed++;
      }
      
      console.log();
      if (actionsPerformed > 0) {
        console.log(chalk.green('✅ Uninstallation complete!'));
        console.log(chalk.gray(`${actionsPerformed} cleanup action(s) performed.`));
      } else {
        console.log(chalk.yellow('ℹ️  No cleanup needed - package was already clean.'));
      }
      
      console.log();
      console.log(chalk.blue('Thank you for using MCP RAG Indexer! 👋'));
      
    } catch (error) {
      console.log();
      console.error(chalk.red('❌ Uninstallation failed:'), error.message);
      process.exit(1);
    }
  }
  
  async removeClaudeConfig() {
    try {
      // Check if Claude config exists
      if (!await this.fileExists(this.claudeConfigPath)) {
        console.log(chalk.gray('Claude configuration not found - nothing to remove'));
        return false;
      }
      
      // Read current config
      const configContent = await fs.readFile(this.claudeConfigPath, 'utf8');
      const config = JSON.parse(configContent);
      
      // Check if our MCP server is configured
      if (!config.mcpServers || !config.mcpServers['rag-indexer']) {
        console.log(chalk.gray('MCP server not found in Claude configuration'));
        return false;
      }
      
      // Remove our MCP server
      delete config.mcpServers['rag-indexer'];
      
      // If mcpServers is now empty, remove the section
      if (Object.keys(config.mcpServers).length === 0) {
        delete config.mcpServers;
      }
      
      // Save updated config
      await fs.writeFile(this.claudeConfigPath, JSON.stringify(config, null, 2));
      
      console.log(chalk.green('✓ Removed MCP server from Claude configuration'));
      console.log(chalk.yellow('  Please restart Claude Code to apply changes'));
      
      return true;
      
    } catch (error) {
      console.log(chalk.yellow(`⚠ Failed to remove Claude configuration: ${error.message}`));
      return false;
    }
  }
  
  async askRemoveUserData() {
    // In a real implementation, you'd use a prompt library
    // For now, we'll be conservative and NOT remove user data by default
    
    if (!await this.fileExists(this.dataDir)) {
      return false; // No data to remove
    }
    
    // Check environment variable for automated removal
    if (process.env.MCP_RAG_REMOVE_DATA === 'true') {
      return true;
    }
    
    // Default to keeping user data for safety
    console.log(chalk.blue('User data found at:'), this.dataDir);
    console.log(chalk.gray('This includes indexed projects, logs, and configuration.'));
    console.log(chalk.gray('To remove this data, set environment variable: MCP_RAG_REMOVE_DATA=true'));
    console.log(chalk.yellow('Keeping user data (recommended)'));
    
    return false;
  }
  
  async removeUserData() {
    try {
      if (!await this.fileExists(this.dataDir)) {
        return false;
      }
      
      // Calculate data size
      const dataSize = await this.calculateDirectorySize(this.dataDir);
      
      // Remove the entire data directory
      await this.removeDirectory(this.dataDir);
      
      console.log(chalk.green(`✓ Removed user data directory (${this.formatSize(dataSize)})`));
      console.log(chalk.gray(`  Deleted: ${this.dataDir}`));
      
      return true;
      
    } catch (error) {
      console.log(chalk.yellow(`⚠ Failed to remove user data: ${error.message}`));
      return false;
    }
  }
  
  async removeDirectory(dir) {
    try {
      await fs.rm(dir, { recursive: true, force: true });
    } catch (error) {
      // Fallback for older Node.js versions
      await this.rimraf(dir);
    }
  }
  
  async rimraf(dir) {
    const files = await fs.readdir(dir).catch(() => []);
    
    for (const file of files) {
      const filePath = path.join(dir, file);
      const stat = await fs.stat(filePath).catch(() => null);
      
      if (stat && stat.isDirectory()) {
        await this.rimraf(filePath);
      } else {
        await fs.unlink(filePath).catch(() => {});
      }
    }
    
    await fs.rmdir(dir).catch(() => {});
  }
  
  async calculateDirectorySize(dir) {
    let size = 0;
    
    try {
      const files = await fs.readdir(dir);
      
      for (const file of files) {
        const filePath = path.join(dir, file);
        const stat = await fs.stat(filePath);
        
        if (stat.isDirectory()) {
          size += await this.calculateDirectorySize(filePath);
        } else {
          size += stat.size;
        }
      }
    } catch (error) {
      // Directory might not exist or be accessible
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
  
  async fileExists(filePath) {
    try {
      await fs.access(filePath);
      return true;
    } catch {
      return false;
    }
  }
}

// Main function for direct execution
async function main() {
  const uninstaller = new Uninstaller();
  await uninstaller.run();
}

// Run uninstaller if called directly
if (require.main === module) {
  main().catch((error) => {
    console.error(chalk.red('Uninstallation failed:'), error);
    process.exit(1);
  });
}

module.exports = { Uninstaller };