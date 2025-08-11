use std::io::{self, BufRead, BufReader, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::{json, Value};
use anyhow::Result;
use tracing::{info, error, warn, debug};

use embed_search::{HybridSearch, SymbolExtractor};

#[derive(Debug)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

pub struct EmbedSearchMCPServer {
    search_engine: Arc<Mutex<Option<HybridSearch>>>,
    symbol_extractor: Arc<Mutex<SymbolExtractor>>,
    db_path: String,
}

impl EmbedSearchMCPServer {
    pub fn new() -> Result<Self> {
        let db_path = "./simple_embed.db".to_string();
        let symbol_extractor = SymbolExtractor::new()?;
        
        Ok(Self {
            search_engine: Arc::new(Mutex::new(None)),
            symbol_extractor: Arc::new(Mutex::new(symbol_extractor)),
            db_path,
        })
    }

    async fn ensure_search_engine(&self) -> Result<()> {
        let mut engine = self.search_engine.lock().await;
        if engine.is_none() {
            *engine = Some(HybridSearch::new(&self.db_path).await?);
        }
        Ok(())
    }

    pub fn get_tools(&self) -> Vec<MCPTool> {
        vec![
            MCPTool {
                name: "embed_search".to_string(),
                description: "Search through indexed code using hybrid semantic and text search".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query text"
                        },
                        "limit": {
                            "type": "integer", 
                            "description": "Maximum number of results to return",
                            "default": 10,
                            "minimum": 1,
                            "maximum": 50
                        },
                        "search_type": {
                            "type": "string",
                            "description": "Type of search to perform",
                            "enum": ["hybrid", "semantic", "text", "symbol"],
                            "default": "hybrid"
                        }
                    },
                    "required": ["query"]
                }),
            },
            MCPTool {
                name: "embed_index".to_string(),
                description: "Index files in a directory for searching".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Directory path to index"
                        },
                        "file_extensions": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "File extensions to include (e.g., ['rs', 'py', 'js'])",
                            "default": ["rs", "py", "js", "ts"]
                        },
                        "max_file_size": {
                            "type": "integer",
                            "description": "Maximum file size in bytes to index",
                            "default": 100000
                        }
                    },
                    "required": ["path"]
                }),
            },
            MCPTool {
                name: "embed_extract_symbols".to_string(),
                description: "Extract code symbols (functions, classes, etc.) from source code".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "Source code to analyze"
                        },
                        "file_extension": {
                            "type": "string", 
                            "description": "File extension (rs, py, js, ts)",
                            "enum": ["rs", "py", "js", "ts"]
                        }
                    },
                    "required": ["code", "file_extension"]
                }),
            },
            MCPTool {
                name: "embed_status".to_string(),
                description: "Get status and health information about the search system".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            MCPTool {
                name: "embed_clear".to_string(),
                description: "Clear all indexed data from the search system".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "confirm": {
                            "type": "boolean",
                            "description": "Confirm deletion of all indexed data",
                            "default": false
                        }
                    }
                }),
            }
        ]
    }

    pub async fn handle_tool_call(&self, tool_name: &str, arguments: Value) -> Result<Value> {
        match tool_name {
            "embed_search" => self.handle_search(arguments).await,
            "embed_index" => self.handle_index(arguments).await,
            "embed_extract_symbols" => self.handle_extract_symbols(arguments).await,
            "embed_status" => self.handle_status().await,
            "embed_clear" => self.handle_clear(arguments).await,
            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name))
        }
    }

    async fn handle_search(&self, args: Value) -> Result<Value> {
        let query = args["query"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?;
        
        let limit = args["limit"].as_u64().unwrap_or(10) as usize;
        let search_type = args["search_type"].as_str().unwrap_or("hybrid");
        
        info!("Performing {} search for: {}", search_type, query);
        
        self.ensure_search_engine().await?;
        let mut engine = self.search_engine.lock().await;
        let engine = engine.as_mut().ok_or_else(|| anyhow::anyhow!("Search engine not initialized"))?;
        
        let results = match search_type {
            "hybrid" => engine.search(query, limit).await?,
            "semantic" | "text" => engine.search(query, limit).await?, // For now, both use hybrid
            "symbol" => {
                // Extract symbols from query context if available
                let mut symbol_extractor = self.symbol_extractor.lock().await;
                let symbols = if let Some(code) = args["code"].as_str() {
                    let ext = args["file_extension"].as_str().unwrap_or("rs");
                    symbol_extractor.extract(code, ext)?
                } else {
                    vec![]
                };
                
                // For symbol search, we'd typically search through indexed symbols
                // For now, fall back to regular search
                engine.search(query, limit).await?
            },
            _ => return Err(anyhow::anyhow!("Unknown search type: {}", search_type))
        };

        let response = json!({
            "results": results.iter().map(|r| json!({
                "content": r.content,
                "file_path": r.file_path,
                "score": r.score,
                "match_type": r.match_type
            })).collect::<Vec<_>>(),
            "total": results.len(),
            "search_type": search_type,
            "query": query
        });

        info!("Search completed: {} results", results.len());
        Ok(response)
    }

    async fn handle_index(&self, args: Value) -> Result<Value> {
        let path = args["path"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        
        let file_extensions: Vec<String> = args["file_extensions"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_else(|| vec!["rs".to_string(), "py".to_string(), "js".to_string(), "ts".to_string()]);
        
        let max_file_size = args["max_file_size"].as_u64().unwrap_or(100000) as usize;
        
        info!("Starting indexing of path: {} with extensions: {:?}", path, file_extensions);
        
        self.ensure_search_engine().await?;
        let mut engine = self.search_engine.lock().await;
        let engine = engine.as_mut().ok_or_else(|| anyhow::anyhow!("Search engine not initialized"))?;
        
        use walkdir::WalkDir;
        use std::fs;
        
        let mut total_files = 0;
        let mut indexed_files = 0;
        let mut skipped_files = 0;
        
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            total_files += 1;
            
            let file_path = entry.path();
            let extension = file_path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");
            
            if !file_extensions.contains(&extension.to_string()) {
                continue;
            }
            
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    if content.len() > max_file_size {
                        warn!("Skipping large file: {} ({} bytes)", file_path.display(), content.len());
                        skipped_files += 1;
                        continue;
                    }
                    
                    // Index single file
                    let file_path_str = file_path.display().to_string();
                    if let Err(e) = engine.index(vec![content], vec![file_path_str]).await {
                        warn!("Failed to index file {}: {}", file_path.display(), e);
                        skipped_files += 1;
                    } else {
                        indexed_files += 1;
                        if indexed_files % 10 == 0 {
                            info!("Indexed {} files so far...", indexed_files);
                        }
                    }
                },
                Err(e) => {
                    debug!("Could not read file {}: {}", file_path.display(), e);
                    skipped_files += 1;
                }
            }
        }

        let response = json!({
            "status": "completed",
            "total_files_found": total_files,
            "files_indexed": indexed_files,
            "files_skipped": skipped_files,
            "path": path,
            "file_extensions": file_extensions
        });

        info!("Indexing completed: {}/{} files indexed", indexed_files, total_files);
        Ok(response)
    }

    async fn handle_extract_symbols(&self, args: Value) -> Result<Value> {
        let code = args["code"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: code"))?;
        let file_extension = args["file_extension"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: file_extension"))?;
        
        debug!("Extracting symbols from {} code ({} chars)", file_extension, code.len());
        
        let mut extractor = self.symbol_extractor.lock().await;
        let symbols = extractor.extract(code, file_extension)?;
        
        let response = json!({
            "symbols": symbols.iter().map(|s| json!({
                "name": s.name,
                "kind": format!("{:?}", s.kind),
                "line": s.line,
                "definition": s.definition
            })).collect::<Vec<_>>(),
            "total": symbols.len(),
            "file_extension": file_extension
        });

        info!("Extracted {} symbols", symbols.len());
        Ok(response)
    }

    async fn handle_status(&self) -> Result<Value> {
        let engine_initialized = self.search_engine.lock().await.is_some();
        
        // Check if database exists
        let db_exists = std::path::Path::new(&self.db_path).exists();
        
        let response = json!({
            "status": "healthy",
            "search_engine_initialized": engine_initialized,
            "database_path": self.db_path,
            "database_exists": db_exists,
            "available_tools": self.get_tools().iter().map(|t| &t.name).collect::<Vec<_>>(),
            "supported_languages": ["rust", "python", "javascript", "typescript"],
            "version": env!("CARGO_PKG_VERSION")
        });

        Ok(response)
    }

    async fn handle_clear(&self, args: Value) -> Result<Value> {
        let confirm = args["confirm"].as_bool().unwrap_or(false);
        
        if !confirm {
            return Ok(json!({
                "status": "confirmation_required",
                "message": "Set 'confirm': true to clear all indexed data"
            }));
        }
        
        info!("Clearing all indexed data");
        
        self.ensure_search_engine().await?;
        let mut engine = self.search_engine.lock().await;
        let engine = engine.as_mut().ok_or_else(|| anyhow::anyhow!("Search engine not initialized"))?;
        
        engine.clear().await?;
        
        let response = json!({
            "status": "cleared",
            "message": "All indexed data has been cleared"
        });

        info!("All indexed data cleared successfully");
        Ok(response)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .init();

    info!("Starting Embed Search MCP Server v{}", env!("CARGO_PKG_VERSION"));
    
    let server = EmbedSearchMCPServer::new()?;
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    // MCP initialization handshake
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "embed-search-mcp",
                "version": env!("CARGO_PKG_VERSION")
            }
        }
    });
    
    // Send server info
    let server_info = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": "embed-search-mcp",
                "version": env!("CARGO_PKG_VERSION")
            }
        }
    });
    
    info!("MCP Server initialized and ready");
    
    let reader = BufReader::new(stdin);
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }
                
                match serde_json::from_str::<Value>(&line) {
                    Ok(request) => {
                        let response = handle_request(&server, request).await;
                        
                        let response_json = serde_json::to_string(&response)?;
                        writeln!(stdout, "{}", response_json)?;
                        stdout.flush()?;
                    },
                    Err(e) => {
                        error!("Failed to parse JSON: {}", e);
                        let error_response = json!({
                            "jsonrpc": "2.0",
                            "id": null,
                            "error": {
                                "code": -32700,
                                "message": "Parse error",
                                "data": format!("{}", e)
                            }
                        });
                        let response_json = serde_json::to_string(&error_response)?;
                        writeln!(stdout, "{}", response_json)?;
                        stdout.flush()?;
                    }
                }
            },
            Err(e) => {
                error!("Failed to read line: {}", e);
                break;
            }
        }
    }
    
    info!("MCP Server shutting down");
    Ok(())
}

