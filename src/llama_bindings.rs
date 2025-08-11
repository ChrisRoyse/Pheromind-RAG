// Complete FFI bindings for llama-cpp-2 v0.1.54
// This module provides safe Rust wrappers around the C FFI
use anyhow::{Result, Context};
use std::ffi::{CStr, CString};

// Re-export the working implementation as the primary binding
pub use crate::llama_wrapper_working::{GGUFModel, GGUFContext};

/// CPU optimization configuration for embedding models
#[derive(Debug, Clone)]
pub struct CPUConfig {
    pub threads: usize,
    pub batch_size: usize, 
    pub context_size: u32,
    pub use_mlock: bool,
    pub use_mmap: bool,
}

impl Default for CPUConfig {
    fn default() -> Self {
        Self {
            threads: num_cpus::get(),
            batch_size: 256,
            context_size: 2048,
            use_mlock: false,
            use_mmap: true,
        }
    }
}

/// FFI-safe embedding interface
pub trait EmbeddingFFI: Send + Sync {
    /// Generate embedding for text with error recovery
    fn embed_safe(&self, text: &str) -> Result<Vec<f32>>;
    
    /// Batch embedding with automatic chunking
    fn embed_batch_safe(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>>;
    
    /// Get embedding dimension
    fn dimension(&self) -> usize;
    
    /// Check if model is loaded correctly
    fn is_ready(&self) -> bool;
}

/// Thread-safe wrapper implementing EmbeddingFFI
pub struct SafeGGUFEmbedder {
    context: std::sync::Mutex<GGUFContext>,
    dimension: usize,
    ready: bool,
}

impl SafeGGUFEmbedder {
    pub fn new(model_path: &str, gpu_layers: i32, config: CPUConfig) -> Result<Self> {
        let model = GGUFModel::load_from_file(model_path, gpu_layers)
            .context("Failed to load GGUF model for embeddings")?;
        
        let context = GGUFContext::new_with_model(&model, config.context_size)
            .context("Failed to create embedding context")?;
        
        let dimension = model.embedding_dim();
        
        Ok(Self {
            context: std::sync::Mutex::new(context),
            dimension,
            ready: true,
        })
    }
}

impl EmbeddingFFI for SafeGGUFEmbedder {
    fn embed_safe(&self, text: &str) -> Result<Vec<f32>> {
        if !self.ready {
            anyhow::bail!("Embedder not ready");
        }
        
        let mut ctx = self.context.lock().unwrap();
        ctx.embed(text)
    }
    
    fn embed_batch_safe(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if !self.ready {
            anyhow::bail!("Embedder not ready");
        }
        
        let mut ctx = self.context.lock().unwrap();
        ctx.embed_batch(texts)
    }
    
    fn dimension(&self) -> usize {
        self.dimension
    }
    
    fn is_ready(&self) -> bool {
        self.ready
    }
}

/// Memory-safe C-compatible interface for FFI
#[no_mangle]
pub extern "C" fn create_gguf_embedder(
    model_path: *const std::os::raw::c_char,
    gpu_layers: i32,
) -> *mut SafeGGUFEmbedder {
    if model_path.is_null() {
        return std::ptr::null_mut();
    }
    
    let path_str = unsafe {
        match CStr::from_ptr(model_path).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };
    
    match SafeGGUFEmbedder::new(path_str, gpu_layers, CPUConfig::default()) {
        Ok(embedder) => Box::into_raw(Box::new(embedder)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn destroy_gguf_embedder(embedder: *mut SafeGGUFEmbedder) {
    if !embedder.is_null() {
        unsafe {
            drop(Box::from_raw(embedder));
        }
    }
}

#[no_mangle]
pub extern "C" fn gguf_embed_text(
    embedder: *mut SafeGGUFEmbedder,
    text: *const std::os::raw::c_char,
    output: *mut f32,
    output_len: usize,
) -> i32 {
    if embedder.is_null() || text.is_null() || output.is_null() {
        return -1;
    }
    
    let embedder = unsafe { &*embedder };
    let text_str = unsafe {
        match CStr::from_ptr(text).to_str() {
            Ok(s) => s,
            Err(_) => return -2,
        }
    };
    
    match embedder.embed_safe(text_str) {
        Ok(embedding) => {
            if embedding.len() > output_len {
                return -3; // Buffer too small
            }
            
            unsafe {
                std::ptr::copy_nonoverlapping(
                    embedding.as_ptr(),
                    output,
                    embedding.len()
                );
            }
            
            embedding.len() as i32
        }
        Err(_) => -4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cpu_config() {
        let config = CPUConfig::default();
        assert!(config.threads > 0);
        assert!(config.batch_size > 0);
        assert!(config.context_size > 0);
    }
    
    #[test]
    fn test_ffi_safety() {
        // Test null pointer handling
        let result = unsafe { 
            create_gguf_embedder(std::ptr::null(), 0)
        };
        assert!(result.is_null());
        
        // Safe to call with null
        destroy_gguf_embedder(std::ptr::null_mut());
    }
}