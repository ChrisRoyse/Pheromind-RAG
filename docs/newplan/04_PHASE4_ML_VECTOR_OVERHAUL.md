# Phase 4: ML/Vector Overhaul - Making Embeddings Actually Work

**Duration**: 2 days  
**Goal**: Get Nomic Embed v1.5 GGUF working with Candle and LanceDB  
**Success Metric**: Can generate embeddings and perform semantic search

## The Reality Check

The ML system has 8+ compilation errors. We need to:
1. Fix all type mismatches
2. Get the GGUF model loading
3. Make LanceDB actually store vectors
4. Implement proper caching

## Task 4.1: Download and Verify Nomic Model (2 hours)

### Download the Correct Model

```bash
# Create model directory
mkdir -p models

# Download Nomic Embed v1.5 GGUF (Q4_K_M quantized)
curl -L https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF/resolve/main/nomic-embed-text-v1.5.Q4_K_M.gguf \
     -o models/nomic-embed-text-v1.5.Q4_K_M.gguf

# Verify file (should be ~137MB)
ls -lh models/
```

### Update Model Path

```rust
// File: src/embedding/nomic.rs

const MODEL_PATH: &str = "models/nomic-embed-text-v1.5.Q4_K_M.gguf";
const MODEL_REPO: &str = "nomic-ai/nomic-embed-text-v1.5-GGUF";
const MODEL_FILE: &str = "nomic-embed-text-v1.5.Q4_K_M.gguf";

impl NomicEmbedder {
    pub fn new() -> Result<Self> {
        // Check if model exists locally first
        let model_path = Path::new(MODEL_PATH);
        if !model_path.exists() {
            return Err(EmbedError::ModelNotFound(MODEL_PATH.to_string()));
        }
        
        // Load GGUF model with Candle
        let device = Device::Cpu;  // Start with CPU, CUDA later
        let model = load_gguf_model(model_path, &device)?;
        
        Ok(Self {
            model,
            device,
            dimension: 768,  // Nomic v1.5 is 768-dimensional
        })
    }
}
```

## Task 4.2: Fix GGUF Loading with Candle (3 hours)

### Implement GGUF Loader

```rust
// File: src/embedding/gguf_loader.rs

use candle_core::{Device, Tensor, DType};
use std::fs::File;
use std::io::Read;

pub struct GGUFModel {
    pub embeddings: Tensor,
    pub layers: Vec<TransformerLayer>,
    pub config: ModelConfig,
}

pub struct ModelConfig {
    pub hidden_size: usize,      // 768 for Nomic
    pub num_layers: usize,        // 12 for Nomic
    pub num_heads: usize,         // 12 for Nomic
    pub max_seq_length: usize,    // 8192 for Nomic
    pub vocab_size: usize,        // 30522 for Nomic
}

pub fn load_gguf_model(path: &Path, device: &Device) -> Result<GGUFModel> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    // Parse GGUF header
    let magic = &buffer[0..4];
    if magic != b"GGUF" {
        return Err(EmbedError::InvalidModel("Not a GGUF file".into()));
    }
    
    // Parse version
    let version = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
    if version != 3 {
        return Err(EmbedError::InvalidModel(format!("Unsupported GGUF version: {}", version)));
    }
    
    // Parse tensors
    let tensors = parse_gguf_tensors(&buffer)?;
    
    // Build model
    Ok(GGUFModel {
        embeddings: tensors.get("token_embeddings.weight")?.to_device(device)?,
        layers: build_transformer_layers(&tensors, device)?,
        config: ModelConfig {
            hidden_size: 768,
            num_layers: 12,
            num_heads: 12,
            max_seq_length: 8192,
            vocab_size: 30522,
        },
    })
}
```

### Test GGUF Loading

