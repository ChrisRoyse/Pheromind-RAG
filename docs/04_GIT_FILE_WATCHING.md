# Phase 4: MCP Server & Tools

## **PHASE OVERVIEW - FULL MCP INTEGRATION**

**GOAL**: MCP server with complete tool suite for LLM integration  
**APPROACH**: Implement 4 essential tools for search and management  
**MEASUREMENT**: Verify all tools work correctly via MCP protocol  
**TIMELINE**: Week 3-4 (Tasks 031-040)

## **KEY INSIGHT: COMPLETE LLM CONTROL**

**REQUIRED TOOLS**:
1. **search_code**: Search with any query complexity
2. **clear_database**: Clear/reset entire vector database
3. **reindex_all**: Re-embed and store all vectors
4. **toggle_watch**: Turn file watching on/off

## **MCP SERVER TASK BREAKDOWN (031-040)**

### **Core MCP Tasks (031-035): Server & Tools**

#### **Task 031: MCP Server Foundation**
**Goal**: Basic MCP server setup with tool registration  
**Duration**: 4 hours  
**Dependencies**: Phase 3 completion

**Implementation**:
```rust
use mcp_rs::{Server, Tool, ToolResult, Request};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use std::sync::Arc;

pub struct MCPServer {
    searcher: Arc<RwLock<UnifiedSearcher>>,
    storage: Arc<RwLock<VectorStorage>>,
    git_watch: Arc<RwLock<Phase3GitWatch>>,
    project_path: PathBuf,
}

impl MCPServer {
    pub fn new(project_path: PathBuf) -> Self {
        // Initialize all components
        let storage = Arc::new(RwLock::new(VectorStorage::new(project_path.join(".embeddings"))?));
        let embedder = MiniLMEmbedder::new()?;
        let chunker = SimpleRegexChunker::new();
        
        let searcher = Arc::new(RwLock::new(UnifiedSearcher {
            tantivy: TantivySearcher,
            embedder: embedder.clone(),
            storage: storage.clone(),
            chunker: chunker.clone(),
            expander: ThreeChunkExpander,
            fusion: SimpleFusion,
        }));
        
        let git_watch = Arc::new(RwLock::new(Phase3GitWatch {
            watcher: GitWatcher::new(project_path.clone()),
            updater: VectorUpdater {
                storage: storage.clone(),
                chunker,
                embedder,
            },
            watch_command: WatchCommand::new(project_path.clone()),
        }));
        
        Self {
            searcher,
            storage,
            git_watch,
            project_path,
        }
    }
    
    pub fn register_tools(&mut self, server: &mut Server) {
        // Register all 4 required tools
        server.register_tool(self.search_tool());
        server.register_tool(self.clear_database_tool());
        server.register_tool(self.reindex_all_tool());
        server.register_tool(self.toggle_watch_tool());
    }
    
    fn search_tool(&self) -> Tool {
        Tool {
            name: "search_code".to_string(),
            description: "Search code with 3-chunk context. Handles any query complexity.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query (can be simple text or complex expression)"
                    }
                },
                "required": ["query"]
            }),
        }
    }
}
```

#### **Task 032: Search Tool Implementation**
**Goal**: Implement the search_code tool  
**Duration**: 3 hours  
**Dependencies**: Task 031

**Implementation**:
```rust
#[derive(Deserialize)]
struct SearchParams {
    query: String,
}

#[derive(Serialize)]
struct SearchResponse {
    results: Vec<SearchResultMCP>,
    total_found: usize,
    search_time_ms: u64,
}

#[derive(Serialize)]
struct SearchResultMCP {
    file: String,
    relevance_score: f32,
    match_type: String,
    three_chunk_context: ThreeChunkContextMCP,
}

#[derive(Serialize)]
struct ThreeChunkContextMCP {
    above_context: Option<ChunkMCP>,
    target_chunk: ChunkMCP,
    below_context: Option<ChunkMCP>,
}

#[derive(Serialize)]
struct ChunkMCP {
    content: String,
    start_line: usize,
    end_line: usize,
}

impl MCPServer {
    pub async fn handle_search(&self, params: SearchParams) -> ToolResult {
        let start = Instant::now();
        
        // Execute search
        let searcher = self.searcher.read().await;
        let results = searcher.search(&params.query, &self.project_path).await?;
        
        // Convert to MCP format
        let mcp_results: Vec<SearchResultMCP> = results.into_iter()
            .map(|r| SearchResultMCP {
                file: r.file,
                relevance_score: r.score,
                match_type: format!("{:?}", r.match_type),
                three_chunk_context: ThreeChunkContextMCP {
                    above_context: r.three_chunk_context.above.map(|c| ChunkMCP {
                        content: c.content,
                        start_line: c.start_line,
                        end_line: c.end_line,
                    }),
                    target_chunk: ChunkMCP {
                        content: r.three_chunk_context.target.content,
                        start_line: r.three_chunk_context.target.start_line,
                        end_line: r.three_chunk_context.target.end_line,
                    },
                    below_context: r.three_chunk_context.below.map(|c| ChunkMCP {
                        content: c.content,
                        start_line: c.start_line,
                        end_line: c.end_line,
                    }),
                },
            })
            .collect();
        
        let response = SearchResponse {
            results: mcp_results,
            total_found: results.len(),
            search_time_ms: start.elapsed().as_millis() as u64,
        };
        
        ToolResult::Success(serde_json::to_value(response)?)
    }
}
```

