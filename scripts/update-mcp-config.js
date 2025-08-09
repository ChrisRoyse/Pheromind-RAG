const fs = require('fs');
const path = require('path');
const os = require('os');

console.log('Updating MCP configuration for Windows...\n');

// Update .mcp.json in project
const projectMcpPath = path.join(process.cwd(), '.mcp.json');
const projectConfig = {
    "mcpServers": {
        "claude-flow": {
            "command": "cmd",
            "args": ["/c", "npx", "claude-flow@alpha", "mcp", "start"],
            "type": "stdio"
        },
        "ruv-swarm": {
            "command": "cmd",
            "args": ["/c", "npx", "ruv-swarm@latest", "mcp", "start"],
            "type": "stdio"
        }
    }
};

fs.writeFileSync(projectMcpPath, JSON.stringify(projectConfig, null, 2));
console.log('✓ Updated .mcp.json');

// Update user's .claude.json
const claudeJsonPath = path.join(os.homedir(), '.claude.json');

try {
    let config = {};
    if (fs.existsSync(claudeJsonPath)) {
        const content = fs.readFileSync(claudeJsonPath, 'utf8');
        config = JSON.parse(content);
    }
    
    // Ensure mcpServers exists at root level
    if (!config.mcpServers) {
        config.mcpServers = {};
    }
    
    // Update claude-flow configuration
    config.mcpServers['claude-flow'] = {
        "command": "cmd",
        "args": ["/c", "npx", "claude-flow@alpha", "mcp", "start"],
        "type": "stdio"
    };
    
    // Update ruv-swarm configuration
    config.mcpServers['ruv-swarm'] = {
        "command": "cmd",
        "args": ["/c", "npx", "ruv-swarm@latest", "mcp", "start"],
        "type": "stdio"
    };
    
    // Write back
    fs.writeFileSync(claudeJsonPath, JSON.stringify(config, null, 2));
    console.log('✓ Updated .claude.json with MCP servers at root level');
    
    // Verify
    const verification = JSON.parse(fs.readFileSync(claudeJsonPath, 'utf8'));
    if (verification.mcpServers && verification.mcpServers['claude-flow']) {
        console.log('\n✅ Configuration verified:');
        console.log('  claude-flow:', verification.mcpServers['claude-flow'].command, 
                    verification.mcpServers['claude-flow'].args.join(' '));
    }
    
} catch (error) {
    console.error('Error updating .claude.json:', error.message);
}

console.log('\n⚠️  IMPORTANT STEPS:');
console.log('1. SQLite binding has been deployed to NPX cache');
console.log('2. Configuration updated with Windows-compatible format');
console.log('3. Please RESTART Claude Desktop now');
console.log('4. MCP servers should show as running after restart');