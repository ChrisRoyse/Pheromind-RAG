#!/usr/bin/env node

const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

console.log('===========================================');
console.log('Global SQLite Bindings Fix for Claude Flow');
console.log('===========================================\n');

const globalNpmRoot = execSync('npm root -g', { encoding: 'utf8' }).trim();
console.log('Global npm root:', globalNpmRoot);

// Find all better-sqlite3 installations
const claudeFlowPath = path.join(globalNpmRoot, 'claude-flow', 'node_modules', 'better-sqlite3');
const claudeFlowPath2 = path.join(globalNpmRoot, 'better-sqlite3');

const pathsToFix = [
    claudeFlowPath,
    claudeFlowPath2,
    path.join(globalNpmRoot, 'claude-chris', 'node_modules', 'better-sqlite3'),
    path.join(globalNpmRoot, 'ruv-swarm', 'node_modules', 'better-sqlite3')
].filter(p => {
    try {
        return fs.existsSync(p);
    } catch (e) {
        return false;
    }
});

console.log('\nFound better-sqlite3 installations to fix:');
pathsToFix.forEach(p => console.log('  -', p));

if (pathsToFix.length === 0) {
    console.error('\n❌ No better-sqlite3 installations found in global npm!');
    console.error('Run: npm install -g better-sqlite3');
    process.exit(1);
}

// Rebuild each installation
let successCount = 0;
let failCount = 0;

for (const sqlitePath of pathsToFix) {
    console.log(`\n📦 Rebuilding ${sqlitePath}...`);
    
    try {
        // Check if build script exists
        const packageJsonPath = path.join(sqlitePath, 'package.json');
        if (!fs.existsSync(packageJsonPath)) {
            console.error(`  ❌ No package.json found at ${sqlitePath}`);
            failCount++;
            continue;
        }
        
        // Run the build
        console.log('  Building native bindings...');
        execSync('npm run build-release', {
            cwd: sqlitePath,
            stdio: 'inherit'
        });
        
        // Verify the build
        const bindingPath = path.join(sqlitePath, 'build', 'Release', 'better_sqlite3.node');
        if (fs.existsSync(bindingPath)) {
            const stats = fs.statSync(bindingPath);
            console.log(`  ✅ Successfully built! (${stats.size} bytes)`);
            successCount++;
        } else {
            console.error('  ❌ Build completed but binding not found');
            failCount++;
        }
        
    } catch (error) {
        console.error(`  ❌ Build failed: ${error.message}`);
        failCount++;
    }
}

console.log('\n===========================================');
console.log(`Results: ${successCount} successful, ${failCount} failed`);

if (successCount > 0) {
    console.log('\n✅ Global SQLite bindings have been fixed!');
    console.log('You can now use claude-flow from any directory.');
} else {
    console.error('\n❌ Failed to fix SQLite bindings.');
    console.error('Please ensure you have build tools installed:');
    console.error('  - Visual Studio Build Tools');
    console.error('  - Python 3.x');
    process.exit(1);
}