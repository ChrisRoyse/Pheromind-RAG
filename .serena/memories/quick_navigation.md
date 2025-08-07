# Quick Navigation Guide for AI Models

## 🎯 Jump-To Locations by Task

### "I need to modify search logic"
```
1. get_symbols_overview "src/search/unified.rs"
2. find_symbol "UnifiedSearcher" depth=1
3. find_symbol "search_documents" include_body=true
```

### "I need to add a new CLI command"
```
1. get_symbols_overview "src/main.rs"
2. find_symbol "Commands" depth=1
3. find_symbol "main" include_body=true
4. Add new variant to Commands enum
5. Add handler function
```

### "I need to modify embeddings"
```
1. get_symbols_overview "src/embedding/nomic.rs"
2. find_symbol "NomicEmbedding" depth=1
3. Check feature flag: search_for_pattern "#\[cfg\(feature.*ml"
```

### "I need to fix a test"
```
1. find_file "*.rs" "tests"
2. search_for_pattern "#\[test\]" relative_path="tests"
3. find_symbol "test_name" include_body=true
```

### "I need to add a feature flag"
```
1. Edit Cargo.toml [features] section
2. search_for_pattern "#\[cfg\(feature" to see examples
3. Add cfg attributes to gated code
```

### "I need to optimize performance"
```
1. find_file "*bench*.rs" "benches"
2. search_for_pattern "criterion::" 
3. find_symbol "bench_" substring_matching=true
4. Check src/cache/ for caching logic
```

### "I need to debug an error"
```
1. get_symbols_overview "src/error.rs"
2. search_for_pattern "Error.*enum"
3. find_referencing_symbols for error type
```

### "I need to modify configuration"
```
1. get_symbols_overview "src/config/mod.rs"
2. find_symbol "Config" include_body=true
3. search_for_pattern "serde.*Config"
```

## 🚀 Speed Patterns

### Fast Overview of Module
```
# Instead of reading whole file:
get_symbols_overview "src/module/mod.rs"

# Then drill down:
find_symbol "SpecificStruct" depth=1
```

### Fast Function Location
```
# Don't search broadly:
search_for_pattern "fn function_name"

# Use symbol search:
find_symbol "function_name" substring_matching=true
```

### Fast Implementation Finding
```
# Find trait implementations:
search_for_pattern "impl.*TraitName.*for"

# Find struct implementations:
find_symbol "impl StructName" substring_matching=true
```

### Fast Test Location
```
# For unit tests in file:
search_for_pattern "#\[cfg\(test\)\]" relative_path="src/module.rs"

# For integration tests:
find_file "*test*.rs" "tests"
```

## 📁 Module Quick Reference

| Task | Primary File | Key Symbol |
|------|-------------|------------|
| CLI Entry | src/main.rs | `Commands` enum |
| Search Logic | src/search/unified.rs | `UnifiedSearcher` |
| Embeddings | src/embedding/nomic.rs | `NomicEmbedding` |
| Storage | src/storage/lancedb.rs | `LanceDBStorage` |
| Config | src/config/mod.rs | `Config` struct |
| Git Watch | src/git/mod.rs | `GitWatcher` |
| Chunking | src/chunking/mod.rs | `Chunker` trait |
| MCP Tools | (See docs/) | Tool handlers |

## 🔍 Search Shortcuts

### By Feature
```
# ML features:
search_for_pattern "#\[cfg\(feature.*\"ml\"\)\]"

# Tantivy features:
search_for_pattern "#\[cfg\(feature.*\"tantivy\"\)\]"

# Vector DB:
search_for_pattern "#\[cfg\(feature.*\"vectordb\"\)\]"
```

### By Pattern Type
```
# Async functions:
search_for_pattern "async fn" relative_path="src"

# Public APIs:
search_for_pattern "^pub fn" relative_path="src"

# Trait definitions:
search_for_pattern "^(pub )?trait "

# Error handling:
search_for_pattern "Result<.*Error>"
```

## ⚡ Performance Critical Paths

Quick access to hot paths:
1. `src/search/fusion.rs` - Score fusion
2. `src/embedding/cache.rs` - Embedding cache
3. `src/search/bm25.rs` - Text scoring
4. `src/chunking/regex_chunker.rs` - File chunking

## 🎭 Remember

- **Never** read entire large files
- **Always** use symbol overview first
- **Prefer** symbol search over pattern search
- **Restrict** searches to specific directories
- **Check** feature flags before modifying gated code