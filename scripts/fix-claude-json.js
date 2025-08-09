const fs = require('fs');
const path = require('path');
const os = require('os');

const claudeJsonPath = path.join(os.homedir(), '.claude.json');

console.log('Fixing MCP configuration in .claude.json...');

try {
    // Read the current configuration
    const config = JSON.parse(fs.readFileSync(claudeJsonPath, 'utf8'));
    
    // Fix claude-flow configuration
    if (config.mcpServers && config.mcpServers['claude-flow']) {
        config.mcpServers['claude-flow'] = {
            command: 'cmd',
            args: ['/c', 'npx', 'claude-flow@alpha', 'mcp', 'start'],
            type: 'stdio'
        };
        console.log('✓ Fixed claude-flow MCP server');
    }
    
    // Fix ruv-swarm configuration
    if (config.mcpServers && config.mcpServers['ruv-swarm']) {
        config.mcpServers['ruv-swarm'] = {
            command: 'cmd',
            args: ['/c', 'npx', 'ruv-swarm@latest', 'mcp', 'start'],
            type: 'stdio'
        };
        console.log('✓ Fixed ruv-swarm MCP server');
    }
    
    // Write the updated configuration
    fs.writeFileSync(claudeJsonPath, JSON.stringify(config, null, 2));
    console.log('✓ Configuration saved successfully');
    console.log('\nPlease restart Claude Desktop for changes to take effect.');
    
} catch (error) {
    console.error('Error updating .claude.json:', error.message);
    process.exit(1);
}