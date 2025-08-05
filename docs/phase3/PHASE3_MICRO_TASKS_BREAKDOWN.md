# Phase 3: Git File Watching - Atomic Micro Tasks Breakdown

## **OVERVIEW**

This document breaks down Phase 3 (Tasks 021-030) into atomic micro tasks. Each micro task:
- Takes maximum 15 minutes to complete
- Follows TDD approach: RED → GREEN → REFACTOR
- Has clear, measurable success criteria
- Can be completed independently

## **TDD APPROACH FOR EACH MICRO TASK**

1. **RED**: Write failing test(s) that define expected behavior
2. **GREEN**: Write minimal code to make test(s) pass
3. **REFACTOR**: Clean up code while keeping tests green

---

## **TASK 021: Git Status Parser (3 hours = 12 micro tasks)**

### **021.1: Setup Git Watcher Test Infrastructure**
- **RED**: Create test module and write test for GitWatcher struct creation
- **GREEN**: Implement minimal GitWatcher struct with new() method
- **REFACTOR**: Add proper error handling and documentation
- **Time**: 15 minutes

### **021.2: Test Git Command Execution**
- **RED**: Write test for executing git status command
- **GREEN**: Implement basic Command execution for git status
- **REFACTOR**: Extract command execution to helper method
- **Time**: 15 minutes

### **021.3: Parse Git Status Output - Added Files**
- **RED**: Write test for parsing added files from git status
- **GREEN**: Implement parsing for "A " and "AM" status codes
- **REFACTOR**: Create FileChange enum and use pattern matching
- **Time**: 15 minutes

### **021.4: Parse Git Status Output - Modified Files**
- **RED**: Write test for parsing modified files
- **GREEN**: Implement parsing for " M", "M ", "MM" status codes
- **REFACTOR**: Consolidate status code matching logic
- **Time**: 15 minutes

### **021.5: Parse Git Status Output - Deleted Files**
- **RED**: Write test for parsing deleted files
- **GREEN**: Implement parsing for " D", "D " status codes
- **REFACTOR**: Complete FileChange enum implementation
- **Time**: 15 minutes

### **021.6: Parse Git Status Output - Untracked Files**
- **RED**: Write test for parsing untracked files (??)
- **GREEN**: Implement parsing for "??" status
- **REFACTOR**: Ensure consistent handling of all file states
- **Time**: 15 minutes

### **021.7: Implement Code File Filter**
- **RED**: Write tests for is_code_file() with various extensions
- **GREEN**: Implement basic extension matching
- **REFACTOR**: Use match expression for cleaner code
- **Time**: 15 minutes

### **021.8: Add More Language Support**
- **RED**: Write tests for additional language extensions
- **GREEN**: Add support for 15+ programming languages
- **REFACTOR**: Organize extensions by category
- **Time**: 15 minutes

### **021.9: Handle Git Command Errors**
- **RED**: Write tests for git command failure scenarios
- **GREEN**: Implement error handling for command execution
- **REFACTOR**: Create custom error types
- **Time**: 15 minutes

### **021.10: Path Resolution and Normalization**
- **RED**: Write tests for relative/absolute path handling
- **GREEN**: Implement proper path resolution
- **REFACTOR**: Use PathBuf methods consistently
- **Time**: 15 minutes

### **021.11: Empty Repository Handling**
- **RED**: Write test for empty repo/no changes scenario
- **GREEN**: Handle empty git status output
- **REFACTOR**: Return empty Vec instead of error
- **Time**: 15 minutes

### **021.12: Integration Test with Real Git Repo**
- **RED**: Create test git repository with known changes
- **GREEN**: Run GitWatcher against test repo
- **REFACTOR**: Clean up test helpers and fixtures
- **Time**: 15 minutes

---

## **TASK 022: Vector Database Updater (4 hours = 16 micro tasks)**

### **022.1: Setup VectorUpdater Test Infrastructure**
- **RED**: Create test module and mock dependencies
- **GREEN**: Implement VectorUpdater struct
- **REFACTOR**: Define clear interfaces for dependencies
- **Time**: 15 minutes

### **022.2: Test File Deletion from Vector DB**
- **RED**: Write test for delete_file_embeddings
- **GREEN**: Implement basic deletion logic
- **REFACTOR**: Add proper error handling
- **Time**: 15 minutes

