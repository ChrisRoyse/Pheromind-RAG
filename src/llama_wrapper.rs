use llama_cpp_2::{
    context::{LlamaContext, params::LlamaContextParams},
    model::{LlamaModel, params::LlamaModelParams},
    llama_batch::LlamaBatch,
};
use llama_cpp_sys_2::llama_backend_init;
use anyhow::{Result, Context};
use std::path::Path;
use std::sync::Arc;
use std::num::NonZeroU32;

/// Thread-safe GGUF model wrapper using llama-cpp-2
pub struct GGUFModel {
    model: Arc<LlamaModel>,
    embedding_dim: usize,
    model_path: String,
}

impl GGUFModel {
    /// Load GGUF model from file
    pub fn load_from_file<P: AsRef<Path>>(path: P, gpu_layers: i32) -> Result<Self> {
        let path = path.as_ref();
        let path_str = path.to_str()
            .context("Invalid path encoding")?;
        
        // Initialize llama backend
        unsafe {
            llama_backend_init();
        }
        
        // Set model parameters using llama-cpp-2 builder
        let model_params = LlamaModelParams::default()
            .with_n_gpu_layers(gpu_layers as u32)
            .with_use_mmap(true)
            .with_use_mlock(false);
        
        // Load model using llama-cpp-2
        let model = LlamaModel::load_from_file(path, model_params)
            .context(format!("Failed to load model from: {}", path.display()))?;
        
        // Get embedding dimension
        let embedding_dim = model.n_embd() as usize;
        
        Ok(Self {
            model: Arc::new(model),
            embedding_dim,
            model_path: path_str.to_string(),
        })
    }
    
    pub fn embedding_dim(&self) -> usize {
        self.embedding_dim
    }
    
    pub fn model(&self) -> &Arc<LlamaModel> {
        &self.model
    }
}

/// GGUF embedding context using llama-cpp-2
pub struct GGUFContext<'a> {
    context: LlamaContext<'a>,
    model: Arc<GGUFModel>,
    embedding_dim: usize,
}

impl<'a> GGUFContext<'a> {
    /// Create new context for embeddings
    pub fn new_with_model(model: Arc<GGUFModel>, context_size: u32) -> Result<Self> {
        // Context parameters for embeddings using llama-cpp-2
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(context_size))
            .with_n_batch(2048)
            .with_n_threads(num_cpus::get() as u32)
            .with_n_threads_batch(num_cpus::get() as u32)
            .with_embeddings(true);  // CRITICAL for embeddings
        
        // Create context using llama-cpp-2
        let context = model.model()
            .new_context(ctx_params)
            .context("Failed to create context")?;
        
        Ok(Self {
            context,
            model: model.clone(),
            embedding_dim: model.embedding_dim(),
        })
    }
    
    /// Generate embeddings for text
    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        // Tokenize using llama-cpp-2
        let tokens = self.model.model()
            .tokenize(text, true)?;  // add_special = true
        
        // Create batch for decoding
        let mut batch = LlamaBatch::new(tokens.len(), 1);
        batch.add(tokens, 0, &[1], false)?;
        
        // Decode the batch
        self.context.decode(&mut batch)
            .context("Failed to decode tokens")?;
        
        // Extract embeddings using llama-cpp-2
        let embeddings = self.context.embeddings()
            .context("Failed to get embeddings")?;
        
        // Convert to Vec<f32> and normalize
        let mut embedding_vec: Vec<f32> = embeddings.to_vec();
        
        // L2 normalization
        let norm = embedding_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for emb in &mut embedding_vec {
                *emb /= norm;
            }
        }
        
        Ok(embedding_vec)
    }
    
    /// Batch embedding generation
    pub fn embed_batch(&mut self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let mut all_embeddings = Vec::new();
        
        for text in texts {
            let embedding = self.embed(&text)?;
            all_embeddings.push(embedding);
        }
        
        Ok(all_embeddings)
    }
}