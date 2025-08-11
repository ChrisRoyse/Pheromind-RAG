# CLAUDE CODE HOOK SYSTEM FIX

## THE PROBLEM

Your Claude Code hook system is failing with:
```
spawn /bin/sh ENOENT: Error occurred while executing hook command
```

## ROOT CAUSE ANALYSIS

1. **Shell Path Issue**: WSL2 doesn't have `/bin/sh` where expected
2. **JSON Structure Issue**: Hook expects `.tool_input.command` but structure may be different  
3. **claude-flow Missing**: `npx claude-flow@alpha` not accessible

## THE COMPLETE FIX

### Option 1: DISABLE HOOKS (Recommended)
```bash
# Run in your terminal (not through Claude)
claude config set hooks.enabled false
```

### Option 2: FIX THE HOOK COMMAND

Your current hook:
```bash
PreToolUse:Bash [cat | jq -r '.tool_input.command // empty' | tr '\n' '\0' | xargs -0 -I {} npx claude-flow@alpha hooks pre-command --command '{}' --validate-safety true --prepare-resources true]
```

**FIXED VERSION:**
```bash
PreToolUse:Bash [cat | jq -r 'if has("command") then .command elif has("tool_input") and (.tool_input | has("command")) then .tool_input.command elif has("parameters") and (.parameters | has("command")) then .parameters.command else empty end' | tr '\n' '\0' | xargs -0 -I {} /bin/bash -c 'command -v npx >/dev/null 2>&1 && npx claude-flow@alpha hooks pre-command --command "{}" --validate-safety true --prepare-resources true || echo "claude-flow not available"']
```

**KEY CHANGES:**
1. **Universal JSON Parser**: Handles multiple JSON structures
2. **Fixed Shell**: Uses `/bin/bash` instead of `/bin/sh`  
3. **Error Handling**: Checks if claude-flow exists before calling

### Option 3: MINIMAL SAFE HOOK

If you want to keep hooks but make them safe:
```bash
PreToolUse:Bash [echo "Hook: Bash command detected" && cat > /dev/null]
PostToolUse:Bash [echo "Hook: Bash command completed"]
```

## HOW TO APPLY THE FIX

1. **Find Claude Code Settings:**
   ```bash
   claude config list
   claude config show hooks
   ```

2. **Option A - Disable Hooks:**
   ```bash
   claude config set hooks.enabled false
   ```

3. **Option B - Update Hook Command:**
   ```bash
   claude config set hooks.PreToolUse.Bash "NEW_COMMAND_HERE"
   ```

4. **Option C - Reset Everything:**
   ```bash
   claude config reset
   ```

## VERIFICATION

After applying the fix, test with a simple command:
```bash
# This should work without hook errors
pwd
```

## WHY THIS HAPPENS

- WSL2 environments have different shell paths than expected
- claude-flow might not be installed globally
- JSON structure varies between Claude Code versions
- Hook system assumes specific environment setup

The hook system is **blocking all Bash operations** until fixed.