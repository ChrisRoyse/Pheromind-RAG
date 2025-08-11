use llama_cpp_2::{
    context::{LlamaContext, params::LlamaContextParams},
    model::{LlamaModel, params::LlamaModelParams},
    llama_batch::LlamaBatch,
    llama_backend::LlamaBackend,
    token::LlamaToken,
};
use anyhow::{Result, Context, bail};
use std::path::Path;
use std::sync::Arc;
use std::num::NonZeroU32;
use once_cell::sync::Lazy;

// Global backend instance
static BACKEND: Lazy<LlamaBackend> = Lazy::new(|| {
    LlamaBackend::init().expect("Failed to initialize llama backend")
});

/// Thread-safe GGUF model wrapper using llama-cpp-2
pub struct GGUFModel {
    model: Arc<LlamaModel>,
    pub embedding_dim: usize,
    model_path: String,
}

impl GGUFModel {
    /// Load GGUF model from file
    pub fn load_from_file<P: AsRef<Path>>(path: P, gpu_layers: i32) -> Result<Self> {
        let path = path.as_ref();
        let path_str = path.to_str()
            .context("Invalid path encoding")?;
        
        // Ensure backend is initialized
        Lazy::force(&BACKEND);
        
        // Set model parameters using llama-cpp-2 builder
        let model_params = LlamaModelParams::default()
            .with_n_gpu_layers(gpu_layers.max(0) as u32);
        
        // Load model using correct llama-cpp-2 v0.1.54 API
        let model = LlamaModel::load_from_file(&BACKEND, path, &model_params)
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
pub struct GGUFContext {
    context: LlamaContext<'static>,
    model: Arc<GGUFModel>,
    embedding_dim: usize,
}

impl GGUFContext {
    /// Create new context for embeddings  
    pub fn new_with_model(model: Arc<GGUFModel>, context_size: u32) -> Result<Self> {
        // Context parameters for embeddings using llama-cpp-2
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(context_size))
            .with_n_batch(512) // Reduced for stability
            .with_n_threads(num_cpus::get() as u32)
            .with_n_threads_batch(num_cpus::get() as u32)
            .with_embeddings(true);  // CRITICAL for embeddings
        
        // Create context using correct llama-cpp-2 v0.1.54 API
        // Clone the model to avoid lifetime issues
        let model_clone = model.clone();
        let context = model_clone.model()
            .new_context(&BACKEND, ctx_params)
            .context("Failed to create context")?;
        
        Ok(Self {
            context,
            model,
            embedding_dim: model_clone.embedding_dim(),
        })
    }
    
    /// Generate embeddings for text
    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        // Tokenize using correct llama-cpp-2 v0.1.54 API  
        let tokens: Vec<LlamaToken> = self.model.model()
            .str_to_token(text, llama_cpp_2::model::AddBos::Never)?;
        
        if tokens.is_empty() {
            bail!("Failed to tokenize text: {}", text);
        }
        
        // Create batch for decoding
        let mut batch = LlamaBatch::new(tokens.len(), 1);
        for (i, token) in tokens.iter().enumerate() {
            batch.add(*token, i as i32, &[0], false)?;
        }
        
        // Decode the batch
        self.context.decode(&mut batch)
            .context("Failed to decode tokens")?;
        
        // Extract embeddings using correct API - get embeddings for sequence 0
        let embeddings = self.context.embeddings_ith(0)
            .context("Failed to get embeddings")?;
        
        // Convert to Vec<f32> and normalize
        let mut embedding_vec: Vec<f32> = embeddings.to_vec();
        
        // L2 normalization for better similarity computations
        let norm = embedding_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-8 {
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