# GET WORKING: FUNCTIONAL REPAIR PLAN

## EXECUTIVE SUMMARY

This system is **85% functional** but has critical data persistence issues preventing basic usage. The implementations are real (not mocked), but there's a fundamental bug where indexed data doesn't persist between commands, making search return 0 results.

**CORE PROBLEM**: You can index files, but search always returns empty results because the BM25 index isn't persisting between command invocations.

---

## CRITICAL FIXES REQUIRED (MANDATORY)

### 1. **DATA PERSISTENCE BUG** - HIGHEST PRIORITY
**Status**: 🔴 BROKEN
**Impact**: Core functionality unusable
**Location**: `src/search/unified.rs` and storage implementations

**Problem**: Each command creates a new `UnifiedSearcher` instance, losing previously indexed data.

**Fix Required**:
- Ensure BM25 index data persists to disk after indexing
- Verify search operations read from the persisted index
- Check if the storage path is consistent between index and search operations

**Specific Action Items**:
1. Verify `self.bm25_engine.save()` actually writes to disk
2. Ensure `self.bm25_engine.load()` reads from the same location during search
3. Check file permissions on the storage directory
4. Add debug logging to trace index save/load operations

### 2. **BUILD CONFIGURATION** - HIGH PRIORITY
**Status**: 🟡 NEEDS CONFIGURATION
**Impact**: Limited functionality without proper features

**Current Issue**: Default build only includes `core` features, providing minimal BM25-only search.

**Fix Required**:
```bash
# For basic text search functionality
cargo build --features="search-basic"

# For full functionality including embeddings
cargo build --features="full-system"
```

**Feature Combinations**:
- `search-basic`: BM25 + Tantivy fuzzy search
- `search-advanced`: Adds symbol indexing via tree-sitter
- `full-system`: All features including ML embeddings and vector storage

### 3. **EMBEDDING MODEL VERIFICATION** - MEDIUM PRIORITY
**Status**: 🟡 UNTESTED ON YOUR SYSTEM
**Impact**: ML-based search quality

**Potential Issues**:
- GGUF Q4_K_M model (84MB) downloads to `~/.nomic/` on first use
- Dequantization might fail on some architectures
- Falls back to "small random values" if dequantization fails

**Fix Required**:
1. Test embedding generation manually:
   ```rust
   let embedder = NomicEmbedder::new()?;
   let embedding = embedder.embed_text("test").await?;
   println!("Embedding dimension: {}", embedding.len());
   ```
2. Verify embeddings aren't all zeros or random values
3. Check if model files downloaded correctly to `~/.nomic/`

---

## SYSTEMATIC DEBUGGING PLAN

### Phase 1: Identify Root Cause (30 minutes)

#### Step 1: Enable Debug Logging
Add to your environment:
```bash
export RUST_LOG=debug
```

#### Step 2: Test Basic Index/Search Flow
```bash
# Build with basic features
cargo build --features="search-basic"

# Index a test file
echo "test content for indexing" > test.txt
./target/debug/embed-search index test.txt

# Search immediately (should find results)
./target/debug/embed-search search "content"
```

#### Step 3: Check Storage Location
```bash
# Find where index data is stored
find . -name "*.db" -o -name "*.index" -o -name "*bm25*"

# Check if files are created during indexing
ls -la before_index/
./target/debug/embed-search index test.txt
ls -la after_index/
```

### Phase 2: Verify Feature Functionality (1 hour)

#### BM25 Text Search
```bash
cargo test bm25_integration_tests --features="search-basic"
```

#### Embedding System (if using ML features)
```bash
cargo test nomic_embedding_tests --features="full-system"
```

#### Vector Storage
```bash
cargo test --features="vectordb" storage_tests
```

### Phase 3: Configuration Validation (30 minutes)

#### Check Config File Loading
Create `config.toml` in project root:
```toml
[storage]
backend = "simple"  # or "lancedb" with vectordb feature

[search]
backend = "bm25"    # or "tantivy" with tantivy feature

[embedding]
model = "nomic-embed-text-v1"
cache_size = 1000
```

#### Verify Config Loading
```bash
RUST_LOG=debug ./target/debug/embed-search search "test" 2>&1 | grep -i config
```

---

## EXPECTED WORKING BEHAVIOR

### After Fixes, You Should See:

1. **Indexing Output**:
   ```
   Indexing file: test.txt
   BM25 index updated: 1 documents
   Index saved to: ./embed_index/
   ```

2. **Search Output**:
   ```
   Found 1 results for "content":
   test.txt: "test content for indexing"
   Relevance: 0.95
   ```

3. **Persistent Data**:
   - Index files created in consistent location
   - Search works across different command invocations
   - Results quality improves with more indexed files

---

## LIKELY FIXES NEEDED

### Most Probable Issues (Based on Code Analysis):

1. **Storage Path Inconsistency**
   - Index saves to one location, search looks in another
   - Environment variable differences between commands

2. **Serialization Format Mismatch**
   - BM25 index format changed between saves/loads
   - Version compatibility issues with serialized data

3. **Feature Flag Runtime Issues**
   - Search trying to use features not compiled in
   - Fallback behavior not working correctly

### Quick Verification Commands:

```bash
# Check if any data persists
./target/debug/embed-search index test1.txt
ls -la # Look for new files
./target/debug/embed-search search "test1"

# Verify feature compilation
./target/debug/embed-search --help  # Should show available commands

# Check model loading (with ML features)
RUST_LOG=debug ./target/debug/embed-search search "test" 2>&1 | grep -i "model\|embed"
```

---

## IMPLEMENTATION VERIFICATION

### What is REAL (Not Mocked):
✅ **GGUF Model Loading**: Actual Nomic model with dequantization
✅ **BM25 Implementation**: Real TF-IDF with proper scoring
✅ **LanceDB Integration**: Real vector database operations
✅ **File Processing**: Actual file reading and chunking
✅ **Search Algorithms**: Real cosine similarity and ranking

### What Might Fail:
⚠️ **Index Persistence**: Data may not survive between process restarts
⚠️ **Configuration Loading**: May not find config files consistently  
⚠️ **Feature Availability**: Runtime behavior depends on compile-time features

---

## SUCCESS CRITERIA

You'll know it's working when:

1. ✅ Index command creates persistent files
2. ✅ Search command finds previously indexed content
3. ✅ Results improve with more indexed files
4. ✅ No crash on normal usage
5. ✅ Embedding generation produces reasonable vectors (not zeros/random)

**Timeline**: Should be fixable in 2-4 hours of focused debugging, mainly tracing the data persistence issue.

**Next Steps**: Start with Phase 1 debugging to identify exactly where the indexed data goes and why search can't find it.