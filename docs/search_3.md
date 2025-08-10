# Search 3: "error Result" Query

**Query:** error Result  
**Limit:** 12  
**Results Found:** 12  

## Search Output

```
🔍 Searching for: error Result

Found 12 results:

1. src\error.rs
   // Comprehensive Error Handling System - Phase 1: Foundation & Safety
   // This module provides robust error types to replace panic-prone patterns
   ...

2. src\utils\retry.rs
   use std::time::Duration;
   use std::future::Future;
   use std::pin::Pin;...

3. src\chunking\line_validator.rs
   // Line validation for code chunking
   
   use anyhow::Result;...

4. src\bin\mcp_server.rs
   use std::io::{self, BufRead, BufReader, Write};
   use std::sync::Arc;
   use tokio::sync::Mutex;...

5. src\chunking\regex_chunker.rs
   use regex::Regex;
   use std::path::Path;
   // Config temporarily removed...

6. src\chunking\three_chunk.rs
   use crate::chunking::Chunk;
   
   /// Expands a target chunk to include surrounding context (above/target/below)...

7. src\lib.rs
   // Balanced architecture - sophisticated but not over-engineered
   
   pub mod error;...

8. src\search\fusion.rs
   use std::collections::HashSet;
   use serde::{Serialize, Deserialize};
   use crate::error::SearchError;...

9. src\indexer.rs
   // Incremental indexing with change detection
   
   use anyhow::Result;...

10. src\utils\memory_monitor.rs
   /// Memory monitoring utilities for detecting and preventing OOM conditions
   /// 
   /// This module provides utilities to monitor memory usage and prevent...

11. src\cache\bounded_cache.rs
   // Bounded Cache Implementation - Phase 1: Foundation & Safety
   // This module provides memory-safe caching with LRU eviction
   ...

12. src\advanced_search.rs
   use anyhow::Result;
   use tantivy::{Index, IndexWriter, schema::{Schema, Field, TEXT, STORED, Value}};
   use tantivy::query::QueryParser;...
```

## Analysis

This search effectively identifies error handling and Result type usage across the codebase:
- Comprehensive error handling system (error.rs)
- Retry mechanisms and utilities
- Input validation for code chunks
- Memory monitoring and safety utilities  
- MCP server error handling
- Search fusion error management
- Cache safety implementations
- Module-level error exports and integration