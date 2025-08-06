#!/bin/bash

# Script to find .md files in .claude/agents/ that don't start with "---"
# Returns first 50 files that need YAML front matter fixes

AGENTS_DIR="C:/code/embed/.claude/agents"
count=0
max_files=50

find "$AGENTS_DIR" -name "*.md" -type f | while read -r file; do
    if [ $count -ge $max_files ]; then
        break
    fi
    
    # Check if first line starts with "---"
    first_line=$(head -n1 "$file" 2>/dev/null)
    if [[ "$first_line" != "---" ]]; then
        echo "$file"
        ((count++))
    fi
done