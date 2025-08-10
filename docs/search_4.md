# Search 4: "struct impl" Query

**Query:** struct impl  
**Limit:** 10  
**Results Found:** 10  

## Search Output

```
🔍 Searching for: struct impl

Found 10 results:

1. src\config.rs
   // Configuration management - simple but flexible
   
   use serde::{Deserialize, Serialize};...

2. src\utils\retry.rs
   use std::time::Duration;
   use std::future::Future;
   use std::pin::Pin;...

3. src\symbol_extractor.rs
   // Tree-sitter based symbol extraction - sophisticated but focused
   
   use anyhow::Result;...

4. src\search\preprocessing.rs
   use std::collections::HashSet;
   
   pub struct QueryPreprocessor;...

5. src\cache\bounded_cache.rs
   // Bounded Cache Implementation - Phase 1: Foundation & Safety
   // This module provides memory-safe caching with LRU eviction
   ...

6. src\utils\memory_monitor.rs
   /// Memory monitoring utilities for detecting and preventing OOM conditions
   /// 
   /// This module provides utilities to monitor memory usage and prevent...

7. src\embedding_cache.rs
   // High-performance embedding cache to avoid redundant computations
   // Critical for meeting <100ms search latency target
   ...

8. src\chunking\line_validator.rs
   // Line validation for code chunking
   
   use anyhow::Result;...

9. src\fusion.rs
   // Configurable fusion algorithm for hybrid search
   // Production-ready implementation based on research
   ...

10. src\chunking\three_chunk.rs
   use crate::chunking::Chunk;
   
   /// Expands a target chunk to include surrounding context (above/target/below)...
```

## Analysis

This search identifies struct and implementation patterns throughout the codebase:
- Configuration management structures
- Retry and async utilities with complex implementations
- Symbol extraction system architectures  
- Query preprocessing components
- Memory-safe cache implementations
- Performance monitoring structures
- High-performance embedding caches
- Code validation systems
- Fusion algorithm implementations
- Context-aware chunking structures