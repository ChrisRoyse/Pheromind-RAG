use anyhow::{Result, Context};
use tokenizers::Tokenizer;
use ort::{Environment, ExecutionProvider, Session, SessionBuilder, Value};
use std::sync::Arc;
use rayon::prelude::*;

/// Types of embeddings supported
#[derive(Clone, Debug)]
pub enum EmbeddingType {
    Semantic,
    Hybrid,
    Neural,
}

/// Configuration for embedding generation
#[derive(Clone)]
pub struct EmbeddingConfig {
    pub model_path: String,
    pub tokenizer_path: String,
    pub use_gpu: bool,
    pub max_sequence_length: usize,
    pub embedding_dimension: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            tokenizer_path: String::new(),
            use_gpu: false,
            max_sequence_length: 512,
            embedding_dimension: 384,
        }
    }
}

/// Main embedding generator
pub struct EmbeddingGenerator {
    config: EmbeddingConfig,
    tokenizer: Option<Arc<Tokenizer>>,
    onnx_session: Option<Arc<Session>>,
    environment: Option<Arc<Environment>>,
    initialized: bool,
}

impl EmbeddingGenerator {
    pub fn new() -> Self {
        Self {
            config: EmbeddingConfig::default(),
            tokenizer: None,
            onnx_session: None,
            environment: None,
            initialized: false,
        }
    }

    /// Initialize the embedding generator with model and tokenizer
    pub fn initialize(&mut self, model_path: &str, tokenizer_path: &str, use_gpu: bool) -> Result<()> {
        self.config = EmbeddingConfig {
            model_path: model_path.to_string(),
            tokenizer_path: tokenizer_path.to_string(),
            use_gpu,
            ..Default::default()
        };

        // Initialize ONNX Runtime environment
        let environment = Arc::new(
            Environment::builder()
                .with_name("embedding_env")
                .build()
                .context("Failed to create ONNX Runtime environment")?
        );

        // Create session builder
        let mut session_builder = SessionBuilder::new(&environment)?;
        
        if use_gpu {
            // Try to add GPU providers
            if let Err(e) = session_builder.with_execution_providers([ExecutionProvider::CUDA(Default::default())]) {
                log::warn!("Failed to initialize CUDA provider: {}. Falling back to CPU.", e);
            }
        }

        // Load the ONNX model
        let session = Arc::new(
            session_builder
                .with_model_from_file(&self.config.model_path)
                .context("Failed to load ONNX model")?
        );

        // Load tokenizer
        let tokenizer = Arc::new(
            Tokenizer::from_file(&self.config.tokenizer_path)
                .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?
        );

        self.environment = Some(environment);
        self.onnx_session = Some(session);
        self.tokenizer = Some(tokenizer);
        self.initialized = true;

        log::info!("Embedding generator initialized successfully");
        Ok(())
    }

