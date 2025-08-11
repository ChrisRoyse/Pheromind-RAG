use crate::llama_wrapper_working::{GGUFModel, GGUFContext};
use crate::embedding_prefixes::{EmbeddingTask, CodeFormatter, BatchProcessor};
use anyhow::Result;
use std::sync::Arc;
use parking_lot::Mutex;
use lru::LruCache;
use std::num::NonZeroUsize;

/// Configuration for GGUF embedder
#[derive(Debug, Clone)]
pub struct GGUFEmbedderConfig {
    pub model_path: String,
    pub context_size: u32,
    pub gpu_layers: i32,
    pub batch_size: usize,
    pub cache_size: usize,
    pub normalize: bool,
    pub threads: usize,
}

impl Default for GGUFEmbedderConfig {
    fn default() -> Self {
        // CPU-optimized configuration
        let cpu_count = num_cpus::get();
        let optimal_threads = std::cmp::max(1, (cpu_count * 3) / 4);  // Use 75% of cores
        
        Self {
            model_path: "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf".to_string(),
            context_size: 8192,
            gpu_layers: 0,  // CPU-ONLY: No GPU layers
            batch_size: 16,  // Reduced for CPU efficiency
            cache_size: 2000,  // Increased cache for CPU compensation
            normalize: true,
            threads: optimal_threads,
        }
    }
}

/// Statistics for embedder performance
#[derive(Debug, Default)]
pub struct EmbedderStats {
    pub total_embeddings: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub batch_operations: usize,
    pub total_tokens_processed: usize,
}

impl EmbedderStats {
    pub fn cache_hit_rate(&self) -> f64 {
        if self.total_embeddings == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_embeddings as f64
        }
    }
}

/// Thread-safe GGUF embedder with caching and performance monitoring
pub struct GGUFEmbedder {
    model: Arc<GGUFModel>,
    context: Arc<Mutex<GGUFContext>>,
    cache: Arc<Mutex<LruCache<String, Vec<f32>>>>,
    config: GGUFEmbedderConfig,
    stats: Arc<Mutex<EmbedderStats>>,
}

impl GGUFEmbedder {
    /// Create new embedder with configuration
    pub fn new(config: GGUFEmbedderConfig) -> Result<Self> {
        // Load GGUF model
        let model = Arc::new(GGUFModel::load_from_file(
            &config.model_path,
            config.gpu_layers,
        )?);
        
        // Create context for embeddings
        let context = Arc::new(Mutex::new(
            GGUFContext::new_with_model(&model, config.context_size)?
        ));
        
        // Initialize LRU cache
        let cache_size = NonZeroUsize::new(config.cache_size)
            .expect("Cache size must be greater than 0");
        let cache = Arc::new(Mutex::new(LruCache::new(cache_size)));
        
        // Initialize statistics
        let stats = Arc::new(Mutex::new(EmbedderStats::default()));
        
        Ok(Self {
            model,
            context,
            cache,
            config,
            stats,
        })
    }

    /// Create embedder with default configuration
    pub fn with_model_path(model_path: &str) -> Result<Self> {
        let mut config = GGUFEmbedderConfig::default();
        config.model_path = model_path.to_string();
        Self::new(config)
    }

    /// Generate embedding with caching and task-specific prefixes
    pub fn embed(&self, text: &str, task: EmbeddingTask) -> Result<Vec<f32>> {
        // Apply task-specific prefix
        let prefixed_text = task.apply_prefix(text);
        
        // Check cache first
        {
            let mut cache = self.cache.lock();
            if let Some(cached) = cache.get(&prefixed_text) {
                // Update statistics
                let mut stats = self.stats.lock();
                stats.total_embeddings += 1;
                stats.cache_hits += 1;
                return Ok(cached.clone());
            }
        }
        
        // Generate embedding using GGUF context
        let embedding = {
            let mut ctx = self.context.lock();
            let result = ctx.embed(&prefixed_text)?;
            
            // Apply L2 normalization if configured
            if self.config.normalize {
                self.normalize_embedding(result)
            } else {
                result
            }
        };
        
        // Cache the result
        {
            let mut cache = self.cache.lock();
            cache.put(prefixed_text.clone(), embedding.clone());
        }
        
        // Update statistics
        {
            let mut stats = self.stats.lock();
            stats.total_embeddings += 1;
            stats.cache_misses += 1;
            stats.total_tokens_processed += text.split_whitespace().count();
        }
        
        Ok(embedding)
    }
    
