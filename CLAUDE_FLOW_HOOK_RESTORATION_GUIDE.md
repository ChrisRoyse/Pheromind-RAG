# 🚀 Claude-Flow Hook System Complete Restoration Guide

## 🎯 OBJECTIVE: Get claude-flow working flawlessly as designed

Your hook system is currently failing with:
```
PreToolUse:Bash [...] spawn /bin/sh ENOENT
```

This guide provides a **systematic, automated solution** to restore full functionality.

## 🔧 WHAT THE RESTORATION DOES

### **Issues Fixed:**
1. **Shell Path Problem**: WSL2 `/bin/sh ENOENT` → Detects correct shell path
2. **JSON Structure Mismatch**: Wrong `.tool_input.command` → Universal JSON parser  
3. **claude-flow Missing**: `npx claude-flow@alpha` fails → Installs and verifies
4. **Error Handling**: Hook failures block all operations → Graceful fallbacks

### **End Result:**
- ✅ Hook system works flawlessly
- ✅ claude-flow commands execute properly  
- ✅ No more `spawn /bin/sh ENOENT` errors
- ✅ Graceful fallback if claude-flow unavailable
- ✅ Full MCP integration as designed

## 🚀 AUTOMATED RESTORATION (Recommended)

### **Step 1: Run Master Restoration Script**
```bash
# Navigate to project directory
cd /home/cabdru/rags/Pheromind-RAG

# Make scripts executable
chmod +x *.sh

# Run complete restoration
./complete_hook_restoration.sh
```

**This script automatically:**
- ✅ Analyzes your environment
- ✅ Debugs JSON structure being passed to hooks
- ✅ Detects correct shell path for WSL2
- ✅ Installs claude-flow@alpha if needed
- ✅ Generates fixed hook commands
- ✅ Attempts automatic configuration
- ✅ Tests the complete system
- ✅ Provides detailed status report

### **Step 2: Verify Results**
After running the script:
```bash
# Check hook configuration
claude config show hooks

# Test with simple command (through Claude Code)
# Should work without errors now
pwd
```

## 🔧 MANUAL RESTORATION (If Automated Fails)

### **Step 1: Debug JSON Structure**
```bash
./debug_hook_json_structure.sh
```

This creates `/tmp/json_capture.sh` - **Replace your current hook temporarily:**

**Replace:**
```
PreToolUse:Bash [cat | jq -r '.tool_input.command // empty' | ...]
```

**With:**  
```
PreToolUse:Bash [/tmp/json_capture.sh]
```

Run some Claude commands, then check `/tmp/claude_hook_samples.log` for the actual JSON structure.

### **Step 2: Fix Shell & Install claude-flow**
```bash
./fix_shell_and_claude_flow.sh
```

This will:
- Detect the correct shell path for your WSL2 system
- Install claude-flow@alpha globally
- Generate optimized hook commands in `/tmp/fixed_hooks.txt`

### **Step 3: Update Hook Configuration**
Copy the commands from `/tmp/fixed_hooks.txt` and update your Claude Code configuration:

```bash
# Get the fixed commands
cat /tmp/fixed_hooks.txt

# Update Claude Code hooks
claude config set "hooks.PreToolUse.Bash" "PASTE_PRE_COMMAND_HERE"
claude config set "hooks.PostToolUse.Bash" "PASTE_POST_COMMAND_HERE"
```

## 🎯 THE TECHNICAL FIX EXPLAINED

### **Original Broken Hook:**
```bash
PreToolUse:Bash [cat | jq -r '.tool_input.command // empty' | tr '\n' '\0' | xargs -0 -I {} npx claude-flow@alpha hooks pre-command --command '{}' --validate-safety true --prepare-resources true]
```

**Problems:**
- ❌ `.tool_input.command` - wrong JSON path
- ❌ `/bin/sh` - doesn't exist in WSL2  
- ❌ No error handling for missing claude-flow

### **Fixed Universal Hook:**
```bash
PreToolUse:Bash [cat | jq -r 'if has("command") then .command elif has("tool_input") and (.tool_input | has("command")) then .tool_input.command elif has("parameters") and (.parameters | has("command")) then .parameters.command else empty end' | tr '\n' '\0' | xargs -0 -I {} /bin/bash -c 'command -v npx >/dev/null 2>&1 && npx claude-flow@alpha hooks pre-command --command "{}" --validate-safety true --prepare-resources true || echo "claude-flow hook: {}"']
```

**Fixes:**
- ✅ Universal JSON parser handles multiple structures
- ✅ `/bin/bash` instead of `/bin/sh` for WSL2
- ✅ Error handling with graceful fallback
- ✅ Validates npx/claude-flow availability before execution

## 🧪 VERIFICATION & TESTING

### **Test Commands (after restoration):**
```bash
# These should work without hook errors:
pwd
ls
echo "test"

# Check logs for errors:
tail -f /tmp/claude_hook_master.log
```

### **Success Indicators:**
- ✅ No more `spawn /bin/sh ENOENT` errors
- ✅ Bash commands execute normally through Claude
- ✅ claude-flow hooks provide feedback/logging
- ✅ `claude config show hooks` shows valid configuration

### **Troubleshooting:**
If issues persist:
1. Check `/tmp/claude_hook_master.log` for detailed diagnosis
2. Verify `npx claude-flow@alpha --version` works independently
3. Test hook commands manually in terminal
4. Consider disabling hooks temporarily: `claude config set hooks.enabled false`

## 📊 EXPECTED OUTCOMES

**Before Fix:**
- ❌ All bash operations blocked by hook errors
- ❌ `spawn /bin/sh ENOENT` in every command
- ❌ claude-flow functionality non-functional
- ❌ Normal Claude Code workflow broken

**After Fix:**
- ✅ Smooth bash operations through Claude Code
- ✅ claude-flow hooks execute properly
- ✅ MCP integration works as designed
- ✅ Full functionality restored
- ✅ Performance monitoring and metrics collection
- ✅ Safety validation and resource preparation

## 🎉 CLAUDE-FLOW FEATURES RESTORED

With working hooks, you get:
- **Pre-command validation** - Safety checks before execution
- **Resource preparation** - Auto-setup for commands
- **Performance tracking** - Metrics collection  
- **Post-command processing** - Result storage and analysis
- **Integration with 87 MCP tools** - Full orchestration
- **Neural pattern learning** - Command optimization
- **Cross-session memory** - Persistent state management

## 🆘 EMERGENCY FALLBACK

If nothing works, disable hooks completely:
```bash
claude config set hooks.enabled false
# OR
claude config unset hooks
# OR
claude config reset
```

This restores basic Claude Code functionality while you debug the claude-flow integration.

---

## 📋 RESTORATION CHECKLIST

- [ ] Run `./complete_hook_restoration.sh`
- [ ] Verify no `spawn /bin/sh ENOENT` errors
- [ ] Test basic bash commands work through Claude
- [ ] Check `claude config show hooks` configuration
- [ ] Verify `npx claude-flow@alpha --version` works
- [ ] Test MCP tool integration
- [ ] Document working configuration

**The goal: claude-flow working flawlessly as designed with full MCP orchestration capabilities.**