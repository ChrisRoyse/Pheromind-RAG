# Search 2: "function search" Query

**Query:** function search  
**Limit:** 8  
**Results Found:** 8  

## Search Output

```
🔍 Searching for: function search

Found 8 results:

1. src\chunking\regex_chunker.rs
   use regex::Regex;
   use std::path::Path;
   // Config temporarily removed...

2. src\symbol_extractor.rs
   // Tree-sitter based symbol extraction - sophisticated but focused
   
   use anyhow::Result;...

3. src\search\text_processor.rs
   use std::collections::HashSet;
   use rust_stemmers::{Algorithm, Stemmer};
   use unicode_normalization::UnicodeNormalization;...

4. src\search\preprocessing.rs
   use std::collections::HashSet;
   
   pub struct QueryPreprocessor;...

5. src\search\fusion.rs
   use std::collections::HashSet;
   use serde::{Serialize, Deserialize};
   use crate::error::SearchError;...

6. src\simple_search.rs
   use anyhow::Result;
   use tantivy::{Index, IndexWriter, schema::{Schema, Field, TEXT, STORED, Value}};
   use tantivy::query::QueryParser;...

7. src\bin\embed_cli.rs
   // Traditional CLI for embed-search that can be used globally
   // This is separate from the MCP server and can be added to PATH
   ...

8. src\semantic_chunker.rs
   // AST-based semantic code chunking using Tree-sitter
   // This is the REAL implementation for production use
   ...
```

## Analysis

This search demonstrates semantic understanding of function and search-related concepts, finding:
- Symbol extraction and AST parsing modules
- Text processing and query preprocessing components  
- Search fusion and ranking algorithms
- Semantic chunking for code analysis
- Core search implementation modules
- Processing utilities for text analysis