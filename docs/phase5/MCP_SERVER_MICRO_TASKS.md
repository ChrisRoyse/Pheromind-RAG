# MCP Server Implementation - Micro Task Breakdown

## Overview
This document breaks down Phase 4 (MCP Server & Tools) into atomic micro tasks following TDD red-green-refactor approach. Each task is designed to take no more than 15 minutes.

## Task 031: MCP Server Foundation (4 hours → 16 micro tasks)

### 031.1: Create MCP Server Module Structure
- Create `src/mcp/mod.rs`
- Create `src/mcp/server.rs`
- Update `src/lib.rs` to include mcp module
- **Test**: Write failing test for module existence

### 031.2: Define MCPServer Struct
- Create MCPServer struct with basic fields
- **Test**: Write test for MCPServer::new() constructor
- **RED**: Test fails (no implementation)
- **GREEN**: Implement basic constructor

### 031.3: Add Dependencies to Cargo.toml
- Add mcp_rs crate
- Add serde, serde_json
- Add tokio with full features
- **Test**: Cargo build succeeds

### 031.4: Create Tool Registration Interface
- Define register_tools method signature
- **Test**: Write test for registering a dummy tool
- **RED**: Test fails
- **GREEN**: Implement basic registration

### 031.5: Initialize Storage Component
- Add VectorStorage field to MCPServer
- **Test**: Test storage initialization
- **RED**: Storage not connected
- **GREEN**: Connect storage in constructor

### 031.6: Initialize Embedder Component
- Add MiniLMEmbedder field
- **Test**: Test embedder initialization
- **RED**: Embedder not created
- **GREEN**: Create embedder instance

### 031.7: Initialize Chunker Component
- Add SimpleRegexChunker field
- **Test**: Test chunker availability
- **RED**: Chunker not initialized
- **GREEN**: Initialize chunker

### 031.8: Create UnifiedSearcher Instance
- Combine components into UnifiedSearcher
- **Test**: Test searcher creation
- **RED**: Searcher not assembled
- **GREEN**: Assemble searcher

### 031.9: Add Three-Chunk Expander
- Add ThreeChunkExpander to searcher
- **Test**: Test expander integration
- **RED**: Expander not connected
- **GREEN**: Connect expander

### 031.10: Add Simple Fusion Component
- Add SimpleFusion to searcher
- **Test**: Test fusion availability
- **RED**: Fusion not integrated
- **GREEN**: Integrate fusion

### 031.11: Create GitWatcher Instance
- Initialize GitWatcher component
- **Test**: Test watcher creation
- **RED**: Watcher not created
- **GREEN**: Create watcher

### 031.12: Create VectorUpdater Instance
- Initialize VectorUpdater with components
- **Test**: Test updater initialization
- **RED**: Updater not created
- **GREEN**: Create updater

### 031.13: Assemble GitWatch Component
- Combine watcher and updater
- **Test**: Test GitWatch assembly
- **RED**: Not assembled
- **GREEN**: Assemble components

### 031.14: Add Arc<RwLock> Wrappers
- Wrap components in Arc<RwLock>
- **Test**: Test concurrent access
- **RED**: No concurrency support
- **GREEN**: Add thread-safe wrappers

### 031.15: Add Project Path Field
- Store project path in server
- **Test**: Test path storage
- **RED**: Path not stored
- **GREEN**: Store path

### 031.16: Error Handling for Constructor
- Add Result return type
- **Test**: Test error propagation
- **RED**: Errors not handled
- **GREEN**: Add error handling

## Task 032: Search Tool Implementation (3 hours → 12 micro tasks)

### 032.1: Define Search Tool Metadata
- Create search_tool() method
- **Test**: Test tool metadata
- **RED**: Method doesn't exist
- **GREEN**: Return tool definition

### 032.2: Define SearchParams Struct
- Create SearchParams with query field
- **Test**: Test deserialization
- **RED**: Struct doesn't exist
- **GREEN**: Create struct with Deserialize

### 032.3: Define SearchResponse Struct
- Create response structure
- **Test**: Test serialization
- **RED**: Struct doesn't exist
- **GREEN**: Create with Serialize

### 032.4: Define SearchResultMCP Struct
- Create MCP-specific result type
- **Test**: Test field access
- **RED**: Type doesn't exist
- **GREEN**: Create struct

### 032.5: Define ThreeChunkContextMCP
- Create MCP chunk context type
- **Test**: Test structure
- **RED**: Type missing
- **GREEN**: Create type

