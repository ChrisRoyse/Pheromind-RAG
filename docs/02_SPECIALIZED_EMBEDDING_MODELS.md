# Phase 2: Single Model Implementation

## **PHASE OVERVIEW - PROVEN TECHNOLOGY APPROACH**

**GOAL**: Add semantic search with single proven embedding model  
**APPROACH**: Implement one high-quality embedding model with vector storage  
**VALIDATION**: Measure semantic search accuracy against Phase 1 baseline  
**TIMELINE**: 4 weeks (Tasks 016-030)

## **KEY INSIGHT: SINGLE MODEL FOCUS**

**APPROACH**: Implement one proven model exceptionally well rather than multiple models poorly  
**OPTIONS**: Choose ONE of the best available embedding models  
**VALIDATION**: Measure against baseline before considering additional complexity

**Model Selection Criteria**:
- **Proven Performance**: Documented success on code similarity tasks
- **Reasonable Cost**: Either free (local) or affordable API costs (<$25/day)  
- **Reliable Access**: High uptime and stable API
- **Good Documentation**: Clear integration path and known limitations

## **FOCUSED TASK BREAKDOWN (016-030)**

### **Model Selection Tasks (016-020): Choose Optimal Model**

#### **Task 016: Embedding Model Research and Selection**
**Goal**: Research and select the single best embedding model for code search  
**Duration**: 8 hours (important decision)  
**Dependencies**: Phase 1 completion

**Research Tasks**:
1. **Evaluate OpenAI text-embedding-3-small**: Cost, performance, reliability
2. **Evaluate CodeT5 local models**: Resource requirements, accuracy, setup complexity  
3. **Evaluate Sentence Transformers**: all-MiniLM-L6-v2, all-mpnet-base-v2
4. **Decision Matrix**: Create comparison across cost, accuracy, complexity, reliability
5. **Final Selection**: Choose ONE model based on evidence and project constraints

#### **Task 017: Model Prototype Implementation**
**Goal**: Create minimal prototype of selected embedding model  
**Duration**: 6 hours  
**Dependencies**: Task 016

#### **Task 018: Model Performance Benchmarking**
**Goal**: Benchmark model performance on code similarity tasks  
**Duration**: 4 hours  
**Dependencies**: Task 017

#### **Task 019: Cost and Resource Analysis**
**Goal**: Measure actual costs and resource usage  
**Duration**: 2 hours  
**Dependencies**: Task 018

#### **Task 020: Model Selection Validation**
**Goal**: Final validation of model choice against requirements  
**Duration**: 2 hours  
**Dependencies**: Task 019

### **Core Implementation Tasks (021-025): Model Integration**

#### **Task 021: Production Model Implementation**
**Goal**: Implement chosen embedding model for production use  
**Duration**: 8 hours  
**Dependencies**: Task 020

```rust
// Example for OpenAI embedding model
pub struct EmbeddingModel {
    client: OpenAIClient,
    model_name: String,
    cache: LRU<String, Vec<f32>>,
    cost_tracker: CostTracker,
}

impl EmbeddingModel {
    pub async fn encode(&mut self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        if let Some(cached) = self.cache.get(text) {
            return Ok(cached.clone());
        }
        
        // Generate embedding
        let response = self.client.create_embedding(CreateEmbeddingRequest {
            model: self.model_name.clone(),
            input: text.to_string(),
        }).await?;
        
        let embedding = response.data[0].embedding.clone();
        
        // Track cost and cache
        self.cost_tracker.record_request(response.usage.total_tokens);
        self.cache.put(text.to_string(), embedding.clone());
        
        Ok(embedding)
    }
}
```

#### **Task 022: Vector Storage Integration**
**Goal**: Implement LanceDB vector storage for embeddings  
**Duration**: 6 hours  
**Dependencies**: Task 021

**TDD Cycle**:
1. **RED Phase**: Test vector storage fails to store and retrieve embeddings
2. **GREEN Phase**: Basic LanceDB integration with simple similarity search
3. **REFACTOR Phase**: Optimize for performance and add batch operations

```rust
pub struct VectorStorage {
    db: LanceDB,
    table_name: String,
    dimension: usize,
}

impl VectorStorage {
    pub fn new(db_path: &Path, dimension: usize) -> Result<Self> {
        let db = LanceDB::connect(db_path)?;
        let table_name = "code_embeddings".to_string();
        
        // Create table if not exists
        if !db.table_exists(&table_name)? {
            db.create_table(&table_name, dimension)?;
        }
        
        Ok(Self {
            db,
            table_name,
            dimension,
        })
    }
    
    pub fn add_embeddings(&mut self, documents: Vec<Document>) -> Result<()> {
        let vectors: Vec<VectorRecord> = documents.into_iter()
            .map(|doc| VectorRecord {
                id: doc.id,
                vector: doc.embedding,
                metadata: doc.metadata,
            })
            .collect();
            
        self.db.insert(&self.table_name, vectors)?;
        Ok(())
    }
    
    pub fn search_similar(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<SimilarityResult>> {
        self.db.similarity_search(&self.table_name, query_embedding, limit)
    }
}
```