#### **Task 033: Clear Database Tool**
**Goal**: Implement clear_database to reset vector storage  
**Duration**: 2 hours  
**Dependencies**: Task 032

**Implementation**:
```rust
impl MCPServer {
    fn clear_database_tool(&self) -> Tool {
        Tool {
            name: "clear_database".to_string(),
            description: "Clear/reset the entire vector database. WARNING: This removes all embeddings!".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "confirm": {
                        "type": "boolean",
                        "description": "Must be true to confirm database clearing"
                    }
                },
                "required": ["confirm"]
            }),
        }
    }
    
    pub async fn handle_clear_database(&self, params: Value) -> ToolResult {
        let confirm = params.get("confirm")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if !confirm {
            return ToolResult::Error("Must confirm=true to clear database".to_string());
        }
        
        let mut storage = self.storage.write().await;
        
        // Delete all data
        storage.clear_all()?;
        
        // Reinitialize schema
        storage.init_schema()?;
        
        ToolResult::Success(json!({
            "status": "success",
            "message": "Vector database cleared and reset"
        }))
    }
}
```

#### **Task 034: Reindex All Tool**
**Goal**: Implement reindex_all to rebuild entire database  
**Duration**: 4 hours  
**Dependencies**: Task 033

**Implementation**:
```rust
impl MCPServer {
    fn reindex_all_tool(&self) -> Tool {
        Tool {
            name: "reindex_all".to_string(),
            description: "Re-embed and store all code files in a directory".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "Directory path to index (absolute or relative). If not provided, uses current project directory."
                    },
                    "show_progress": {
                        "type": "boolean",
                        "description": "Show progress during reindexing",
                        "default": true
                    }
                },
                "required": []
            }),
        }
    }
    
    pub async fn handle_reindex_all(&self, params: Value) -> ToolResult {
        let show_progress = params.get("show_progress")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
            
        // Get directory from params or use project path
        let directory = params.get("directory")
            .and_then(|v| v.as_str())
            .map(PathBuf::from)
            .unwrap_or_else(|| self.project_path.clone());
            
        // Resolve relative paths
        let target_dir = if directory.is_relative() {
            self.project_path.join(&directory)
        } else {
            directory
        };
        
        // Verify directory exists
        if !target_dir.exists() || !target_dir.is_dir() {
            return ToolResult::Error(format!("Directory not found: {}", target_dir.display()));
        }
        
        let start_time = Instant::now();
        let mut stats = ReindexStats::default();
        
        println!("Indexing directory: {}", target_dir.display());
        
        // Find all code files
        let files = self.find_all_code_files(&target_dir)?;
        stats.total_files = files.len();
        
        // Clear existing data first
        let mut storage = self.storage.write().await;
        storage.clear_all()?;
        storage.init_schema()?;
        drop(storage); // Release lock
        
        // Reindex all files
        let mut git_watch = self.git_watch.write().await;
        let mut processed = 0;
        
        for file in files {
            match git_watch.updater.index_file(&file).await {
                Ok(_) => {
                    stats.indexed_files += 1;
                    processed += 1;
                    
                    if show_progress && processed % 10 == 0 {
                        println!("Indexed {}/{} files...", processed, stats.total_files);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to index {}: {}", file.display(), e);
                    stats.failed_files += 1;
                }
            }
        }
        
        stats.total_time = start_time.elapsed();
        
        ToolResult::Success(json!({
            "status": "success",
            "stats": {
                "total_files": stats.total_files,
                "indexed_files": stats.indexed_files,
                "failed_files": stats.failed_files,
                "time_seconds": stats.total_time.as_secs(),
            }
        }))
    }
    
    fn find_all_code_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        for entry in WalkDir::new(dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !self.is_ignored(e.path())) 
        {
            let entry = entry?;
            if entry.file_type().is_file() && self.is_code_file(entry.path()) {
                files.push(entry.path().to_owned());
            }
        }
        
        Ok(files)
    }
}
```

