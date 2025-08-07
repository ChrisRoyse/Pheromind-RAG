# Task 1.018: Fix Schema Compatibility Checking

**Time Estimate**: 8 minutes
**Dependencies**: None
**File(s) to Modify**: `src/search/tantivy_search.rs`

## Objective
Improve schema compatibility checking to prevent silent corruption and data loss.

## Success Criteria
- [ ] Schema mismatches are detected reliably
- [ ] Field type compatibility is verified
- [ ] Clear error messages for incompatible schemas
- [ ] No silent schema migrations

## Instructions

### Step 1: Enhance is_schema_compatible function
```rust
// Improve schema compatibility checking around line 177:
fn is_schema_compatible(index: &Index, expected_schema: &Schema) -> bool {
    let index_schema = index.schema();
    
    // Check field count
    if index_schema.num_fields() != expected_schema.num_fields() {
        log::warn!("Schema field count mismatch: index has {}, expected {}", 
                   index_schema.num_fields(), expected_schema.num_fields());
        return false;
    }
    
    // Check each required field exists with correct type
    let required_fields = [
        ("file_path", tantivy::schema::TEXT | tantivy::schema::STORED),
        ("line_number", tantivy::schema::STORED),
        ("content", tantivy::schema::TEXT | tantivy::schema::STORED),
        ("line_content", tantivy::schema::STORED),
    ];
    
    for (field_name, expected_options) in &required_fields {
        match (expected_schema.get_field(field_name), index_schema.get_field(field_name)) {
            (Ok(expected_field), Ok(index_field)) => {
                let expected_entry = expected_schema.get_field_entry(expected_field);
                let index_entry = index_schema.get_field_entry(index_field);
                
                // Check field options compatibility
                if expected_entry.field_type() != index_entry.field_type() {
                    log::warn!("Field '{}' type mismatch", field_name);
                    return false;
                }
            }
            _ => {
                log::warn!("Required field '{}' missing from schema", field_name);
                return false;
            }
        }
    }
    
    true
}
```

### Step 2: Add schema version tracking
```rust
// Add schema version to detect changes:
const SCHEMA_VERSION: u32 = 1;

fn create_schema_with_version() -> Schema {
    let mut schema_builder = Schema::builder();
    
    // Add a version field to track schema changes
    let _version_field = schema_builder.add_u64_field("__schema_version", STORED);
    
    // ... existing field definitions
    schema_builder.build()
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features tantivy
```

## Next Task
task_019 - Validate index rebuild safety