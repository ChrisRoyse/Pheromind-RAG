#!/bin/bash
# Debug script to fix Claude Code hook JSON parsing

# Universal jq command that handles multiple possible JSON structures
UNIVERSAL_JQ='jq -r "
  if has(\"command\") then .command
  elif has(\"tool_input\") and (.tool_input | has(\"command\")) then .tool_input.command  
  elif has(\"parameters\") and (.parameters | has(\"command\")) then .parameters.command
  elif has(\"input\") and (.input | has(\"command\")) then .input.command
  elif has(\"tool_use\") and (.tool_use | has(\"command\")) then .tool_use.command
  elif has(\"args\") and (.args | has(\"command\")) then .args.command
  else empty
  end"'

echo "BROKEN jq command:"
echo 'jq -r ".tool_input.command // empty"'
echo ""
echo "FIXED universal jq command:"
echo "$UNIVERSAL_JQ"
echo ""
echo "To fix your hook, replace the jq command in your Claude Code settings with:"
echo "$UNIVERSAL_JQ"