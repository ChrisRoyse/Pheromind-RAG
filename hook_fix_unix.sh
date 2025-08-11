#!/bin/bash
# Complete Claude-Flow Hook System Fix - Unix Line Endings
# This fixes the line ending issue and restores hook functionality

echo "🔧 CLAUDE-FLOW HOOK SYSTEM FIX"
echo "==============================="
echo ""

# Step 1: Fix line endings in all scripts
echo "📝 Step 1: Fixing line endings in scripts..."
for script in complete_hook_restoration.sh debug_hook_json_structure.sh fix_shell_and_claude_flow.sh; do
    if [ -f "$script" ]; then
        # Remove Windows line endings
        sed -i 's/\r$//' "$script" 2>/dev/null
        echo "✅ Fixed line endings: $script"
    else
        echo "⚠️  Script not found: $script"
    fi
done

echo ""

# Step 2: Check shell environment
echo "🐚 Step 2: Shell Environment Check"
echo "Available shells:"
ls -la /bin/bash /usr/bin/bash /bin/sh 2>/dev/null || echo "Checking other locations..."

BEST_SHELL=""
if [ -x "/bin/bash" ]; then
    BEST_SHELL="/bin/bash"
    echo "✅ Selected: /bin/bash"
elif [ -x "/usr/bin/bash" ]; then
    BEST_SHELL="/usr/bin/bash"  
    echo "✅ Selected: /usr/bin/bash"
else
    echo "❌ No bash found"
    exit 1
fi

echo ""

# Step 3: Check Node.js/npm
echo "📦 Step 3: Node.js Environment"
if command -v node >/dev/null 2>&1; then
    echo "✅ Node.js: $(node --version)"
else
    echo "❌ Node.js not found"
fi

if command -v npm >/dev/null 2>&1; then
    echo "✅ npm: $(npm --version)"
else
    echo "❌ npm not found"
fi

if command -v npx >/dev/null 2>&1; then
    echo "✅ npx: $(npx --version)"
else
    echo "❌ npx not found"
fi

echo ""

# Step 4: Test claude-flow
echo "🧪 Step 4: Claude-Flow Test"
if npx claude-flow@alpha --version >/dev/null 2>&1; then
    CF_VERSION=$(npx claude-flow@alpha --version 2>&1 | head -1)
    echo "✅ claude-flow@alpha: $CF_VERSION"
    CLAUDE_FLOW_OK=true
else
    echo "❌ claude-flow@alpha not accessible"
    echo "Attempting installation..."
    if npm install -g claude-flow@alpha >/dev/null 2>&1; then
        echo "✅ claude-flow@alpha installed"
        CLAUDE_FLOW_OK=true
    else
        echo "⚠️  Installation failed, will use fallback"
        CLAUDE_FLOW_OK=false
    fi
fi

echo ""

# Step 5: Generate working hook commands
echo "⚙️  Step 5: Generate Hook Commands"

# Universal JSON parser that handles multiple structures
JSON_PARSER='if has("command") then .command elif has("tool_input") and (.tool_input | has("command")) then .tool_input.command elif has("parameters") and (.parameters | has("command")) then .parameters.command elif has("input") and (.input | has("command")) then .input.command else empty end'

if [ "$CLAUDE_FLOW_OK" = true ]; then
    # Full claude-flow integration
    PRE_HOOK="cat | jq -r '$JSON_PARSER' | tr '\n' '\0' | xargs -0 -I {} $BEST_SHELL -c 'command -v npx >/dev/null 2>&1 && npx claude-flow@alpha hooks pre-command --command \"{}\" --validate-safety true --prepare-resources true || echo \"hook: {}\"'"
    
    POST_HOOK="cat | jq -r '$JSON_PARSER' | tr '\n' '\0' | xargs -0 -I {} $BEST_SHELL -c 'command -v npx >/dev/null 2>&1 && npx claude-flow@alpha hooks post-command --command \"{}\" --track-metrics true --store-results true || echo \"completed: {}\"'"
else
    # Minimal working hooks without claude-flow
    PRE_HOOK="cat | jq -r '$JSON_PARSER' | tr '\n' '\0' | xargs -0 -I {} echo \"hook: {}\""
    POST_HOOK="cat | jq -r '$JSON_PARSER' | tr '\n' '\0' | xargs -0 -I {} echo \"completed: {}\""
fi

echo "Generated hook commands:"
echo "PRE:  $PRE_HOOK"
echo "POST: $POST_HOOK"

# Save to files
echo "PreToolUse:Bash [$PRE_HOOK]" > fixed_hooks_unix.txt
echo "PostToolUse:Bash [$POST_HOOK]" >> fixed_hooks_unix.txt

echo "✅ Hook commands saved to: fixed_hooks_unix.txt"
echo ""

# Step 6: Instructions
echo "📋 MANUAL CONFIGURATION REQUIRED:"
echo "=================================="
echo ""
echo "Your hook system needs these commands to be configured in Claude Code:"
echo ""
echo "1. Copy these commands:"
cat fixed_hooks_unix.txt
echo ""
echo "2. Configure Claude Code hooks:"
echo "   claude config set 'hooks.PreToolUse.Bash' '$PRE_HOOK'"
echo "   claude config set 'hooks.PostToolUse.Bash' '$POST_HOOK'"
echo ""
echo "3. Or disable hooks if still having issues:"
echo "   claude config set hooks.enabled false"
echo ""
echo "4. Test with a simple command through Claude Code"

echo ""
echo "🎯 Fix completed! Check fixed_hooks_unix.txt for commands to configure."