```rust
#[test]
fn test_gguf_model_loading() {
    let model_path = Path::new("models/nomic-embed-text-v1.5.Q4_K_M.gguf");
    assert!(model_path.exists(), "Model file not found");
    
    let device = Device::Cpu;
    let model = load_gguf_model(model_path, &device).unwrap();
    
    assert_eq!(model.config.hidden_size, 768);
    assert_eq!(model.config.num_layers, 12);
}
```

## Task 4.3: Fix LanceDB Storage (2 hours)

### Remove Broken Sled Code

```rust
// File: src/storage/lancedb.rs

// DELETE ALL SLED-RELATED CODE
// Remove migration_db field and all methods using it

use lancedb::{Connection, Table};
use arrow::array::{Float32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};

pub struct LanceDBStorage {
    connection: Connection,
    table: Table,
    dimension: usize,
}

impl LanceDBStorage {
    pub async fn new(path: &Path, dimension: usize) -> Result<Self> {
        // Connect to LanceDB
        let connection = Connection::new(path).await?;
        
        // Create or open table
        let schema = Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("content", DataType::Utf8, false),
            Field::new("embedding", DataType::FixedSizeList(
                Box::new(Field::new("item", DataType::Float32, true)),
                dimension as i32,
            ), false),
        ]);
        
        let table = connection
            .create_table("embeddings", schema)
            .await
            .or_else(|_| connection.open_table("embeddings").await)?;
        
        Ok(Self {
            connection,
            table,
            dimension,
        })
    }
    
    pub async fn add_embedding(
        &mut self,
        id: &str,
        content: &str,
        embedding: &[f32],
    ) -> Result<()> {
        if embedding.len() != self.dimension {
            return Err(StorageError::InvalidVector {
                expected: self.dimension,
                actual: embedding.len(),
            });
        }
        
        // Create record batch
        let id_array = StringArray::from(vec![id]);
        let content_array = StringArray::from(vec![content]);
        let embedding_array = Float32Array::from(embedding.to_vec());
        
        // Insert into table
        self.table.add_data(vec![
            Arc::new(id_array),
            Arc::new(content_array),
            Arc::new(embedding_array),
        ]).await?;
        
        Ok(())
    }
    
    pub async fn search(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // Perform vector similarity search
        let results = self.table
            .search(query_embedding)
            .limit(limit)
            .execute()
            .await?;
        
        Ok(results)
    }
}
```

## Task 4.4: Fix Embedding Cache (1 hour)

### Fix Type Mismatches

```rust
// File: src/embedding/cache.rs

use lru::LruCache;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct EmbeddingCache {
    cache: Arc<RwLock<LruCache<String, Vec<f32>>>>,
    max_size: usize,
}

impl EmbeddingCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(max_size))),
            max_size,
        }
    }
    
    pub async fn get(&self, key: &str) -> Option<Vec<f32>> {
        let mut cache = self.cache.write().await;
        cache.get(key).cloned()
    }
    
    pub async fn insert(&self, key: String, value: Vec<f32>) {
        let mut cache = self.cache.write().await;
        cache.put(key, value);
    }
}
```

### Fix Usage in Nomic

```rust
// File: src/embedding/nomic.rs

// FIX the cache usage
pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
    let cache_key = format!("{:x}", md5::compute(text));
    
    // Check cache - FIXED TYPE HANDLING
    if let Some(cached) = self.cache.get(&cache_key).await {
        return Ok(cached);
    }
    
    // Generate embedding
    let embedding = self.generate_embedding_internal(text)?;
    
    // Store in cache
    self.cache.insert(cache_key, embedding.clone()).await;
    
    Ok(embedding)
}
```

## Task 4.5: Implement Embedding Generation (3 hours)

### Core Embedding Logic

