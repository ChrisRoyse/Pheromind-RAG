#!/usr/bin/env node

const { execSync } = require('child_process');

console.log('🧪 Testing Claude Flow SQLite3 Memory System\n');
console.log('=' . repeat(60));

function runCommand(cmd, description) {
    console.log(`\n📝 ${description}`);
    console.log(`   Command: ${cmd}`);
    try {
        const output = execSync(cmd, { encoding: 'utf-8', stdio: 'pipe' });
        console.log('✅ Success!');
        if (output && output.trim()) {
            console.log('   Output:', output.trim().substring(0, 200));
        }
        return { success: true, output };
    } catch (error) {
        console.log('❌ Failed:', error.message.substring(0, 100));
        return { success: false, error: error.message };
    }
}

// Test suite
const tests = [
    {
        name: 'Initialize Swarm',
        cmd: 'npx claude-flow@alpha swarm init mesh --max-agents 5'
    },
    {
        name: 'Store Memory - Config',
        cmd: 'npx claude-flow@alpha memory store "swarm_config" "{\\"topology\\":\\"mesh\\",\\"agents\\":5}" --type config'
    },
    {
        name: 'Store Memory - Session',
        cmd: 'npx claude-flow@alpha memory store "session_001" "{\\"start\\":\\"' + new Date().toISOString() + '\\"}" --type session'
    },
    {
        name: 'Store Memory - Agent Data',
        cmd: 'npx claude-flow@alpha memory store "agent_metrics" "{\\"cpu\\":45,\\"memory\\":128}" --type metrics'
    },
    {
        name: 'List All Memory Keys',
        cmd: 'npx claude-flow@alpha memory list'
    },
    {
        name: 'Get Specific Memory',
        cmd: 'npx claude-flow@alpha memory get "swarm_config"'
    },
    {
        name: 'Search Memory by Type',
        cmd: 'npx claude-flow@alpha memory search --type config'
    },
    {
        name: 'Memory Status',
        cmd: 'npx claude-flow@alpha memory status'
    },
    {
        name: 'Export Memory',
        cmd: 'npx claude-flow@alpha memory export --format json'
    },
    {
        name: 'Benchmark Memory Operations',
        cmd: 'npx claude-flow@alpha benchmark memory --iterations 100'
    }
];

// Run tests
let passed = 0;
let failed = 0;

tests.forEach((test, index) => {
    const result = runCommand(test.cmd, `Test ${index + 1}: ${test.name}`);
    if (result.success) {
        passed++;
    } else {
        failed++;
    }
});

// Summary
console.log('\n' + '=' . repeat(60));
console.log('📊 Test Summary:');
console.log(`   ✅ Passed: ${passed}/${tests.length}`);
console.log(`   ❌ Failed: ${failed}/${tests.length}`);

if (passed === tests.length) {
    console.log('\n🎉 All memory tests passed! SQLite3 memory system is working.');
} else if (passed > 0) {
    console.log('\n⚠️  Some tests passed. Memory system is partially working.');
} else {
    console.log('\n❌ All tests failed. Memory system needs investigation.');
}

// Additional integration test
console.log('\n' + '=' . repeat(60));
console.log('🔧 Running Integration Test...\n');

try {
    // Create a workflow that uses memory
    console.log('Creating memory-based workflow...');
    execSync('npx claude-flow@alpha memory store "workflow_state" "{\\"step\\":1,\\"status\\":\\"running\\"}" --type workflow', { stdio: 'pipe' });
    
    // Update the workflow state
    console.log('Updating workflow state...');
    execSync('npx claude-flow@alpha memory store "workflow_state" "{\\"step\\":2,\\"status\\":\\"completed\\"}" --type workflow', { stdio: 'pipe' });
    
    // Retrieve final state
    const finalState = execSync('npx claude-flow@alpha memory get "workflow_state"', { encoding: 'utf-8', stdio: 'pipe' });
    
    if (finalState.includes('completed')) {
        console.log('✅ Integration test passed! Memory persistence verified.');
    } else {
        console.log('⚠️  Integration test incomplete.');
    }
} catch (error) {
    console.log('❌ Integration test failed:', error.message.substring(0, 100));
}

console.log('\n✨ Claude Flow Memory System Test Complete!');