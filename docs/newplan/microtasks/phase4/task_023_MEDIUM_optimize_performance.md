# Task 023 - MEDIUM: Optimize LanceDB Performance Settings

## Priority: MEDIUM
## Estimated Time: 10 minutes
## Dependencies: Task 022

## Objective
Optimize LanceDB configuration and settings for better search and insertion performance.

## Current Issue
- Default settings may not be optimal
- Index configuration needs tuning
- Connection pooling and caching needed

## Tasks
1. **Optimize index configuration** (4 min)
   ```rust
   // Update src/storage/lancedb_store.rs
   impl LanceDBStore {
       pub async fn create_optimized_vector_index(&self) -> Result<()> {
           let table = self.get_table().await?;
           
           // Create optimized IVF-PQ index for large datasets
           let index_result = table
               .create_index(&["embedding"])
               .index_type(lancedb::index::IndexType::IVF_PQ)
               .num_partitions(self.calculate_optimal_partitions().await?)
               .num_sub_vectors(32) // Good balance for 768-dim embeddings
               .max_iterations(100)
               .sample_rate(0.1)
               .execute()
               .await;
               
           match index_result {
               Ok(_) => {
                   println!("Created optimized vector index");
                   Ok(())
               },
               Err(e) => {
                   eprintln!("Vector index creation failed: {}", e);
                   // Fall back to exact search if index creation fails
                   Ok(())
               }
           }
       }
       
       async fn calculate_optimal_partitions(&self) -> Result<usize> {
           let count = self.count().await.unwrap_or(1000);
           
           // Rule of thumb: sqrt(n) partitions, with reasonable bounds
           let partitions = ((count as f64).sqrt() as usize).max(8).min(1024);
           
           // Ensure partitions are power of 2 for better performance
           let partitions = partitions.next_power_of_two();
           
           Ok(partitions)
       }
   }
   ```

2. **Add connection optimization** (4 min)
   ```rust
   // Add connection pooling and caching
   use std::sync::Arc;
   use tokio::sync::RwLock;
   use std::collections::HashMap;
   use lru::LruCache;
   
   pub struct OptimizedLanceDBStore {
       base_store: LanceDBStore,
       table_cache: Arc<RwLock<Option<Table>>>,
       query_cache: Arc<RwLock<LruCache<String, Vec<SearchResult>>>>,
       config: PerformanceConfig,
   }
   
   #[derive(Debug, Clone)]
   pub struct PerformanceConfig {
       pub cache_size: usize,
       pub prefetch_size: usize,
       pub batch_size: usize,
       pub enable_query_cache: bool,
       pub cache_ttl_seconds: u64,
   }
   
   impl Default for PerformanceConfig {
       fn default() -> Self {
           Self {
               cache_size: 1000,
               prefetch_size: 100,
               batch_size: 100,
               enable_query_cache: true,
               cache_ttl_seconds: 300, // 5 minutes
           }
       }
   }
   
   impl OptimizedLanceDBStore {
       pub async fn new(
           db_path: &str,
           table_name: &str,
           embedding_dim: usize,
           config: PerformanceConfig,
       ) -> Result<Self> {
           let base_store = LanceDBStore::new(db_path, table_name, embedding_dim).await?;
           
           Ok(Self {
               base_store,
               table_cache: Arc::new(RwLock::new(None)),
               query_cache: Arc::new(RwLock::new(
                   LruCache::new(std::num::NonZeroUsize::new(config.cache_size).unwrap())
               )),
               config,
           })
       }
       
       async fn get_cached_table(&self) -> Result<Table> {
           // Check cache first
           {
               let cache = self.table_cache.read().await;
               if let Some(table) = cache.as_ref() {
                   return Ok(table.clone());
               }
           }
           
           // Get table and cache it
           let table = self.base_store.get_table().await?;
           {
               let mut cache = self.table_cache.write().await;
               *cache = Some(table.clone());
           }
           
           Ok(table)
       }
   }
   ```

3. **Add batch optimization** (2 min)
   ```rust
   impl OptimizedLanceDBStore {
       pub async fn add_embeddings_optimized(
           &self,
           data: Vec<(String, EmbeddingVector, serde_json::Value)>,
       ) -> Result<()> {
           if data.is_empty() {
               return Ok(());
           }
           
           let batch_size = self.config.batch_size;
           
           // Process in optimized batches
           for chunk in data.chunks(batch_size) {
               let (ids, embeddings, metadata_list): (Vec<_>, Vec<_>, Vec<_>) = 
                   chunk.iter().cloned().map(|(id, emb, meta)| (id, emb, meta)).multiunzip();
               
               let batch = self.base_store.create_embedding_batch(
                   ids,
                   embeddings,
                   metadata_list,
               )?;
               
               let table = self.get_cached_table().await?;
               table.add(vec![batch]).execute().await?;
           }
           
           // Clear table cache to ensure fresh data
           {
               let mut cache = self.table_cache.write().await;
               *cache = None;
           }
           
           Ok(())
       }
       
       pub async fn search_with_cache(
           &self,
           query_embedding: &EmbeddingVector,
           limit: usize,
           threshold: Option<f32>,
       ) -> Result<Vec<SearchResult>> {
           if !self.config.enable_query_cache {
               return self.base_store.search(query_embedding, limit, threshold).await;
           }
           
           // Create cache key
           let cache_key = format!(
               "{}_{}_{}_{}",
               hash_embedding(query_embedding),
               limit,
               threshold.map(|t| (t * 1000.0) as i32).unwrap_or(-1),
               chrono::Utc::now().timestamp() / self.config.cache_ttl_seconds as i64
           );
           
           // Check cache
           {
               let mut cache = self.query_cache.write().await;
               if let Some(results) = cache.get(&cache_key) {
                   return Ok(results.clone());
               }
           }
           
           // Execute search
           let results = self.base_store.search(query_embedding, limit, threshold).await?;
           
           // Cache results
           {
               let mut cache = self.query_cache.write().await;
               cache.put(cache_key, results.clone());
           }
           
           Ok(results)
       }
   }
   
   fn hash_embedding(embedding: &[f32]) -> u64 {
       use std::hash::{Hash, Hasher};
       use std::collections::hash_map::DefaultHasher;
       
       let mut hasher = DefaultHasher::new();
       for &value in embedding.iter().take(10) { // Hash first 10 values for speed
           ((value * 1000.0) as i32).hash(&mut hasher);
       }
       hasher.finish()
   }
   ```

## Success Criteria
- [ ] Index configuration optimized
- [ ] Connection pooling implemented
- [ ] Query caching works
- [ ] Batch operations optimized
- [ ] Performance metrics improved

## Files to Modify
- `src/storage/lancedb_store.rs`
- `Cargo.toml` (add lru dependency)

## Dependencies to Add
```toml
[dependencies]
lru = "0.12"
```

## Performance Targets
- Search latency: <100ms for 10k embeddings
- Insert throughput: >1000 embeddings/second
- Memory usage: <2GB for 100k embeddings
- Cache hit ratio: >80% for repeated queries

## Next Task
→ Task 024: Fix embedding cache type mismatches