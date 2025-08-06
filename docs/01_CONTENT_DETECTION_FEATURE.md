# Phase 1: Regex + Embeddings Foundation

## **PHASE OVERVIEW - CORE FOUNDATION**

**GOAL**: Build regex chunking + MiniLM embeddings + 3-chunk context  
**APPROACH**: Fast chunking, single model embeddings, vector storage  
**MEASUREMENT**: Basic accuracy testing with simple queries  
**TIMELINE**: Week 1 (Tasks 001-010)

## **KEY INSIGHT: SIMPLICITY + CONTEXT**

**CORE COMPONENTS**:
1. **Regex Chunking**: Fast pattern-based chunking
2. **MiniLM Embeddings**: all-MiniLM-L6-v2 for all embeddings
3. **3-Chunk Context**: Always return above + target + below
4. **LanceDB Storage**: Simple vector database

**Foundation Requirements**:
- **Regex Patterns**: Basic patterns for code boundaries
- **Single Model**: MiniLM only (no complex routing)
- **Vector Storage**: Set up LanceDB with proper schema
- **Basic Testing**: Simple queries to validate system

## **FOUNDATION TASK BREAKDOWN (001-010)**

### **Core Setup Tasks (001-005): Regex + Embeddings**

#### **Task 001: Basic Regex Patterns**
**Goal**: Simple regex patterns for code chunking  
**Duration**: 3 hours  
**Dependencies**: None

**Implementation**:
```rust
pub struct SimpleRegexChunker {
    // Simple patterns for common code structures
    function_pattern: Regex,
    class_pattern: Regex,
    chunk_size_target: usize, // ~100 lines per chunk
}

impl SimpleRegexChunker {
    pub fn new() -> Self {
        Self {
            // Universal patterns that work across languages
            function_pattern: Regex::new(r"^\s*(pub|public|private|protected|static|async|def|function|fn|func)\s+\w+").unwrap(),
            class_pattern: Regex::new(r"^\s*(class|struct|interface|enum)\s+\w+").unwrap(),
            chunk_size_target: 100,
        }
    }
    
    pub fn chunk_file(&self, content: &str) -> Vec<Chunk> {
        // Simple line-based chunking with pattern hints
        let lines: Vec<&str> = content.lines().collect();
        let mut chunks = Vec::new();
        let mut current_chunk = Vec::new();
        let mut start_line = 0;
        
        for (i, line) in lines.iter().enumerate() {
            current_chunk.push(*line);
            
            // Start new chunk on pattern match or size limit
            if self.is_chunk_boundary(line) || current_chunk.len() >= self.chunk_size_target {
                if !current_chunk.is_empty() {
                    chunks.push(Chunk {
                        content: current_chunk.join("\n"),
                        start_line,
                        end_line: i,
                    });
                    current_chunk.clear();
                    start_line = i + 1;
                }
            }
        }
        
        // Don't forget the last chunk
        if !current_chunk.is_empty() {
            chunks.push(Chunk {
                content: current_chunk.join("\n"),
                start_line,
                end_line: lines.len() - 1,
            });
        }
        
        chunks
    }
}
```

#### **Task 002: MiniLM Embedder Setup**  
**Goal**: Set up all-MiniLM-L6-v2 embedding model  
**Duration**: 3 hours  
**Dependencies**: None

**Implementation**:
```rust
use candle::{Device, Tensor};
use tokenizers::Tokenizer;

pub struct MiniLMEmbedder {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl MiniLMEmbedder {
    pub fn new() -> Result<Self> {
        // Load all-MiniLM-L6-v2
        let device = Device::Cpu;
        let model_path = "models/all-MiniLM-L6-v2";
        
        let config = Config::from_file(&format!("{}/config.json", model_path))?;
        let model = BertModel::load(&format!("{}/model.safetensors", model_path), &config, &device)?;
        let tokenizer = Tokenizer::from_file(&format!("{}/tokenizer.json", model_path))?;
        
        Ok(Self { model, tokenizer, device })
    }
    
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Tokenize
        let encoding = self.tokenizer.encode(text, true)?;
        let input_ids = Tensor::new(encoding.get_ids(), &self.device)?;
        
        // Get embeddings
        let embeddings = self.model.forward(&input_ids)?;
        
        // Mean pooling
        let pooled = self.mean_pool(&embeddings)?;
        
        // Convert to Vec<f32>
        Ok(pooled.to_vec1()?)
    }
    
    pub fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        texts.iter()
            .map(|text| self.embed(text))
            .collect()
    }
}
```

