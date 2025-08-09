const fs = require('fs');
const path = require('path');
const os = require('os');

console.log('Finding project-specific MCP configuration...\n');

const claudeJsonPath = path.join(os.homedir(), '.claude.json');
const projectPath = 'C:\\code\\embed';

try {
    const config = JSON.parse(fs.readFileSync(claudeJsonPath, 'utf8'));
    
    // Find all keys that might be project-specific
    const keys = Object.keys(config);
    
    console.log('Looking for project-specific configurations...\n');
    
    // Check for project path as key
    const projectKeys = keys.filter(k => 
        k.includes('embed') || 
        k.includes('C:') || 
        k.includes('code') ||
        k.includes('project')
    );
    
    if (projectKeys.length > 0) {
        console.log('Found potential project keys:');
        projectKeys.forEach(key => {
            console.log(`  - "${key}"`);
            if (config[key] && config[key].mcpServers) {
                console.log('    Has mcpServers:', Object.keys(config[key].mcpServers));
            }
        });
    }
    
    // Look for the exact project path
    const normalizedPaths = [
        projectPath,
        projectPath.replace(/\\/g, '/'),
        projectPath.replace(/\\/g, '\\\\'),
        'project:' + projectPath,
        'project:' + projectPath.replace(/\\/g, '/'),
    ];
    
    console.log('\nChecking normalized paths:');
    normalizedPaths.forEach(p => {
        if (config[p]) {
            console.log(`  ✓ Found: "${p}"`);
            if (config[p].mcpServers) {
                console.log('    MCP Servers:', JSON.stringify(config[p].mcpServers, null, 2));
            }
        }
    });
    
    // Check if there's a projects section
    if (config.projects) {
        console.log('\n✓ Found "projects" section');
        const projectEntries = Object.keys(config.projects);
        projectEntries.forEach(p => {
            if (p.includes('embed')) {
                console.log(`  Project: "${p}"`);
                if (config.projects[p].mcpServers) {
                    console.log('    Has MCP servers');
                }
            }
        });
    }
    
} catch (error) {
    console.error('Error reading .claude.json:', error.message);
}