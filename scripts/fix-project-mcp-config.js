const fs = require('fs');
const path = require('path');
const os = require('os');

console.log('===========================================');
console.log('FIXING PROJECT-SPECIFIC MCP CONFIGURATION');
console.log('===========================================\n');

const claudeJsonPath = path.join(os.homedir(), '.claude.json');
const projectPath = 'C:\\code\\embed';

try {
    // Read configuration
    console.log('Reading .claude.json...');
    const config = JSON.parse(fs.readFileSync(claudeJsonPath, 'utf8'));
    
    // Ensure projects section exists
    if (!config.projects) {
        config.projects = {};
    }
    
    // Ensure project entry exists
    if (!config.projects[projectPath]) {
        config.projects[projectPath] = {};
    }
    
    // Update project-specific MCP servers with Windows cmd wrapper
    console.log(`\nUpdating project "${projectPath}" MCP servers...`);
    
    config.projects[projectPath].mcpServers = {
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
    };
    
    console.log('✓ Updated claude-flow with cmd wrapper');
    console.log('✓ Updated ruv-swarm with cmd wrapper');
    
    // Write back the configuration
    console.log('\nSaving configuration...');
    fs.writeFileSync(claudeJsonPath, JSON.stringify(config, null, 2));
    console.log('✓ Configuration saved');
    
    // Verify the update
    console.log('\n===========================================');
    console.log('VERIFICATION');
    console.log('===========================================');
    
    const verification = JSON.parse(fs.readFileSync(claudeJsonPath, 'utf8'));
    if (verification.projects && 
        verification.projects[projectPath] && 
        verification.projects[projectPath].mcpServers &&
        verification.projects[projectPath].mcpServers['claude-flow']) {
        
        const claudeFlow = verification.projects[projectPath].mcpServers['claude-flow'];
        console.log('\n✅ PROJECT-SPECIFIC CONFIG UPDATED:');
        console.log(`  Project: ${projectPath}`);
        console.log(`  Command: ${claudeFlow.command}`);
        console.log(`  Args: ${claudeFlow.args.join(' ')}`);
        console.log(`  Type: ${claudeFlow.type}`);
        
        if (claudeFlow.command === 'cmd' && claudeFlow.args[0] === '/c') {
            console.log('\n✅ WINDOWS CMD WRAPPER CORRECTLY APPLIED');
        } else {
            console.log('\n❌ WARNING: Configuration may not be correct for Windows');
        }
    } else {
        console.log('\n❌ ERROR: Could not verify configuration');
    }
    
    console.log('\n===========================================');
    console.log('REQUIRED ACTION');
    console.log('===========================================');
    console.log('1. Configuration has been updated');
    console.log('2. RESTART Claude Desktop now');
    console.log('3. The MCP server should show as running');
    console.log('');
    
} catch (error) {
    console.error('❌ Error updating .claude.json:', error.message);
    process.exit(1);
}