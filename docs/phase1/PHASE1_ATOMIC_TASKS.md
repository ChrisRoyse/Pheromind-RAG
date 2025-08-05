# Phase 1: Regex + Embeddings Foundation - Atomic Tasks

## Overview
Phase 1 delivers the core foundation for 85% search accuracy through:
- **3-Chunk Context**: The single most impactful feature (55% accuracy gain)
- **MiniLM Embeddings**: Fast, proven all-MiniLM-L6-v2 model
- **Regex Chunking**: Simple, effective code boundaries
- **LanceDB Storage**: Vector database for semantic search

**Total Tasks**: 10 (each ~15 minutes)
**Approach**: TDD with pragmatic Red-Green-Refactor cycles

## Task 001: Regex Chunker Core (15 min)
**RED**: Write test for `SimpleRegexChunker` that chunks code by patterns and size
**GREEN**: Implement chunker with function/class patterns, ~100 line target
**REFACTOR**: Extract patterns to constants, add boundary detection method
**Deliverable**: Working chunker that splits code intelligently

## Task 002: Chunk Structure & Line Tracking (15 min)
**RED**: Test that chunks track start/end line numbers accurately
**GREEN**: Add `Chunk` struct with content, start_line, end_line fields
**REFACTOR**: Ensure chunks maintain proper line number continuity
**Deliverable**: Chunks with accurate line tracking for search results

## Task 003: Three-Chunk Context Expander (15 min)
**RED**: Test `ThreeChunkExpander` returns above/target/below chunks
**GREEN**: Implement expand() method handling edge cases (first/last chunks)
**REFACTOR**: Add display formatting for clear context presentation
**Deliverable**: **Critical 55% accuracy feature working**

## Task 004: MiniLM Embedder Setup (15 min)
**RED**: Test `MiniLMEmbedder` loads model and produces 384-dim embeddings
**GREEN**: Implement embedder with all-MiniLM-L6-v2, mock if model not available
**REFACTOR**: Add batch embedding support and normalization
**Deliverable**: Embedder ready for semantic search

## Task 005: LanceDB Vector Storage (15 min)
**RED**: Test `VectorStorage` schema includes all required fields
**GREEN**: Create schema with file_path, chunk_index, embedding, content, lines
**REFACTOR**: Add insert_embedding() and search preparation
**Deliverable**: Vector database ready for embeddings

## Task 006: Ripgrep Text Search (15 min)
**RED**: Test `RipgrepSearcher` finds exact text matches
**GREEN**: Implement search() using ripgrep --json output
**REFACTOR**: Parse results into Match structs with file/line info
**Deliverable**: Fast exact text search capability

## Task 007: Simple Indexing Pipeline (15 min)
**RED**: Test indexing a single file end-to-end
**GREEN**: Connect chunker → embedder → storage for one file
**REFACTOR**: Add error handling and progress tracking
**Deliverable**: Can index individual files

## Task 008: Basic Search Integration (15 min)
**RED**: Test `SimpleSearcher` returns results with 3-chunk context
**GREEN**: Wire ripgrep results through chunk expansion
**REFACTOR**: Add SearchResult struct with score and context
**Deliverable**: Basic search returning contextualized results

## Task 009: Integration Testing (15 min)
**RED**: Write end-to-end test indexing and searching small codebase
**GREEN**: Fix integration issues, ensure all components work together
**REFACTOR**: Add test fixtures and performance assertions
**Deliverable**: Validated Phase 1 functionality

## Task 010: Performance & Polish (15 min)
**RED**: Test performance targets (<50ms chunk, <100ms embed, <500ms search)
**GREEN**: Basic optimizations to meet targets
**REFACTOR**: Code cleanup, documentation, prepare for Phase 2
**Deliverable**: Phase 1 complete and ready for fusion layer

## Success Criteria Checklist
- [ ] Regex chunking splits code at ~100 lines
- [ ] Function/class boundaries detected
- [ ] **3-chunk context always returned** (55% accuracy gain)
- [ ] MiniLM embeddings are 384 dimensions
- [ ] LanceDB stores vectors with metadata
- [ ] Ripgrep provides exact matches
- [ ] Search results include full context
- [ ] Performance: <50ms chunk, <100ms embed
- [ ] Memory usage < 1GB
- [ ] All tests passing

## Key Implementation Notes

### Critical Path
The 3-chunk context is the **most important feature** - it alone provides 55% accuracy improvement. Prioritize getting this working correctly over perfect code structure.

### Pragmatic TDD
- RED: Write minimal test that captures the requirement
- GREEN: Implement just enough to pass - use mocks/stubs where needed
- REFACTOR: Only refactor if it helps deliver functionality faster

### File Structure
```
src/
  chunking/
    regex_chunker.rs    # Tasks 001-002
    three_chunk.rs      # Task 003
  embedding/
    minilm.rs          # Task 004
  storage/
    lancedb.rs         # Task 005
  search/
    ripgrep.rs         # Task 006
    searcher.rs        # Task 008
  lib.rs               # Integration
tests/
  integration_test.rs  # Task 009
```

### Dependencies
- regex = "1.10"
- candle = "0.3" (or mock embeddings initially)
- lance = "0.10"
- serde_json = "1.0"
- tokio = { version = "1.35", features = ["full"] }

## Phase 1 Completion
After these 10 tasks, you'll have:
1. Working regex chunker with smart boundaries
2. 3-chunk context expansion (the key feature!)
3. MiniLM embeddings ready
4. Vector storage initialized
5. Basic search with exact matching
6. Foundation for 85% search accuracy

**Next**: Phase 2 adds semantic search and simple fusion