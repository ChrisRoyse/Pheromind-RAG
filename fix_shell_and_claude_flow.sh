#!/bin/bash
# Shell Detection and Claude-Flow Installation Script
# This fixes the shell path and ensures claude-flow works properly

LOG_FILE="/tmp/claude_hook_fix.log"

echo "🔧 SHELL & CLAUDE-FLOW INSTALLATION FIX" | tee "$LOG_FILE"
echo "=======================================" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

## STEP 1: Shell Detection and Path Fixing
echo "📍 STEP 1: Shell Detection" | tee -a "$LOG_FILE"
echo "==========================" | tee -a "$LOG_FILE"

# Detect available shells
AVAILABLE_SHELLS=""
for shell in /bin/bash /usr/bin/bash /bin/sh /usr/bin/sh /bin/dash /usr/bin/dash; do
    if [ -x "$shell" ]; then
        AVAILABLE_SHELLS="$AVAILABLE_SHELLS $shell"
        echo "✅ Found: $shell" | tee -a "$LOG_FILE"
    fi
done

if [ -z "$AVAILABLE_SHELLS" ]; then
    echo "❌ ERROR: No shells found!" | tee -a "$LOG_FILE"
    exit 1
fi

# Determine best shell for WSL2
BEST_SHELL=""
if [ -x "/bin/bash" ]; then
    BEST_SHELL="/bin/bash"
    echo "🎯 Selected: /bin/bash (preferred)" | tee -a "$LOG_FILE"
elif [ -x "/usr/bin/bash" ]; then
    BEST_SHELL="/usr/bin/bash"
    echo "🎯 Selected: /usr/bin/bash (alternative)" | tee -a "$LOG_FILE"
elif [ -x "/bin/sh" ]; then
    BEST_SHELL="/bin/sh"
    echo "🎯 Selected: /bin/sh (fallback)" | tee -a "$LOG_FILE"
else
    BEST_SHELL=$(echo $AVAILABLE_SHELLS | cut -d' ' -f1)
    echo "🎯 Selected: $BEST_SHELL (first available)" | tee -a "$LOG_FILE"
fi

echo "" | tee -a "$LOG_FILE"

## STEP 2: Claude-Flow Installation
echo "📦 STEP 2: Claude-Flow Installation" | tee -a "$LOG_FILE"
echo "====================================" | tee -a "$LOG_FILE"

# Check if npm/npx is available
if ! command -v npx >/dev/null 2>&1; then
    echo "❌ ERROR: npx not found. Installing Node.js..." | tee -a "$LOG_FILE"
    
    # Try to install Node.js (different methods for different systems)
    if command -v apt >/dev/null 2>&1; then
        echo "Installing Node.js via apt..." | tee -a "$LOG_FILE"
        sudo apt update && sudo apt install -y nodejs npm | tee -a "$LOG_FILE"
    elif command -v yum >/dev/null 2>&1; then
        echo "Installing Node.js via yum..." | tee -a "$LOG_FILE"
        sudo yum install -y nodejs npm | tee -a "$LOG_FILE"
    elif command -v brew >/dev/null 2>&1; then
        echo "Installing Node.js via brew..." | tee -a "$LOG_FILE"
        brew install node | tee -a "$LOG_FILE"
    else
        echo "⚠️  Please install Node.js manually from https://nodejs.org" | tee -a "$LOG_FILE"
        echo "   Then re-run this script" | tee -a "$LOG_FILE"
        exit 1
    fi
else
    echo "✅ npx found: $(which npx)" | tee -a "$LOG_FILE"
fi

# Test npm registry access
echo "Testing npm registry access..." | tee -a "$LOG_FILE"
if npm view claude-flow >/dev/null 2>&1; then
    echo "✅ npm registry accessible" | tee -a "$LOG_FILE"
else
    echo "⚠️  npm registry issues detected" | tee -a "$LOG_FILE"
fi

# Install claude-flow@alpha
echo "Installing claude-flow@alpha..." | tee -a "$LOG_FILE"
if npx claude-flow@alpha --version >/dev/null 2>&1; then
    CLAUDE_FLOW_VERSION=$(npx claude-flow@alpha --version 2>&1 | head -1)
    echo "✅ claude-flow@alpha already available: $CLAUDE_FLOW_VERSION" | tee -a "$LOG_FILE"
else
    echo "Installing claude-flow@alpha globally..." | tee -a "$LOG_FILE"
    if npm install -g claude-flow@alpha; then
        echo "✅ claude-flow@alpha installed globally" | tee -a "$LOG_FILE"
    else
        echo "⚠️  Global install failed, will use npx" | tee -a "$LOG_FILE"
        # Test npx access
        if npx --yes claude-flow@alpha --version >/dev/null 2>&1; then
            echo "✅ claude-flow@alpha accessible via npx" | tee -a "$LOG_FILE"
        else
            echo "❌ ERROR: Cannot access claude-flow@alpha" | tee -a "$LOG_FILE"
            echo "   Manual installation required:" | tee -a "$LOG_FILE"
            echo "   npm install -g claude-flow@alpha" | tee -a "$LOG_FILE"
            exit 1
        fi
    fi
fi