### **022.3: Test Single File Reading**
- **RED**: Write test for reading file content
- **GREEN**: Implement file reading with error handling
- **REFACTOR**: Handle encoding issues gracefully
- **Time**: 15 minutes

### **022.4: Test File Chunking Integration**
- **RED**: Write test for chunking file content
- **GREEN**: Integrate with SimpleRegexChunker
- **REFACTOR**: Handle empty files and edge cases
- **Time**: 15 minutes

### **022.5: Test Single Chunk Embedding**
- **RED**: Write test for embedding one chunk
- **GREEN**: Integrate with MiniLMEmbedder
- **REFACTOR**: Add embedding validation
- **Time**: 15 minutes

### **022.6: Test Embedding Storage**
- **RED**: Write test for storing single embedding
- **GREEN**: Implement storage.insert_embedding call
- **REFACTOR**: Add metadata to stored embeddings
- **Time**: 15 minutes

### **022.7: Test Complete File Indexing**
- **RED**: Write test for index_file method
- **GREEN**: Implement full file processing pipeline
- **REFACTOR**: Add progress tracking
- **Time**: 15 minutes

### **022.8: Test Modified File Update**
- **RED**: Write test for updating modified file
- **GREEN**: Implement delete-then-reindex pattern
- **REFACTOR**: Optimize for minimal downtime
- **Time**: 15 minutes

### **022.9: Test Added File Processing**
- **RED**: Write test for processing new files
- **GREEN**: Implement add file logic
- **REFACTOR**: Share code with modified file handling
- **Time**: 15 minutes

### **022.10: Test Deleted File Handling**
- **RED**: Write test for FileChange::Deleted
- **GREEN**: Implement deletion-only logic
- **REFACTOR**: Ensure no orphaned embeddings
- **Time**: 15 minutes

### **022.11: Test Large File Handling**
- **RED**: Write test with large file (>1MB)
- **GREEN**: Implement streaming/chunked processing
- **REFACTOR**: Add memory usage constraints
- **Time**: 15 minutes

### **022.12: Test Binary File Rejection**
- **RED**: Write test for binary file detection
- **GREEN**: Implement binary file checks
- **REFACTOR**: Fail gracefully with clear errors
- **Time**: 15 minutes

### **022.13: Test Concurrent File Updates**
- **RED**: Write test for parallel file processing
- **GREEN**: Add basic async/await support
- **REFACTOR**: Ensure thread safety
- **Time**: 15 minutes

### **022.14: Test Update Error Recovery**
- **RED**: Write test for partial update failure
- **GREEN**: Implement per-file error handling
- **REFACTOR**: Log errors without stopping batch
- **Time**: 15 minutes

### **022.15: Test Embedding Retry Logic**
- **RED**: Write test for transient embedding failures
- **GREEN**: Implement retry with backoff
- **REFACTOR**: Make retry configurable
- **Time**: 15 minutes

### **022.16: Integration Test Full Update Flow**
- **RED**: Create test with multiple file changes
- **GREEN**: Run complete update pipeline
- **REFACTOR**: Verify database state consistency
- **Time**: 15 minutes

---

## **TASK 023: Batch Update Processing (3 hours = 12 micro tasks)**

### **023.1: Setup Batch Processing Tests**
- **RED**: Create test infrastructure for batch updates
- **GREEN**: Implement UpdateStats struct
- **REFACTOR**: Add Display trait for stats
- **Time**: 15 minutes

### **023.2: Test Change Grouping by Type**
- **RED**: Write test for grouping changes
- **GREEN**: Implement change categorization
- **REFACTOR**: Use iterators for efficiency
- **Time**: 15 minutes

### **023.3: Test Deletion Batch Processing**
- **RED**: Write test for batch deletions
- **GREEN**: Implement parallel deletion logic
- **REFACTOR**: Add deletion confirmation
- **Time**: 15 minutes

### **023.4: Test Modification Batch Processing**
- **RED**: Write test for batch modifications
- **GREEN**: Implement modification processing
- **REFACTOR**: Share code with single updates
- **Time**: 15 minutes

