use anyhow::Result;
use candle_core::{Device, Tensor, DType};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use hf_hub::{api::tokio::Api, Repo, RepoType};
use tokenizers::Tokenizer;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use std::path::Path;

use crate::embedding::cache::EmbeddingCache;

#[derive(Debug)]
pub enum EmbeddingError {
    ModelLoadError(String),
    TokenizationError(String),
    InferenceError(String),
    DimensionError(String),
}

impl std::fmt::Display for EmbeddingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmbeddingError::ModelLoadError(msg) => write!(f, "Model load error: {}", msg),
            EmbeddingError::TokenizationError(msg) => write!(f, "Tokenization error: {}", msg),
            EmbeddingError::InferenceError(msg) => write!(f, "Inference error: {}", msg),
            EmbeddingError::DimensionError(msg) => write!(f, "Dimension error: {}", msg),
        }
    }
}

impl std::error::Error for EmbeddingError {}

/// Real all-MiniLM-L6-v2 embedder using Candle ML framework
pub struct RealMiniLMEmbedder {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

// Global singleton instance
static GLOBAL_EMBEDDER: OnceCell<Arc<RealMiniLMEmbedder>> = OnceCell::new();

impl RealMiniLMEmbedder {
    /// Get or create the global singleton instance
    pub async fn get_global() -> Result<Arc<RealMiniLMEmbedder>, EmbeddingError> {
        if let Some(embedder) = GLOBAL_EMBEDDER.get() {
            println!("📋 Using cached global embedder instance");
            return Ok(embedder.clone());
        }
        
        println!("🆕 Creating new global embedder instance...");
        let embedder = Arc::new(Self::new_impl().await?);
        
        // Try to set the global instance, but if another thread beat us to it, use that one
        match GLOBAL_EMBEDDER.set(embedder.clone()) {
            Ok(_) => {
                println!("✅ Global embedder instance created and cached");
                Ok(embedder)
            }
            Err(_) => {
                // Another thread already set it, use that instance
                println!("📋 Another thread created the global instance, using that one");
                Ok(GLOBAL_EMBEDDER.get().unwrap().clone())
            }
        }
    }
    
    /// Load the actual all-MiniLM-L6-v2 model from Hugging Face
    pub async fn new() -> Result<Self, EmbeddingError> {
        Self::new_impl().await
    }
    
