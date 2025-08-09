const { spawn } = require('child_process');

console.log('===========================================');
console.log('DEBUGGING CUSTOM SERVER STARTUP');
console.log('===========================================\n');

// Test 1: Can we run the server directly?
console.log('[TEST 1] Testing direct server execution...');

const serverPath = 'C:/Users/hotra/AppData/Local/claude-flow-mcp/claude-flow-custom-server.js';

try {
    const server = spawn('node', [serverPath], {
        stdio: ['pipe', 'pipe', 'pipe']
    });
    
    let started = false;
    
    server.stdout.on('data', (data) => {
        console.log('Server stdout:', data.toString());
        started = true;
    });
    
    server.stderr.on('data', (data) => {
        console.log('Server stderr:', data.toString());
    });
    
    server.on('exit', (code, signal) => {
        console.log(`Server exited with code ${code}, signal ${signal}`);
    });
    
    server.on('error', (err) => {
        console.log('Server error:', err.message);
    });
    
    // Send initialization after delay
    setTimeout(() => {
        console.log('\nSending initialization request...');
        const initRequest = JSON.stringify({
            jsonrpc: "2.0",
            id: 1,
            method: "initialize",
            params: { capabilities: {} }
        }) + '\n';
        
        server.stdin.write(initRequest);
    }, 1000);
    
    // Check status after delay
    setTimeout(() => {
        if (started) {
            console.log('\n✅ Server started and is responding');
        } else {
            console.log('\n❌ Server started but not responding');
        }
        
        console.log('\n[TEST 2] Testing with exact Claude Code command...');
        
        // Now test with exact same command Claude Code uses
        const claudeCodeTest = spawn('node', [serverPath], {
            stdio: ['pipe', 'pipe', 'pipe']
        });
        
        let claudeCodeResponse = false;
        
        claudeCodeTest.stdout.on('data', (data) => {
            console.log('Claude Code test response:', data.toString());
            claudeCodeResponse = true;
        });
        
        claudeCodeTest.stderr.on('data', (data) => {
            console.log('Claude Code test error:', data.toString());
        });
        
        // Send the same kind of request Claude Code would send
        setTimeout(() => {
            const healthCheck = JSON.stringify({
                jsonrpc: "2.0",
                id: 1,
                method: "initialize",
                params: {
                    capabilities: {},
                    clientInfo: {
                        name: "claude-code",
                        version: "1.0.0"
                    }
                }
            }) + '\n';
            
            claudeCodeTest.stdin.write(healthCheck);
        }, 500);
        
        setTimeout(() => {
            if (claudeCodeResponse) {
                console.log('\n✅ Server works with Claude Code style requests');
            } else {
                console.log('\n❌ Server not responding to Claude Code style requests');
            }
            
            server.kill();
            claudeCodeTest.kill();
            process.exit();
        }, 2000);
        
    }, 3000);
    
} catch (err) {
    console.log('Failed to start server:', err.message);
}