### **023.5: Test Stats Collection**
- **RED**: Write test for statistics tracking
- **GREEN**: Implement counters for each operation
- **REFACTOR**: Add timing information
- **Time**: 15 minutes

### **023.6: Test Partial Batch Failure**
- **RED**: Write test with some failing updates
- **GREEN**: Continue processing on failures
- **REFACTOR**: Collect all errors for reporting
- **Time**: 15 minutes

### **023.7: Test Empty Batch Handling**
- **RED**: Write test for empty change list
- **GREEN**: Handle gracefully with zero stats
- **REFACTOR**: Skip processing entirely
- **Time**: 15 minutes

### **023.8: Test Large Batch Performance**
- **RED**: Write test with 100+ file changes
- **GREEN**: Ensure reasonable performance
- **REFACTOR**: Add batch size limits
- **Time**: 15 minutes

### **023.9: Test Duplicate Change Handling**
- **RED**: Write test with duplicate file paths
- **GREEN**: Deduplicate before processing
- **REFACTOR**: Keep latest change only
- **Time**: 15 minutes

### **023.10: Test Batch Cancellation**
- **RED**: Write test for mid-batch cancellation
- **GREEN**: Add cancellation token support
- **REFACTOR**: Clean up partial state
- **Time**: 15 minutes

### **023.11: Test Memory Usage Limits**
- **RED**: Write test for memory constraints
- **GREEN**: Process in chunks if needed
- **REFACTOR**: Make chunk size configurable
- **Time**: 15 minutes

### **023.12: Integration Test Real Batch**
- **RED**: Create realistic batch scenario
- **GREEN**: Process complete batch
- **REFACTOR**: Verify final database state
- **Time**: 15 minutes

---

## **TASK 024: Watch Command Implementation (2 hours = 8 micro tasks)**

### **024.1: Setup Watch Command Tests**
- **RED**: Create test infrastructure for watch command
- **GREEN**: Implement WatchCommand struct
- **REFACTOR**: Define clear start/stop interface
- **Time**: 15 minutes

### **024.2: Test Watch Enable/Disable**
- **RED**: Write test for atomic bool flag
- **GREEN**: Implement enabled flag with Arc
- **REFACTOR**: Add thread-safe accessors
- **Time**: 15 minutes

### **024.3: Test Watch Loop Basic**
- **RED**: Write test for basic watch loop
- **GREEN**: Implement simple loop with sleep
- **REFACTOR**: Extract loop to separate method
- **Time**: 15 minutes

### **024.4: Test Change Detection Interval**
- **RED**: Write test for 5-second interval
- **GREEN**: Implement configurable interval
- **REFACTOR**: Use Duration type consistently
- **Time**: 15 minutes

### **024.5: Test Background Thread Spawn**
- **RED**: Write test for thread spawning
- **GREEN**: Implement thread::spawn logic
- **REFACTOR**: Add thread naming for debugging
- **Time**: 15 minutes

### **024.6: Test Watch Stop Mechanism**
- **RED**: Write test for stopping watch
- **GREEN**: Implement atomic bool checking
- **REFACTOR**: Ensure clean thread shutdown
- **Time**: 15 minutes

### **024.7: Test Error Handling in Loop**
- **RED**: Write test for check_and_update errors
- **GREEN**: Log errors without crashing
- **REFACTOR**: Add error counting/reporting
- **Time**: 15 minutes

### **024.8: Integration Test Watch Lifecycle**
- **RED**: Test complete start/run/stop cycle
- **GREEN**: Verify proper cleanup
- **REFACTOR**: Add status reporting
- **Time**: 15 minutes

---

## **TASK 025: State Persistence (2 hours = 8 micro tasks)**

### **025.1: Setup State Persistence Tests**
- **RED**: Create test infrastructure for state saving
- **GREEN**: Define state structure
- **REFACTOR**: Use serde for serialization
- **Time**: 15 minutes

### **025.2: Test State File Creation**
- **RED**: Write test for initial state file
- **GREEN**: Implement state file writing
- **REFACTOR**: Use atomic writes
- **Time**: 15 minutes

### **025.3: Test State Loading**
- **RED**: Write test for loading saved state
- **GREEN**: Implement state deserialization
- **REFACTOR**: Handle missing/corrupt files
- **Time**: 15 minutes

