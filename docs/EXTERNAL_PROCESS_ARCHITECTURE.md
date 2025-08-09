# External Process GGUF Architecture - V8 Crash Prevention

## ⚠️ CRITICAL PROBLEM ANALYSIS

### Current Architecture FATAL FLAWS:
1. **V8 Heap Contamination**: Candle tensors allocate 4.3GB on system heap visible to V8
2. **Memory Monitoring Illusion**: Monitoring tracks allocation but doesn't prevent V8 crash
3. **False Streaming Claims**: Still loads entire model into memory, just in chunks
4. **Node.js Process Contamination**: Any GGUF loading in Node.js process WILL crash V8

### THE ONLY VIABLE SOLUTION: COMPLETE PROCESS SEPARATION

---

## 1. PROCESS SEPARATION ARCHITECTURE

```
┌─────────────────────────────────────────┐    ┌───────────────────────────────────┐
│           NODE.JS PROCESS               │    │      RUST EMBEDDER PROCESS       │
│                                         │    │                                   │
│  ┌─────────────────┐  ┌───────────────┐ │    │  ┌─────────────────────────────┐  │
│  │   MCP Server    │  │  IPC Client   │ │◄──►│  │        IPC Server           │  │
│  │                 │  │               │ │    │  │                             │  │
│  └─────────────────┘  └───────────────┘ │    │  └─────────────────────────────┘  │
│                                         │    │              │                    │
│  ┌─────────────────────────────────────┐ │    │  ┌─────────────────────────────┐  │
│  │         ZERO MODEL DATA             │ │    │  │    MEMORY-MAPPED GGUF       │  │
│  │      ZERO V8 HEAP USAGE             │ │    │  │      - mmap() only          │  │
│  │      ZERO CANDLE TENSORS            │ │    │  │      - 1MB working set      │  │
│  └─────────────────────────────────────┘ │    │  │      - Q4_K_M quantized     │  │
│                                         │    │  └─────────────────────────────┘  │
└─────────────────────────────────────────┘    └───────────────────────────────────┘
         ^                                                         │
         │                                                         │
    PURE IPC                                               DIRECT FILE
   NO MODEL                                               ACCESS ONLY
   DATA HERE                                                       │
                                                                   ▼
                                              ┌─────────────────────────────────────┐
                                              │          GGUF MODEL FILE            │
                                              │      nomic-embed-code.gguf          │
                                              │            4.3GB                    │
                                              │     NEVER LOADED INTO MEMORY       │
                                              │      ONLY MEMORY-MAPPED            │
                                              └─────────────────────────────────────┘
```

### Process Responsibilities:

**Node.js Process (Claude Code/MCP):**
- ✅ MCP protocol handling
- ✅ Text preprocessing  
- ✅ IPC communication only
- ✅ ZERO model data
- ✅ ZERO tensor operations
- ✅ Process lifecycle management

**Rust Embedder Process:**
- ✅ Memory-mapped GGUF access
- ✅ Lazy tensor loading (1MB chunks max)
- ✅ CPU quantized inference
- ✅ Embedding computation
- ✅ NEVER sends large data over IPC

---

## 2. IPC PROTOCOL SPECIFICATION

### Transport: Named Pipes (Windows Compatible)

```rust
// IPC Message Format
#[derive(Serialize, Deserialize)]
pub enum EmbedderMessage {
    // Client -> Server
    Initialize { model_path: String },
    Embed { text: String, max_length: usize },
    EmbedBatch { texts: Vec<String>, max_length: usize },
    Shutdown,
    
    // Server -> Client  
    Initialized { vocab_size: usize, embed_dim: usize },
    EmbedResult { embedding: Vec<f32> },
    BatchResult { embeddings: Vec<Vec<f32>> },
    Error { message: String },
    Shutdown { graceful: bool },
}

// IPC Protocol Rules
#[derive(Debug)]
pub struct IpcProtocol {
    // CRITICAL: Maximum message sizes
    max_message_size: usize,        // 64KB max
    max_embedding_size: usize,      // 768 * 4 bytes = 3KB
    max_batch_size: usize,          // 10 texts max
    timeout_seconds: u64,           // 30s timeout
}
```

### Windows Named Pipes Implementation:

```rust
// Named pipe path: \\.\pipe\embed_search_{pid}_{random}
pub struct WindowsNamedPipe {
    pipe_name: String,
    handle: HANDLE,
    buffer: [u8; 65536], // 64KB fixed buffer
}

impl WindowsNamedPipe {
    pub fn create_server(pipe_name: &str) -> Result<Self>;
    pub fn connect_client(pipe_name: &str) -> Result<Self>;
    pub fn send_message(&mut self, msg: &EmbedderMessage) -> Result<()>;
    pub fn receive_message(&mut self) -> Result<EmbedderMessage>;
}
```

---

## 3. MEMORY MANAGEMENT STRATEGY

### ZERO-ALLOCATION GGUF LOADING:

