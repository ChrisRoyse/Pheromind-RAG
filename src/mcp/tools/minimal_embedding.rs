//! Minimal embedding MCP tool - hash-based vector generation
//!
//! This tool provides deterministic, V8-safe embedding generation using
//! hash-based vectors instead of ML models.

use serde_json::{json, Value};
use crate::embedding::MinimalEmbedder;
use crate::mcp::{McpError, McpResult};
use crate::mcp::protocol::JsonRpcResponse;

/// Execute minimal embedding generation
pub async fn execute_minimal_embed(
    params: &Value,
    id: Option<Value>,
) -> McpResult<JsonRpcResponse> {
    let text = params.get("text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidRequest {
            message: "Missing required 'text' parameter".to_string(),
        })?;
    
    let embedder = MinimalEmbedder::new();
    let embedding = embedder.embed(text);
    
    let response = json!({
        "embedding": embedding,
        "dimension": embedder.dimension(),
        "text_length": text.len(),
        "method": "hash-based",
        "deterministic": true,
        "memory_safe": true
    });
    
    JsonRpcResponse::success(response, id)
}

/// Get minimal embedder status and capabilities
pub async fn execute_minimal_embedder_status(
    _params: &Value,
    id: Option<Value>,
) -> McpResult<JsonRpcResponse> {
    let embedder = MinimalEmbedder::new();
    
    let response = json!({
        "type": "minimal-hash",
        "status": "active",
        "dimension": embedder.dimension(),
        "description": "Hash-based embedder (40 lines vs 138,000)",
        "capabilities": {
            "deterministic": true,
            "memory_safe": true,
            "v8_safe": true,
            "zero_dependencies": true,
            "instant_startup": true
        },
        "trade_offs": {
            "pros": [
                "Zero crashes",
                "Deterministic output",
                "Fast generation",
                "Tiny memory footprint",
                "No model files needed"
            ],
            "cons": [
                "No semantic understanding",
                "Similarity based on text patterns only",
                "Lower quality than ML embeddings"
            ]
        }
    });
    
    JsonRpcResponse::success(response, id)
}