### **025.4: Test Last Update Timestamp**
- **RED**: Write test for timestamp tracking
- **GREEN**: Add timestamp to state
- **REFACTOR**: Use SystemTime for accuracy
- **Time**: 15 minutes

### **025.5: Test Processed Files List**
- **RED**: Write test for file tracking
- **GREEN**: Track processed file paths
- **REFACTOR**: Limit list size for memory
- **Time**: 15 minutes

### **025.6: Test State Migration**
- **RED**: Write test for version upgrades
- **GREEN**: Implement version field
- **REFACTOR**: Add migration logic
- **Time**: 15 minutes

### **025.7: Test Concurrent State Access**
- **RED**: Write test for multi-thread access
- **GREEN**: Add file locking
- **REFACTOR**: Use lock files properly
- **Time**: 15 minutes

### **025.8: Integration Test State Recovery**
- **RED**: Test crash recovery scenario
- **GREEN**: Verify state consistency
- **REFACTOR**: Add state validation
- **Time**: 15 minutes

---

## **TASK 026: Progress Reporting (2 hours = 8 micro tasks)**

### **026.1: Setup Progress Bar Tests**
- **RED**: Create test for progress bar creation
- **GREEN**: Integrate indicatif crate
- **REFACTOR**: Create progress bar factory
- **Time**: 15 minutes

### **026.2: Test Progress Updates**
- **RED**: Write test for progress increments
- **GREEN**: Implement set_position calls
- **REFACTOR**: Add percentage display
- **Time**: 15 minutes

### **026.3: Test Progress Messages**
- **RED**: Write test for status messages
- **GREEN**: Implement set_message calls
- **REFACTOR**: Truncate long file paths
- **Time**: 15 minutes

### **026.4: Test Progress Bar Styling**
- **RED**: Write test for custom style
- **GREEN**: Implement progress bar template
- **REFACTOR**: Make style configurable
- **Time**: 15 minutes

### **026.5: Test Multi-Bar Support**
- **RED**: Write test for nested progress
- **GREEN**: Support deletion + update bars
- **REFACTOR**: Use MultiProgress manager
- **Time**: 15 minutes

### **026.6: Test Progress in Quiet Mode**
- **RED**: Write test for --quiet flag
- **GREEN**: Conditionally show progress
- **REFACTOR**: Add verbosity levels
- **Time**: 15 minutes

### **026.7: Test ETA Calculation**
- **RED**: Write test for time estimates
- **GREEN**: Add ETA to progress bar
- **REFACTOR**: Smooth ETA calculations
- **Time**: 15 minutes

### **026.8: Integration Test Full Progress**
- **RED**: Test with real file updates
- **GREEN**: Verify smooth updates
- **REFACTOR**: Handle terminal resize
- **Time**: 15 minutes

---

## **TASK 027: Ignore Patterns (2 hours = 8 micro tasks)**

### **027.1: Setup Ignore Pattern Tests**
- **RED**: Create test for pattern parsing
- **GREEN**: Implement basic pattern struct
- **REFACTOR**: Use glob crate
- **Time**: 15 minutes

### **027.2: Test .gitignore Loading**
- **RED**: Write test for reading .gitignore
- **GREEN**: Implement file reading
- **REFACTOR**: Handle nested .gitignore files
- **Time**: 15 minutes

### **027.3: Test Simple Glob Patterns**
- **RED**: Write test for *.log patterns
- **GREEN**: Implement glob matching
- **REFACTOR**: Cache compiled patterns
- **Time**: 15 minutes

### **027.4: Test Directory Patterns**
- **RED**: Write test for dir/ patterns
- **GREEN**: Handle directory exclusions
- **REFACTOR**: Optimize path checking
- **Time**: 15 minutes

### **027.5: Test Negation Patterns**
- **RED**: Write test for ! patterns
- **GREEN**: Implement pattern negation
- **REFACTOR**: Handle precedence correctly
- **Time**: 15 minutes

### **027.6: Test Comment Handling**
- **RED**: Write test for # comments
- **GREEN**: Skip comment lines
- **REFACTOR**: Handle inline comments
- **Time**: 15 minutes

### **027.7: Test Pattern Priority**
- **RED**: Write test for override rules
- **GREEN**: Implement last-match-wins
- **REFACTOR**: Document behavior clearly
- **Time**: 15 minutes