#### **Task 003: LanceDB Setup**
**Goal**: Set up LanceDB for vector storage  
**Duration**: 2 hours  
**Dependencies**: Task 002

**Implementation**:
```rust
use lance::dataset::Dataset;
use lance::table::Table;

pub struct VectorStorage {
    db_path: PathBuf,
    dataset: Option<Dataset>,
}

impl VectorStorage {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        Ok(Self {
            db_path,
            dataset: None,
        })
    }
    
    pub fn init_schema(&mut self) -> Result<()> {
        // Create schema for embeddings
        let schema = Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("file_path", DataType::Utf8, false),
            Field::new("chunk_index", DataType::Int32, false),
            Field::new("content", DataType::Utf8, false),
            Field::new("embedding", DataType::FixedSizeList(
                Box::new(Field::new("item", DataType::Float32, true)),
                384, // MiniLM dimension
            ), false),
            Field::new("start_line", DataType::Int32, false),
            Field::new("end_line", DataType::Int32, false),
        ]);
        
        // Create dataset
        self.dataset = Some(Dataset::create(&self.db_path, schema)?);        
        Ok(())
    }
    
    pub fn insert_embedding(&mut self, file_path: &str, chunk_idx: usize, chunk: &Chunk, embedding: Vec<f32>) -> Result<()> {
        let batch = RecordBatch::try_new(
            self.dataset.as_ref().unwrap().schema(),
            vec![
                Arc::new(StringArray::from(vec![format!("{}-{}", file_path, chunk_idx)])),
                Arc::new(StringArray::from(vec![file_path])),
                Arc::new(Int32Array::from(vec![chunk_idx as i32])),
                Arc::new(StringArray::from(vec![&chunk.content])),
                Arc::new(FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                    vec![Some(embedding)],
                    384
                )),
                Arc::new(Int32Array::from(vec![chunk.start_line as i32])),
                Arc::new(Int32Array::from(vec![chunk.end_line as i32])),
            ],
        )?;
        
        self.dataset.as_mut().unwrap().append_batch(batch)?;
        Ok(())
    }
}
```

#### **Task 004: Three-Chunk Expander**
**Goal**: Implement 3-chunk context expansion  
**Duration**: 2 hours  
**Dependencies**: Task 001

**Implementation**:
```rust
pub struct ThreeChunkExpander;

pub struct ThreeChunkResult {
    pub above: Option<Chunk>,
    pub target: Chunk,
    pub below: Option<Chunk>,
}

impl ThreeChunkExpander {
    pub fn expand(&self, chunks: &[Chunk], target_idx: usize) -> ThreeChunkResult {
        ThreeChunkResult {
            above: if target_idx > 0 { 
                Some(chunks[target_idx - 1].clone()) 
            } else { 
                None 
            },
            target: chunks[target_idx].clone(),
            below: if target_idx < chunks.len() - 1 { 
                Some(chunks[target_idx + 1].clone()) 
            } else { 
                None 
            },
        }
    }
    
    pub fn format_for_display(&self, result: &ThreeChunkResult) -> String {
        let mut output = String::new();
        
        if let Some(above) = &result.above {
            output.push_str("// === ABOVE CONTEXT ===\n");
            output.push_str(&above.content);
            output.push_str("\n\n");
        }
        
        output.push_str("// === TARGET CHUNK ===\n");
        output.push_str(&result.target.content);
        output.push_str("\n");
        
        if let Some(below) = &result.below {
            output.push_str("\n// === BELOW CONTEXT ===\n");
            output.push_str(&below.content);
        }
        
        output
    }
}
```

#### **Task 005: Initial Indexing**
**Goal**: Index a test codebase with embeddings  
**Duration**: 3 hours  
**Dependencies**: Tasks 001-004

### **Integration Tasks (006-010): Bringing It Together**

#### **Task 006: Tantivy Integration**
**Goal**: Set up tantivy for exact text search and fuzzy matching  
**Duration**: 2 hours  
**Dependencies**: None

