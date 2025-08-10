# Search 1: "embed" Query

**Query:** embed  
**Limit:** 10  
**Results Found:** 10  

## Search Output

```
🔍 Searching for: embed

Found 10 results:

1. src\simple_embedder.rs
   use anyhow::Result;
   use fastembed::TextEmbedding;
   ...

2. src\bin\mcp_server.rs
   use std::io::{self, BufRead, BufReader, Write};
   use std::sync::Arc;
   use tokio::sync::Mutex;...

3. src\embedding_cache.rs
   // High-performance embedding cache to avoid redundant computations
   // Critical for meeting <100ms search latency target
   ...

4. src\bin\embed_cli.rs
   // Traditional CLI for embed-search that can be used globally
   // This is separate from the MCP server and can be added to PATH
   ...

5. src\main.rs
   use anyhow::Result;
   use clap::{Parser, Subcommand};
   use walkdir::WalkDir;...

6. src\error.rs
   // Comprehensive Error Handling System - Phase 1: Foundation & Safety
   // This module provides robust error types to replace panic-prone patterns
   ...

7. src\config.rs
   // Configuration management - simple but flexible
   
   use serde::{Deserialize, Serialize};...

8. src\simple_search.rs
   use anyhow::Result;
   use tantivy::{Index, IndexWriter, schema::{Schema, Field, TEXT, STORED, Value}};
   use tantivy::query::QueryParser;...

9. src\advanced_search.rs
   use anyhow::Result;
   use tantivy::{Index, IndexWriter, schema::{Schema, Field, TEXT, STORED, Value}};
   use tantivy::query::QueryParser;...

10. src\indexer.rs
   // Incremental indexing with change detection
   
   use anyhow::Result;...
```

## Analysis

This search demonstrates the hybrid search capability finding files related to embedding functionality across the codebase, including:
- Core embedding modules (simple_embedder.rs)  
- MCP server implementation
- Embedding cache for performance optimization
- CLI tools for embedding operations
- Main application entry points
- Configuration and error handling systems