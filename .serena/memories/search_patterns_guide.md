# Serena Search Patterns for This Codebase

## Common Symbol Search Patterns

### Finding Main Components
```
# Find all search implementations
find_symbol with name_path="SearchImpl" or "Searcher" with substring_matching=true

# Find all services
find_symbol with name_path="Service" with substring_matching=true

# Find configuration structs
find_symbol with name_path="Config" with substring_matching=true

# Find error types
find_symbol with name_path="Error" with substring_matching=true
```

### Finding Methods in Classes
```
# Find all methods in a struct
find_symbol with name_path="StructName" depth=1

# Find specific method
find_symbol with name_path="StructName/method_name"

# Find constructors
find_symbol with name_path="new" with substring_matching=true
```

## Pattern Search Examples

### Finding Feature-Gated Code
```
search_for_pattern with pattern="#\[cfg\(feature.*?ml"
search_for_pattern with pattern="#\[cfg\(feature.*?tantivy"
```

### Finding TODO/FIXME Comments
```
search_for_pattern with pattern="// (TODO|FIXME|HACK|NOTE):"
```

### Finding Error Handling
```
search_for_pattern with pattern="\.context\("
search_for_pattern with pattern="anyhow::Result"
search_for_pattern with pattern="thiserror"
```

### Finding Async Functions
```
search_for_pattern with pattern="async fn"
search_for_pattern with pattern="\.await"
```

### Finding Tests
```
search_for_pattern with pattern="#\[test\]"
search_for_pattern with pattern="#\[cfg\(test\)\]"
```

## Efficient Navigation Strategies

### 1. Start with Overview
```
get_symbols_overview on main files:
- src/lib.rs (exports)
- src/main.rs (entry points)
- src/search/mod.rs (search exports)
```

### 2. Follow Imports
```
search_for_pattern with pattern="^use .*search::"
search_for_pattern with pattern="^use .*embedding::"
```

### 3. Find Implementations
```
find_symbol with name_path="impl.*SearchAdapter" with substring_matching=true
find_symbol with name_path="impl.*Service" with substring_matching=true
```

### 4. Trace Call Chains
```
find_referencing_symbols for key functions to see usage
```

## Performance Tips

### AVOID These Patterns
```
# DON'T: Read entire files
read_file on large files

# DON'T: Search without context restrictions
search_for_pattern with pattern="fn" (too broad)

# DON'T: Get all symbols without filtering
find_symbol with name_path="" 
```

### USE These Instead
```
# DO: Use symbol overview first
get_symbols_overview on specific file

# DO: Restrict searches to directories
search_for_pattern with relative_path="src/search"

# DO: Use specific patterns
search_for_pattern with pattern="fn search_documents"

# DO: Read only symbol bodies you need
find_symbol with include_body=true only when needed
```

## Quick Searches for Common Tasks

### Finding Database Operations
```
search_for_pattern with pattern="LanceDB|lancedb"
search_for_pattern with pattern="CREATE TABLE|INSERT INTO"
```

### Finding Configuration Usage
```
search_for_pattern with pattern="\.get\(.*config"
find_symbol with name_path="Config"
```

### Finding CLI Commands
```
find_symbol with name_path="Commands" depth=1
search_for_pattern with pattern="clap.*command"
```

### Finding Benchmarks
```
search_for_pattern with relative_path="benches"
search_for_pattern with pattern="#\[bench\]"
```