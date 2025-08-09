const fs = require('fs');
const path = require('path');

const configPath = path.join(process.env.USERPROFILE || process.env.HOME, '.claude.json');

console.log('Reading Claude config from:', configPath);
console.log('');

try {
  const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
  
  // Find all MCP server configurations
  const mcpServers = {};
  
  for (const [key, value] of Object.entries(config)) {
    if (value && typeof value === 'object' && value.type === 'stdio') {
      mcpServers[key] = value;
    }
  }
  
  console.log('Found MCP Servers:');
  console.log('==================');
  
  for (const [name, config] of Object.entries(mcpServers)) {
    console.log(`\n📦 ${name}:`);
    console.log(`   Type: ${config.type}`);
    console.log(`   Command: ${config.command}`);
    console.log(`   Args: ${JSON.stringify(config.args || [])}`);
    
    // Check if it's claude-flow or ruv-swarm
    if (name.includes('claude-flow') || config.command?.includes('claude-flow')) {
      console.log('   ⚠️  This is claude-flow server');
    }
    if (name.includes('ruv-swarm') || config.command?.includes('ruv-swarm')) {
      console.log('   ✅ This is ruv-swarm server');
    }
  }
  
  // Look for project-specific config
  const projects = config.projects || {};
  for (const [projectPath, projectConfig] of Object.entries(projects)) {
    if (projectConfig.mcpServers) {
      console.log(`\n\n📂 Project: ${projectPath}`);
      console.log('Project MCP Servers:');
      for (const [name, serverConfig] of Object.entries(projectConfig.mcpServers)) {
        console.log(`   ${name}: ${serverConfig.command} ${(serverConfig.args || []).join(' ')}`);
      }
    }
  }
  
} catch (error) {
  console.error('Error reading config:', error.message);
}