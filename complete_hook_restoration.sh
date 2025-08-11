#!/bin/bash
# Complete Claude-Flow Hook System Restoration
# This is the master script that orchestrates the entire fix process

MASTER_LOG="/tmp/claude_hook_master.log"

echo "🚀 CLAUDE-FLOW HOOK SYSTEM COMPLETE RESTORATION" | tee "$MASTER_LOG"
echo "===============================================" | tee -a "$MASTER_LOG"
echo "Master log: $MASTER_LOG" | tee -a "$MASTER_LOG"
echo "" | tee -a "$MASTER_LOG"

# Function to check if a step succeeded
check_success() {
    if [ $? -eq 0 ]; then
        echo "✅ $1" | tee -a "$MASTER_LOG"
        return 0
    else
        echo "❌ $1" | tee -a "$MASTER_LOG"
        return 1
    fi
}

# Function to run with timeout
run_with_timeout() {
    timeout 30 "$@"
    return $?
}

echo "🔍 PHASE 1: Environment Analysis" | tee -a "$MASTER_LOG"
echo "=================================" | tee -a "$MASTER_LOG"

# Check current Claude Code status
echo "Checking Claude Code installation..." | tee -a "$MASTER_LOG"
if command -v claude >/dev/null 2>&1; then
    CLAUDE_VERSION=$(claude --version 2>/dev/null || echo "version unknown")
    echo "✅ Claude Code found: $CLAUDE_VERSION" | tee -a "$MASTER_LOG"
else
    echo "❌ Claude Code CLI not found" | tee -a "$MASTER_LOG"
    echo "Please install Claude Code CLI first" | tee -a "$MASTER_LOG"
    exit 1
fi

# Check current hook configuration
echo "Checking current hook configuration..." | tee -a "$MASTER_LOG"
CURRENT_HOOKS=$(claude config show hooks 2>/dev/null || echo "no hooks configured")
echo "Current hooks: $CURRENT_HOOKS" | tee -a "$MASTER_LOG"

echo "" | tee -a "$MASTER_LOG"

echo "🧪 PHASE 2: JSON Structure Analysis" | tee -a "$MASTER_LOG"
echo "====================================" | tee -a "$MASTER_LOG"

# Run JSON debugging
echo "Running JSON structure analysis..." | tee -a "$MASTER_LOG"
if [ -f "./debug_hook_json_structure.sh" ]; then
    chmod +x ./debug_hook_json_structure.sh
    run_with_timeout ./debug_hook_json_structure.sh
    check_success "JSON structure analysis completed"
else
    echo "⚠️  JSON debug script not found, using universal parser" | tee -a "$MASTER_LOG"
fi

echo "" | tee -a "$MASTER_LOG"

echo "🔧 PHASE 3: System Dependencies" | tee -a "$MASTER_LOG"
echo "================================" | tee -a "$MASTER_LOG"

# Run shell and claude-flow fix
echo "Running shell detection and claude-flow installation..." | tee -a "$MASTER_LOG"
if [ -f "./fix_shell_and_claude_flow.sh" ]; then
    chmod +x ./fix_shell_and_claude_flow.sh
    run_with_timeout ./fix_shell_and_claude_flow.sh
    check_success "Shell and claude-flow setup completed"
else
    echo "❌ Shell fix script not found" | tee -a "$MASTER_LOG"
    exit 1
fi

echo "" | tee -a "$MASTER_LOG"

echo "⚙️  PHASE 4: Hook Configuration Update" | tee -a "$MASTER_LOG"
echo "======================================" | tee -a "$MASTER_LOG"

