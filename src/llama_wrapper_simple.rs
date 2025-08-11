// Simple wrapper using the working implementation
use crate::llama_wrapper_working::{GGUFModel as WorkingGGUFModel, GGUFContext as WorkingGGUFContext};
use anyhow::Result;
use std::path::Path;

/// Simple GGUF model wrapper
pub struct GGUFModel {
    inner: WorkingGGUFModel,
    pub embedding_dim: usize,
}

impl GGUFModel {
    pub fn load_from_file<P: AsRef<Path>>(path: P, gpu_layers: i32) -> Result<Self> {
        let inner = WorkingGGUFModel::load_from_file(path, gpu_layers)?;
        let embedding_dim = inner.embedding_dim();
        
        Ok(Self {
            inner,
            embedding_dim,
        })
    }
    
    pub fn inner(&self) -> &WorkingGGUFModel {
        &self.inner
    }
}

/// Simple GGUF context wrapper  
pub struct GGUFContext {
    inner: WorkingGGUFContext,
    embedding_dim: usize,
}

impl GGUFContext {
    pub fn new_with_model(model: &GGUFModel, context_size: u32) -> Result<Self> {
        let inner = WorkingGGUFContext::new_with_model(model.inner(), context_size)?;
        
        Ok(Self {
            inner,
            embedding_dim: model.embedding_dim,
        })
    }
    
    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        self.inner.embed(text)
    }
    
    pub fn embed_batch(&mut self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        self.inner.embed_batch(texts)
    }
}