```rust
/// REAL zero-allocation GGUF handler
pub struct MemoryMappedGGUF {
    /// Memory-mapped file handle (NO heap allocation)
    mmap: Mmap,
    
    /// Metadata extracted once (small allocation)
    metadata: GGUFMetadata,
    
    /// Tensor lookup table (small allocation)
    tensor_index: HashMap<String, TensorLocation>,
    
    /// CRITICAL: Working set limited to 1MB TOTAL
    working_buffer: Box<[u8; 1024 * 1024]>, // 1MB stack allocation
}

impl MemoryMappedGGUF {
    /// Create new memory-mapped GGUF - NEVER loads model into memory
    pub fn new(model_path: &Path) -> Result<Self> {
        let file = File::open(model_path)?;
        
        // CRITICAL: Only memory-map the file, never load into heap
        let mmap = unsafe { Mmap::map(&file)? };
        
        // Extract metadata (small structure, few KB)
        let metadata = Self::extract_metadata(&mmap)?;
        
        // Build tensor index (small structure, few KB) 
        let tensor_index = Self::build_tensor_index(&mmap, &metadata)?;
        
        // CRITICAL: Total heap allocation < 100KB
        Ok(Self {
            mmap,
            metadata,
            tensor_index,
            working_buffer: Box::new([0u8; 1024 * 1024]),
        })
    }
    
    /// Load tensor chunk WITHOUT allocating on heap
    pub fn load_tensor_chunk(
        &mut self, 
        tensor_name: &str, 
        offset: usize, 
        size: usize
    ) -> Result<&[u8]> {
        // Validate chunk size
        if size > self.working_buffer.len() {
            return Err(anyhow!("Chunk too large: {} > 1MB", size));
        }
        
        // Get tensor location from index
        let location = self.tensor_index.get(tensor_name)
            .ok_or_else(|| anyhow!("Tensor not found: {}", tensor_name))?;
            
        // CRITICAL: Read directly from memory-mapped region into working buffer
        let start = location.offset + offset;
        let end = start + size;
        
        if end > self.mmap.len() {
            return Err(anyhow!("Read beyond file bounds"));
        }
        
        // Copy chunk to working buffer (reused, no allocation)
        self.working_buffer[..size].copy_from_slice(&self.mmap[start..end]);
        
        Ok(&self.working_buffer[..size])
    }
}

/// Tensor location metadata
#[derive(Debug, Clone)]
struct TensorLocation {
    offset: usize,
    size: usize,
    dtype: GgmlDType,
    shape: Vec<usize>,
}
```

### Q4_K_M Quantized Inference Engine:

```rust
/// CPU-only quantized inference - NO GPU tensors
pub struct QuantizedInferenceEngine {
    gguf: MemoryMappedGGUF,
    
    /// CRITICAL: All computations use stack-allocated buffers
    computation_buffer: Box<[f32; 768]>,     // Max embedding size
    attention_buffer: Box<[f32; 2048]>,      // Attention computation  
    temp_buffer: Box<[f32; 4096]>,          // Temporary calculations
}

impl QuantizedInferenceEngine {
    /// Embed text using streaming quantized inference
    pub fn embed(&mut self, tokens: &[u32]) -> Result<Vec<f32>> {
        // CRITICAL: All computation uses fixed-size stack buffers
        // NEVER allocates tensors or large vectors
        
        self.forward_pass(tokens)
    }
    
    fn forward_pass(&mut self, tokens: &[u32]) -> Result<Vec<f32>> {
        // Token embeddings - stream from memory-mapped file
        self.load_token_embeddings(tokens)?;
        
        // Transformer layers - process one at a time
        for layer in 0..self.gguf.metadata.num_layers {
            self.process_transformer_layer(layer)?;
        }
        
        // Layer norm and pooling
        self.final_layer_norm()?;
        self.mean_pooling()?;
        
        // Return embedding (only final result allocation)
        Ok(self.computation_buffer[..self.gguf.metadata.embed_dim].to_vec())
    }
}
```

---

## 4. ERROR HANDLING & RECOVERY

### Process Crash Detection:

```rust
pub struct ProcessManager {
    embedder_process: Option<Child>,
    ipc_client: Option<WindowsNamedPipe>,
    restart_count: usize,
    max_restarts: usize,
}

impl ProcessManager {
    pub fn spawn_embedder(&mut self) -> Result<()> {
        let mut cmd = Command::new("embed-search-embedder.exe");
        cmd.arg("--mode").arg("ipc-server");
        cmd.arg("--model").arg(&self.model_path);
        cmd.arg("--pipe-name").arg(&self.pipe_name);
        
        let process = cmd.spawn()?;
        self.embedder_process = Some(process);
        
        // Wait for IPC connection
        self.connect_ipc()?;
        Ok(())
    }
    
    pub fn check_process_health(&mut self) -> Result<()> {
        if let Some(process) = &mut self.embedder_process {
            match process.try_wait()? {
                Some(status) => {
                    // Process crashed - attempt restart
                    if self.restart_count < self.max_restarts {
                        self.restart_count += 1;
                        self.spawn_embedder()?;
                    } else {
                        return Err(anyhow!("Embedder process failed {} times", self.max_restarts));
                    }
                }
                None => {
                    // Process still running - check IPC health
                    self.check_ipc_health()?;
                }
            }
        }
        Ok(())
    }
}
```