# Test claude-flow functionality
echo "" | tee -a "$LOG_FILE"
echo "🧪 STEP 3: Testing Claude-Flow" | tee -a "$LOG_FILE"
echo "==============================" | tee -a "$LOG_FILE"

echo "Testing basic claude-flow commands..." | tee -a "$LOG_FILE"

# Test version
if CLAUDE_FLOW_VERSION=$(npx claude-flow@alpha --version 2>&1 | head -1); then
    echo "✅ Version: $CLAUDE_FLOW_VERSION" | tee -a "$LOG_FILE"
else
    echo "❌ Version check failed" | tee -a "$LOG_FILE"
fi

# Test help command
if npx claude-flow@alpha --help >/dev/null 2>&1; then
    echo "✅ Help command works" | tee -a "$LOG_FILE"
else
    echo "❌ Help command failed" | tee -a "$LOG_FILE"
fi

# Test hook subcommand specifically
if npx claude-flow@alpha hooks --help >/dev/null 2>&1; then
    echo "✅ Hook subcommand available" | tee -a "$LOG_FILE"
else
    echo "⚠️  Hook subcommand not found (may need different claude-flow version)" | tee -a "$LOG_FILE"
fi

echo "" | tee -a "$LOG_FILE"

## STEP 4: Generate Fixed Hook Commands
echo "🎯 STEP 4: Generate Fixed Hook Commands" | tee -a "$LOG_FILE"
echo "=======================================" | tee -a "$LOG_FILE"

# Check if we have JSON structure analysis
SAMPLE_LOG="/tmp/claude_hook_samples.log"
RECOMMENDED_JQ=""

if [ -f "$SAMPLE_LOG" ] && grep -q "RECOMMENDED jq COMMAND" /tmp/claude_hook_debug.log; then
    RECOMMENDED_JQ=$(grep "jq -r" /tmp/claude_hook_debug.log | tail -1 | sed "s/.*jq -r '//" | sed "s/'.*//")
    echo "✅ Using analyzed JSON structure: $RECOMMENDED_JQ" | tee -a "$LOG_FILE"
else
    echo "⚠️  No JSON analysis found. Using universal parser..." | tee -a "$LOG_FILE"
    RECOMMENDED_JQ='if has("command") then .command elif has("tool_input") and (.tool_input | has("command")) then .tool_input.command elif has("parameters") and (.parameters | has("command")) then .parameters.command elif has("input") and (.input | has("command")) then .input.command else empty end'
fi

# Generate the fixed hook command
FIXED_PRE_HOOK="cat | jq -r '$RECOMMENDED_JQ' | tr '\\n' '\\0' | xargs -0 -I {} $BEST_SHELL -c 'command -v npx >/dev/null 2>&1 && npx claude-flow@alpha hooks pre-command --command \"{}\" --validate-safety true --prepare-resources true || echo \"claude-flow hook: {}\"'"

FIXED_POST_HOOK="cat | jq -r '$RECOMMENDED_JQ' | tr '\\n' '\\0' | xargs -0 -I {} $BEST_SHELL -c 'command -v npx >/dev/null 2>&1 && npx claude-flow@alpha hooks post-command --command \"{}\" --track-metrics true --store-results true || echo \"claude-flow completed: {}\"'"

echo "" | tee -a "$LOG_FILE"
echo "🎉 FINAL FIXED HOOK COMMANDS:" | tee -a "$LOG_FILE"
echo "=============================" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"
echo "PreToolUse:Bash [$FIXED_PRE_HOOK]" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"
echo "PostToolUse:Bash [$FIXED_POST_HOOK]" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# Save to file for easy copy-paste
echo "PreToolUse:Bash [$FIXED_PRE_HOOK]" > /tmp/fixed_hooks.txt
echo "PostToolUse:Bash [$FIXED_POST_HOOK]" >> /tmp/fixed_hooks.txt

echo "✅ Fixed hook commands saved to: /tmp/fixed_hooks.txt" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

echo "📋 MANUAL STEPS TO COMPLETE:" | tee -a "$LOG_FILE"
echo "============================" | tee -a "$LOG_FILE"
echo "1. Copy the hook commands from /tmp/fixed_hooks.txt" | tee -a "$LOG_FILE"
echo "2. Update your Claude Code configuration:" | tee -a "$LOG_FILE"
echo "   - claude config set hooks.PreToolUse.Bash 'COMMAND_HERE'" | tee -a "$LOG_FILE"
echo "   - claude config set hooks.PostToolUse.Bash 'COMMAND_HERE'" | tee -a "$LOG_FILE"
echo "3. Test with a simple bash command through Claude" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

echo "🎯 SUMMARY:" | tee -a "$LOG_FILE"
echo "============" | tee -a "$LOG_FILE"
echo "Shell selected: $BEST_SHELL" | tee -a "$LOG_FILE"
echo "Claude-flow status: $(npx claude-flow@alpha --version 2>/dev/null || echo 'Installation needed')" | tee -a "$LOG_FILE"
echo "JSON parser: $( [ -n "$RECOMMENDED_JQ" ] && echo "Custom" || echo "Universal" )" | tee -a "$LOG_FILE"
echo "Fix log: $LOG_FILE" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

echo "Hook system fix completed! Check $LOG_FILE for details."