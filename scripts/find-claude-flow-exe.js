const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

console.log('Finding claude-flow executable location...\n');

// Get NPM cache
const npmCache = execSync('npm config get cache', { encoding: 'utf8' }).trim();
const npxDir = path.join(npmCache, '_npx');

console.log('NPM Cache:', npmCache);
console.log('NPX Directory:', npxDir);
console.log('\nSearching for claude-flow...\n');

// Find claude-flow in NPX cache
if (fs.existsSync(npxDir)) {
    const dirs = fs.readdirSync(npxDir);
    
    for (const dir of dirs) {
        const fullPath = path.join(npxDir, dir);
        const packagePath = path.join(fullPath, 'node_modules', 'claude-flow');
        
        if (fs.existsSync(packagePath)) {
            console.log(`Found claude-flow in: ${dir}`);
            
            // Look for executable files
            const binPath = path.join(fullPath, 'node_modules', '.bin');
            if (fs.existsSync(binPath)) {
                const binFiles = fs.readdirSync(binPath);
                console.log('  Binary files:', binFiles.filter(f => f.includes('claude')));
            }
            
            // Check for main entry point
            const packageJsonPath = path.join(packagePath, 'package.json');
            if (fs.existsSync(packageJsonPath)) {
                const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
                console.log('  Main:', pkg.main);
                console.log('  Bin:', pkg.bin);
                
                // Find the actual Node.js script
                if (pkg.bin && pkg.bin['claude-flow']) {
                    const scriptPath = path.join(packagePath, pkg.bin['claude-flow']);
                    console.log('  Script path:', scriptPath);
                    
                    if (fs.existsSync(scriptPath)) {
                        console.log('  ✓ Script exists');
                        
                        // Check for Windows executable wrappers
                        const cmdPath = path.join(binPath, 'claude-flow.cmd');
                        const ps1Path = path.join(binPath, 'claude-flow.ps1');
                        
                        if (fs.existsSync(cmdPath)) {
                            console.log('  ✓ Windows CMD wrapper:', cmdPath);
                        }
                        if (fs.existsSync(ps1Path)) {
                            console.log('  ✓ PowerShell wrapper:', ps1Path);
                        }
                    }
                }
            }
        }
    }
}

console.log('\n===========================================');
console.log('SOLUTION APPROACH');
console.log('===========================================');
console.log('\nBased on Neo4j example, we need to:');
console.log('1. Create a direct executable wrapper');
console.log('2. Use full path to that executable');
console.log('3. Pass "mcp start" as arguments');
console.log('4. Avoid npx completely');