### 032.6: Define ChunkMCP Struct
- Create chunk representation
- **Test**: Test fields
- **RED**: Type missing
- **GREEN**: Create struct

### 032.7: Create handle_search Method Signature
- Define async method
- **Test**: Test method exists
- **RED**: Method missing
- **GREEN**: Add method stub

### 032.8: Implement Query Parsing
- Parse search params
- **Test**: Test param extraction
- **RED**: Not implemented
- **GREEN**: Parse params

### 032.9: Execute Search Operation
- Call searcher.search()
- **Test**: Test search execution
- **RED**: Not calling search
- **GREEN**: Execute search

### 032.10: Convert Results to MCP Format
- Map internal results to MCP
- **Test**: Test conversion
- **RED**: No conversion
- **GREEN**: Implement mapping

### 032.11: Add Timing Measurement
- Track search duration
- **Test**: Test timing data
- **RED**: No timing
- **GREEN**: Add Instant tracking

### 032.12: Return ToolResult
- Wrap response in ToolResult
- **Test**: Test success case
- **RED**: Not returning result
- **GREEN**: Return Success

## Task 033: Clear Database Tool (2 hours → 8 micro tasks)

### 033.1: Define Clear Database Tool Metadata
- Create clear_database_tool() method
- **Test**: Test tool definition
- **RED**: Method missing
- **GREEN**: Add method

### 033.2: Add Confirmation Parameter
- Define confirm parameter in schema
- **Test**: Test parameter schema
- **RED**: No parameters
- **GREEN**: Add parameter

### 033.3: Create handle_clear_database Signature
- Define async handler method
- **Test**: Test method exists
- **RED**: Method missing
- **GREEN**: Add stub

### 033.4: Parse Confirmation Parameter
- Extract confirm from params
- **Test**: Test param parsing
- **RED**: Not parsing
- **GREEN**: Parse boolean

### 033.5: Validate Confirmation
- Check confirm is true
- **Test**: Test validation
- **RED**: No validation
- **GREEN**: Add check

### 033.6: Acquire Storage Write Lock
- Get mutable storage access
- **Test**: Test lock acquisition
- **RED**: Not locking
- **GREEN**: Acquire lock

### 033.7: Execute Clear Operation
- Call storage.clear_all()
- **Test**: Test clear execution
- **RED**: Not clearing
- **GREEN**: Execute clear

### 033.8: Reinitialize Schema
- Call storage.init_schema()
- **Test**: Test reinitialization
- **RED**: Not reinitializing
- **GREEN**: Reinitialize

## Task 034: Reindex All Tool (4 hours → 16 micro tasks)

### 034.1: Define Reindex Tool Metadata
- Create reindex_all_tool() method
- **Test**: Test tool definition
- **RED**: Method missing
- **GREEN**: Add method

### 034.2: Add Directory Parameter
- Define optional directory param
- **Test**: Test parameter schema
- **RED**: No parameter
- **GREEN**: Add parameter

### 034.3: Add Progress Parameter
- Define show_progress param
- **Test**: Test default value
- **RED**: No parameter
- **GREEN**: Add with default

### 034.4: Create handle_reindex_all Signature
- Define async handler
- **Test**: Test method exists
- **RED**: Method missing
- **GREEN**: Add stub

### 034.5: Parse Directory Parameter
- Extract directory from params
- **Test**: Test parsing
- **RED**: Not parsing
- **GREEN**: Parse path

### 034.6: Resolve Relative Paths
- Convert relative to absolute
- **Test**: Test path resolution
- **RED**: Not resolving
- **GREEN**: Resolve paths

### 034.7: Validate Directory Exists
- Check directory validity
- **Test**: Test validation
- **RED**: No validation
- **GREEN**: Add checks

### 034.8: Create ReindexStats Struct
- Define statistics tracking
- **Test**: Test struct creation
- **RED**: Struct missing
- **GREEN**: Create struct

### 034.9: Create find_all_code_files Method
- Define file discovery method
- **Test**: Test method exists
- **RED**: Method missing
- **GREEN**: Add stub

### 034.10: Implement File Walking
- Use WalkDir for traversal
- **Test**: Test file discovery
- **RED**: Not walking
- **GREEN**: Walk directory

### 034.11: Add File Type Filtering
- Filter for code files only
- **Test**: Test filtering
- **RED**: No filtering
- **GREEN**: Add filter

### 034.12: Clear Existing Data
- Remove old embeddings
- **Test**: Test clearing
- **RED**: Not clearing
- **GREEN**: Clear data

### 034.13: Process Files in Batch
- Index files with progress
- **Test**: Test processing
- **RED**: Not processing
- **GREEN**: Process files

