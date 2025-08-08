/// Comprehensive MCP Server Verification Script
/// 
/// This script performs a complete verification of the MCP server implementation,
/// testing all tools, functionality, and integration points.
/// 
/// BRUTAL HONESTY: This verifies what ACTUALLY works vs what is claimed.

use std::process::{Command, Stdio};
use std::io::{Write, BufRead, BufReader};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use serde_json::{json, Value};

const TIMEOUT_SECONDS: u64 = 30;

#[derive(Debug)]
struct VerificationResult {
    test_name: String,
    success: bool,
    error: Option<String>,
    duration: Duration,
    response: Option<Value>,
}

struct McpVerifier {
    server_process: Option<std::process::Child>,
    temp_dir: TempDir,
    request_id: i32,
    binary_path: PathBuf,
}

impl McpVerifier {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let binary_path = PathBuf::from("target/release/mcp_server.exe");
        
        if !binary_path.exists() {
            return Err(format!("MCP server binary not found at {}", binary_path.display()).into());
        }
        
        Ok(Self {
            server_process: None,
            temp_dir,
            request_id: 1,
            binary_path,
        })
    }
    
    fn start_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Starting MCP server...");
        
        let mut child = Command::new(&self.binary_path)
            .arg(self.temp_dir.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        
        // Give server time to initialize
        std::thread::sleep(Duration::from_millis(1000));
        
        // Check if process is still running
        if let Ok(Some(exit_status)) = child.try_wait() {
            return Err(format!("Server process exited immediately with status: {}", exit_status).into());
        }
        
        self.server_process = Some(child);
        println!("✅ MCP server started successfully");
        Ok(())
    }
    
    fn send_request(&mut self, method: &str, params: Option<Value>) -> Result<Value, Box<dyn std::error::Error>> {
        let process = self.server_process.as_mut()
            .ok_or("Server process not started")?;
        
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params.unwrap_or(json!({})),
            "id": self.request_id
        });
        
        self.request_id += 1;
        
        // Send request
        if let Some(stdin) = process.stdin.as_mut() {
            writeln!(stdin, "{}", request)?;
            stdin.flush()?;
        } else {
            return Err("Failed to write to server stdin".into());
        }
        
        // Read response with timeout
        if let Some(stdout) = process.stdout.as_mut() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            
            // Use a timeout mechanism
            let start = Instant::now();
            while start.elapsed() < Duration::from_secs(TIMEOUT_SECONDS) {
                if let Ok(line) = lines.next().ok_or("No response from server")? {
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    return Ok(serde_json::from_str(&line)?);
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            
            Err("Server response timeout".into())
        } else {
            Err("Failed to read from server stdout".into())
        }
    }
    
    fn verify_tool(&mut self, tool_name: &str, method: &str, params: Option<Value>) -> VerificationResult {
        let start = Instant::now();
        
        match self.send_request(method, params) {
            Ok(response) => {
                let success = response.get("result").is_some() && response.get("error").is_none();
                VerificationResult {
                    test_name: tool_name.to_string(),
                    success,
                    error: if success { None } else { Some(format!("Server returned error: {:?}", response.get("error"))) },
                    duration: start.elapsed(),
                    response: Some(response),
                }
            }
            Err(e) => VerificationResult {
                test_name: tool_name.to_string(),
                success: false,
                error: Some(e.to_string()),
                duration: start.elapsed(),
                response: None,
            }
        }
    }
    
    fn stop_server(&mut self) {
        if let Some(mut process) = self.server_process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
    }
    
    fn create_test_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create some test files to index and search
        let src_dir = self.temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir)?;
        
        // Rust file
        std::fs::write(
            src_dir.join("main.rs"),
            r#"
fn main() {
    println!("Hello, world!");
}

struct TestStruct {
    field: String,
}

impl TestStruct {
    fn new() -> Self {
        Self { field: "test".to_string() }
    }
    
    fn search_function(&self) -> &str {
        &self.field
    }
}
"#
        )?;
        
        // Python file
        std::fs::write(
            src_dir.join("test.py"),
            r#"
def search_function():
    return "test result"

class TestClass:
    def __init__(self):
        self.field = "test"
    
    def method(self):
        return self.field
"#
        )?;
        
        // JSON file
        std::fs::write(
            src_dir.join("config.json"),
            r#"{"name": "test", "version": "1.0.0", "search": true}"#
        )?;
        
        println!("📁 Created test files in {}", src_dir.display());
        Ok(())
    }
}

impl Drop for McpVerifier {
    fn drop(&mut self) {
        self.stop_server();
    }
}

