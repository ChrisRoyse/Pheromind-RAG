const fs = require('fs');
const path = require('path');
const os = require('os');

console.log('===========================================');
console.log('FIXING CLAUDE-FLOW MCP - WINDOWS SOLUTION');
console.log('===========================================\n');
console.log('Using Neo4j pattern: Direct executable paths\n');

const claudeJsonPath = path.join(os.homedir(), '.claude.json');
const projectPath = 'C:\\code\\embed';

// Path to our stable wrapper
const wrapperPath = 'C:\\\\Users\\\\hotra\\\\AppData\\\\Local\\\\claude-flow-mcp\\\\claude-flow-mcp.bat';

try {
    // Read configuration
    console.log('Reading .claude.json...');
    const config = JSON.parse(fs.readFileSync(claudeJsonPath, 'utf8'));
    
    // Update PROJECT-SPECIFIC configuration (like Neo4j example)
    if (!config.projects) {
        config.projects = {};
    }
    
    if (!config.projects[projectPath]) {
        config.projects[projectPath] = {};
    }
    
    console.log('Updating project configuration...\n');
    
    // Remove cmd /c wrapper, use direct executable like Neo4j
    config.projects[projectPath].mcpServers = {
        "claude-flow": {
            "type": "stdio",
            "command": wrapperPath
            // No args needed - the batch file handles everything
        },
        "ruv-swarm": {
            "type": "stdio", 
            "command": "C:\\\\Users\\\\hotra\\\\AppData\\\\Local\\\\claude-flow-mcp\\\\ruv-swarm-mcp.bat"
        }
    };
    
    console.log('✓ Updated claude-flow with direct executable path');
    console.log('  Command:', wrapperPath);
    console.log('  Type: stdio');
    console.log('  Pattern: Like Neo4j example (direct .bat path)');
    
    // Also update global mcpServers for consistency
    if (!config.mcpServers) {
        config.mcpServers = {};
    }
    
    config.mcpServers['claude-flow'] = {
        "type": "stdio",
        "command": wrapperPath
    };
    
    // Save configuration
    console.log('\nSaving configuration...');
    fs.writeFileSync(claudeJsonPath, JSON.stringify(config, null, 2));
    console.log('✓ Configuration saved');
    
    // Verify
    console.log('\n===========================================');
    console.log('VERIFICATION');
    console.log('===========================================\n');
    
    const verification = JSON.parse(fs.readFileSync(claudeJsonPath, 'utf8'));
    const projectConfig = verification.projects?.[projectPath]?.mcpServers?.['claude-flow'];
    
    if (projectConfig) {
        console.log('✅ PROJECT CONFIGURATION UPDATED:');
        console.log('  Project:', projectPath);
        console.log('  Command:', projectConfig.command);
        console.log('  Type:', projectConfig.type);
        console.log('\n✅ USING NEO4J PATTERN:');
        console.log('  - Direct executable path ✓');
        console.log('  - No cmd /c wrapper ✓');
        console.log('  - Stable location ✓');
    }
    
    // Update .mcp.json too
    const mcpJsonPath = path.join(process.cwd(), '.mcp.json');
    const mcpConfig = {
        "mcpServers": {
            "claude-flow": {
                "type": "stdio",
                "command": wrapperPath
            },
            "ruv-swarm": {
                "type": "stdio",
                "command": "C:\\\\Users\\\\hotra\\\\AppData\\\\Local\\\\claude-flow-mcp\\\\ruv-swarm-mcp.bat"
            }
        }
    };
    
    fs.writeFileSync(mcpJsonPath, JSON.stringify(mcpConfig, null, 2));
    console.log('\n✓ Updated .mcp.json with same pattern');
    
    console.log('\n===========================================');
    console.log('NEXT STEPS');
    console.log('===========================================');
    console.log('\n1. Configuration updated using Neo4j pattern');
    console.log('2. Direct executable path (no npx in config)');
    console.log('3. RESTART Claude Desktop now');
    console.log('4. MCP server should connect successfully');
    
} catch (error) {
    console.error('Error:', error.message);
}