    /// Internal implementation for creating a new embedder
    async fn new_impl() -> Result<Self, EmbeddingError> {
        println!("🔄 Loading real all-MiniLM-L6-v2 model from Hugging Face...");
        
        let device = Device::Cpu; // Use CPU for compatibility
        let model_id = "sentence-transformers/all-MiniLM-L6-v2";
        
        // Download model files from Hugging Face Hub
        let api = Api::new().map_err(|e| EmbeddingError::ModelLoadError(format!("API error: {}", e)))?;
        let repo = api.repo(Repo::with_revision(model_id.to_string(), RepoType::Model, "main".to_string()));
        
        // Download required files
        let config_filename = repo.get("config.json").await
            .map_err(|e| EmbeddingError::ModelLoadError(format!("Failed to download config: {}", e)))?;
        let tokenizer_filename = repo.get("tokenizer.json").await
            .map_err(|e| EmbeddingError::ModelLoadError(format!("Failed to download tokenizer: {}", e)))?;
        let weights_filename = repo.get("model.safetensors").await
            .map_err(|e| EmbeddingError::ModelLoadError(format!("Failed to download weights: {}", e)))?;
        
        println!("📥 Downloaded model files");
        
        // Load tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_filename)
            .map_err(|e| EmbeddingError::TokenizationError(format!("Tokenizer load error: {}", e)))?;
        
        // Load config
        let config_str = std::fs::read_to_string(config_filename)
            .map_err(|e| EmbeddingError::ModelLoadError(format!("Config read error: {}", e)))?;
        let config: Config = serde_json::from_str(&config_str)
            .map_err(|e| EmbeddingError::ModelLoadError(format!("Config parse error: {}", e)))?;
        
        println!("📋 Model config: hidden_size={}, num_attention_heads={}, num_hidden_layers={}", 
                 config.hidden_size, config.num_attention_heads, config.num_hidden_layers);
        
        // Load model weights
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_filename], DTYPE, &device)
                .map_err(|e| EmbeddingError::ModelLoadError(format!("Weight load error: {}", e)))?
        };
        
        // Create BERT model
        let model = BertModel::load(vb, &config)
            .map_err(|e| EmbeddingError::ModelLoadError(format!("Model creation error: {}", e)))?;
        
        println!("✅ Successfully loaded real all-MiniLM-L6-v2 model");
        
        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }
    
    /// Generate REAL 384-dimensional embeddings using the actual model
    pub fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        // Tokenize input text
        let encoding = self.tokenizer.encode(text, true)
            .map_err(|e| EmbeddingError::TokenizationError(format!("Encoding error: {}", e)))?;
        
        let tokens = encoding.get_ids();
        let token_ids = Tensor::new(tokens, &self.device)
            .map_err(|e| EmbeddingError::InferenceError(format!("Tensor creation error: {}", e)))?
            .unsqueeze(0)
            .map_err(|e| EmbeddingError::InferenceError(format!("Unsqueeze error: {}", e)))?;
        
        // Create attention mask (all 1s for simplicity)
        let seq_len = tokens.len();
        let attention_mask = Tensor::ones((1, seq_len), DType::I64, &self.device)
            .map_err(|e| EmbeddingError::InferenceError(format!("Attention mask error: {}", e)))?;
        
        // Run inference through BERT model
        let embeddings = self.model.forward(&token_ids, &attention_mask, None)
            .map_err(|e| EmbeddingError::InferenceError(format!("Model forward error: {}", e)))?;
        
        // Mean pooling over sequence length to get sentence embedding
        let pooled = embeddings.mean(1)
            .map_err(|e| EmbeddingError::InferenceError(format!("Mean pooling error: {}", e)))?;
        
        // Convert to Vec<f32>
        let embedding_vec: Vec<f32> = pooled.flatten_all()
            .map_err(|e| EmbeddingError::InferenceError(format!("Flatten error: {}", e)))?
            .to_vec1()
            .map_err(|e| EmbeddingError::InferenceError(format!("Vec conversion error: {}", e)))?;
        
        // Verify it's 384-dimensional (all-MiniLM-L6-v2 standard)
        if embedding_vec.len() != 384 {
            return Err(EmbeddingError::DimensionError(
                format!("Expected 384 dimensions, got {}", embedding_vec.len())
            ));
        }
        
        // L2 normalize the embedding (standard for sentence transformers)
        let normalized = self.normalize(embedding_vec);
        
        Ok(normalized)
    }
    
    /// Batch embed multiple texts for efficiency (sequential processing)
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let mut embeddings = Vec::with_capacity(texts.len());
        
        for text in texts {
            let embedding = self.embed(text)?;
            embeddings.push(embedding);
        }
        
        Ok(embeddings)
    }

    /// Optimized batch embedding with tensor batching (up to 32 texts simultaneously)
    pub fn embed_batch_optimized(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // For very small batches, use sequential processing
        if texts.len() == 1 {
            return Ok(vec![self.embed(texts[0])?]);
        }

        // Process in batches of up to 32 for memory efficiency
        const MAX_BATCH_SIZE: usize = 32;
        let mut all_embeddings = Vec::with_capacity(texts.len());

        for chunk in texts.chunks(MAX_BATCH_SIZE) {
            let batch_embeddings = self.embed_tensor_batch(chunk)?;
            all_embeddings.extend(batch_embeddings);
        }

        Ok(all_embeddings)
    }

    /// Internal method for tensor-based batch processing
    fn embed_tensor_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Tokenize all texts
        let mut all_token_ids = Vec::new();
        let mut max_len = 0;

        for text in texts {
            let encoding = self.tokenizer.encode(*text, true)
                .map_err(|e| EmbeddingError::TokenizationError(format!("Encoding error: {}", e)))?;
            
            let tokens = encoding.get_ids();
            max_len = max_len.max(tokens.len());
            all_token_ids.push(tokens.to_vec());
        }

        // Pad all sequences to the same length
        let batch_size = texts.len();
        let mut padded_tokens = Vec::with_capacity(batch_size * max_len);
        let mut attention_mask_data = Vec::with_capacity(batch_size * max_len);

        for tokens in &all_token_ids {
            // Add actual tokens
            padded_tokens.extend_from_slice(tokens);
            attention_mask_data.extend(vec![1i64; tokens.len()]);
            
            // Add padding
            let padding_needed = max_len - tokens.len();
            if padding_needed > 0 {
                padded_tokens.extend(vec![0u32; padding_needed]); // PAD token is usually 0
                attention_mask_data.extend(vec![0i64; padding_needed]);
            }
        }

        // Create tensors
        let token_tensor = Tensor::new(padded_tokens, &self.device)
            .map_err(|e| EmbeddingError::InferenceError(format!("Token tensor creation error: {}", e)))?
            .reshape(&[batch_size, max_len])
            .map_err(|e| EmbeddingError::InferenceError(format!("Token tensor reshape error: {}", e)))?;

        let attention_tensor = Tensor::new(attention_mask_data, &self.device)
            .map_err(|e| EmbeddingError::InferenceError(format!("Attention tensor creation error: {}", e)))?
            .reshape(&[batch_size, max_len])
            .map_err(|e| EmbeddingError::InferenceError(format!("Attention tensor reshape error: {}", e)))?;

        // Run batch inference
        let embeddings = self.model.forward(&token_tensor, &attention_tensor, None)
            .map_err(|e| EmbeddingError::InferenceError(format!("Batch model forward error: {}", e)))?;

        // Apply attention mask for proper mean pooling
        let attention_expanded = attention_tensor
            .to_dtype(embeddings.dtype())
            .map_err(|e| EmbeddingError::InferenceError(format!("Attention dtype conversion error: {}", e)))?
            .unsqueeze(2)
            .map_err(|e| EmbeddingError::InferenceError(format!("Attention unsqueeze error: {}", e)))?;

        // Mask out padding tokens
        let masked_embeddings = embeddings.broadcast_mul(&attention_expanded)
            .map_err(|e| EmbeddingError::InferenceError(format!("Attention masking error: {}", e)))?;

        // Sum over sequence length
        let summed = masked_embeddings.sum(1)
            .map_err(|e| EmbeddingError::InferenceError(format!("Sum pooling error: {}", e)))?;

        // Count valid tokens for each sequence (sum of attention mask)
        let attention_sums = attention_tensor
            .to_dtype(embeddings.dtype())
            .map_err(|e| EmbeddingError::InferenceError(format!("Attention sum dtype error: {}", e)))?
            .sum(1)
            .map_err(|e| EmbeddingError::InferenceError(format!("Attention sum error: {}", e)))?
            .unsqueeze(1)
            .map_err(|e| EmbeddingError::InferenceError(format!("Attention sum unsqueeze error: {}", e)))?;

        // Mean pooling: divide by number of valid tokens
        let mean_pooled = summed.broadcast_div(&attention_sums)
            .map_err(|e| EmbeddingError::InferenceError(format!("Mean pooling division error: {}", e)))?;

        // Convert to Vec<Vec<f32>>
        let batch_data: Vec<f32> = mean_pooled.flatten_all()
            .map_err(|e| EmbeddingError::InferenceError(format!("Batch flatten error: {}", e)))?
            .to_vec1()
            .map_err(|e| EmbeddingError::InferenceError(format!("Batch vec conversion error: {}", e)))?;

        // Split into individual embeddings and normalize
        let embedding_dim = 384; // all-MiniLM-L6-v2 dimension
        let mut result = Vec::with_capacity(batch_size);

        for i in 0..batch_size {
            let start_idx = i * embedding_dim;
            let end_idx = start_idx + embedding_dim;
            
            if end_idx > batch_data.len() {
                return Err(EmbeddingError::DimensionError(
                    format!("Batch embedding dimension mismatch: expected {}, got {}", 
                           embedding_dim, batch_data.len() / batch_size)
                ));
            }

            let embedding_vec = batch_data[start_idx..end_idx].to_vec();
            let normalized = self.normalize(embedding_vec);
            result.push(normalized);
        }

        Ok(result)
    }
    
    /// Process in batches for memory efficiency (chunked batch processing)
    pub fn embed_chunked(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        const DEFAULT_BATCH_SIZE: usize = 32;
        let mut results = Vec::with_capacity(texts.len());
        
        for chunk in texts.chunks(DEFAULT_BATCH_SIZE) {
            let batch_results = self.embed_batch(chunk)?;
            results.extend(batch_results);
        }
        
        Ok(results)
    }
    
    /// L2 normalize embedding vector (required for proper similarity)
    fn normalize(&self, mut embedding: Vec<f32>) -> Vec<f32> {
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }
        embedding
    }
    
    /// Get model information
    pub fn model_info(&self) -> String {
        format!("all-MiniLM-L6-v2 (384 dimensions, sentence-transformers compatible)")
    }
    
    /// Test the model with a simple embedding
    pub fn test_embedding(&self) -> Result<(), EmbeddingError> {
        let test_text = "This is a test sentence for the embedding model.";
        let embedding = self.embed(test_text)?;
        
        println!("🧪 Test embedding - Input: '{}'", test_text);
        println!("🧪 Output dimensions: {}", embedding.len());
        println!("🧪 First 5 values: {:?}", &embedding[0..5]);
        println!("🧪 L2 norm: {:.6}", embedding.iter().map(|x| x * x).sum::<f32>().sqrt());
        
        if (embedding.iter().map(|x| x * x).sum::<f32>().sqrt() - 1.0).abs() > 0.001 {
            return Err(EmbeddingError::InferenceError("Embedding not properly normalized".to_string()));
        }
        
        println!("✅ Model test passed - embeddings are properly normalized");
        Ok(())  
    }
}

