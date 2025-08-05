# Phase 2: Search & Simple Fusion - Atomic Micro Tasks Breakdown

## Overview
Phase 2 implements unified search with simple fusion of exact and semantic results. Each atomic task is designed to take no more than 15 minutes following TDD (Red-Green-Refactor) methodology.

**Total Timeline**: Week 2 (Tasks 011-020)  
**Goal**: Achieve 70%+ search accuracy with <500ms latency  
**Approach**: TDD with comprehensive test coverage

---

## Task 011: Vector Similarity Search (3 hours = 12 micro tasks)

### 011.1: Setup Vector Search Test Infrastructure
- Create test module for vector search
- Write failing test for basic similarity search
- **Time**: 15 min
- **TDD**: Red - test fails, no implementation

### 011.2: Create VectorStorage Search Interface
- Define search_similar method signature
- Add Result and error types
- **Time**: 15 min
- **TDD**: Red - compilation passes, test still fails

### 011.3: Implement Basic Query Vector Creation
- Convert query embedding to LanceDB format
- Test vector dimensions match
- **Time**: 15 min
- **TDD**: Green - basic vector creation works

### 011.4: Add LanceDB ANN Search Call
- Implement dataset.scan().nearest() call
- Handle limit parameter
- **Time**: 15 min
- **TDD**: Green - search executes

### 011.5: Parse Search Results from LanceDB
- Extract file_path, chunk_index, content
- Handle batch iteration
- **Time**: 15 min
- **TDD**: Green - results parsed

### 011.6: Calculate Similarity Scores
- Convert distance to similarity (1.0 - distance)
- Normalize scores
- **Time**: 15 min
- **TDD**: Green - scores calculated

### 011.7: Create SimilarityMatch Structs
- Build match objects with all fields
- Sort by similarity descending
- **Time**: 15 min
- **TDD**: Green - matches created

### 011.8: Add Error Handling
- Handle missing dataset
- Handle malformed queries
- **Time**: 15 min
- **TDD**: Refactor - improve robustness

### 011.9: Optimize Search Performance
- Add query caching consideration
- Batch result processing
- **Time**: 15 min
- **TDD**: Refactor - optimize

### 011.10: Add Search Logging
- Log query and result count
- Add timing metrics
- **Time**: 15 min
- **TDD**: Refactor - add observability

### 011.11: Write Integration Tests
- Test with real embeddings
- Test edge cases (empty results, large limits)
- **Time**: 15 min
- **TDD**: Validate all paths

### 011.12: Document Vector Search API
- Add inline documentation
- Create usage examples
- **Time**: 15 min
- **TDD**: Complete documentation

---

## Task 012: Simple Fusion Algorithm (3 hours = 12 micro tasks)

### 012.1: Create SimpleFusion Test Suite
- Setup test module
- Write failing test for basic fusion
- **Time**: 15 min
- **TDD**: Red - no implementation

### 012.2: Define Fusion Types and Interfaces
- Create MatchType enum (Exact, Semantic)
- Define FusedResult struct
- **Time**: 15 min
- **TDD**: Red - types defined

### 012.3: Implement Deduplication Logic
- Create HashSet for seen items
- Define deduplication key (file + location)
- **Time**: 15 min
- **TDD**: Green - dedup works

### 012.4: Process Exact Matches First
- Iterate exact matches
- Assign score 1.0
- **Time**: 15 min
- **TDD**: Green - exact processed

### 012.5: Add Exact Matches to Results
- Create FusedResult objects
- Add to results vector
- **Time**: 15 min
- **TDD**: Green - exact added

### 012.6: Process Semantic Matches
- Check for existing exact matches
- Apply 0.8x score multiplier
- **Time**: 15 min
- **TDD**: Green - semantic processed

### 012.7: Implement Score-Based Sorting
- Sort results by score descending
- Handle score ties
- **Time**: 15 min
- **TDD**: Green - sorting works

### 012.8: Add Result Truncation
- Limit to top 20 results
- Make limit configurable
- **Time**: 15 min
- **TDD**: Refactor - add limits