    /// Generate embedding for a single text
    pub fn generate_embedding(&self, text: &str, embedding_type: EmbeddingType) -> Result<Vec<f32>> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Embedding generator not initialized"));
        }

        let tokenizer = self.tokenizer.as_ref().unwrap();
        let session = self.onnx_session.as_ref().unwrap();

        // Tokenize input
        let encoding = tokenizer
            .encode(text, true)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;

        // Prepare input tensors
        let input_ids = encoding.get_ids();
        let attention_mask = encoding.get_attention_mask();

        // Truncate or pad to max sequence length
        let mut padded_input_ids = vec![0i64; self.config.max_sequence_length];
        let mut padded_attention_mask = vec![0i64; self.config.max_sequence_length];

        let seq_len = input_ids.len().min(self.config.max_sequence_length);
        
        for i in 0..seq_len {
            padded_input_ids[i] = input_ids[i] as i64;
            padded_attention_mask[i] = attention_mask[i] as i64;
        }

        // Create ONNX input tensors
        let input_ids_tensor = Value::from_array(
            session.allocator(), 
            &[padded_input_ids]
        )?;
        
        let attention_mask_tensor = Value::from_array(
            session.allocator(), 
            &[padded_attention_mask]
        )?;

        // Run inference
        let inputs = vec![
            ("input_ids", input_ids_tensor),
            ("attention_mask", attention_mask_tensor),
        ];

        let outputs = session.run(inputs)?;
        
        // Extract embeddings from output
        let output_tensor = outputs
            .get("last_hidden_state")
            .or_else(|| outputs.get(0))
            .context("No output tensor found")?;

        let raw_embeddings: &[f32] = output_tensor
            .try_extract::<f32>()?
            .view()
            .as_slice()
            .context("Failed to extract embeddings")?;

        // Apply pooling and post-processing based on embedding type
        let pooled_embedding = self.pool_embeddings(raw_embeddings, &padded_attention_mask, embedding_type)?;

        Ok(pooled_embedding)
    }

    /// Generate embeddings for multiple texts in parallel
    pub fn generate_batch_embeddings(&self, texts: &[String], embedding_type: EmbeddingType) -> Result<Vec<Vec<f32>>> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Embedding generator not initialized"));
        }

        // Use parallel processing for batch generation
        texts
            .par_iter()
            .map(|text| self.generate_embedding(text, embedding_type.clone()))
            .collect::<Result<Vec<_>, _>>()
    }

    /// Pool embeddings using different strategies based on type
    fn pool_embeddings(&self, raw_embeddings: &[f32], attention_mask: &[i64], embedding_type: EmbeddingType) -> Result<Vec<f32>> {
        let seq_len = attention_mask.len();
        let embedding_dim = raw_embeddings.len() / seq_len;
        
        match embedding_type {
            EmbeddingType::Semantic => {
                // Mean pooling with attention mask
                self.mean_pooling(raw_embeddings, attention_mask, embedding_dim)
            }
            EmbeddingType::Hybrid => {
                // Combination of mean pooling and max pooling
                let mean_pooled = self.mean_pooling(raw_embeddings, attention_mask, embedding_dim)?;
                let max_pooled = self.max_pooling(raw_embeddings, attention_mask, embedding_dim)?;
                
                // Combine mean and max pooling
                Ok(mean_pooled.iter()
                    .zip(max_pooled.iter())
                    .map(|(mean, max)| (mean + max) / 2.0)
                    .collect())
            }
            EmbeddingType::Neural => {
                // Advanced pooling with learnable weights (simplified version)
                let mean_pooled = self.mean_pooling(raw_embeddings, attention_mask, embedding_dim)?;
                
                // Apply neural transformation (simplified - in real implementation would use learned weights)
                Ok(mean_pooled.iter()
                    .map(|&x| x.tanh()) // Non-linear activation
                    .collect())
            }
        }
    }

    /// Mean pooling with attention mask
    fn mean_pooling(&self, embeddings: &[f32], attention_mask: &[i64], embedding_dim: usize) -> Result<Vec<f32>> {
        let seq_len = attention_mask.len();
        let mut pooled = vec![0.0; embedding_dim];
        let mut mask_sum = 0i64;

        // Sum embeddings where attention mask is 1
        for i in 0..seq_len {
            if attention_mask[i] == 1 {
                mask_sum += 1;
                for j in 0..embedding_dim {
                    pooled[j] += embeddings[i * embedding_dim + j];
                }
            }
        }

        // Average by number of non-masked tokens
        if mask_sum > 0 {
            let mask_sum_f = mask_sum as f32;
            for value in &mut pooled {
                *value /= mask_sum_f;
            }
        }

        // L2 normalize
        self.l2_normalize(&mut pooled);

        Ok(pooled)
    }

    /// Max pooling with attention mask
    fn max_pooling(&self, embeddings: &[f32], attention_mask: &[i64], embedding_dim: usize) -> Result<Vec<f32>> {
        let seq_len = attention_mask.len();
        let mut pooled = vec![f32::NEG_INFINITY; embedding_dim];

        // Take max over non-masked positions
        for i in 0..seq_len {
            if attention_mask[i] == 1 {
                for j in 0..embedding_dim {
                    let val = embeddings[i * embedding_dim + j];
                    if val > pooled[j] {
                        pooled[j] = val;
                    }
                }
            }
        }

        // Handle case where all values were masked
        for value in &mut pooled {
            if *value == f32::NEG_INFINITY {
                *value = 0.0;
            }
        }

        // L2 normalize
        self.l2_normalize(&mut pooled);

        Ok(pooled)
    }

    /// L2 normalization
    fn l2_normalize(&self, vector: &mut [f32]) {
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in vector {
                *value /= norm;
            }
        }
    }

    /// Get embedding dimension
    pub fn get_embedding_dimension(&self) -> usize {
        self.config.embedding_dimension
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Cleanup resources
    pub fn cleanup(&mut self) -> Result<()> {
        self.tokenizer = None;
        self.onnx_session = None;
        self.environment = None;
        self.initialized = false;
        
        log::info!("Embedding generator cleaned up");
        Ok(())
    }
}

impl Default for EmbeddingGenerator {
    fn default() -> Self {
        Self::new()
    }
}