### 034.14: Track Statistics
- Update stats during processing
- **Test**: Test stat tracking
- **RED**: Not tracking
- **GREEN**: Track stats

### 034.15: Handle Processing Errors
- Catch and report errors
- **Test**: Test error handling
- **RED**: No error handling
- **GREEN**: Add try-catch

### 034.16: Return Statistics
- Format and return results
- **Test**: Test response format
- **RED**: Not returning stats
- **GREEN**: Return stats

## Task 035: Toggle Watch Tool (2 hours → 8 micro tasks)

### 035.1: Define Toggle Watch Tool Metadata
- Create toggle_watch_tool() method
- **Test**: Test tool definition
- **RED**: Method missing
- **GREEN**: Add method

### 035.2: Add Enabled Parameter
- Define boolean parameter
- **Test**: Test parameter schema
- **RED**: No parameter
- **GREEN**: Add parameter

### 035.3: Create handle_toggle_watch Signature
- Define async handler
- **Test**: Test method exists
- **RED**: Method missing
- **GREEN**: Add stub

### 035.4: Parse Enabled Parameter
- Extract enabled from params
- **Test**: Test parsing
- **RED**: Not parsing
- **GREEN**: Parse boolean

### 035.5: Acquire GitWatch Write Lock
- Get mutable access
- **Test**: Test lock acquisition
- **RED**: Not locking
- **GREEN**: Acquire lock

### 035.6: Implement Enable Logic
- Call start_watching()
- **Test**: Test enabling
- **RED**: Not enabling
- **GREEN**: Enable watching

### 035.7: Implement Disable Logic
- Call stop_watching()
- **Test**: Test disabling
- **RED**: Not disabling
- **GREEN**: Disable watching

### 035.8: Return Status Response
- Format success response
- **Test**: Test response format
- **RED**: No response
- **GREEN**: Return status

## Task 036: MCP Request Router (3 hours → 12 micro tasks)

### 036.1: Create Request Router Structure
- Define handle_request method
- **Test**: Test method signature
- **RED**: Method missing
- **GREEN**: Add method

### 036.2: Add Method Name Extraction
- Extract method from request
- **Test**: Test extraction
- **RED**: Not extracting
- **GREEN**: Extract method

### 036.3: Create Match Statement
- Add match on method name
- **Test**: Test matching
- **RED**: No matching
- **GREEN**: Add match

### 036.4: Route search_code Requests
- Add search_code case
- **Test**: Test routing
- **RED**: Case missing
- **GREEN**: Add case

### 036.5: Parse Search Parameters
- Deserialize search params
- **Test**: Test parsing
- **RED**: Not parsing
- **GREEN**: Parse params

### 036.6: Route clear_database Requests
- Add clear_database case
- **Test**: Test routing
- **RED**: Case missing
- **GREEN**: Add case

### 036.7: Route reindex_all Requests
- Add reindex_all case
- **Test**: Test routing
- **RED**: Case missing
- **GREEN**: Add case

### 036.8: Route toggle_watch Requests
- Add toggle_watch case
- **Test**: Test routing
- **RED**: Case missing
- **GREEN**: Add case

### 036.9: Add Unknown Method Handling
- Handle unmatched methods
- **Test**: Test error case
- **RED**: No handling
- **GREEN**: Return error

### 036.10: Add Parameter Validation
- Validate params exist
- **Test**: Test validation
- **RED**: No validation
- **GREEN**: Add checks

### 036.11: Add Error Propagation
- Propagate handler errors
- **Test**: Test error flow
- **RED**: Not propagating
- **GREEN**: Add ? operator

### 036.12: Add Response Formatting
- Ensure consistent responses
- **Test**: Test formatting
- **RED**: Inconsistent
- **GREEN**: Standardize

## Task 037: MCP Transport Layer (2 hours → 8 micro tasks)

### 037.1: Create Transport Module
- Create transport.rs file
- **Test**: Test module loads
- **RED**: File missing
- **GREEN**: Create file

### 037.2: Define StdioTransport Struct
- Create transport type
- **Test**: Test struct creation
- **RED**: Type missing
- **GREEN**: Define struct

### 037.3: Implement Transport Constructor
- Add new() method
- **Test**: Test construction
- **RED**: Method missing
- **GREEN**: Add constructor

### 037.4: Add Stdin Reader
- Create stdin reader thread
- **Test**: Test reading
- **RED**: No reader
- **GREEN**: Add reader

