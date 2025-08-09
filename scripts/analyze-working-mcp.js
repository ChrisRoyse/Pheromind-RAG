const { spawn } = require('child_process');

console.log('===========================================');
console.log('ANALYZING WORKING RUV-SWARM MCP SERVER');
console.log('===========================================\n');

console.log('Testing ruv-swarm server to understand what makes it work...\n');

// Test the working ruv-swarm server
const ruvSwarm = spawn('C:/Users/hotra/AppData/Local/claude-flow-mcp/ruv-swarm-mcp.bat', [], {
    stdio: ['pipe', 'pipe', 'pipe']
});

let ruvResponded = false;

ruvSwarm.stdout.on('data', (data) => {
    console.log('ruv-swarm stdout:', data.toString());
    ruvResponded = true;
});

ruvSwarm.stderr.on('data', (data) => {
    console.log('ruv-swarm stderr:', data.toString());
});

ruvSwarm.on('exit', (code, signal) => {
    console.log(`ruv-swarm exited with code ${code}, signal ${signal}`);
});

// Send initialization request
setTimeout(() => {
    console.log('\nSending initialization to ruv-swarm...');
    const initRequest = JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: { capabilities: {} }
    }) + '\n';
    
    ruvSwarm.stdin.write(initRequest);
}, 1000);

// Check results
setTimeout(() => {
    if (ruvResponded) {
        console.log('\n✅ ruv-swarm responds to JSON-RPC');
        console.log('This confirms it implements proper MCP protocol');
    } else {
        console.log('\n❌ ruv-swarm not responding to JSON-RPC');
        console.log('It may implement a different protocol or exit immediately');
    }
    
    console.log('\n===========================================');
    console.log('COMPARISON ANALYSIS');
    console.log('===========================================\n');
    
    console.log('ruv-swarm (WORKING):');
    console.log('- Command: C:/Users/.../ruv-swarm-mcp.bat');
    console.log('- Runs: npx ruv-swarm@latest mcp start');
    console.log('- Status: ✓ Connected');
    
    console.log('\nclaude-flow custom (FAILING):');
    console.log('- Command: C:/Users/.../start-custom-server.bat');
    console.log('- Runs: node custom-server.js');
    console.log('- Status: ✗ Failed to connect');
    
    console.log('\nPOSSIBLE ISSUES:');
    console.log('1. Claude Code may timeout during health check');
    console.log('2. Custom server may not send expected responses');
    console.log('3. Different initialization sequence expected');
    console.log('4. Windows path/command execution differences');
    
    ruvSwarm.kill();
    process.exit();
}, 3000);