```rust
// File: src/embedding/nomic.rs

impl NomicEmbedder {
    fn generate_embedding_internal(&self, text: &str) -> Result<Vec<f32>> {
        // Tokenize with instruction prefix
        let input_text = format!("search_document: {}", text);
        let tokens = self.tokenizer.encode(&input_text, true)?;
        
        // Convert to tensor
        let input_ids = Tensor::new(
            tokens.get_ids(),
            &self.device,
        )?;
        
        // Get token embeddings
        let embeddings = self.model.embeddings.index_select(
            &input_ids,
            0,
        )?;
        
        // Pass through transformer layers
        let mut hidden_states = embeddings;
        for layer in &self.model.layers {
            hidden_states = layer.forward(&hidden_states)?;
        }
        
        // Mean pooling
        let pooled = hidden_states.mean(0)?;
        
        // L2 normalization
        let norm = pooled.sqr()?.sum_all()?.sqrt()?;
        let normalized = pooled.div(&norm)?;
        
        // Convert to Vec<f32>
        let embedding: Vec<f32> = normalized.to_vec1()?;
        
        // Verify dimension
        if embedding.len() != 768 {
            return Err(EmbedError::InvalidDimension {
                expected: 768,
                actual: embedding.len(),
            });
        }
        
        Ok(embedding)
    }
}
```

## Task 4.6: Integration Testing (2 hours)

### End-to-End Test

```rust
#[tokio::test]
async fn test_ml_pipeline_end_to_end() {
    // Initialize components
    let embedder = NomicEmbedder::new().await.unwrap();
    let storage = LanceDBStorage::new("test_vectors", 768).await.unwrap();
    
    // Test documents
    let docs = vec![
        ("doc1", "The quick brown fox jumps over the lazy dog"),
        ("doc2", "Machine learning is fascinating"),
        ("doc3", "Rust programming language is fast and safe"),
    ];
    
    // Generate and store embeddings
    for (id, content) in &docs {
        let embedding = embedder.generate_embedding(content).await.unwrap();
        storage.add_embedding(id, content, &embedding).await.unwrap();
    }
    
    // Search with query
    let query = "programming languages";
    let query_embedding = embedder.generate_embedding(query).await.unwrap();
    let results = storage.search(&query_embedding, 2).await.unwrap();
    
    // Verify results
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].id, "doc3");  // Should be most similar
}
```

### Performance Test

```rust
#[tokio::test]
async fn test_embedding_performance() {
    let embedder = NomicEmbedder::new().await.unwrap();
    
    let start = Instant::now();
    let embedding = embedder.generate_embedding("test text").await.unwrap();
    let duration = start.elapsed();
    
    println!("Embedding generation took: {:?}", duration);
    assert!(duration < Duration::from_millis(100));  // Should be < 100ms
    
    // Test cache performance
    let start = Instant::now();
    let cached = embedder.generate_embedding("test text").await.unwrap();
    let cache_duration = start.elapsed();
    
    println!("Cached retrieval took: {:?}", cache_duration);
    assert!(cache_duration < Duration::from_millis(1));  // Should be < 1ms
}
```

## Success Criteria

- [ ] GGUF model loads successfully
- [ ] Can generate 768-dimensional embeddings
- [ ] Embeddings are L2 normalized
- [ ] LanceDB stores vectors without errors
- [ ] Vector similarity search returns relevant results
- [ ] Cache improves performance by 100x
- [ ] All ML tests pass
- [ ] Performance meets targets (<100ms generation)

## Performance Targets

- Embedding generation: <100ms
- Cached retrieval: <1ms
- Batch processing: 10 texts/second
- Storage insertion: <10ms
- Similarity search: <50ms for 100k vectors

## Common Issues and Solutions

1. **Model not found**: Ensure model is downloaded to correct path
2. **Out of memory**: Use smaller batch sizes or enable disk swap
3. **Slow generation**: Check if using CPU instead of GPU
4. **Wrong dimensions**: Verify model is Nomic v1.5 (768 dims)
5. **Cache misses**: Check cache key generation

## Next Phase

Proceed to Phase 5 (Integration Surgery) only after ML pipeline works end-to-end.