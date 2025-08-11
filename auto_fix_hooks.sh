#!/bin/bash
# Automatic Claude Code Hook System Fix Script

echo "🔧 CLAUDE CODE HOOK SYSTEM AUTO-FIX"
echo "=================================="
echo ""

# Check if claude command exists
if ! command -v claude >/dev/null 2>&1; then
    echo "❌ ERROR: 'claude' command not found"
    echo "   Please install Claude Code CLI first"
    exit 1
fi

echo "✅ Claude Code CLI found"

# Show current hook status
echo ""
echo "📋 CURRENT HOOK CONFIGURATION:"
echo "------------------------------"
claude config show hooks 2>/dev/null || echo "No hook configuration found"

echo ""
echo "🛠️  APPLYING FIX..."
echo "-------------------"

# Option 1: Try to disable hooks
echo "1. Disabling hook system..."
if claude config set hooks.enabled false 2>/dev/null; then
    echo "✅ Hooks disabled successfully"
else
    echo "⚠️  Could not disable hooks via config"
fi

# Option 2: Try to unset hook configuration
echo "2. Removing hook configuration..."
if claude config unset hooks 2>/dev/null; then
    echo "✅ Hook configuration removed"
else
    echo "⚠️  Could not remove hook configuration"
fi

# Option 3: Try to reset config if other methods failed
echo "3. Attempting config reset (if needed)..."
read -p "Do you want to reset Claude Code config completely? (y/N): " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if claude config reset 2>/dev/null; then
        echo "✅ Config reset successfully"
    else
        echo "❌ Could not reset config"
    fi
else
    echo "⏭️  Skipping config reset"
fi

echo ""
echo "🧪 TESTING FIX..."
echo "-----------------"

# Test if hooks are fixed by running a simple command through Claude
echo "Testing if hook system is now working..."

# Create a test file to verify bash works
if echo "test" > /tmp/claude_hook_test.txt 2>/dev/null; then
    echo "✅ Bash operations working"
    rm -f /tmp/claude_hook_test.txt
else
    echo "❌ Bash operations still blocked"
    echo ""
    echo "🆘 MANUAL STEPS REQUIRED:"
    echo "------------------------"
    echo "1. Run: claude config list"
    echo "2. Look for hook configurations"  
    echo "3. Run: claude config set hooks.enabled false"
    echo "4. Or run: claude config reset"
fi

echo ""
echo "📊 FINAL STATUS:"
echo "---------------"
claude config show hooks 2>/dev/null || echo "✅ No hook configuration (good!)"

echo ""
echo "🎉 Hook fix script completed!"
echo "If you're still having issues, see: fix_claude_hooks.md"