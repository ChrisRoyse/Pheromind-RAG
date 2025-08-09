#!/usr/bin/env node

const path = require('path');
const { execSync } = require('child_process');

console.log('=========================================');
console.log('Global SQLite Bindings Test');
console.log('=========================================\n');

console.log('Node.js version:', process.version);
console.log('Platform:', process.platform);
console.log('Architecture:', process.arch);
console.log('');

// Test 1: Direct require from global
console.log('Test 1: Loading better-sqlite3 from global installation...');
try {
    const globalRoot = execSync('npm root -g', { encoding: 'utf8' }).trim();
    const globalSqlitePath = path.join(globalRoot, 'claude-flow', 'node_modules', 'better-sqlite3');
    
    console.log('  Path:', globalSqlitePath);
    
    const Database = require(globalSqlitePath);
    
    // Test in-memory database
    const db = new Database(':memory:');
    
    // Create a test table
    db.exec(`
        CREATE TABLE test (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            value INTEGER
        )
    `);
    
    // Insert test data
    const insert = db.prepare('INSERT INTO test (name, value) VALUES (?, ?)');
    insert.run('test1', 100);
    insert.run('test2', 200);
    
    // Query data
    const rows = db.prepare('SELECT * FROM test').all();
    console.log('  ✅ SQLite working! Rows:', rows.length);
    
    db.close();
    
} catch (error) {
    console.error('  ❌ Failed:', error.message);
    console.error('\n  Full error:', error);
    process.exit(1);
}

// Test 2: Test via npx claude-flow
console.log('\nTest 2: Testing claude-flow memory commands...');
try {
    // Store a value
    execSync('claude-flow memory store test-key "Test value from global" --category testing', {
        stdio: 'pipe',
        encoding: 'utf8'
    });
    
    // Retrieve the value
    const result = execSync('claude-flow memory get test-key', {
        encoding: 'utf8'
    });
    
    if (result.includes('Test value from global')) {
        console.log('  ✅ claude-flow memory commands working!');
    } else {
        console.log('  ⚠️  claude-flow executed but value not retrieved');
        console.log('  Output:', result);
    }
    
} catch (error) {
    console.error('  ❌ Failed:', error.message);
}

// Test 3: Test MCP server startup
console.log('\nTest 3: Testing MCP server startup...');
try {
    const child = require('child_process').spawn('claude-flow', ['mcp', 'start'], {
        stdio: 'pipe'
    });
    
    let output = '';
    let errorOutput = '';
    
    child.stdout.on('data', (data) => {
        output += data.toString();
    });
    
    child.stderr.on('data', (data) => {
        errorOutput += data.toString();
    });
    
    // Wait a bit for the server to start
    setTimeout(() => {
        if (child.pid) {
            console.log('  ✅ MCP server started with PID:', child.pid);
            child.kill();
        } else {
            console.log('  ❌ MCP server failed to start');
        }
        
        console.log('\n=========================================');
        console.log('All tests completed!');
        console.log('=========================================');
        
        process.exit(0);
    }, 2000);
    
} catch (error) {
    console.error('  ❌ Failed to start MCP server:', error.message);
    process.exit(1);
}