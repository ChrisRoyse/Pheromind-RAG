const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

console.log('Fixing SQLite binding for claude-flow MCP server...\n');

// Get NPM cache location
const npmCache = execSync('npm config get cache', { encoding: 'utf8' }).trim();
console.log('NPM Cache:', npmCache);

// Source binding
const bindingSource = path.join('C:\\code\\embed\\better-sqlite3\\build\\Release\\better_sqlite3.node');

if (!fs.existsSync(bindingSource)) {
    console.error('ERROR: Binding not found at', bindingSource);
    process.exit(1);
}

console.log('Source binding:', bindingSource);
console.log('Size:', fs.statSync(bindingSource).size, 'bytes\n');

// Find all NPX cache directories with better-sqlite3
const npxDir = path.join(npmCache, '_npx');
let deployCount = 0;

if (fs.existsSync(npxDir)) {
    const dirs = fs.readdirSync(npxDir);
    
    for (const dir of dirs) {
        const sqliteDir = path.join(npxDir, dir, 'node_modules', 'better-sqlite3');
        
        if (fs.existsSync(sqliteDir)) {
            console.log(`Found better-sqlite3 in: ${dir}`);
            
            // Create build/Release directory
            const targetDir = path.join(sqliteDir, 'build', 'Release');
            if (!fs.existsSync(targetDir)) {
                fs.mkdirSync(targetDir, { recursive: true });
            }
            
            // Copy binding
            const targetFile = path.join(targetDir, 'better_sqlite3.node');
            try {
                fs.copyFileSync(bindingSource, targetFile);
                console.log('  ✓ Deployed binding successfully');
                deployCount++;
            } catch (err) {
                console.log('  ✗ Failed:', err.message);
            }
        }
    }
}

console.log(`\nDeployed to ${deployCount} NPX cache directories`);

if (deployCount > 0) {
    console.log('\n✅ SQLite binding fix applied successfully!');
    console.log('The MCP server should now work.');
} else {
    console.log('\n⚠️  No deployments successful. Try running:');
    console.log('npx claude-flow@alpha init --force');
    console.log('Then run this script again.');
}