/// Count the actual number of MCP tools implemented
fn count_implemented_tools() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let tools_dir = PathBuf::from("src/mcp/tools");
    let mut tools = Vec::new();
    
    // Look for tool implementation files
    let tool_files = [
        ("search", "search.rs"),
        ("index", "index.rs"),
        ("status", "status.rs"),
        ("clear", "clear.rs"),
        ("orchestrated_search", "orchestrated_search.rs"),
        ("watcher", "watcher.rs"),
    ];
    
    for (tool_name, file_name) in &tool_files {
        let file_path = tools_dir.join(file_name);
        if file_path.exists() {
            let content = std::fs::read_to_string(&file_path)?;
            
            // Count the number of methods/functions in each tool
            let method_count = content.matches("pub async fn").count();
            tools.push(format!("{} ({} methods)", tool_name, method_count));
        }
    }
    
    Ok(tools)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 MCP SERVER COMPREHENSIVE VERIFICATION");
    println!("========================================");
    
    // Step 1: Verify binary exists
    println!("\n1️⃣ Binary Verification");
    let binary_path = PathBuf::from("target/release/mcp_server.exe");
    if binary_path.exists() {
        let metadata = std::fs::metadata(&binary_path)?;
        println!("✅ MCP server binary exists: {} ({:.1} MB)", 
                 binary_path.display(), 
                 metadata.len() as f64 / 1_048_576.0);
    } else {
        println!("❌ MCP server binary NOT found at {}", binary_path.display());
        return Err("Binary not found. Run: cargo build --release --bin mcp_server --features=\"mcp\"".into());
    }
    
    // Step 2: Count tools
    println!("\n2️⃣ Tool Implementation Count");
    let tools = count_implemented_tools()?;
    println!("📊 Implemented tools: {}", tools.len());
    for (i, tool) in tools.iter().enumerate() {
        println!("   {}. {}", i + 1, tool);
    }
    
    // Step 3: Start server and test functionality
    println!("\n3️⃣ Functional Verification");
    let mut verifier = McpVerifier::new()?;
    
    // Create test files
    verifier.create_test_files()?;
    
    // Start the server
    verifier.start_server()?;
    
    let mut results = Vec::new();
    
    // Test 1: Initialize
    println!("\n🧪 Testing initialize...");
    let result = verifier.verify_tool("initialize", "initialize", Some(json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {}
    })));
    println!("   Result: {} ({}ms)", if result.success { "✅ PASS" } else { "❌ FAIL" }, result.duration.as_millis());
    if let Some(ref error) = result.error {
        println!("   Error: {}", error);
    }
    results.push(result);
    
    // Test 2: Search
    println!("\n🔍 Testing search...");
    let result = verifier.verify_tool("search", "tools/call", Some(json!({
        "name": "search",
        "arguments": {
            "query": "search_function",
            "max_results": 10
        }
    })));
    println!("   Result: {} ({}ms)", if result.success { "✅ PASS" } else { "❌ FAIL" }, result.duration.as_millis());
    if let Some(ref error) = result.error {
        println!("   Error: {}", error);
    }
    results.push(result);
    
    // Test 3: Index
    println!("\n📚 Testing index...");
    let result = verifier.verify_tool("index", "tools/call", Some(json!({
        "name": "index_directory",
        "arguments": {
            "path": verifier.temp_dir.path().to_str().unwrap(),
            "recursive": true
        }
    })));
    println!("   Result: {} ({}ms)", if result.success { "✅ PASS" } else { "❌ FAIL" }, result.duration.as_millis());
    if let Some(ref error) = result.error {
        println!("   Error: {}", error);
    }
    results.push(result);
    
    // Test 4: Status
    println!("\n📊 Testing status...");
    let result = verifier.verify_tool("status", "tools/call", Some(json!({
        "name": "get_status",
        "arguments": {}
    })));
    println!("   Result: {} ({}ms)", if result.success { "✅ PASS" } else { "❌ FAIL" }, result.duration.as_millis());
    if let Some(ref error) = result.error {
        println!("   Error: {}", error);
    }
    results.push(result);
    
    // Test 5: Clear
    println!("\n🧹 Testing clear...");
    let result = verifier.verify_tool("clear", "tools/call", Some(json!({
        "name": "clear_index",
        "arguments": {}
    })));
    println!("   Result: {} ({}ms)", if result.success { "✅ PASS" } else { "❌ FAIL" }, result.duration.as_millis());
    if let Some(ref error) = result.error {
        println!("   Error: {}", error);
    }
    results.push(result);
    
    // Step 4: Results Summary
    println!("\n4️⃣ VERIFICATION SUMMARY");
    println!("======================");
    
    let total_tests = results.len();
    let passed_tests = results.iter().filter(|r| r.success).count();
    let failed_tests = total_tests - passed_tests;
    
    println!("📈 Total tests: {}", total_tests);
    println!("✅ Passed: {}", passed_tests);
    println!("❌ Failed: {}", failed_tests);
    println!("🎯 Success rate: {:.1}%", (passed_tests as f64 / total_tests as f64) * 100.0);
    
    if failed_tests > 0 {
        println!("\n❌ FAILED TESTS:");
        for result in results.iter().filter(|r| !r.success) {
            println!("   • {}: {}", result.test_name, result.error.as_ref().unwrap_or(&"Unknown error".to_string()));
        }
    }
    
    // Final assessment
    println!("\n🏆 FINAL ASSESSMENT");
    println!("==================");
    
    if passed_tests == total_tests {
        println!("🎉 ALL TESTS PASSED - MCP server is fully functional!");
        println!("✅ Binary compiles: YES");
        println!("✅ Server starts: YES");
        println!("✅ Tools working: {}/{}", passed_tests, total_tests);
        println!("✅ Production ready: LIKELY");
    } else if passed_tests > 0 {
        println!("⚠️  PARTIAL FUNCTIONALITY - Some issues detected");
        println!("✅ Binary compiles: YES");
        println!("✅ Server starts: YES");
        println!("⚠️  Tools working: {}/{}", passed_tests, total_tests);
        println!("❌ Production ready: NO (needs fixes)");
    } else {
        println!("❌ MAJOR ISSUES - Server not functional");
        println!("✅ Binary compiles: YES");
        println!("❌ Server starts: UNKNOWN");
        println!("❌ Tools working: 0/{}", total_tests);
        println!("❌ Production ready: NO");
    }
    
    Ok(())
}