    /// Generate embeddings for multiple texts with batch processing
    pub fn embed_batch(&self, texts: Vec<String>, task: EmbeddingTask) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        let mut uncached_indices = Vec::new();
        let mut uncached_texts = Vec::new();
        
        // Check cache for each text
        for (i, text) in texts.iter().enumerate() {
            let prefixed = task.apply_prefix(text);
            let mut cache = self.cache.lock();
            
            if let Some(cached) = cache.get(&prefixed) {
                results.push(Some(cached.clone()));
                
                // Update cache hit stats
                let mut stats = self.stats.lock();
                stats.cache_hits += 1;
            } else {
                results.push(None);
                uncached_indices.push(i);
                uncached_texts.push(prefixed);
                
                // Update cache miss stats
                let mut stats = self.stats.lock();
                stats.cache_misses += 1;
            }
        }
        
        // Process uncached texts in batches
        if !uncached_texts.is_empty() {
            for chunk in uncached_texts.chunks(self.config.batch_size) {
                let mut ctx = self.context.lock();
                let chunk_embeddings = ctx.embed_batch(chunk.to_vec())?;
                
                // Apply normalization if configured
                let normalized_embeddings: Vec<Vec<f32>> = if self.config.normalize {
                    chunk_embeddings.into_iter()
                        .map(|emb| self.normalize_embedding(emb))
                        .collect()
                } else {
                    chunk_embeddings
                };
                
                // Update results and cache
                for (chunk_idx, embedding) in normalized_embeddings.into_iter().enumerate() {
                    if let Some(&result_idx) = uncached_indices.get(chunk_idx) {
                        results[result_idx] = Some(embedding.clone());
                        
                        // Cache the result
                        let mut cache = self.cache.lock();
                        cache.put(chunk[chunk_idx].clone(), embedding);
                    }
                }
            }
            
            // Update batch operation stats
            let mut stats = self.stats.lock();
            stats.batch_operations += 1;
        }
        
        // Update total embedding count
        {
            let mut stats = self.stats.lock();
            stats.total_embeddings += texts.len();
            stats.total_tokens_processed += texts.iter()
                .map(|t| t.split_whitespace().count())
                .sum::<usize>();
        }
        
        // Extract results
        Ok(results.into_iter().map(|r| r.unwrap()).collect())
    }
    
    /// Embed code with language-aware formatting
    pub fn embed_code(&self, code: &str, language: Option<&str>, task: EmbeddingTask) -> Result<Vec<f32>> {
        let formatted_code = match language {
            Some(lang) => CodeFormatter::format_code(code, lang),
            None => code.to_string(),
        };
        
        self.embed(&formatted_code, task)
    }
    
    /// Embed code from file (with automatic language detection)
    pub fn embed_code_file(&self, code: &str, filename: &str, task: EmbeddingTask) -> Result<Vec<f32>> {
        let language = CodeFormatter::detect_language(filename);
        self.embed_code(code, language, task)
    }
    
    /// Batch process code snippets with language detection
    pub fn embed_code_batch(&self, codes: Vec<(&str, Option<&str>)>, task: EmbeddingTask) -> Result<Vec<Vec<f32>>> {
        let formatted_texts = BatchProcessor::process_code_batch(codes, task);
        
        // Convert to owned strings for embed_batch
        let owned_texts: Vec<String> = formatted_texts.into_iter().collect();
        
        // Remove task prefix since it's already applied by BatchProcessor
        let embeddings = self.embed_batch_raw(owned_texts)?;
        
        Ok(embeddings)
    }
    
    /// Internal batch embedding without task prefix application
    fn embed_batch_raw(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        let mut uncached_indices = Vec::new();
        let mut uncached_texts = Vec::new();
        
        // Check cache
        for (i, text) in texts.iter().enumerate() {
            let mut cache = self.cache.lock();
            if let Some(cached) = cache.get(text) {
                results.push(Some(cached.clone()));
            } else {
                results.push(None);
                uncached_indices.push(i);
                uncached_texts.push(text.clone());
            }
        }
        
        // Process uncached
        if !uncached_texts.is_empty() {
            for chunk in uncached_texts.chunks(self.config.batch_size) {
                let mut ctx = self.context.lock();
                let embeddings = ctx.embed_batch(chunk.to_vec())?;
                
                let normalized: Vec<Vec<f32>> = if self.config.normalize {
                    embeddings.into_iter().map(|e| self.normalize_embedding(e)).collect()
                } else {
                    embeddings
                };
                
                for (chunk_idx, embedding) in normalized.into_iter().enumerate() {
                    if let Some(&result_idx) = uncached_indices.get(chunk_idx) {
                        results[result_idx] = Some(embedding.clone());
                        
                        // Cache
                        let mut cache = self.cache.lock();
                        cache.put(chunk[chunk_idx].clone(), embedding);
                    }
                }
            }
        }
        
        Ok(results.into_iter().map(|r| r.unwrap()).collect())
    }
    
    /// Get embedding dimension
    pub fn dimension(&self) -> usize {
        self.model.embedding_dim
    }
    
    /// Get performance statistics
    pub fn stats(&self) -> EmbedderStats {
        self.stats.lock().clone()
    }
    
    /// Clear cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock();
        cache.clear();
    }
    
    /// Get cache statistics
    pub fn cache_info(&self) -> (usize, usize) {
        let cache = self.cache.lock();
        (cache.len(), cache.cap().get())
    }
    
    /// L2 normalization
    fn normalize_embedding(&self, mut embedding: Vec<f32>) -> Vec<f32> {
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-8 {
            for x in &mut embedding {
                *x /= norm;
            }
        }
        embedding
    }
}

