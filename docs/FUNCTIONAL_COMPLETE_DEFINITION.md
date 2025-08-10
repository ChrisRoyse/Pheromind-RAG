# Definition of "Functional and Complete" Code Search System

## Based on Real Production Implementations

### Core Requirements (Non-Negotiable)

#### 1. **All 5 Technologies Must Work Together**
- ✅ **BM25**: Native Tantivy implementation with correct IDF calculation (K1=1.2, B=0.75)
- ✅ **Tantivy**: Full-text indexing with configurable tokenizers (en_stem, etc.)
- ✅ **LanceDB**: Vector storage with Arrow schema, cosine similarity search
- ✅ **Nomic Embed v1**: 768-dim embeddings with "passage:" and "query:" prefixes
- ✅ **Tree-sitter**: AST-based symbol extraction AND semantic chunking

#### 2. **Performance Targets (From Production Systems)**
- Search latency: < 100ms for hybrid search
- Indexing speed: ~500 files/second
- Memory usage: < 500MB for 50k files
- Index size: ~30% of original codebase size

#### 3. **Functional Features Required**

##### **Indexing Pipeline**
```
Code Files → Tree-sitter Parse → Semantic Chunks → 
           ↓                                      ↓
    Symbol Extraction                    Content Chunks (1500 chars)
           ↓                                      ↓
    Symbol Index                         Nomic Embeddings
           ↓                                      ↓
    Tantivy Index                       LanceDB Vectors
```

##### **Search Pipeline**
```
Query → Parse & Expand →
      ↓
   ┌──────────────────┬────────────────┬──────────────┐
   ↓                  ↓                ↓              ↓
BM25 Search    Vector Search    Symbol Search    Fuzzy Search
   ↓                  ↓                ↓              ↓
   └──────────────────┴────────────────┴──────────────┘
                           ↓
                    Hybrid Fusion (RRF)
                           ↓
                    Reranking & Results
```

#### 4. **Integration Architecture**

##### **Correct Integration Pattern**
```rust
pub struct UnifiedSearchEngine {
    // Tantivy for text search (NOT built into LanceDB)
    tantivy_index: Index,
    tantivy_writer: IndexWriter,
    
    // LanceDB for vector search (separate system)
    vector_db: Connection,
    vector_table: Table,
    
    // Nomic embedder with correct prefixes
    embedder: NomicEmbedder, // Uses "passage:" and "query:"
    
    // Tree-sitter for BOTH parsing and chunking
    symbol_extractor: SymbolExtractor,
    semantic_chunker: SemanticChunker,
    
    // BM25 engine (can use Tantivy's or custom)
    bm25_scorer: BM25Scorer,
}
```

##### **Hybrid Fusion Algorithm**
```rust
// NOT arbitrary 70/30 - use configurable weights
pub struct FusionConfig {
    bm25_weight: f32,      // Default: 0.25
    vector_weight: f32,    // Default: 0.40
    symbol_weight: f32,    // Default: 0.25
    fuzzy_weight: f32,     // Default: 0.10
}

// RRF fusion with configurable k parameter
fn reciprocal_rank_fusion(results: Vec<SearchResult>, k: f32 = 60.0) -> Vec<FusedResult> {
    // score = Σ(1 / (k + rank_i))
}
```

#### 5. **Code Quality Standards**

##### **Must Compile and Run**
```bash
cargo check              # ✅ Zero errors
cargo test              # ✅ All tests pass
cargo run --bin search  # ✅ Executes successfully
cargo bench             # ✅ Meets performance targets
```

##### **Error Handling**
- No `unwrap()` in production code
- Proper error propagation with `?`
- Graceful fallbacks for component failures
- Comprehensive logging and metrics

##### **Testing Requirements**
- Unit tests for each component
- Integration tests for full pipeline
- Performance benchmarks with real data
- Edge case handling (empty queries, large files, etc.)

### What Makes It "Complete"

#### 1. **Semantic Code Understanding**
- Extract functions, classes, methods, variables
- Understand import relationships
- Track symbol references
- Preserve code structure in chunks

#### 2. **Multi-Modal Search**
- Exact match for identifiers
- Semantic search for concepts
- Keyword search for text
- Symbol navigation for code structure

#### 3. **Production Ready**
- Configuration management (TOML/JSON)
- Incremental indexing
- Cache management
- Memory monitoring
- Concurrent processing

#### 4. **Language Support**
Minimum viable languages:
- Rust
- Python
- JavaScript/TypeScript
- Go
- Java

### Architecture Components

#### 1. **Chunking Strategy**
```rust
pub struct SemanticChunk {
    content: String,
    file_path: PathBuf,
    start_line: usize,
    end_line: usize,
    symbols: Vec<Symbol>,
    chunk_type: ChunkType, // Function, Class, Module
    parent_context: Option<String>,
}
```

#### 2. **Storage Schema**
```rust
// LanceDB schema
pub struct CodeDocument {
    id: String,
    file_path: String,
    content: String,
    embedding: Vector<768>,
    symbols: Vec<String>,
    language: String,
    metadata: Metadata,
}

// Tantivy schema
schema_builder
    .add_text_field("content", TEXT | STORED)
    .add_text_field("symbols", TEXT)
    .add_u64_field("line_number", INDEXED | STORED)
    .add_facet_field("language", INDEXED);
```

#### 3. **Query Processing**
```rust
pub struct QueryProcessor {
    fn process(query: &str) -> ProcessedQuery {
        // 1. Detect query type (code, natural language, symbol)
        // 2. Extract keywords and identifiers
        // 3. Generate embedding with correct prefix
        // 4. Expand with synonyms/related terms
        // 5. Build sub-queries for each search type
    }
}
```

### Validation Criteria (100/100 Score)

#### Functionality (40 points)
- [ ] All 5 technologies integrated and working (10)
- [ ] Hybrid search returns relevant results (10)
- [ ] Symbol extraction works correctly (10)
- [ ] Incremental indexing functions (10)

#### Performance (30 points)
- [ ] Search latency < 100ms (10)
- [ ] Indexing speed > 100 files/sec (10)
- [ ] Memory usage < 500MB for 50k files (10)

#### Code Quality (20 points)
- [ ] Zero compilation errors (5)
- [ ] All tests pass (5)
- [ ] No unwrap() in production code (5)
- [ ] Comprehensive error handling (5)

#### Completeness (10 points)
- [ ] Configuration management (2)
- [ ] Multi-language support (3)
- [ ] Documentation (2)
- [ ] Benchmarks (3)

### Non-Functional Requirements

1. **Maintainability**
   - Modular architecture
   - Clear separation of concerns
   - Well-documented interfaces

2. **Scalability**
   - Handle codebases up to 1M files
   - Concurrent indexing
   - Streaming results

3. **Reliability**
   - Graceful degradation
   - Recovery from crashes
   - Data consistency

### Implementation Priority

1. **Phase 1: Core Integration** (Week 1)
   - Get all 5 technologies compiling together
   - Basic indexing pipeline
   - Simple search functionality

2. **Phase 2: Optimization** (Week 2)
   - Performance tuning
   - Proper chunking strategies
   - Fusion algorithm refinement

3. **Phase 3: Production Hardening** (Week 3)
   - Error handling
   - Configuration management
   - Testing and benchmarks

This definition is based on actual production systems and represents the minimum viable "functional and complete" implementation.