### 037.5: Add Stdout Writer
- Create stdout writer
- **Test**: Test writing
- **RED**: No writer
- **GREEN**: Add writer

### 037.6: Implement Message Parsing
- Parse JSON-RPC messages
- **Test**: Test parsing
- **RED**: Not parsing
- **GREEN**: Parse JSON

### 037.7: Add Request Handler Loop
- Create main event loop
- **Test**: Test loop runs
- **RED**: No loop
- **GREEN**: Add loop

### 037.8: Connect to Server
- Wire transport to server
- **Test**: Test connection
- **RED**: Not connected
- **GREEN**: Connect handler

## Task 038: Error Handling (2 hours → 8 micro tasks)

### 038.1: Define MCPError Type
- Create error enum
- **Test**: Test error creation
- **RED**: Type missing
- **GREEN**: Define enum

### 038.2: Add Transport Errors
- Add transport error variants
- **Test**: Test variants
- **RED**: Variants missing
- **GREEN**: Add variants

### 038.3: Add Storage Errors
- Add storage error variants
- **Test**: Test variants
- **RED**: Variants missing
- **GREEN**: Add variants

### 038.4: Add Validation Errors
- Add validation variants
- **Test**: Test variants
- **RED**: Variants missing
- **GREEN**: Add variants

### 038.5: Implement Error Display
- Add Display trait
- **Test**: Test formatting
- **RED**: No Display impl
- **GREEN**: Implement trait

### 038.6: Add Error Conversion
- Implement From traits
- **Test**: Test conversions
- **RED**: No conversions
- **GREEN**: Add From impls

### 038.7: Add Error Context
- Add context to errors
- **Test**: Test context
- **RED**: No context
- **GREEN**: Add context field

### 038.8: Add Recovery Logic
- Handle recoverable errors
- **Test**: Test recovery
- **RED**: No recovery
- **GREEN**: Add recovery

## Task 039: Tool Documentation (1 hour → 4 micro tasks)

### 039.1: Create Documentation Module
- Create docs.rs file
- **Test**: Test module exists
- **RED**: File missing
- **GREEN**: Create file

### 039.2: Generate Tool Schemas
- Export tool definitions
- **Test**: Test schema generation
- **RED**: No schemas
- **GREEN**: Generate schemas

### 039.3: Create Usage Examples
- Add example requests
- **Test**: Test examples valid
- **RED**: No examples
- **GREEN**: Add examples

### 039.4: Create README
- Write MCP usage guide
- **Test**: Test file exists
- **RED**: No README
- **GREEN**: Create guide

## Task 040: Phase 4 Completion (2 hours → 8 micro tasks)

### 040.1: Create Integration Test Suite
- Set up test framework
- **Test**: Test framework runs
- **RED**: No framework
- **GREEN**: Add framework

### 040.2: Test Search Tool E2E
- Full search workflow test
- **Test**: Test search works
- **RED**: Test fails
- **GREEN**: Fix issues

### 040.3: Test Clear Database E2E
- Full clear workflow test
- **Test**: Test clear works
- **RED**: Test fails
- **GREEN**: Fix issues

### 040.4: Test Reindex All E2E
- Full reindex workflow test
- **Test**: Test reindex works
- **RED**: Test fails
- **GREEN**: Fix issues

### 040.5: Test Toggle Watch E2E
- Full toggle workflow test
- **Test**: Test toggle works
- **RED**: Test fails
- **GREEN**: Fix issues

### 040.6: Performance Benchmarks
- Measure tool performance
- **Test**: Test benchmarks run
- **RED**: No benchmarks
- **GREEN**: Add benchmarks

### 040.7: LLM Integration Test
- Test with actual LLM
- **Test**: Test LLM connection
- **RED**: Not working
- **GREEN**: Fix integration

### 040.8: Final Documentation
- Update all docs
- **Test**: Test docs complete
- **RED**: Docs incomplete
- **GREEN**: Complete docs

## Summary

Total micro tasks: 96
- Task 031: 16 tasks (4 hours)
- Task 032: 12 tasks (3 hours)
- Task 033: 8 tasks (2 hours)
- Task 034: 16 tasks (4 hours)
- Task 035: 8 tasks (2 hours)
- Task 036: 12 tasks (3 hours)
- Task 037: 8 tasks (2 hours)
- Task 038: 8 tasks (2 hours)
- Task 039: 4 tasks (1 hour)
- Task 040: 8 tasks (2 hours)

Each micro task follows the TDD red-green-refactor pattern and is designed to be completed in 15 minutes or less. This breakdown ensures systematic progress while maintaining quality through continuous testing.