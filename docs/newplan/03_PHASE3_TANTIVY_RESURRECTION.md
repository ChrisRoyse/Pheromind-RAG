# Phase 3: Tantivy Resurrection - Full-Text Search

**Duration**: 1 day  
**Goal**: Make Tantivy full-text search operational  
**Success Metric**: Tantivy indexes and searches successfully

## Task 3.1: Fix Tantivy Index Creation (2 hours)

### Current State
Tantivy won't compile due to API changes in v0.24. The `sort_by_field` option was removed.

### Step 1: Update Index Settings

```rust
// File: src/search/tantivy_search.rs

// DELETE the old code around line 165:
let index_settings = IndexSettings {
    sort_by_field: None,  // REMOVE THIS LINE
    docstore_compression: Compressor::Lz4,
    docstore_blocksize: 16384,
};

// REPLACE with v0.24 compatible code:
let index_settings = IndexSettings {
    docstore_compression: Compressor::Lz4,
    docstore_blocksize: 16384,
};
```

### Step 2: Check for Other API Changes

```rust
// Verify schema building is still valid
let schema = Schema::builder()
    .add_text_field("body", TEXT | STORED)
    .add_text_field("path", STORED)
    .add_u64_field("chunk_index", STORED)
    .add_u64_field("start_line", STORED)
    .add_u64_field("end_line", STORED)
    .build();
```

### Step 3: Update Query Parser

```rust
// Make sure query parser is compatible with v0.24
let query_parser = QueryParser::for_index(
    &self.index,
    vec![self.body_field],
);
```

## Task 3.2: Test Index Creation (1 hour)

### Create Test Index

```rust
#[test]
fn test_tantivy_index_creation() {
    use tempfile::tempdir;
    
    let dir = tempdir().unwrap();
    let index_path = dir.path();
    
    // Create index
    let tantivy_search = TantivySearch::new(index_path).unwrap();
    
    // Add test document
    let doc = Document {
        content: "This is a test document".to_string(),
        path: "test.txt".to_string(),
        chunk_index: 0,
        start_line: 1,
        end_line: 1,
    };
    
    tantivy_search.add_document(doc).unwrap();
    tantivy_search.commit().unwrap();
    
    // Verify index exists
    assert!(index_path.join(".tantivy").exists());
}
```

### Run Test

```bash
cargo test --features tantivy test_tantivy_index_creation
```

## Task 3.3: Implement Fuzzy Search (2 hours)

### Add Fuzzy Query Support

```rust
// File: src/search/tantivy_search.rs

pub fn search_fuzzy(&self, query: &str, max_distance: u8) -> Result<Vec<SearchResult>> {
    let reader = self.index.reader()?;
    let searcher = reader.searcher();
    
    // Build fuzzy query
    let fuzzy_query = FuzzyTermQuery::new(
        Term::from_field_text(self.body_field, query),
        max_distance,  // Levenshtein distance
        true,  // With transpositions
    );
    
    // Execute search
    let top_docs = searcher.search(&fuzzy_query, &TopDocs::with_limit(100))?;
    
    // Convert to results
    let mut results = Vec::new();
    for (_score, doc_address) in top_docs {
        let doc = searcher.doc(doc_address)?;
        results.push(self.doc_to_result(doc)?);
    }
    
    Ok(results)
}
```

### Test Fuzzy Search

```rust
#[test]
fn test_fuzzy_search() {
    let mut tantivy = setup_test_index();
    
    // Add documents with typos
    tantivy.add_document("The quick brown fox").unwrap();
    tantivy.add_document("The quikc brown fox").unwrap();  // Typo
    tantivy.add_document("The quick browm fox").unwrap();  // Typo
    tantivy.commit().unwrap();
    
    // Search with correct spelling
    let results = tantivy.search_fuzzy("quick", 1).unwrap();
    assert_eq!(results.len(), 3);  // Should find all including typos
}
```

## Task 3.4: Integrate with Existing Index (2 hours)

### Check Existing Index Compatibility