### 012.9: Handle Edge Cases
- Empty input handling
- All duplicates scenario
- **Time**: 15 min
- **TDD**: Refactor - handle edges

### 012.10: Add Fusion Metrics
- Track deduplication stats
- Log fusion performance
- **Time**: 15 min
- **TDD**: Refactor - add metrics

### 012.11: Write Comprehensive Tests
- Test various match combinations
- Verify score ordering
- **Time**: 15 min
- **TDD**: Validate fusion logic

### 012.12: Document Fusion Algorithm
- Explain scoring strategy
- Add examples
- **Time**: 15 min
- **TDD**: Complete docs

---

## Task 013: Complete Search Pipeline (4 hours = 16 micro tasks)

### 013.1: Create UnifiedSearcher Test Framework
- Setup comprehensive test suite
- Write failing integration test
- **Time**: 15 min
- **TDD**: Red - no implementation

### 013.2: Define UnifiedSearcher Struct
- Add all component dependencies
- Create constructor
- **Time**: 15 min
- **TDD**: Red - struct defined

### 013.3: Implement Ripgrep Integration
- Call ripgrep.search()
- Handle project path
- **Time**: 15 min
- **TDD**: Green - exact search works

### 013.4: Add Query Embedding Generation
- Call embedder.embed()
- Handle embedding errors
- **Time**: 15 min
- **TDD**: Green - embeddings work

### 013.5: Integrate Vector Search
- Call storage.search_similar()
- Configure result limit
- **Time**: 15 min
- **TDD**: Green - semantic works

### 013.6: Wire in Fusion Component
- Call fusion.fuse_results()
- Pass both result sets
- **Time**: 15 min
- **TDD**: Green - fusion integrated

### 013.7: Implement File Content Reading
- Read files for chunk expansion
- Handle missing files
- **Time**: 15 min
- **TDD**: Green - files read

### 013.8: Add Chunk Finding Logic
- Find chunk for line numbers (exact)
- Use chunk_index (semantic)
- **Time**: 15 min
- **TDD**: Green - chunks found

### 013.9: Integrate 3-Chunk Expander
- Call expander.expand()
- Build three-chunk contexts
- **Time**: 15 min
- **TDD**: Green - expansion works

### 013.10: Create SearchResult Objects
- Populate all fields
- Include match type
- **Time**: 15 min
- **TDD**: Green - results created

### 013.11: Add Async/Await Support
- Make search method async
- Handle async errors
- **Time**: 15 min
- **TDD**: Refactor - async support

### 013.12: Implement Timeout Handling
- Add configurable timeout
- Cancel long-running searches
- **Time**: 15 min
- **TDD**: Refactor - timeouts

### 013.13: Add Search Cancellation
- Support cancellation tokens
- Clean up on cancel
- **Time**: 15 min
- **TDD**: Refactor - cancellation

### 013.14: Write End-to-End Tests
- Test full pipeline
- Verify result quality
- **Time**: 15 min
- **TDD**: Validate pipeline

### 013.15: Add Performance Benchmarks
- Measure search latency
- Profile bottlenecks
- **Time**: 15 min
- **TDD**: Optimize performance

### 013.16: Document Search API
- Complete API documentation
- Add usage examples
- **Time**: 15 min
- **TDD**: Complete docs

---

## Task 014: Result Ranking Optimization (2 hours = 8 micro tasks)

### 014.1: Create Ranking Test Suite
- Setup ranking tests
- Define quality metrics
- **Time**: 15 min
- **TDD**: Red - no ranking

### 014.2: Implement Query-in-Target Boost
- Check if query appears in target chunk
- Apply 1.2x multiplier
- **Time**: 15 min
- **TDD**: Green - boost works

### 014.3: Add Beginning-of-Chunk Boost
- Check first 3 lines for query
- Apply 1.1x multiplier
- **Time**: 15 min
- **TDD**: Green - position boost

### 014.4: Implement Large Chunk Penalty
- Count chunk lines
- Apply 0.95x for >150 lines
- **Time**: 15 min
- **TDD**: Green - penalty applied

