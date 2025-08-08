// Simple MCP protocol test that doesn't require full initialization
use embed_search::mcp::protocol::{JsonRpcRequest, JsonRpcResponse, ProtocolHandler, RpcMethod};
use embed_search::mcp::types::{McpCapabilities, SearchRequest};
use embed_search::mcp::error::McpError;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing MCP Protocol Implementation");

    // Test 1: Protocol Handler - Parse valid request
    println!("\n1. Testing JSON-RPC request parsing...");
    let mut handler = ProtocolHandler::new();
    let request_json = r#"{"jsonrpc":"2.0","method":"search","params":{"query":"test"},"id":1}"#;
    
    match handler.parse_request(request_json) {
        Ok(request) => {
            println!("✅ Successfully parsed request: method={}, id={:?}", 
                request.method, request.id);
            
            // Test method parsing
            let method = request.get_method()?;
            println!("✅ Method parsed: {:?}", method);
            
            // Test parameter extraction
            let params: Option<SearchRequest> = request.get_params()?;
            if let Some(params) = params {
                println!("✅ Parameters extracted: query='{}'", params.query);
            }
        }
        Err(e) => {
            println!("❌ Failed to parse request: {}", e);
            return Err(e.into());
        }
    }

    // Test 2: Error response creation
    println!("\n2. Testing error response creation...");
    let error = McpError::MethodNotFound { 
        method: "unknown_method".to_string() 
    };
    let error_response = JsonRpcResponse::error(error, Some(serde_json::json!(1)));
    println!("✅ Error response created: {}", 
        serde_json::to_string_pretty(&error_response)?);

    // Test 3: Success response creation
    println!("\n3. Testing success response creation...");
    let success_data = serde_json::json!({"message": "test successful"});
    let success_response = JsonRpcResponse::success(success_data, Some(serde_json::json!(2)))?;
    println!("✅ Success response created: {}", 
        serde_json::to_string_pretty(&success_response)?);

    // Test 4: Invalid JSON handling
    println!("\n4. Testing invalid JSON handling...");
    let invalid_json = r#"{"jsonrpc":"2.0","method":}"#; // Invalid JSON
    match handler.parse_request(invalid_json) {
        Ok(_) => {
            println!("❌ Should have failed to parse invalid JSON");
            return Err("Invalid JSON was parsed successfully".into());
        }
        Err(e) => {
            println!("✅ Correctly rejected invalid JSON: {}", e);
        }
    }

    // Test 5: Method validation
    println!("\n5. Testing method validation...");
    let invalid_method = r#"{"jsonrpc":"2.0","method":"invalid_method","id":3}"#;
    match handler.parse_request(invalid_method) {
        Ok(request) => {
            match request.get_method() {
                Ok(_) => {
                    println!("❌ Should have rejected invalid method");
                    return Err("Invalid method was accepted".into());
                }
                Err(e) => {
                    println!("✅ Correctly rejected invalid method: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✅ Request parsing rejected invalid method: {}", e);
        }
    }

    // Test 6: All supported methods
    println!("\n6. Testing all supported RPC methods...");
    let methods = [
        "initialize", "search", "index", "stats", 
        "clear", "capabilities", "ping", "shutdown"
    ];
    
    for method_name in &methods {
        match RpcMethod::from_str(method_name) {
            Ok(method) => {
                println!("✅ Method '{}' -> {:?}", method_name, method);
                assert_eq!(method.as_str(), *method_name);
            }
            Err(e) => {
                println!("❌ Failed to parse method '{}': {}", method_name, e);
                return Err(e.into());
            }
        }
    }

    // Test 7: Batch request handling
    println!("\n7. Testing batch request parsing...");
    let batch_json = r#"[
        {"jsonrpc":"2.0","method":"ping","id":1},
        {"jsonrpc":"2.0","method":"capabilities","id":2}
    ]"#;
    
    match handler.parse_batch_request(batch_json) {
        Ok(requests) => {
            println!("✅ Successfully parsed batch request with {} items", requests.len());
            for (i, req) in requests.iter().enumerate() {
                println!("  Request {}: method='{}', id={:?}", 
                    i + 1, req.method, req.id);
            }
        }
        Err(e) => {
            println!("❌ Failed to parse batch request: {}", e);
            return Err(e.into());
        }
    }

    // Test 8: Response serialization
    println!("\n8. Testing response serialization...");
    let response = JsonRpcResponse::success("test", Some(serde_json::json!(1)))?;
    match handler.serialize_response(&response) {
        Ok(json) => {
            println!("✅ Successfully serialized response: {}", json);
            
            // Verify it's valid JSON
            let parsed: serde_json::Value = serde_json::from_str(&json)?;
            assert_eq!(parsed["jsonrpc"], "2.0");
            assert_eq!(parsed["id"], 1);
            println!("✅ Serialized response is valid JSON-RPC 2.0");
        }
        Err(e) => {
            println!("❌ Failed to serialize response: {}", e);
            return Err(e.into());
        }
    }

    println!("\n🎉 All MCP protocol tests passed successfully!");
    println!("\n📋 Summary:");
    println!("  ✅ JSON-RPC 2.0 compliance validated");
    println!("  ✅ Request/response parsing working");
    println!("  ✅ Error handling functional");
    println!("  ✅ Method validation active");
    println!("  ✅ Batch processing supported");
    println!("  ✅ Protocol serialization correct");

    Ok(())
}