```bash
# Check if existing .tantivy_index is compatible
ls -la .tantivy_index/
```

### Migration Strategy

```rust
// File: src/bin/tantivy_migrator.rs

fn migrate_index(old_path: &Path, new_path: &Path) -> Result<()> {
    println!("Migrating Tantivy index from {:?} to {:?}", old_path, new_path);
    
    // Try to open old index
    match Index::open_in_dir(old_path) {
        Ok(old_index) => {
            // Old index is compatible, just copy
            fs_extra::dir::copy(old_path, new_path, &CopyOptions::new())?;
            println!("Index migrated successfully");
        }
        Err(_) => {
            // Old index incompatible, rebuild
            println!("Old index incompatible, rebuilding...");
            rebuild_index(new_path)?;
        }
    }
    
    Ok(())
}
```

## Task 3.5: Performance Optimization (1 hour)

### Configure Optimal Settings

```rust
// File: src/search/tantivy_search.rs

impl TantivySearch {
    pub fn new_optimized(path: &Path) -> Result<Self> {
        let index = Index::builder()
            .schema(Self::build_schema())
            .settings(IndexSettings {
                docstore_compression: Compressor::Lz4,
                docstore_blocksize: 16384,
            })
            .create_in_dir(path)?;
            
        // Configure writer with optimal settings
        let writer = index.writer_with_num_threads(
            num_cpus::get(),  // Use all CPU cores
            50_000_000,  // 50MB buffer
        )?;
        
        Ok(Self {
            index,
            writer: Arc::new(Mutex::new(writer)),
            // ...
        })
    }
}
```

### Benchmark Performance

```rust
#[bench]
fn bench_tantivy_indexing(b: &mut Bencher) {
    let tantivy = setup_test_tantivy();
    let content = "The quick brown fox jumps over the lazy dog";
    
    b.iter(|| {
        tantivy.add_document(content).unwrap();
    });
}

#[bench]
fn bench_tantivy_search(b: &mut Bencher) {
    let tantivy = setup_indexed_tantivy();  // Pre-indexed with 10000 docs
    
    b.iter(|| {
        tantivy.search("quick brown", 10).unwrap();
    });
}
```

## Task 3.6: Integration Testing (1 hour)

### Test with Real Files

```rust
#[test]
fn test_tantivy_with_real_codebase() {
    let tantivy = TantivySearch::new(".tantivy_test").unwrap();
    
    // Index some real Rust files
    for entry in WalkDir::new("src").into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension() == Some(OsStr::new("rs")) {
            let content = fs::read_to_string(entry.path()).unwrap();
            tantivy.add_document(Document {
                content,
                path: entry.path().to_string_lossy().to_string(),
                // ...
            }).unwrap();
        }
    }
    
    tantivy.commit().unwrap();
    
    // Search for Rust-specific terms
    let results = tantivy.search("impl trait", 10).unwrap();
    assert!(!results.is_empty());
    
    // Test fuzzy search
    let fuzzy_results = tantivy.search_fuzzy("impll", 1).unwrap();
    assert!(!fuzzy_results.is_empty());
}
```

## Success Criteria

- [ ] Tantivy compiles without errors
- [ ] Can create new indexes
- [ ] Can add documents to index
- [ ] Basic search returns results
- [ ] Fuzzy search works with Levenshtein distance
- [ ] Existing index (if any) is migrated or rebuilt
- [ ] Performance meets targets (<5ms search time)
- [ ] Integration with codebase files works

## Performance Targets

- Indexing: <10ms per document
- Search: <5ms for 10,000 documents
- Fuzzy search: <20ms for 10,000 documents
- Memory: <500MB for 100,000 documents

## Common Issues and Solutions

1. **Index corruption**: Delete `.tantivy_index` and rebuild
2. **Memory issues**: Reduce writer buffer size
3. **Slow search**: Add more RAM to searcher
4. **Fuzzy too broad**: Reduce max_distance parameter

## Next Phase

Proceed to Phase 4 (ML/Vector Overhaul) only after Tantivy search works correctly.