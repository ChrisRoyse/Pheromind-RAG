#!/bin/bash
# JSON Structure Debugger for Claude Code Hooks
# This script captures and analyzes the actual JSON being passed to hooks

DEBUG_LOG="/tmp/claude_hook_debug.log"
SAMPLE_LOG="/tmp/claude_hook_samples.log"

echo "🔍 CLAUDE HOOK JSON STRUCTURE DEBUGGER" | tee "$DEBUG_LOG"
echo "=====================================" | tee -a "$DEBUG_LOG"
echo "Debug log: $DEBUG_LOG" | tee -a "$DEBUG_LOG"
echo "Sample log: $SAMPLE_LOG" | tee -a "$DEBUG_LOG"
echo "" | tee -a "$DEBUG_LOG"

# Create a JSON capture script
cat > /tmp/json_capture.sh << 'EOF'
#!/bin/bash
# Capture and analyze JSON input

JSON_INPUT=$(cat)
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')

# Log the raw JSON
echo "=== TIMESTAMP: $TIMESTAMP ===" >> /tmp/claude_hook_samples.log
echo "RAW JSON INPUT:" >> /tmp/claude_hook_samples.log
echo "$JSON_INPUT" >> /tmp/claude_hook_samples.log

# Try to pretty-print if it's valid JSON
if echo "$JSON_INPUT" | jq . >/dev/null 2>&1; then
    echo "PRETTY JSON:" >> /tmp/claude_hook_samples.log
    echo "$JSON_INPUT" | jq . >> /tmp/claude_hook_samples.log
    
    # Analyze structure
    echo "JSON STRUCTURE ANALYSIS:" >> /tmp/claude_hook_samples.log
    echo "Has 'command': $(echo "$JSON_INPUT" | jq 'has("command")')" >> /tmp/claude_hook_samples.log
    echo "Has 'tool_input': $(echo "$JSON_INPUT" | jq 'has("tool_input")')" >> /tmp/claude_hook_samples.log
    echo "Has 'parameters': $(echo "$JSON_INPUT" | jq 'has("parameters")')" >> /tmp/claude_hook_samples.log
    echo "Has 'input': $(echo "$JSON_INPUT" | jq 'has("input")')" >> /tmp/claude_hook_samples.log
    echo "Has 'args': $(echo "$JSON_INPUT" | jq 'has("args")')" >> /tmp/claude_hook_samples.log
    
    # Try different command extraction paths
    echo "COMMAND EXTRACTION TESTS:" >> /tmp/claude_hook_samples.log
    echo "  .command: $(echo "$JSON_INPUT" | jq -r '.command // "null"')" >> /tmp/claude_hook_samples.log
    echo "  .tool_input.command: $(echo "$JSON_INPUT" | jq -r '.tool_input.command // "null"')" >> /tmp/claude_hook_samples.log
    echo "  .parameters.command: $(echo "$JSON_INPUT" | jq -r '.parameters.command // "null"')" >> /tmp/claude_hook_samples.log
    echo "  .input.command: $(echo "$JSON_INPUT" | jq -r '.input.command // "null"')" >> /tmp/claude_hook_samples.log
    echo "  .args.command: $(echo "$JSON_INPUT" | jq -r '.args.command // "null"')" >> /tmp/claude_hook_samples.log
    
    # Get all keys at root level
    echo "ROOT LEVEL KEYS:" >> /tmp/claude_hook_samples.log
    echo "$JSON_INPUT" | jq -r 'keys | @json' >> /tmp/claude_hook_samples.log
    
else
    echo "INVALID JSON OR NON-JSON INPUT" >> /tmp/claude_hook_samples.log
    echo "Raw content: $JSON_INPUT" >> /tmp/claude_hook_samples.log
fi

echo "===========================================" >> /tmp/claude_hook_samples.log
echo "" >> /tmp/claude_hook_samples.log

# Return empty to prevent command execution during debug
echo ""
EOF

chmod +x /tmp/json_capture.sh

echo "✅ JSON capture script created: /tmp/json_capture.sh" | tee -a "$DEBUG_LOG"
echo "" | tee -a "$DEBUG_LOG"

