@echo off
:: Claude Code wrapper to work around installation issues

set CLAUDE_CMD="C:/Users/hotra/AppData/Roaming/npm/node_modules/@anthropic-ai/claude-code/cli.js"

node %CLAUDE_CMD% %*