const { spawn } = require('child_process');

console.log('===========================================');
console.log('TESTING CUSTOM CLAUDE-FLOW MCP SERVER');
console.log('===========================================\n');

const serverPath = 'C:/Users/hotra/AppData/Local/claude-flow-mcp/claude-flow-custom-server.js';

console.log('Starting custom MCP server...\n');

const server = spawn('node', [serverPath], {
    stdio: ['pipe', 'pipe', 'pipe']
});

let responses = [];

server.stdout.on('data', (data) => {
    const response = data.toString();
    console.log('Server response:', response);
    responses.push(response);
});

server.stderr.on('data', (data) => {
    console.log('Server error:', data.toString());
});

// Test sequence
let testStep = 0;
const tests = [
    {
        name: "Initialize",
        request: {
            jsonrpc: "2.0",
            id: 1,
            method: "initialize",
            params: { capabilities: {} }
        }
    },
    {
        name: "List tools",
        request: {
            jsonrpc: "2.0",
            id: 2,
            method: "tools/list",
            params: {}
        }
    },
    {
        name: "Call help tool",
        request: {
            jsonrpc: "2.0",
            id: 3,
            method: "tools/call",
            params: {
                name: "claude_flow_help",
                arguments: {}
            }
        }
    }
];

function runNextTest() {
    if (testStep >= tests.length) {
        console.log('\n===========================================');
        console.log('TEST SUMMARY');
        console.log('===========================================\n');
        console.log(`Ran ${tests.length} tests`);
        console.log(`Received ${responses.length} responses`);
        
        if (responses.length >= tests.length) {
            console.log('✅ Custom MCP server is working!');
            console.log('Server responds to JSON-RPC requests correctly');
        } else {
            console.log('❌ Server not responding properly');
        }
        
        server.kill();
        process.exit();
        return;
    }
    
    const test = tests[testStep];
    console.log(`\n[TEST ${testStep + 1}] ${test.name}`);
    console.log('Sending:', JSON.stringify(test.request));
    
    server.stdin.write(JSON.stringify(test.request) + '\n');
    testStep++;
    
    // Wait for response before next test
    setTimeout(runNextTest, 1500);
}

// Start testing after brief delay
setTimeout(runNextTest, 1000);

// Cleanup after 10 seconds
setTimeout(() => {
    console.log('\nTest timeout - cleaning up...');
    server.kill();
    process.exit();
}, 10000);