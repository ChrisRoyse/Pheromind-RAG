const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync } = require('child_process');

console.log('===========================================');
console.log('FIXING CLAUDE-FLOW - DIRECT NODE APPROACH');
console.log('===========================================\n');
console.log('Using Neo4j pattern with Node.exe directly\n');

// Get Node.exe path
const nodeExePath = process.execPath;
console.log('Node.exe location:', nodeExePath);

// Script location
const scriptPath = 'C:\\Users\\hotra\\AppData\\Local\\claude-flow-mcp\\claude-flow-direct.js';
console.log('Script location:', scriptPath);

// Update configuration
const claudeJsonPath = path.join(os.homedir(), '.claude.json');
const projectPath = 'C:\\code\\embed';

try {
    console.log('\nReading .claude.json...');
    const config = JSON.parse(fs.readFileSync(claudeJsonPath, 'utf8'));
    
    // Ensure project structure exists
    if (!config.projects) {
        config.projects = {};
    }
    if (!config.projects[projectPath]) {
        config.projects[projectPath] = {};
    }
    
    // Update with direct Node.exe execution (like Neo4j uses Python.exe)
    console.log('Updating configuration with direct Node.exe...\n');
    
    // Escape paths for JSON
    const escapedNodePath = nodeExePath.replace(/\\/g, '\\\\');
    const escapedScriptPath = scriptPath.replace(/\\/g, '\\\\');
    
    config.projects[projectPath].mcpServers = {
        "claude-flow": {
            "type": "stdio",
            "command": escapedNodePath,
            "args": [escapedScriptPath]
        },
        "ruv-swarm": {
            "type": "stdio",
            "command": "C:\\\\Users\\\\hotra\\\\AppData\\\\Local\\\\claude-flow-mcp\\\\ruv-swarm-mcp.bat"
        }
    };
    
    console.log('✅ Configuration updated:');
    console.log('  Command:', escapedNodePath);
    console.log('  Args:', [escapedScriptPath]);
    console.log('  Pattern: Direct executable (like Neo4j)');
    
    // Also update global mcpServers
    if (!config.mcpServers) {
        config.mcpServers = {};
    }
    config.mcpServers['claude-flow'] = {
        "type": "stdio",
        "command": escapedNodePath,
        "args": [escapedScriptPath]
    };
    
    // Save configuration
    console.log('\nSaving configuration...');
    fs.writeFileSync(claudeJsonPath, JSON.stringify(config, null, 2));
    console.log('✓ Configuration saved');
    
    // Update .mcp.json
    const mcpJsonPath = path.join(process.cwd(), '.mcp.json');
    const mcpConfig = {
        "mcpServers": {
            "claude-flow": {
                "type": "stdio",
                "command": escapedNodePath,
                "args": [escapedScriptPath]
            },
            "ruv-swarm": {
                "type": "stdio",
                "command": "C:\\\\Users\\\\hotra\\\\AppData\\\\Local\\\\claude-flow-mcp\\\\ruv-swarm-mcp.bat"
            }
        }
    };
    
    fs.writeFileSync(mcpJsonPath, JSON.stringify(mcpConfig, null, 2));
    console.log('✓ Updated .mcp.json');
    
    // Test the command
    console.log('\n===========================================');
    console.log('TESTING DIRECT EXECUTION');
    console.log('===========================================\n');
    
    console.log('Testing command...');
    try {
        const testCmd = `"${nodeExePath}" "${scriptPath}" --version`;
        console.log('Command:', testCmd);
        execSync(testCmd, { 
            timeout: 3000,
            stdio: 'pipe'
        });
        console.log('✓ Command executes without error');
    } catch (err) {
        if (err.code === 'ETIMEDOUT') {
            console.log('⚠ Command timed out (may be normal for server)');
        } else {
            console.log('✗ Command failed:', err.message);
        }
    }
    
    console.log('\n===========================================');
    console.log('COMPARISON WITH NEO4J');
    console.log('===========================================\n');
    
    console.log('Neo4j pattern:');
    console.log('  Command: C:\\\\...\\\\Python\\\\Scripts\\\\mcp-neo4j-cypher.exe');
    console.log('  Args: ["--db-url", "...", "--password", "..."]');
    console.log('');
    console.log('Claude-flow pattern (NEW):');
    console.log('  Command:', escapedNodePath);
    console.log('  Args:', [escapedScriptPath]);
    console.log('');
    console.log('✅ Both use direct executable paths');
    console.log('✅ Both avoid npx/cmd wrappers');
    console.log('✅ Both use stable locations');
    
    console.log('\n===========================================');
    console.log('NEXT STEPS');
    console.log('===========================================\n');
    console.log('1. Configuration updated with Node.exe directly');
    console.log('2. Pattern matches Neo4j (direct executable)');
    console.log('3. RESTART Claude Desktop now');
    console.log('4. Check MCP server status');
    
} catch (error) {
    console.error('Error:', error.message);
}