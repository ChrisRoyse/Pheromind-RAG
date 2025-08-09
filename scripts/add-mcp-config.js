const fs = require('fs');
const path = require('path');
const os = require('os');

const claudeJsonPath = path.join(os.homedir(), '.claude.json');

console.log('Adding MCP configuration to .claude.json...');

try {
    // Read the current configuration
    let config = {};
    if (fs.existsSync(claudeJsonPath)) {
        config = JSON.parse(fs.readFileSync(claudeJsonPath, 'utf8'));
    }
    
    // Ensure mcpServers exists
    if (!config.mcpServers) {
        config.mcpServers = {};
    }
    
    // Add/Update claude-flow configuration
    config.mcpServers['claude-flow'] = {
        command: 'cmd',
        args: ['/c', 'npx', 'claude-flow@alpha', 'mcp', 'start'],
        type: 'stdio'
    };
    console.log('✓ Added claude-flow MCP server');
    
    // Add/Update ruv-swarm configuration  
    config.mcpServers['ruv-swarm'] = {
        command: 'cmd',
        args: ['/c', 'npx', 'ruv-swarm@latest', 'mcp', 'start'],
        type: 'stdio'
    };
    console.log('✓ Added ruv-swarm MCP server');
    
    // Write the updated configuration
    fs.writeFileSync(claudeJsonPath, JSON.stringify(config, null, 2));
    console.log('✓ Configuration saved successfully');
    
    // Verify the configuration
    const verification = JSON.parse(fs.readFileSync(claudeJsonPath, 'utf8'));
    if (verification.mcpServers && verification.mcpServers['claude-flow']) {
        console.log('\n✅ Verification: MCP servers configured correctly');
        console.log('\nClaude-flow command:', verification.mcpServers['claude-flow'].command);
        console.log('Claude-flow args:', verification.mcpServers['claude-flow'].args.join(' '));
    }
    
    console.log('\n⚠️  IMPORTANT: Please restart Claude Desktop for changes to take effect.');
    
} catch (error) {
    console.error('Error updating .claude.json:', error.message);
    process.exit(1);
}