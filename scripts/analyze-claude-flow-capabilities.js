const { spawn } = require('child_process');
const { promisify } = require('util');
const exec = promisify(require('child_process').exec);

console.log('===========================================');
console.log('ANALYZING FULL CLAUDE-FLOW CAPABILITIES');
console.log('===========================================\n');

async function analyzeClaudeFlow() {
    const commands = [
        { cmd: 'npx claude-flow@alpha --help', name: 'Main Help' },
        { cmd: 'npx claude-flow@alpha sparc --help', name: 'SPARC Help' },
        { cmd: 'npx claude-flow@alpha hooks --help', name: 'Hooks Help' },
        { cmd: 'npx claude-flow@alpha memory --help', name: 'Memory Help' },
        { cmd: 'npx claude-flow@alpha github --help', name: 'GitHub Help' },
        { cmd: 'npx claude-flow@alpha monitoring --help', name: 'Monitoring Help' },
        { cmd: 'npx claude-flow@alpha workflows --help', name: 'Workflows Help' },
        { cmd: 'npx claude-flow@alpha coordination --help', name: 'Coordination Help' },
        { cmd: 'npx claude-flow@alpha optimization --help', name: 'Optimization Help' },
        { cmd: 'npx claude-flow@alpha training --help', name: 'Training Help' },
        { cmd: 'npx claude-flow@alpha analysis --help', name: 'Analysis Help' },
        { cmd: 'npx claude-flow@alpha automation --help', name: 'Automation Help' }
    ];
    
    console.log('Testing all claude-flow command categories...\n');
    
    const results = [];
    
    for (const test of commands) {
        try {
            console.log(`[TESTING] ${test.name}...`);
            const { stdout, stderr } = await exec(test.cmd, { 
                timeout: 10000,
                cwd: process.cwd()
            });
            
            if (stdout.trim()) {
                console.log(`✅ ${test.name}: Available`);
                console.log(`   Commands found: ${(stdout.match(/^\s*\w+/gm) || []).length}`);
                results.push({
                    category: test.name,
                    available: true,
                    output: stdout,
                    commands: extractCommands(stdout)
                });
            } else {
                console.log(`❌ ${test.name}: No output`);
                results.push({
                    category: test.name,
                    available: false,
                    error: 'No output'
                });
            }
        } catch (error) {
            if (error.message.includes('Unknown command') || error.message.includes('not found')) {
                console.log(`❌ ${test.name}: Not available`);
            } else {
                console.log(`⚠️  ${test.name}: Error - ${error.message.substring(0, 50)}...`);
            }
            results.push({
                category: test.name,
                available: false,
                error: error.message
            });
        }
        console.log('');
    }
    
    // Summary
    console.log('===========================================');
    console.log('CAPABILITIES SUMMARY');
    console.log('===========================================\n');
    
    const available = results.filter(r => r.available);
    const unavailable = results.filter(r => !r.available);
    
    console.log(`✅ Available Categories: ${available.length}`);
    available.forEach(r => {
        console.log(`   - ${r.category} (${r.commands ? r.commands.length : 0} commands)`);
        if (r.commands) {
            r.commands.slice(0, 3).forEach(cmd => console.log(`     • ${cmd}`));
            if (r.commands.length > 3) {
                console.log(`     • ... and ${r.commands.length - 3} more`);
            }
        }
    });
    
    console.log(`\n❌ Unavailable Categories: ${unavailable.length}`);
    unavailable.forEach(r => {
        console.log(`   - ${r.category}`);
    });
    
    // Generate comprehensive tool list
    console.log('\n===========================================');
    console.log('COMPREHENSIVE TOOL REQUIREMENTS');
    console.log('===========================================\n');
    
    console.log('Based on analysis, the MCP server needs:');
    
    let totalTools = 0;
    available.forEach(category => {
        if (category.commands) {
            console.log(`\n${category.category.toUpperCase()}:`);
            category.commands.forEach(cmd => {
                const toolName = `claude_flow_${cmd.toLowerCase().replace(/[^a-z0-9]/g, '_')}`;
                console.log(`  ${toolName}: ${cmd}`);
                totalTools++;
            });
        }
    });
    
    console.log(`\n📊 TOTAL TOOLS NEEDED: ${totalTools}`);
    console.log('Current MCP server has: 3 tools');
    console.log(`Missing: ${totalTools - 3} tools\n`);
    
    return { available, unavailable, totalTools };
}

function extractCommands(helpOutput) {
    // Extract command names from help output
    const lines = helpOutput.split('\n');
    const commands = [];
    
    let inCommandsSection = false;
    for (const line of lines) {
        if (line.toLowerCase().includes('commands:') || line.toLowerCase().includes('available commands')) {
            inCommandsSection = true;
            continue;
        }
        
        if (inCommandsSection) {
            const match = line.match(/^\s*([a-zA-Z][a-zA-Z0-9-_]*)/);
            if (match && !match[1].includes('Options') && !match[1].includes('Usage')) {
                commands.push(match[1]);
            }
            
            if (line.trim() === '' && commands.length > 0) {
                break;
            }
        }
    }
    
    return commands;
}

analyzeClaudeFlow().catch(console.error);