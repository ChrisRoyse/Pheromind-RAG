const fs = require('fs');
const path = require('path');
const os = require('os');

console.log('===========================================');
console.log('VERIFYING NEO4J PATTERN FIX');
console.log('===========================================\n');

let score = 0;
const checks = [];

// Check 1: Wrapper files exist
console.log('[CHECK 1] Executable wrappers exist...');
const claudeFlowWrapper = 'C:\\Users\\hotra\\AppData\\Local\\claude-flow-mcp\\claude-flow-mcp.bat';
const ruvSwarmWrapper = 'C:\\Users\\hotra\\AppData\\Local\\claude-flow-mcp\\ruv-swarm-mcp.bat';

if (fs.existsSync(claudeFlowWrapper)) {
    console.log('  ✅ claude-flow-mcp.bat exists');
    score += 25;
    checks.push({ name: 'Claude-Flow Wrapper', status: 'PASS' });
} else {
    console.log('  ❌ claude-flow-mcp.bat not found');
    checks.push({ name: 'Claude-Flow Wrapper', status: 'FAIL' });
}

if (fs.existsSync(ruvSwarmWrapper)) {
    console.log('  ✅ ruv-swarm-mcp.bat exists');
    score += 25;
    checks.push({ name: 'Ruv-Swarm Wrapper', status: 'PASS' });
} else {
    console.log('  ❌ ruv-swarm-mcp.bat not found');
    checks.push({ name: 'Ruv-Swarm Wrapper', status: 'FAIL' });
}

// Check 2: Configuration uses Neo4j pattern
console.log('\n[CHECK 2] Configuration follows Neo4j pattern...');
try {
    const claudeJsonPath = path.join(os.homedir(), '.claude.json');
    const config = JSON.parse(fs.readFileSync(claudeJsonPath, 'utf8'));
    const projectPath = 'C:\\code\\embed';
    
    const projectConfig = config.projects?.[projectPath]?.mcpServers?.['claude-flow'];
    
    if (projectConfig) {
        // Check for Neo4j pattern: direct path, no cmd /c
        if (projectConfig.command && 
            projectConfig.command.includes('.bat') &&
            !projectConfig.command.includes('cmd') &&
            projectConfig.type === 'stdio') {
            
            console.log('  ✅ Uses direct executable path (Neo4j pattern)');
            console.log('  ✅ No cmd /c wrapper');
            console.log('  ✅ Type is stdio');
            score += 30;
            checks.push({ name: 'Neo4j Pattern', status: 'PASS' });
        } else {
            console.log('  ❌ Not following Neo4j pattern');
            checks.push({ name: 'Neo4j Pattern', status: 'FAIL' });
        }
    } else {
        console.log('  ❌ Project configuration not found');
        checks.push({ name: 'Neo4j Pattern', status: 'FAIL' });
    }
} catch (error) {
    console.log('  ❌ Error reading configuration:', error.message);
    checks.push({ name: 'Neo4j Pattern', status: 'FAIL' });
}

// Check 3: SQLite binding deployed
console.log('\n[CHECK 3] SQLite binding available...');
const bindingSource = 'C:\\code\\embed\\better-sqlite3\\build\\Release\\better_sqlite3.node';
if (fs.existsSync(bindingSource)) {
    const stats = fs.statSync(bindingSource);
    if (stats.size > 1000000) {
        console.log('  ✅ SQLite binding exists (', stats.size, 'bytes)');
        score += 20;
        checks.push({ name: 'SQLite Binding', status: 'PASS' });
    } else {
        console.log('  ❌ SQLite binding too small');
        checks.push({ name: 'SQLite Binding', status: 'FAIL' });
    }
} else {
    console.log('  ❌ SQLite binding not found');
    checks.push({ name: 'SQLite Binding', status: 'FAIL' });
}

// Summary
console.log('\n===========================================');
console.log('SUMMARY');
console.log('===========================================\n');
console.log(`SCORE: ${score}/100\n`);

checks.forEach(check => {
    const icon = check.status === 'PASS' ? '✅' : '❌';
    console.log(`${icon} ${check.name}: ${check.status}`);
});

console.log('\n===========================================');
console.log('ASSESSMENT');
console.log('===========================================\n');

if (score >= 90) {
    console.log('✅ NEO4J PATTERN SUCCESSFULLY APPLIED');
    console.log('\nThe configuration now follows the same pattern as your');
    console.log('working Neo4j servers. After restarting Claude Desktop,');
    console.log('claude-flow MCP server should connect successfully.');
} else if (score >= 70) {
    console.log('⚠️  PARTIALLY CONFIGURED');
    console.log('\nSome elements are in place but configuration may need adjustment.');
} else {
    console.log('❌ CONFIGURATION INCOMPLETE');
    console.log('\nThe Neo4j pattern has not been fully applied.');
}

console.log('\n📝 Neo4j Pattern Checklist:');
console.log('  [' + (checks.find(c => c.name === 'Claude-Flow Wrapper')?.status === 'PASS' ? '✓' : ' ') + '] Direct executable wrapper created');
console.log('  [' + (checks.find(c => c.name === 'Neo4j Pattern')?.status === 'PASS' ? '✓' : ' ') + '] Configuration uses direct path (no cmd /c)');
console.log('  [' + (checks.find(c => c.name === 'SQLite Binding')?.status === 'PASS' ? '✓' : ' ') + '] SQLite binding available');
console.log('\nNEXT STEP: Restart Claude Desktop');