// Thread safety: GGUFEmbedder implements Send + Sync
unsafe impl Send for GGUFEmbedder {}
unsafe impl Sync for GGUFEmbedder {}

impl Clone for EmbedderStats {
    fn clone(&self) -> Self {
        Self {
            total_embeddings: self.total_embeddings,
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            batch_operations: self.batch_operations,
            total_tokens_processed: self.total_tokens_processed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_embedder() -> Result<GGUFEmbedder> {
        let mut config = GGUFEmbedderConfig::default();
        config.cache_size = 10;
        config.batch_size = 2;
        GGUFEmbedder::new(config)
    }

    #[test]
    fn test_embedder_creation() -> Result<()> {
        let embedder = create_test_embedder()?;
        assert_eq!(embedder.dimension(), 768);
        Ok(())
    }

    #[test]
    fn test_single_embedding() -> Result<()> {
        let embedder = create_test_embedder()?;
        let result = embedder.embed("test text", EmbeddingTask::SearchQuery)?;
        
        assert_eq!(result.len(), 768);
        
        // Check normalization
        let norm: f32 = result.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6, "Embedding should be normalized");
        
        Ok(())
    }

    #[test]
    fn test_batch_embedding() -> Result<()> {
        let embedder = create_test_embedder()?;
        let texts = vec!["text1".to_string(), "text2".to_string(), "text3".to_string()];
        let results = embedder.embed_batch(texts, EmbeddingTask::SearchDocument)?;
        
        assert_eq!(results.len(), 3);
        for result in results {
            assert_eq!(result.len(), 768);
        }
        
        Ok(())
    }

    #[test]
    fn test_caching() -> Result<()> {
        let embedder = create_test_embedder()?;
        
        // First embedding - cache miss
        let _result1 = embedder.embed("test", EmbeddingTask::SearchQuery)?;
        let stats1 = embedder.stats();
        assert_eq!(stats1.cache_misses, 1);
        assert_eq!(stats1.cache_hits, 0);
        
        // Second embedding - cache hit
        let _result2 = embedder.embed("test", EmbeddingTask::SearchQuery)?;
        let stats2 = embedder.stats();
        assert_eq!(stats2.cache_hits, 1);
        
        Ok(())
    }

    #[test]
    fn test_code_embedding() -> Result<()> {
        let embedder = create_test_embedder()?;
        
        let code = "fn main() { println!(\"Hello\"); }";
        let result = embedder.embed_code(code, Some("rust"), EmbeddingTask::CodeDefinition)?;
        
        assert_eq!(result.len(), 768);
        Ok(())
    }

    #[test]
    fn test_thread_safety() -> Result<()> {
        use std::thread;
        use std::sync::Arc;
        
        let embedder = Arc::new(create_test_embedder()?);
        let mut handles = vec![];
        
        // Spawn multiple threads
        for i in 0..3 {
            let embedder_clone = embedder.clone();
            let handle = thread::spawn(move || {
                let text = format!("test text {}", i);
                embedder_clone.embed(&text, EmbeddingTask::SearchQuery)
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap()?;
        }
        
        Ok(())
    }
}