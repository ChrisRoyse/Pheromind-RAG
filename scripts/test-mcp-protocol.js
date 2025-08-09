const { spawn } = require('child_process');

console.log('===========================================');
console.log('TESTING MCP PROTOCOL COMMUNICATION');
console.log('===========================================\n');

console.log('Starting claude-flow MCP server and sending JSON-RPC...\n');

const child = spawn('npx', ['claude-flow@alpha', 'mcp', 'start'], {
    shell: true,
    stdio: ['pipe', 'pipe', 'pipe']
});

let responseReceived = false;

// Send initialization request
const initRequest = JSON.stringify({
    jsonrpc: "2.0",
    id: 1,
    method: "initialize",
    params: {
        capabilities: {}
    }
}) + '\n';

console.log('Sending initialization request:');
console.log(initRequest);

// Listen for responses
child.stdout.on('data', (data) => {
    responseReceived = true;
    console.log('Response received:');
    console.log(data.toString());
});

child.stderr.on('data', (data) => {
    console.log('Error output:');
    console.log(data.toString());
});

// Send the request
setTimeout(() => {
    child.stdin.write(initRequest);
    console.log('Request sent, waiting for response...\n');
}, 500);

// Check after 3 seconds
setTimeout(() => {
    if (!responseReceived) {
        console.log('❌ No response received after 3 seconds');
        console.log('\nPOSSIBLE ISSUES:');
        console.log('1. MCP server not implementing JSON-RPC protocol');
        console.log('2. Server exiting immediately after start');
        console.log('3. Server not reading from stdin');
        console.log('4. Server buffering output without flushing');
    } else {
        console.log('\n✅ MCP server is responding to JSON-RPC');
    }
    
    console.log('\nChecking if process is still running...');
    try {
        process.kill(child.pid, 0);
        console.log('✓ Process is still running (PID:', child.pid, ')');
    } catch (e) {
        console.log('✗ Process has exited');
    }
    
    child.kill();
    process.exit();
}, 3000);

child.on('exit', (code) => {
    console.log(`\nProcess exited with code: ${code}`);
});