### 014.5: Add Re-sorting Logic
- Sort after score adjustments
- Maintain stable sort
- **Time**: 15 min
- **TDD**: Green - resorting works

### 014.6: Test Ranking Improvements
- Verify boost effects
- Check edge cases
- **Time**: 15 min
- **TDD**: Validate ranking

### 014.7: Add Ranking Configuration
- Make multipliers configurable
- Add ranking profiles
- **Time**: 15 min
- **TDD**: Refactor - configurable

### 014.8: Document Ranking Strategy
- Explain heuristics
- Provide tuning guide
- **Time**: 15 min
- **TDD**: Complete docs

---

## Task 015: Parallel Search Execution (2 hours = 8 micro tasks)

### 015.1: Create Parallel Execution Tests
- Setup concurrency tests
- Write failing test
- **Time**: 15 min
- **TDD**: Red - sequential only

### 015.2: Add Tokio Spawn for Exact Search
- Wrap ripgrep in spawn
- Return JoinHandle
- **Time**: 15 min
- **TDD**: Green - exact parallel

### 015.3: Add Tokio Spawn for Semantic Search
- Wrap embedding + vector search
- Return JoinHandle
- **Time**: 15 min
- **TDD**: Green - semantic parallel

### 015.4: Implement Join Logic
- Use tokio::join! macro
- Handle both results
- **Time**: 15 min
- **TDD**: Green - parallel works

### 015.5: Add Error Propagation
- Handle spawn errors
- Propagate search errors
- **Time**: 15 min
- **TDD**: Refactor - errors handled

### 015.6: Measure Parallel Performance
- Compare vs sequential
- Log timing improvements
- **Time**: 15 min
- **TDD**: Validate speedup

### 015.7: Add Resource Limits
- Limit concurrent tasks
- Add backpressure
- **Time**: 15 min
- **TDD**: Refactor - resource limits

### 015.8: Document Parallel Strategy
- Explain concurrency model
- Add performance notes
- **Time**: 15 min
- **TDD**: Complete docs

---

## Task 016: Search Result Caching (2 hours = 8 micro tasks)

### 016.1: Create Cache Test Suite
- Setup LRU cache tests
- Write failing test
- **Time**: 15 min
- **TDD**: Red - no cache

### 016.2: Implement SearchCache Struct
- Add LRU dependency
- Create with Mutex
- **Time**: 15 min
- **TDD**: Green - cache created

### 016.3: Implement Cache Get Method
- Lock mutex safely
- Clone results on hit
- **Time**: 15 min
- **TDD**: Green - get works

### 016.4: Implement Cache Insert Method
- Lock mutex safely
- Handle capacity limits
- **Time**: 15 min
- **TDD**: Green - insert works

### 016.5: Integrate Cache in Search Pipeline
- Check cache before search
- Store results after search
- **Time**: 15 min
- **TDD**: Green - cache integrated

### 016.6: Add Cache Invalidation
- Clear on file changes
- Selective invalidation
- **Time**: 15 min
- **TDD**: Refactor - invalidation

### 016.7: Add Cache Metrics
- Track hit/miss ratio
- Log cache performance
- **Time**: 15 min
- **TDD**: Refactor - metrics

### 016.8: Document Cache Strategy
- Explain cache benefits
- Configuration guide
- **Time**: 15 min
- **TDD**: Complete docs

---

## Task 017: Query Preprocessing (2 hours = 8 micro tasks)

### 017.1: Create Preprocessor Test Suite
- Setup preprocessing tests
- Write failing test
- **Time**: 15 min
- **TDD**: Red - no preprocessing

### 017.2: Implement Basic Normalization
- Convert to lowercase
- Trim whitespace
- **Time**: 15 min
- **TDD**: Green - basic works

### 017.3: Add Noise Word Removal
- Define noise word list
- Remove from query
- **Time**: 15 min
- **TDD**: Green - noise removed

### 017.4: Implement Whitespace Normalization
- Split and rejoin
- Handle multiple spaces
- **Time**: 15 min
- **TDD**: Green - whitespace fixed