#### **Task 023: Codebase Indexing System**
**Goal**: Build system to generate embeddings for entire codebase  
**Duration**: 6 hours  
**Dependencies**: Task 022

#### **Task 024: Semantic Search Implementation**
**Goal**: Implement semantic search using embeddings and vector storage  
**Duration**: 4 hours  
**Dependencies**: Task 023

#### **Task 025: Integration with Baseline System**
**Goal**: Integrate semantic search with Phase 1 baseline system  
**Duration**: 4 hours  
**Dependencies**: Task 024

### **Validation Tasks (026-030): Semantic Search Validation**

#### **Task 026: Accuracy Measurement Against Baseline**
**Goal**: Measure semantic search accuracy vs Phase 1 baseline  
**Duration**: 4 hours  
**Dependencies**: Task 025

#### **Task 027: Performance Benchmarking**
**Goal**: Measure latency and resource usage of semantic search  
**Duration**: 2 hours  
**Dependencies**: Task 026

#### **Task 028: Error Analysis and Improvement**
**Goal**: Analyze where semantic search fails and identify improvements  
**Duration**: 4 hours  
**Dependencies**: Task 027

#### **Task 029: Cost and Resource Optimization**
**Goal**: Optimize embedding model usage for cost and resource efficiency  
**Duration**: 3 hours  
**Dependencies**: Task 028

#### **Task 030: Phase 2 System Validation**
**Goal**: Final validation that semantic search adds measurable value  
**Duration**: 1 hour  
**Dependencies**: Task 029

## **SUCCESS CRITERIA**

### **Phase 2 Targets**
- **Semantic Search Implementation**: Working embedding-based search system
- **Accuracy Improvement**: Measurable improvement over Phase 1 baseline
- **Performance**: <1s search latency including embedding generation
- **Resource Usage**: <8GB memory, <$25/day API costs
- **Integration**: Seamless integration with baseline exact search

### **Model Selection Requirements**
- **Documented Performance**: Published benchmarks on code similarity tasks
- **Reliability**: >99% uptime or robust local fallback
- **Cost Efficiency**: Sustainable cost structure for target usage
- **Technical Feasibility**: Clear integration path with known dependencies

## **ARCHITECTURE**

```rust
pub struct SemanticSearchSystem {
    // Core components
    embedding_model: EmbeddingModel,
    vector_storage: VectorStorage,
    indexing_system: CodebaseIndexer,
    
    // Integration with baseline
    baseline_search: BaselineSearchSystem,
    result_merger: SearchResultMerger,
    
    // Performance and monitoring
    performance_monitor: PerformanceMonitor,
    cost_tracker: CostTracker,
}

impl SemanticSearchSystem {
    pub async fn search(&mut self, query: &str, project_path: &Path) -> Result<SearchResult> {
        // Run both exact and semantic search in parallel
        let (baseline_future, semantic_future) = tokio::join!(
            self.baseline_search.search(query, project_path),
            self.semantic_search(query)
        );
        
        // Merge results intelligently
        let baseline_results = baseline_future?;
        let semantic_results = semantic_future?;
        
        let merged = self.result_merger.merge(baseline_results, semantic_results);
        
        Ok(merged)
    }
    
    async fn semantic_search(&mut self, query: &str) -> Result<Vec<SimilarityResult>> {
        // Generate query embedding
        let query_embedding = self.embedding_model.encode(query).await?;
        
        // Search for similar vectors
        let similar = self.vector_storage.search_similar(query_embedding, 50)?;
        
        Ok(similar)
    }
}
```

## **OPTIMIZATION RESULTS**

**BEFORE (Dual Model Strategy)**:
- 25 tasks for complex dual model implementation
- Unproven accuracy benefits with high complexity
- Multiple models to maintain and coordinate
- Complex routing and fallback logic

**AFTER (Single Model Implementation)**:
- 15 focused tasks for single proven model
- Evidence-based approach with baseline comparison
- Single model to maintain and optimize
- Simple integration with clear value measurement

**Result**: Proven semantic search capability with measurable accuracy improvement over baseline.