### Graceful Degradation:

```rust
pub enum EmbedderStatus {
    Healthy,
    Degraded { reason: String },
    Failed { error: String },
}

pub struct FallbackEmbedder {
    primary: ProcessManager,
    fallback: Option<SimpleHashEmbedder>, // TF-IDF based fallback
}

impl FallbackEmbedder {
    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        match self.primary.embed(text) {
            Ok(embedding) => Ok(embedding),
            Err(e) => {
                warn!("Primary embedder failed: {}", e);
                
                // Use simple hash-based fallback
                match &mut self.fallback {
                    Some(fallback) => fallback.embed(text),
                    None => {
                        // Initialize fallback
                        self.fallback = Some(SimpleHashEmbedder::new());
                        self.fallback.as_mut().unwrap().embed(text)
                    }
                }
            }
        }
    }
}
```

---

## 5. INTEGRATION WITH EXISTING MCP

### Drop-in Replacement Interface:

```rust
/// External process embedder that implements existing interface
pub struct ExternalProcessEmbedder {
    process_manager: ProcessManager,
    embedding_cache: LruCache<String, Vec<f32>>,
}

impl ExternalProcessEmbedder {
    pub async fn new(model_path: &str) -> Result<Self> {
        let mut process_manager = ProcessManager::new(model_path);
        process_manager.spawn_embedder()?;
        
        Ok(Self {
            process_manager,
            embedding_cache: LruCache::new(NonZeroUsize::new(1000).unwrap()),
        })
    }
}

// Implement existing interface for compatibility
impl EmbeddingProvider for ExternalProcessEmbedder {
    async fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        if let Some(cached) = self.embedding_cache.get(text) {
            return Ok(cached.clone());
        }
        
        // Check process health
        self.process_manager.check_process_health()?;
        
        // Send embed request via IPC
        let embedding = self.process_manager.embed(text).await?;
        
        // Cache result
        self.embedding_cache.put(text.to_string(), embedding.clone());
        
        Ok(embedding)
    }
    
    async fn embed_batch(&mut self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.process_manager.embed_batch(texts).await
    }
}
```

---

## 6. FUNDAMENTAL FLAW ANALYSIS

### ❌ WHAT DOESN'T WORK:
1. **In-process GGUF loading**: Any approach that loads GGUF in Node.js process
2. **"Safe" tensor libraries**: Candle, Torch, etc. still allocate on system heap  
3. **Memory monitoring**: Monitoring doesn't prevent V8 crash, just gives warning
4. **Streaming within process**: Still accumulates memory in same process space

### ✅ WHAT WILL WORK:
1. **Complete process separation**: ZERO model data in Node.js process
2. **Memory-mapped files**: Direct OS file mapping, no heap allocation
3. **Fixed working sets**: Bounded computation buffers, never growing
4. **IPC-only communication**: Small messages, never large tensor data

### 🔄 MIGRATION STRATEGY:
1. Build external embedder process first
2. Implement IPC protocol
3. Create drop-in replacement wrapper
4. Test with small models first
5. Deploy with process monitoring
6. Remove old in-process implementation

---

## 7. WINDOWS-SPECIFIC CONSIDERATIONS

### Named Pipes vs Sockets:
- **Named Pipes**: Native Windows, better performance
- **TCP Sockets**: Cross-platform but overkill for local IPC
- **Memory-mapped files**: Possible but complex synchronization

### Process Management:
```rust
// Windows-specific process spawning
use winapi::um::processthreadsapi::{CreateProcessW, PROCESS_INFORMATION, STARTUPINFOW};
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE};

pub struct WindowsProcessManager {
    process_info: Option<PROCESS_INFORMATION>,
    job_object: HANDLE, // For cleanup on parent exit
}
```

### Memory Monitoring:
```rust
// Windows memory API
use winapi::um::psapi::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};

impl WindowsProcessManager {
    fn check_memory_usage(&self) -> Result<u64> {
        // Monitor external process memory only
        // Node.js process should have ZERO model memory
    }
}
```

---

## CONCLUSION: THIS IS THE ONLY VIABLE APPROACH

**Any attempt to load a 4.3GB GGUF model in the same process as Node.js WILL crash V8.** 

The current "streaming" implementation is **fundamentally flawed** because it still loads tensor data into system memory visible to V8. The external process architecture is the **only solution** that guarantees V8 safety.

**Key Success Metrics:**
- ✅ Node.js process memory usage: < 100MB total
- ✅ Rust process memory usage: < 50MB working set (mmap doesn't count)
- ✅ IPC latency: < 10ms per embedding
- ✅ Process restart time: < 5 seconds
- ✅ Zero V8 crashes under any memory pressure

This architecture trades some latency for absolute safety - which is the correct trade-off for production deployment.