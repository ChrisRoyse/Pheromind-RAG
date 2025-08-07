# Task 3.003: Update Query Parser for v0.24 API

**Time Estimate**: 8 minutes
**Priority**: CRITICAL
**Dependencies**: task_002
**File(s) to Modify**: `src/search/tantivy_search.rs`

## Objective
Ensure QueryParser is compatible with Tantivy v0.24 API and can create valid queries.

## Success Criteria
- [ ] QueryParser::for_index works with v0.24
- [ ] Query parsing doesn't crash
- [ ] Basic queries can be executed
- [ ] Field references are correct

## Instructions

### Step 1: Check QueryParser instantiation
```rust
// Verify this pattern works:
let query_parser = QueryParser::for_index(
    &self.index,
    vec![self.body_field],
);
```

### Step 2: Test query parsing
```rust
// Add test method:
fn test_query_parsing(&self) -> Result<()> {
    let query_parser = QueryParser::for_index(
        &self.index, 
        vec![self.body_field]
    );
    
    let query = query_parser.parse_query("test query")?;
    println!("Query parsed successfully: {:?}", query);
    Ok(())
}
```

### Step 3: Check imports
```rust
// Ensure these are imported:
use tantivy::query::QueryParser;
use tantivy::query::Query;
```

### Step 4: Test query execution
```bash
cargo test --features tantivy test_query_parser
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features tantivy
cargo test --features tantivy test_query_parser -v
```

## Troubleshooting
- If QueryParser::for_index fails, check v0.24 docs for API changes
- If parse_query fails, verify query syntax is valid
- Check that field references are properly initialized

## Next Task
task_004 - Create basic index creation test