const { spawn } = require('child_process');
const path = require('path');

console.log('===========================================');
console.log('DEBUGGING CLAUDE-FLOW EXECUTION');
console.log('===========================================\n');

// Test 1: Simple npx execution
console.log('[TEST 1] Testing npx claude-flow@alpha with different commands...\n');

const tests = [
    { args: ['claude-flow@alpha', '--version'], name: 'Version check' },
    { args: ['claude-flow@alpha', 'help'], name: 'Help command' },
    { args: ['claude-flow@alpha', 'mcp'], name: 'MCP command' },
    { args: ['claude-flow@alpha', 'mcp', 'help'], name: 'MCP help' },
    { args: ['claude-flow@alpha', 'mcp', 'start'], name: 'MCP start' },
];

let testIndex = 0;

function runTest() {
    if (testIndex >= tests.length) {
        console.log('\n===========================================');
        console.log('ANALYSIS');
        console.log('===========================================\n');
        console.log('Based on the test results above, we can determine:');
        console.log('1. Whether claude-flow is installed correctly');
        console.log('2. Which commands are available');
        console.log('3. Why MCP server might be failing');
        return;
    }
    
    const test = tests[testIndex];
    console.log(`Running: npx ${test.args.join(' ')} (${test.name})`);
    
    const child = spawn('npx', test.args, {
        shell: true,
        timeout: 3000
    });
    
    let output = '';
    let error = '';
    
    child.stdout.on('data', (data) => {
        output += data.toString();
    });
    
    child.stderr.on('data', (data) => {
        error += data.toString();
    });
    
    // Set a timeout
    const timeout = setTimeout(() => {
        console.log(`  ⚠ Timed out after 3 seconds`);
        child.kill();
    }, 3000);
    
    child.on('exit', (code) => {
        clearTimeout(timeout);
        
        if (code === null) {
            console.log(`  Result: Process killed (timeout)`);
        } else if (code === 0) {
            console.log(`  ✓ Success (exit code 0)`);
            if (output) {
                console.log(`  Output: ${output.trim().substring(0, 100)}`);
            }
        } else {
            console.log(`  ✗ Failed (exit code ${code})`);
            if (error) {
                console.log(`  Error: ${error.trim().substring(0, 100)}`);
            }
        }
        
        console.log('');
        testIndex++;
        setTimeout(runTest, 500);
    });
}

runTest();