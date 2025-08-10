#!/usr/bin/env python3
"""
Fix the embed-search MCP server configuration in Claude Code.
The issue: embed-search was added outside the mcpServers object.
"""

import json
from pathlib import Path
import sys

def fix_embed_search_config():
    """Fix embed-search placement in Claude Code configuration."""
    
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
    
    # Remove embed-search from root level if it exists
    if "embed-search" in config:
        del config["embed-search"]
        print("Removed embed-search from root level")
    
    # Ensure mcpServers exists
    if "mcpServers" not in config:
        config["mcpServers"] = {}
        print("Created mcpServers object")
    
    # Add embed-search to mcpServers if not already there
    if "embed-search" not in config["mcpServers"]:
        embed_path = Path("C:/code/embed/target/release/embed-search-mcp.exe")
        
        if not embed_path.exists():
            print(f"Error: MCP executable not found at: {embed_path}")
            return False
        
        config["mcpServers"]["embed-search"] = {
            "type": "stdio",
            "command": str(embed_path).replace("/", "\\\\").replace("\\", "\\\\"),
            "args": []
        }
        print("Added embed-search to mcpServers object")
    else:
        print("embed-search already in mcpServers")
    
    # Backup original config
    backup_path = config_path.with_suffix('.json.backup2')
    with open(config_path, 'r', encoding='utf-8') as orig:
        with open(backup_path, 'w', encoding='utf-8') as backup:
            backup.write(orig.read())
    print(f"Backed up original config to: {backup_path}")
    
    # Write updated config
    try:
        with open(config_path, 'w', encoding='utf-8') as f:
            json.dump(config, f, indent=2)
        print("Successfully fixed embed-search configuration")
        print("\nConfiguration now in mcpServers:")
        print(json.dumps(config["mcpServers"].get("embed-search", {}), indent=2))
        print("\nIMPORTANT: Restart Claude Code for changes to take effect")
        print("After restart, use /mcp to check server status")
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
    success = fix_embed_search_config()
    sys.exit(0 if success else 1)