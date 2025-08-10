# Search 5: "async await tokio" Query

**Query:** async await tokio  
**Limit:** 15  
**Results Found:** 11  

## Search Output

```
🔍 Searching for: async await tokio

Found 11 results:

1. src\utils\retry.rs
   use std::time::Duration;
   use std::future::Future;
   use std::pin::Pin;...

2. src\bin\mcp_server.rs
   use std::io::{self, BufRead, BufReader, Write};
   use std::sync::Arc;
   use tokio::sync::Mutex;...

3. src\simple_storage.rs
   use anyhow::Result;
   use arrow_array::{RecordBatch, Float32Array, StringArray, FixedSizeListArray, Int32Array, RecordBatchIterator};
   use arrow_schema::{DataType, Field, Schema};...

4. src\bin\embed_cli.rs
   // Traditional CLI for embed-search that can be used globally
   // This is separate from the MCP server and can be added to PATH
   ...

5. src\main.rs
   use anyhow::Result;
   use clap::{Parser, Subcommand};
   use walkdir::WalkDir;...

6. src\simple_search.rs
   use anyhow::Result;
   use tantivy::{Index, IndexWriter, schema::{Schema, Field, TEXT, STORED, Value}};
   use tantivy::query::QueryParser;...

7. src\advanced_search.rs
   use anyhow::Result;
   use tantivy::{Index, IndexWriter, schema::{Schema, Field, TEXT, STORED, Value}};
   use tantivy::query::QueryParser;...

8. src\error.rs
   // Comprehensive Error Handling System - Phase 1: Foundation & Safety
   // This module provides robust error types to replace panic-prone patterns
   ...

9. src\indexer.rs
   // Incremental indexing with change detection
   
   use anyhow::Result;...

10. src\search\text_processor.rs
   use std::collections::HashSet;
   use rust_stemmers::{Algorithm, Stemmer};
   use unicode_normalization::UnicodeNormalization;...

11. src\chunking\regex_chunker.rs
   use regex::Regex;
   use std::path::Path;
   // Config temporarily removed...
```

## Analysis

This search demonstrates the system's ability to find async/concurrent programming patterns:
- Future and async retry mechanisms  
- Tokio-based MCP server with Arc/Mutex coordination
- Async storage operations with Arrow/LanceDB
- CLI tools with async runtime management
- Main application async entry points
- Search engines with async indexing/querying
- Advanced search with concurrent operations
- Error handling in async contexts
- Incremental indexing with async I/O
- Text processing pipelines
- Async-aware chunking systems