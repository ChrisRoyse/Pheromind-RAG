# Embed Search System Architecture - Deep Dive

## System Design Philosophy
- **Modular Components**: Clean separation between search engines
- **Feature-Gated Compilation**: Optional ML/vector capabilities
- **Parallel Execution**: All search methods run concurrently
- **Truth Above All**: Radical candor in error reporting

## Core Components

### 1. Search Adapters (`src/search/search_adapter.rs`)
```rust
pub trait SearchAdapter: Send + Sync {
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
    async fn index_files(&mut self, files: Vec<String>) -> Result<()>;
    async fn update_file(&mut self, file_path: &str, content: &str) -> Result<()>;
}
```

### 2. BM25 Engine (`src/search/bm25.rs`)
- **Algorithm**: Okapi BM25 with k1=1.2, b=0.75
- **IDF Formula**: `ln((N - df + 0.5) / (df + 0.5) + 1.0)`
- **Features**: 
  - Incremental updates without full rebuild
  - Negative IDF handling for common terms
  - Document frequency caching
  - Thread-safe with RwLock

### 3. Tantivy Search (`src/search/tantivy_search.rs`)
- **Schema**: Line-by-line indexing with metadata
- **Fuzzy Logic**: 
  - CamelCase/snake_case splitting
  - Compound word detection
  - Edit distance matching
- **Index Management**:
  - Persistent storage with schema validation
  - Automatic rebuilding on corruption
  - Project-scoped isolation

### 4. Symbol Indexing (`src/search/symbol_index.rs`)
- **Tree-Sitter Integration**: 10+ language support
- **Symbol Types**: Functions, classes, methods, variables
- **Storage**: Triple HashMap (by name, file, kind)
- **Query Detection**: Smart symbol vs text classification

### 5. Vector Search (`src/storage/lancedb_storage.rs`)
- **Embedding Model**: Nomic Embed Text v1.5
- **Indexing**: IVF-PQ (256 partitions, 16 sub-vectors)
- **Features**:
  - Batch operations with atomic commits
  - Checksum validation
  - Corruption recovery
  - Similarity threshold filtering

### 6. Unified Fusion (`src/search/unified.rs`)
- **Parallel Execution**: Tokio async for all engines
- **Score Normalization**: Percentile-based (95th percentile)
- **Ranking Factors**:
  - Content relevance
  - Filename matches
  - Directory structure
  - Test file penalties
- **Deduplication**: Path-based with score aggregation

## Data Flow Architecture

```
User Query
    ↓
UnifiedSearcher
    ↓
┌─────────────────────────────────┐
│  Parallel Execution (Tokio)     │
├──────┬──────┬──────┬───────────┤
│ BM25 │Tantivy│Symbol│ Vector    │
└──────┴──────┴──────┴───────────┘
    ↓        ↓        ↓        ↓
Results  Results  Results  Results
    ↓        ↓        ↓        ↓
┌─────────────────────────────────┐
│    Fusion & Score Normalization │
└─────────────────────────────────┘
    ↓
Deduplicated Results
    ↓
Ranked Output
```

## Watcher System Architecture

### File System Layer
```
File Change Event
    ↓
GitWatcher (notify crate)
    ↓
Debouncer (500ms)
    ↓
Edge Case Validation
    ↓
Event Queue
    ↓
IndexUpdater
    ↓
Incremental Updates
```

### MCP Protocol Layer
```
IDE/Client
    ↓
MCP Server (TypeScript)
    ↓
JSON-RPC 2.0
    ↓
Engine Interface
    ↓
[Missing: Native Bridge]
    ↓
Rust Search System
```

## Memory Architecture

### Caching Layers
1. **Embedding Cache**: LRU with SHA-256 keys
2. **BM25 Term Cache**: Document frequency cache
3. **Symbol Cache**: Parsed symbol database
4. **Search Result Cache**: Query-keyed results

### Storage Patterns
- **Read Path**: Cache → Memory → Disk → Network
- **Write Path**: Memory → Cache → Disk (async)
- **Invalidation**: File-based with timestamp tracking

## Error Handling Philosophy

### Principle 0: Radical Candor
```rust
// Never hide failures
if let Err(e) = operation {
    // Full error chain with context
    return Err(anyhow!("Operation failed: {:#}", e));
}
```

### Error Categories
- **E1xxx**: File system errors
- **E2xxx**: Parser errors
- **E3xxx**: Network errors
- **E4xxx**: Storage errors
- **E5xxx**: Configuration errors

## Performance Characteristics

### Latency Targets
- Native search: <10ms
- BM25 ranking: <50ms
- Tantivy fuzzy: <100ms
- Vector search: <200ms
- Unified fusion: <250ms total

### Concurrency Model
- **Search**: Parallel execution with join
- **Indexing**: Batch processing (10 files/batch)
- **Watching**: Async event processing
- **Updates**: Incremental with locks

## Technical Debt Areas

### High Priority
1. **UnifiedSearcher Size**: 1063 lines (violates 500-line rule)
2. **Duplicate BM25**: Two implementations need merging
3. **MCP Bridge**: Missing native binding

### Medium Priority
1. **Feature Flag Complexity**: Excessive conditional compilation
2. **Lock Contention**: Multiple Arc<RwLock> patterns
3. **Magic Numbers**: Hard-coded thresholds need config

### Low Priority
1. **Test Placeholders**: 70% of stress tests incomplete
2. **Documentation**: API docs need updates
3. **Benchmarks**: Missing performance baselines