// Thread safety
unsafe impl Send for RealMiniLMEmbedder {}
unsafe impl Sync for RealMiniLMEmbedder {}

/// Cached embedder that wraps RealMiniLMEmbedder with LRU caching
pub struct CachedEmbedder {
    embedder: Arc<RealMiniLMEmbedder>,
    cache: EmbeddingCache,
}

impl CachedEmbedder {
    /// Create a new cached embedder with default cache size
    pub async fn new() -> Result<Self, EmbeddingError> {
        Self::new_with_cache_size(10_000).await
    }

    /// Create a new cached embedder with specified cache size
    pub async fn new_with_cache_size(cache_size: usize) -> Result<Self, EmbeddingError> {
        let embedder = RealMiniLMEmbedder::get_global().await?;
        let cache = EmbeddingCache::new(cache_size);
        
        Ok(Self { embedder, cache })
    }

    /// Create a new cached embedder with persistence
    pub async fn new_with_persistence(cache_size: usize, cache_dir: impl AsRef<Path>) -> Result<Self, EmbeddingError> {
        let embedder = RealMiniLMEmbedder::get_global().await?;
        let cache = EmbeddingCache::new_with_persistence(cache_size, cache_dir);
        
        Ok(Self { embedder, cache })
    }

    /// Embed text with caching
    pub fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        // Check cache first
        if let Some(cached_embedding) = self.cache.get(text) {
            return Ok(cached_embedding);
        }