### 017.5: Add Abbreviation Expansion
- Expand common abbreviations
- Make configurable
- **Time**: 15 min
- **TDD**: Green - expansion works

### 017.6: Test Edge Cases
- Empty queries
- All noise words
- **Time**: 15 min
- **TDD**: Refactor - handle edges

### 017.7: Add Language Detection
- Detect programming terms
- Preserve case when needed
- **Time**: 15 min
- **TDD**: Refactor - smarter processing

### 017.8: Document Preprocessing
- List all transformations
- Provide examples
- **Time**: 15 min
- **TDD**: Complete docs

---

## Task 018: Search Metrics (2 hours = 8 micro tasks)

### 018.1: Create Metrics Test Framework
- Setup metrics tests
- Define metric types
- **Time**: 15 min
- **TDD**: Red - no metrics

### 018.2: Implement Search Timer
- Start timer on search
- Calculate duration
- **Time**: 15 min
- **TDD**: Green - timing works

### 018.3: Add Result Count Tracking
- Count exact matches
- Count semantic matches
- **Time**: 15 min
- **TDD**: Green - counts tracked

### 018.4: Implement Accuracy Tracking
- Define accuracy metric
- Track user selections
- **Time**: 15 min
- **TDD**: Green - accuracy tracked

### 018.5: Add Performance Histograms
- Track latency distribution
- Calculate percentiles
- **Time**: 15 min
- **TDD**: Green - histograms work

### 018.6: Create Metrics Reporter
- Format metrics output
- Support multiple formats
- **Time**: 15 min
- **TDD**: Refactor - reporting

### 018.7: Add Metrics Persistence
- Save metrics to file
- Rotation strategy
- **Time**: 15 min
- **TDD**: Refactor - persistence

### 018.8: Document Metrics System
- Explain all metrics
- Analysis guide
- **Time**: 15 min
- **TDD**: Complete docs

---

## Task 019: Error Handling (1 hour = 4 micro tasks)

### 019.1: Define Error Types
- Create error enum
- Add error contexts
- **Time**: 15 min
- **TDD**: Setup error types

### 019.2: Add Graceful Degradation
- Direct error responses
- Clear failure messages
- **Time**: 15 min
- **TDD**: Implement error responses

### 019.3: Implement Error Logging
- Log all errors
- Add context info
- **Time**: 15 min
- **TDD**: Add logging

### 019.4: Write Error Tests
- Test all error paths
- Verify recovery
- **Time**: 15 min
- **TDD**: Validate handling

---

## Task 020: Phase 2 Integration & Documentation (1 hour = 4 micro tasks)

### 020.1: Run Full Integration Tests
- Test all components together
- Verify 70% accuracy target
- **Time**: 15 min
- **TDD**: Validate phase

### 020.2: Performance Benchmarking
- Measure <500ms target
- Identify bottlenecks
- **Time**: 15 min
- **TDD**: Validate performance

### 020.3: Create Phase 2 API Documentation
- Document all public APIs
- Add examples
- **Time**: 15 min
- **TDD**: Complete API docs

### 020.4: Write Phase 2 Summary Report
- List achievements
- Note any issues
- **Time**: 15 min
- **TDD**: Phase complete

---

## Success Criteria Checklist

- [ ] Vector similarity search working with LanceDB
- [ ] Simple fusion algorithm deduplicates and scores correctly
- [ ] Unified search pipeline integrates all components
- [ ] Result ranking improves relevance
- [ ] Parallel execution reduces latency
- [ ] Search result caching improves performance
- [ ] Query preprocessing handles common cases
- [ ] Metrics track search quality and performance
- [ ] Error handling provides graceful degradation
- [ ] All tests passing with >90% coverage
- [ ] Search latency <500ms average
- [ ] Search accuracy >70% on test queries
- [ ] Documentation complete for all components

## TDD Cycle Summary

For each micro task:
1. **Red**: Write failing test first
2. **Green**: Implement minimal code to pass
3. **Refactor**: Improve code quality while keeping tests green

Total micro tasks: 100
Total estimated time: 25 hours (100 × 15 minutes)
Actual phase allocation: 20 hours + 5 hours buffer