async fn handle_request(server: &EmbedSearchMCPServer, request: Value) -> Value {
    let method = request["method"].as_str().unwrap_or("");
    let id = request["id"].clone();
    
    match method {
        "initialize" => {
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        }
                    },
                    "serverInfo": {
                        "name": "embed-search-mcp",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }
            })
        },
        
        "tools/list" => {
            let tools = server.get_tools();
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": tools.iter().map(|tool| json!({
                        "name": tool.name,
                        "description": tool.description,
                        "inputSchema": tool.input_schema
                    })).collect::<Vec<_>>()
                }
            })
        },
        
        "tools/call" => {
            let tool_name = request["params"]["name"].as_str().unwrap_or("");
            let arguments = request["params"]["arguments"].clone();
            
            match server.handle_tool_call(tool_name, arguments).await {
                Ok(result) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{
                            "type": "text",
                            "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Error serializing result".to_string())
                        }]
                    }
                }),
                Err(e) => {
                    error!("Tool call failed: {}", e);
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -1,
                            "message": "Tool execution failed",
                            "data": format!("{}", e)
                        }
                    })
                }
            }
        },
        
        _ => {
            warn!("Unknown method: {}", method);
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": "Method not found",
                    "data": format!("Unknown method: {}", method)
                }
            })
        }
    }
}