#### **Task 035: Toggle Watch Tool**
**Goal**: Implement toggle_watch to control file monitoring  
**Duration**: 2 hours  
**Dependencies**: Task 034

**Implementation**:
```rust
impl MCPServer {
    fn toggle_watch_tool(&self) -> Tool {
        Tool {
            name: "toggle_watch".to_string(),
            description: "Turn file watching on or off".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "enabled": {
                        "type": "boolean",
                        "description": "true to enable watching, false to disable"
                    }
                },
                "required": ["enabled"]
            }),
        }
    }
    
    pub async fn handle_toggle_watch(&self, params: Value) -> ToolResult {
        let enabled = params.get("enabled")
            .and_then(|v| v.as_bool())
            .ok_or("enabled parameter required")?;
        
        let mut git_watch = self.git_watch.write().await;
        
        if enabled {
            git_watch.start_watching();
            ToolResult::Success(json!({
                "status": "success",
                "message": "File watching enabled",
                "check_interval": "5 seconds"
            }))
        } else {
            git_watch.stop_watching();
            ToolResult::Success(json!({
                "status": "success",
                "message": "File watching disabled"
            }))
        }
    }
}
```

### **Integration Tasks (036-040): Server Completion**

#### **Task 036: MCP Request Router**
**Goal**: Route incoming MCP requests to handlers  
**Duration**: 3 hours  
**Dependencies**: Task 035

**Implementation**:
```rust
impl MCPServer {
    pub async fn handle_request(&self, request: Request) -> ToolResult {
        match request.method.as_str() {
            "search_code" => {
                let params: SearchParams = serde_json::from_value(request.params)?;
                self.handle_search(params).await
            },
            "clear_database" => {
                self.handle_clear_database(request.params).await
            },
            "reindex_all" => {
                self.handle_reindex_all(request.params).await
            },
            "toggle_watch" => {
                self.handle_toggle_watch(request.params).await
            },
            _ => ToolResult::Error(format!("Unknown method: {}", request.method))
        }
    }
}
```

#### **Task 037: MCP Transport Layer**
**Goal**: Implement stdio transport for MCP  
**Duration**: 2 hours  
**Dependencies**: Task 036

#### **Task 038: Error Handling**
**Goal**: Robust error handling for all tools  
**Duration**: 2 hours  
**Dependencies**: Task 037

#### **Task 039: Tool Documentation**
**Goal**: Generate comprehensive tool docs  
**Duration**: 1 hour  
**Dependencies**: Task 038

#### **Task 040: Phase 4 Completion**
**Goal**: Full integration testing with LLMs  
**Duration**: 2 hours  
**Dependencies**: Task 039

## **SUCCESS CRITERIA**

### **Phase 4 Targets**
- **All Tools Working**: 4 tools fully functional
- **MCP Compliant**: Follows MCP protocol spec
- **Performance**: <1s response for all tools
- **Reliability**: Graceful error handling
- **LLM Ready**: Works with Claude, GPT, etc.

### **Tool Requirements**
- **search_code**: Returns 3-chunk contexts always
- **clear_database**: Requires confirmation
- **reindex_all**: Accepts directory parameter, shows progress, handles errors
- **toggle_watch**: Simple on/off control

## **FINAL ARCHITECTURE**

```rust
// Complete system
pub struct EmbeddingSearchSystem {
    server: MCPServer,
    transport: StdioTransport,
}

impl EmbeddingSearchSystem {
    pub async fn run() -> Result<()> {
        let project_path = std::env::current_dir()?;
        let mut server = MCPServer::new(project_path)?;
        
        // Set up MCP
        let mut mcp = Server::new("embedding-search");
        server.register_tools(&mut mcp);
        
        // Start stdio transport
        let transport = StdioTransport::new();
        transport.run(|request| async {
            server.handle_request(request).await
        }).await?;
        
        Ok(())
    }
}

// Usage
#[tokio::main]
async fn main() -> Result<()> {
    EmbeddingSearchSystem::run().await
}
```

## **WEEK 3-4 DELIVERABLES**

1. **MCP Server**: Full protocol implementation
2. **Search Tool**: Complex queries with 3-chunk context
3. **Management Tools**: Clear, reindex, toggle watch
4. **Complete System**: All phases integrated
5. **Production Ready**: 85% accuracy, <500ms search

**COMPLETE**: Simple, powerful code search with LLM integration!