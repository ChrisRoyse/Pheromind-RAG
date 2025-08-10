#!/usr/bin/env python3
"""
Configure embed-search MCP server for Claude Code (not Claude Desktop).
This script safely adds the configuration to ~/.claude.json
"""

import json
import os
from pathlib import Path
import sys

def configure_embed_search():
    """Add embed-search MCP server to Claude Code configuration."""
    
    # Claude Code config path (Windows)
    config_path = Path.home() / ".claude.json"
    
    if not config_path.exists():
        print(f"Error: Claude Code config not found at: {config_path}")
        return False
    
    print(f"Found Claude Code config at: {config_path}")
    
    # Load existing config
    try:
        with open(config_path, 'r', encoding='utf-8') as f:
            config = json.load(f)
    except json.JSONDecodeError as e:
        print(f"Error parsing JSON: {e}")
        return False
    
    # Check if embed-search already configured
    if "embed-search" in config:
        print("embed-search already configured in Claude Code")
        return True
    
    # Get the embed-search executable path
    embed_path = Path("C:/code/embed/target/release/embed-search-mcp.exe")
    
    if not embed_path.exists():
        print(f"Error: MCP executable not found at: {embed_path}")
        return False
    
    # Add embed-search configuration at root level
    # Following the pattern of Neo4j servers
    config["embed-search"] = {
        "type": "stdio",
        "command": str(embed_path).replace("/", "\\\\").replace("\\", "\\\\"),
        "args": []
    }
    
    # Backup original config
    backup_path = config_path.with_suffix('.json.backup')
    with open(backup_path, 'w', encoding='utf-8') as f:
        with open(config_path, 'r', encoding='utf-8') as orig:
            f.write(orig.read())
    print(f"Backed up original config to: {backup_path}")
    
    # Write updated config
    try:
        with open(config_path, 'w', encoding='utf-8') as f:
            json.dump(config, f, indent=2)
        print(f"Successfully added embed-search to Claude Code config")
        print("\nConfiguration added:")
        print(json.dumps(config["embed-search"], indent=2))
        print("\nIMPORTANT: Restart Claude Code for changes to take effect")
        print("    After restart, use /mcp to check server status")
        return True
    except Exception as e:
        print(f"Error writing config: {e}")
        # Restore backup
        with open(backup_path, 'r', encoding='utf-8') as backup:
            with open(config_path, 'w', encoding='utf-8') as f:
                f.write(backup.read())
        print("Restored original config from backup")
        return False

if __name__ == "__main__":
    success = configure_embed_search()
    sys.exit(0 if success else 1)