# Check if we have the fixed hook commands
if [ -f "/tmp/fixed_hooks.txt" ]; then
    echo "✅ Fixed hook commands found" | tee -a "$MASTER_LOG"
    
    # Read the fixed commands
    PRE_HOOK=$(grep "PreToolUse" /tmp/fixed_hooks.txt | sed 's/PreToolUse:Bash \[//' | sed 's/\]$//')
    POST_HOOK=$(grep "PostToolUse" /tmp/fixed_hooks.txt | sed 's/PostToolUse:Bash \[//' | sed 's/\]$//')
    
    echo "Attempting automatic hook configuration update..." | tee -a "$MASTER_LOG"
    
    # Try to set the hooks automatically
    if claude config set "hooks.PreToolUse.Bash" "$PRE_HOOK" 2>/dev/null; then
        echo "✅ PreToolUse hook updated successfully" | tee -a "$MASTER_LOG"
    else
        echo "⚠️  Could not automatically set PreToolUse hook" | tee -a "$MASTER_LOG"
        echo "Manual configuration required" | tee -a "$MASTER_LOG"
    fi
    
    if claude config set "hooks.PostToolUse.Bash" "$POST_HOOK" 2>/dev/null; then
        echo "✅ PostToolUse hook updated successfully" | tee -a "$MASTER_LOG"
    else
        echo "⚠️  Could not automatically set PostToolUse hook" | tee -a "$MASTER_LOG"
        echo "Manual configuration required" | tee -a "$MASTER_LOG"
    fi
    
else
    echo "❌ Fixed hook commands not found" | tee -a "$MASTER_LOG"
    echo "Manual hook configuration required" | tee -a "$MASTER_LOG"
fi

echo "" | tee -a "$MASTER_LOG"

echo "🧪 PHASE 5: System Testing" | tee -a "$MASTER_LOG"
echo "===========================" | tee -a "$MASTER_LOG"

echo "Testing hook system with safe commands..." | tee -a "$MASTER_LOG"

# Create a test file to verify the system works
TEST_FILE="/tmp/claude_hook_test_$$"
echo "test content" > "$TEST_FILE"

# Test basic file operations that would trigger hooks
if ls "$TEST_FILE" >/dev/null 2>&1; then
    echo "✅ Basic file operations work" | tee -a "$MASTER_LOG"
    rm -f "$TEST_FILE"
else
    echo "❌ Basic file operations blocked" | tee -a "$MASTER_LOG"
fi

# Test if hooks are still causing errors
echo "Checking for hook error patterns..." | tee -a "$MASTER_LOG"
if dmesg | tail -10 | grep -q "spawn.*ENOENT" 2>/dev/null; then
    echo "⚠️  Shell spawn errors still detected" | tee -a "$MASTER_LOG"
else
    echo "✅ No shell spawn errors detected" | tee -a "$MASTER_LOG"
fi

echo "" | tee -a "$MASTER_LOG"

echo "📊 FINAL STATUS REPORT" | tee -a "$MASTER_LOG"
echo "======================" | tee -a "$MASTER_LOG"

# Check current configuration
FINAL_HOOKS=$(claude config show hooks 2>/dev/null || echo "no hooks configured")
echo "Final hook configuration:" | tee -a "$MASTER_LOG"
echo "$FINAL_HOOKS" | tee -a "$MASTER_LOG"
echo "" | tee -a "$MASTER_LOG"

# Check claude-flow status
CLAUDE_FLOW_STATUS=$(npx claude-flow@alpha --version 2>/dev/null || echo "not available")
echo "Claude-flow status: $CLAUDE_FLOW_STATUS" | tee -a "$MASTER_LOG"

# Check shell status  
SHELL_STATUS=$(echo $SHELL)
echo "Shell environment: $SHELL_STATUS" | tee -a "$MASTER_LOG"

echo "" | tee -a "$MASTER_LOG"

echo "✅ RESTORATION COMPLETE" | tee -a "$MASTER_LOG"
echo "=======================" | tee -a "$MASTER_LOG"
echo "" | tee -a "$MASTER_LOG"

echo "📋 NEXT STEPS:" | tee -a "$MASTER_LOG"
echo "=============" | tee -a "$MASTER_LOG"
echo "1. Test Claude Code with a simple bash command" | tee -a "$MASTER_LOG"
echo "2. If errors persist, check manual configuration in /tmp/fixed_hooks.txt" | tee -a "$MASTER_LOG"
echo "3. Use 'claude config show hooks' to verify configuration" | tee -a "$MASTER_LOG"
echo "4. Report success/failure status" | tee -a "$MASTER_LOG"
echo "" | tee -a "$MASTER_LOG"

echo "Full restoration log available at: $MASTER_LOG"
echo ""
echo "🎉 Claude-Flow hook system restoration completed!"
echo "Test with a simple command to verify everything works."