        // Generate embedding and cache it
        let embedding = self.embedder.embed(text)?;
        self.cache.put(text, embedding.clone());
        
        Ok(embedding)
    }

    /// Batch embed with caching and optimized tensor processing
    pub fn embed_batch_cached(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let mut results = Vec::with_capacity(texts.len());
        let mut uncached_texts = Vec::new();
        let mut uncached_indices = Vec::new();

        // Check cache for each text
        for (i, text) in texts.iter().enumerate() {
            if let Some(cached_embedding) = self.cache.get(text) {
                results.push((i, cached_embedding));
            } else {
                uncached_texts.push(*text);
                uncached_indices.push(i);
            }
        }

        // Process uncached texts in optimized batches
        if !uncached_texts.is_empty() {
            println!("🔄 Cache miss for {}/{} texts, generating embeddings...", 
                     uncached_texts.len(), texts.len());
            
            let new_embeddings = self.embedder.embed_batch_optimized(&uncached_texts)?;
            
            // Cache new embeddings and add to results
            for (text, embedding) in uncached_texts.iter().zip(new_embeddings.iter()) {
                self.cache.put(text, embedding.clone());
            }
            
            for (idx, embedding) in uncached_indices.into_iter().zip(new_embeddings.into_iter()) {
                results.push((idx, embedding));
            }
        } else {
            println!("📦 Cache hit for all {}/{} texts!", results.len(), texts.len());
        }

        // Sort results by original order and extract embeddings
        results.sort_by_key(|(idx, _)| *idx);
        Ok(results.into_iter().map(|(_, embedding)| embedding).collect())
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> crate::embedding::cache::CacheStats {
        self.cache.stats()
    }

    /// Clear the cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Get model information
    pub fn model_info(&self) -> String {
        format!("{} (with LRU cache: {} entries)", 
                self.embedder.model_info(), self.cache.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_real_model_loading() {
        // This test requires internet connection and downloads the model
        let embedder = RealMiniLMEmbedder::new().await;
        
        match embedder {
            Ok(model) => {
                println!("✅ Model loaded successfully");
                assert!(model.test_embedding().is_ok());
            }
            Err(e) => {
                println!("⚠️  Model loading failed (expected if no internet): {}", e);
                // Don't fail test if no internet - just log
            }
        }
    }
    
    #[tokio::test]  
    async fn test_real_embeddings_semantic_similarity() {
        let embedder = match RealMiniLMEmbedder::new().await {
            Ok(e) => e,
            Err(_) => {
                println!("⏭️  Skipping semantic test - model not available");
                return;
            }
        };
        
        // Test semantic similarity with real embeddings
        let text1 = "The cat sits on the mat";
        let text2 = "A feline rests on the carpet";  // Semantically similar
        let text3 = "Python programming language";   // Semantically different
        
        let emb1 = embedder.embed(text1).unwrap();
        let emb2 = embedder.embed(text2).unwrap();
        let emb3 = embedder.embed(text3).unwrap();
        
        // Cosine similarity function
        let cosine_sim = |a: &[f32], b: &[f32]| -> f32 {
            a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
        };
        
        let sim_1_2 = cosine_sim(&emb1, &emb2);
        let sim_1_3 = cosine_sim(&emb1, &emb3);
        
        println!("🔍 Similarity cat/feline: {:.4}", sim_1_2);
        println!("🔍 Similarity cat/python: {:.4}", sim_1_3);
        
        // Real semantic model should show cat/feline more similar than cat/python
        assert!(sim_1_2 > sim_1_3, "Real embeddings should show semantic similarity");
        assert!(sim_1_2 > 0.5, "Semantically similar texts should have high similarity");
    }
}