**Implementation**:
```rust
use std::process::Command;

pub struct TantivySearcher;

impl TantivySearcher {
    pub fn search(&self, query: &str, path: &Path) -> Result<Vec<Match>> {
        // Use tantivy for text search with fuzzy matching
        let results = self.tantivy_engine.search(query)?;
        
        let mut matches = Vec::new();
        
        for line in output.stdout.lines() {
            if let Ok(json) = serde_json::from_str::<RgMatch>(&line?) {
                if json.type_field == "match" {
                    matches.push(Match {
                        file: json.data.path.text,
                        line_number: json.data.line_number,
                        content: json.data.lines.text,
                    });
                }
            }
        }
        
        Ok(matches)
    }
}
```

#### **Task 007: Simple Search API**
**Goal**: Create unified search interface  
**Duration**: 3 hours  
**Dependencies**: Tasks 001-006

**Implementation**:
```rust
pub struct SimpleSearcher {
    chunker: SimpleRegexChunker,
    embedder: MiniLMEmbedder,
    storage: VectorStorage,
    expander: ThreeChunkExpander,
    tantivy: TantivySearcher,
}

impl SimpleSearcher {
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        // 1. Exact and fuzzy search with tantivy
        let exact_matches = self.tantivy.search(query, &self.project_path)?;
        
        // 2. Semantic search
        let query_embedding = self.embedder.embed(query)?;
        let semantic_matches = self.storage.search_similar(query_embedding, 20)?;
        
        // 3. Simple fusion (covered in phase 2)
        let mut all_results = Vec::new();
        
        // For now, just return exact matches with 3-chunk context
        for match_result in exact_matches {
            let file_content = std::fs::read_to_string(&match_result.file)?;
            let chunks = self.chunker.chunk_file(&file_content);
            let chunk_idx = self.find_chunk_for_line(&chunks, match_result.line_number);
            let three_chunk = self.expander.expand(&chunks, chunk_idx);
            
            all_results.push(SearchResult {
                file: match_result.file,
                three_chunk_context: three_chunk,
                score: 1.0, // Exact match
            });
        }
        
        Ok(all_results)
    }
}
```

#### **Task 008: Basic Testing**
**Goal**: Test chunking, embedding, and search  
**Duration**: 2 hours  
**Dependencies**: Task 007

#### **Task 009: Performance Optimization**
**Goal**: Basic caching and performance tuning  
**Duration**: 2 hours  
**Dependencies**: Task 008

#### **Task 010: Phase 1 Completion**
**Goal**: Verify all components work together  
**Duration**: 1 hour  
**Dependencies**: Task 009


## **SUCCESS CRITERIA**

### **Phase 1 Targets**
- **Regex Chunking**: Working chunker with ~100 line chunks
- **MiniLM Setup**: Embedder producing 384-dim vectors
- **Vector Storage**: LanceDB initialized and storing embeddings
- **3-Chunk Context**: All results include context
- **Basic Search**: Tantivy integration working

### **Performance Requirements**
- Chunking: <50ms per file
- Embedding: <100ms per chunk
- Storage: <10ms per insert
- Memory: <1GB for model

## **SIMPLE ARCHITECTURE**

```rust
// Core components for Phase 1
pub struct Phase1Foundation {
    pub chunker: SimpleRegexChunker,
    pub embedder: MiniLMEmbedder,
    pub storage: VectorStorage,
    pub expander: ThreeChunkExpander,
    pub searcher: SimpleSearcher,
}

// Basic types
#[derive(Clone)]
pub struct Chunk {
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
}

pub struct SearchResult {
    pub file: String,
    pub three_chunk_context: ThreeChunkResult,
    pub score: f32,
}

pub struct Match {
    pub file: String,
    pub line_number: usize,
    pub content: String,
}
```

## **WEEK 1 DELIVERABLES**

1. **Working Chunker**: Simple regex-based chunking
2. **MiniLM Embeddings**: Single model setup complete  
3. **Vector Storage**: LanceDB storing embeddings
4. **3-Chunk Context**: Every result has context
5. **Basic Search**: Tantivy integration functional

**Next Phase**: Simple fusion and improved search accuracy