### **027.8: Integration Test Ignore Rules**
- **RED**: Test with complex .gitignore
- **GREEN**: Verify correct filtering
- **REFACTOR**: Add performance tests
- **Time**: 15 minutes

---

## **TASK 028: Error Recovery (2 hours = 8 micro tasks)**

### **028.1: Setup Error Recovery Tests**
- **RED**: Create test infrastructure for errors
- **GREEN**: Define error types
- **REFACTOR**: Use thiserror crate
- **Time**: 15 minutes

### **028.2: Test File Access Errors**
- **RED**: Write test for permission denied
- **GREEN**: Handle IO errors gracefully
- **REFACTOR**: Add helpful error messages
- **Time**: 15 minutes

### **028.3: Test Embedding Service Errors**
- **RED**: Write test for service timeout
- **GREEN**: Implement timeout handling
- **REFACTOR**: Add retry logic
- **Time**: 15 minutes

### **028.4: Test Database Connection Errors**
- **RED**: Write test for DB unavailable
- **GREEN**: Handle connection failures
- **REFACTOR**: Add connection pooling
- **Time**: 15 minutes

### **028.5: Test Partial Update Recovery**
- **RED**: Write test for mid-update failure
- **GREEN**: Save progress periodically
- **REFACTOR**: Implement checkpointing
- **Time**: 15 minutes

### **028.6: Test Rollback Mechanism**
- **RED**: Write test for update rollback
- **GREEN**: Implement transaction support
- **REFACTOR**: Make rollback automatic
- **Time**: 15 minutes

### **028.7: Test Error Reporting**
- **RED**: Write test for error summaries
- **GREEN**: Collect and format errors
- **REFACTOR**: Add error categorization
- **Time**: 15 minutes

### **028.8: Integration Test Recovery**
- **RED**: Test various failure scenarios
- **GREEN**: Verify system resilience
- **REFACTOR**: Add recovery metrics
- **Time**: 15 minutes

---

## **TASK 029: Watch Status API (1 hour = 4 micro tasks)**

### **029.1: Setup Status API Tests**
- **RED**: Create test for status endpoint
- **GREEN**: Define status response struct
- **REFACTOR**: Use consistent JSON format
- **Time**: 15 minutes

### **029.2: Test Watch Running Status**
- **RED**: Write test for is_running check
- **GREEN**: Expose watch state
- **REFACTOR**: Add state enum
- **Time**: 15 minutes

### **029.3: Test Statistics Retrieval**
- **RED**: Write test for getting stats
- **GREEN**: Implement stats aggregation
- **REFACTOR**: Add time-based windows
- **Time**: 15 minutes

### **029.4: Integration Test Status API**
- **RED**: Test full status endpoint
- **GREEN**: Verify all fields populated
- **REFACTOR**: Add API versioning
- **Time**: 15 minutes

---

## **TASK 030: Phase 3 Completion (1 hour = 4 micro tasks)**

### **030.1: Setup Integration Tests**
- **RED**: Create comprehensive test suite
- **GREEN**: Test all components together
- **REFACTOR**: Add performance benchmarks
- **Time**: 15 minutes

### **030.2: Test with Real Repository**
- **RED**: Clone test repository
- **GREEN**: Run full git watch cycle
- **REFACTOR**: Verify vector updates
- **Time**: 15 minutes

### **030.3: Test Performance Targets**
- **RED**: Write performance tests
- **GREEN**: Verify <1s detection time
- **REFACTOR**: Optimize hot paths
- **Time**: 15 minutes

### **030.4: Documentation and Cleanup**
- **RED**: Write API documentation tests
- **GREEN**: Complete all documentation
- **REFACTOR**: Clean up test fixtures
- **Time**: 15 minutes

---

## **SUMMARY**

**Total Micro Tasks**: 100
**Total Time**: 25 hours (100 × 15 minutes)
**Original Estimate**: 23 hours

Each micro task follows the TDD pattern:
1. **RED**: Write failing test first
2. **GREEN**: Implement minimal solution
3. **REFACTOR**: Improve code quality

This breakdown ensures:
- Every feature is tested first
- Implementation is incremental
- Code quality improves continuously
- Progress is measurable every 15 minutes