# Instructions for user
echo "📋 INSTRUCTIONS TO DEBUG JSON STRUCTURE:" | tee -a "$DEBUG_LOG"
echo "========================================" | tee -a "$DEBUG_LOG"
echo "" | tee -a "$DEBUG_LOG"
echo "1. Replace your current hook command with this debug version:" | tee -a "$DEBUG_LOG"
echo "" | tee -a "$DEBUG_LOG"
echo "   REPLACE:" | tee -a "$DEBUG_LOG"
echo "   PreToolUse:Bash [cat | jq -r '.tool_input.command // empty' | ...]" | tee -a "$DEBUG_LOG"
echo "" | tee -a "$DEBUG_LOG"
echo "   WITH:" | tee -a "$DEBUG_LOG"
echo "   PreToolUse:Bash [/tmp/json_capture.sh]" | tee -a "$DEBUG_LOG"
echo "" | tee -a "$DEBUG_LOG"
echo "2. Run a few Claude Code bash commands to capture samples" | tee -a "$DEBUG_LOG"
echo "3. Check the results in: $SAMPLE_LOG" | tee -a "$DEBUG_LOG"
echo "4. Run this script again to analyze the captured JSON" | tee -a "$DEBUG_LOG"
echo "" | tee -a "$DEBUG_LOG"

# Analyze existing samples if they exist
if [ -f "$SAMPLE_LOG" ]; then
    echo "📊 ANALYZING EXISTING CAPTURED SAMPLES:" | tee -a "$DEBUG_LOG"
    echo "======================================" | tee -a "$DEBUG_LOG"
    
    # Count samples
    SAMPLE_COUNT=$(grep -c "TIMESTAMP:" "$SAMPLE_LOG")
    echo "Found $SAMPLE_COUNT captured samples" | tee -a "$DEBUG_LOG"
    
    if [ "$SAMPLE_COUNT" -gt 0 ]; then
        echo "" | tee -a "$DEBUG_LOG"
        echo "🎯 RECOMMENDED FIX BASED ON SAMPLES:" | tee -a "$DEBUG_LOG"
        echo "===================================" | tee -a "$DEBUG_LOG"
        
        # Analyze which command paths work
        if grep -q '"command":' "$SAMPLE_LOG"; then
            echo "✅ Direct .command path found" | tee -a "$DEBUG_LOG"
            RECOMMENDED_JQ='.command'
        elif grep -q '"tool_input".*"command"' "$SAMPLE_LOG"; then
            echo "✅ .tool_input.command path found" | tee -a "$DEBUG_LOG"
            RECOMMENDED_JQ='.tool_input.command'
        elif grep -q '"parameters".*"command"' "$SAMPLE_LOG"; then
            echo "✅ .parameters.command path found" | tee -a "$DEBUG_LOG"
            RECOMMENDED_JQ='.parameters.command'
        else
            echo "⚠️  No clear command path identified" | tee -a "$DEBUG_LOG"
            echo "   Manual analysis of $SAMPLE_LOG required" | tee -a "$DEBUG_LOG"
            RECOMMENDED_JQ=""
        fi
        
        if [ -n "$RECOMMENDED_JQ" ]; then
            echo "" | tee -a "$DEBUG_LOG"
            echo "💡 RECOMMENDED jq COMMAND:" | tee -a "$DEBUG_LOG"
            echo "jq -r '$RECOMMENDED_JQ // empty'" | tee -a "$DEBUG_LOG"
        fi
        
        echo "" | tee -a "$DEBUG_LOG"
        echo "📋 LATEST SAMPLE:" | tee -a "$DEBUG_LOG"
        echo "=================" | tee -a "$DEBUG_LOG"
        tail -20 "$SAMPLE_LOG" | tee -a "$DEBUG_LOG"
    fi
else
    echo "⚠️  No sample data found yet." | tee -a "$DEBUG_LOG"
    echo "   Follow instructions above to capture JSON samples first." | tee -a "$DEBUG_LOG"
fi

echo "" | tee -a "$DEBUG_LOG"
echo "🔧 NEXT STEPS:" | tee -a "$DEBUG_LOG"
echo "=============" | tee -a "$DEBUG_LOG"
echo "1. If no samples exist: Set up debug hook and run Claude commands" | tee -a "$DEBUG_LOG"  
echo "2. If samples exist: Use recommended jq command for final hook" | tee -a "$DEBUG_LOG"
echo "3. Run: ./fix_shell_and_claude_flow.sh (next script)" | tee -a "$DEBUG_LOG"
echo "" | tee -a "$DEBUG_LOG"

echo "Debug script completed. Check $DEBUG_LOG for full output."