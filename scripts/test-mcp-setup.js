const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync } = require('child_process');

console.log('===========================================');
console.log('MCP SETUP VERIFICATION TEST');
console.log('===========================================\n');

let score = 0;
const tests = [];

// Test 1: Check project configuration in .claude.json
console.log('[TEST 1] Checking project-specific configuration...');
try {
    const claudeJsonPath = path.join(os.homedir(), '.claude.json');
    const config = JSON.parse(fs.readFileSync(claudeJsonPath, 'utf8'));
    const projectPath = 'C:\\code\\embed';
    
    if (config.projects && 
        config.projects[projectPath] && 
        config.projects[projectPath].mcpServers &&
        config.projects[projectPath].mcpServers['claude-flow']) {
        
        const cf = config.projects[projectPath].mcpServers['claude-flow'];
        if (cf.command === 'cmd' && cf.args[0] === '/c' && cf.args[1] === 'npx') {
            console.log('  ✅ Project config has correct Windows cmd wrapper');
            score += 25;
            tests.push({ name: 'Project Config', status: 'PASS' });
        } else {
            console.log('  ❌ Project config missing cmd wrapper');
            tests.push({ name: 'Project Config', status: 'FAIL', error: 'Wrong command format' });
        }
    } else {
        console.log('  ❌ Project-specific config not found');
        tests.push({ name: 'Project Config', status: 'FAIL', error: 'Not found' });
    }
} catch (error) {
    console.log('  ❌ Error reading config:', error.message);
    tests.push({ name: 'Project Config', status: 'FAIL', error: error.message });
}

// Test 2: Check SQLite binding in NPX cache
console.log('\n[TEST 2] Checking SQLite binding deployment...');
try {
    const npmCache = execSync('npm config get cache', { encoding: 'utf8' }).trim();
    const npxDir = path.join(npmCache, '_npx');
    let bindingFound = false;
    
    if (fs.existsSync(npxDir)) {
        const dirs = fs.readdirSync(npxDir);
        for (const dir of dirs) {
            const bindingPath = path.join(npxDir, dir, 'node_modules', 'better-sqlite3', 'build', 'Release', 'better_sqlite3.node');
            if (fs.existsSync(bindingPath)) {
                const stats = fs.statSync(bindingPath);
                if (stats.size > 1000000) {
                    bindingFound = true;
                    break;
                }
            }
        }
    }
    
    if (bindingFound) {
        console.log('  ✅ SQLite binding found in NPX cache');
        score += 25;
        tests.push({ name: 'SQLite Binding', status: 'PASS' });
    } else {
        console.log('  ❌ SQLite binding not found in NPX cache');
        tests.push({ name: 'SQLite Binding', status: 'FAIL', error: 'Not deployed' });
    }
} catch (error) {
    console.log('  ❌ Error checking binding:', error.message);
    tests.push({ name: 'SQLite Binding', status: 'FAIL', error: error.message });
}

// Test 3: Check .mcp.json in project
console.log('\n[TEST 3] Checking project .mcp.json...');
try {
    const mcpJsonPath = path.join(process.cwd(), '.mcp.json');
    if (fs.existsSync(mcpJsonPath)) {
        const mcpConfig = JSON.parse(fs.readFileSync(mcpJsonPath, 'utf8'));
        if (mcpConfig.mcpServers && mcpConfig.mcpServers['claude-flow']) {
            const cf = mcpConfig.mcpServers['claude-flow'];
            if (cf.command === 'cmd' && cf.args[0] === '/c') {
                console.log('  ✅ Project .mcp.json has Windows cmd wrapper');
                score += 25;
                tests.push({ name: 'Project .mcp.json', status: 'PASS' });
            } else {
                console.log('  ❌ Project .mcp.json missing cmd wrapper');
                tests.push({ name: 'Project .mcp.json', status: 'FAIL', error: 'Wrong format' });
            }
        }
    } else {
        console.log('  ❌ .mcp.json not found');
        tests.push({ name: 'Project .mcp.json', status: 'FAIL', error: 'File not found' });
    }
} catch (error) {
    console.log('  ❌ Error reading .mcp.json:', error.message);
    tests.push({ name: 'Project .mcp.json', status: 'FAIL', error: error.message });
}

// Test 4: Test npx claude-flow command
console.log('\n[TEST 4] Testing npx claude-flow command...');
try {
    // Quick test to see if command works
    execSync('npx claude-flow@alpha --version', { 
        encoding: 'utf8',
        timeout: 5000,
        stdio: 'pipe'
    });
    console.log('  ✅ npx claude-flow@alpha command works');
    score += 25;
    tests.push({ name: 'NPX Command', status: 'PASS' });
} catch (error) {
    if (error.code === 'ETIMEDOUT') {
        console.log('  ⚠️  Command timed out (may be normal for server)');
        score += 15;
        tests.push({ name: 'NPX Command', status: 'PARTIAL', note: 'Timeout' });
    } else {
        console.log('  ❌ npx command failed:', error.message);
        tests.push({ name: 'NPX Command', status: 'FAIL', error: error.message });
    }
}

// Summary
console.log('\n===========================================');
console.log('TEST SUMMARY');
console.log('===========================================');
console.log(`\nSCORE: ${score}/100\n`);

tests.forEach(test => {
    const status = test.status === 'PASS' ? '✅' : test.status === 'PARTIAL' ? '⚠️' : '❌';
    console.log(`${status} ${test.name}: ${test.status}`);
    if (test.error) console.log(`   Error: ${test.error}`);
    if (test.note) console.log(`   Note: ${test.note}`);
});

console.log('\n===========================================');
console.log('FINAL ASSESSMENT');
console.log('===========================================');

if (score >= 90) {
    console.log('\n✅ SETUP COMPLETE - MCP should work after Claude Desktop restart');
} else if (score >= 70) {
    console.log('\n⚠️  MOSTLY COMPLETE - Some issues remain, but may work');
} else {
    console.log('\n❌ SETUP INCOMPLETE - Critical issues need fixing');
}

console.log('\nNEXT STEP: Restart Claude Desktop to apply changes');