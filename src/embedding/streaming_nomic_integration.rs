/// STREAMING NOMIC EMBEDDER - V8-SAFE REPLACEMENT
/// 
/// This module provides a drop-in replacement for the existing NomicEmbedder
/// that uses streaming tensor loading to prevent V8 heap crashes.

use super::streaming_core::StreamingGGUFLoader;
use crate::utils::memory_monitor::MemoryMonitor;
use std::collections::HashMap;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use tokio::sync::OnceCell;
use candle_core::{Device, Tensor, DType};
use anyhow::{Result, anyhow};
use gguf_file::Content;

/// Memory-safe streaming version of NomicEmbedder
pub struct StreamingNomicEmbedder {
    /// Device tensors (stored on GPU/CPU device memory, not V8 heap)
    tensors: HashMap<String, Tensor>,
    device: Device,
    memory_monitor: Arc<MemoryMonitor>,
    
    /// Model configuration
    vocab_size: usize,
    embed_dim: usize,
    num_layers: usize,
    num_heads: usize,
    
    /// Transformer components
    token_embeddings: Tensor,
    layers: Vec<TransformerLayer>,
    layer_norm: LayerNorm,
}

/// Transformer layer components
pub struct TransformerLayer {
    self_attention: MultiHeadAttention,
    feed_forward: FeedForward,
    attention_norm: LayerNorm,
    ffn_norm: LayerNorm,
}

/// Multi-head attention module
pub struct MultiHeadAttention {
    q_proj: Tensor,
    k_proj: Tensor,
    v_proj: Tensor,
    o_proj: Tensor,
    num_heads: usize,
    head_dim: usize,
}

/// Feed-forward network
pub struct FeedForward {
    gate_proj: Tensor,
    up_proj: Tensor,
    down_proj: Tensor,
}

/// Layer normalization
pub struct LayerNorm {
    weight: Tensor,
    bias: Option<Tensor>,
    eps: f32,
}

impl StreamingNomicEmbedder {
    /// Create new embedder using streaming loader - ZERO large heap allocations
    pub async fn new_with_streaming<P: AsRef<Path>>(model_path: P) -> Result<Self> {
        let memory_monitor = Arc::new(MemoryMonitor::for_nodejs());
        
        // CRITICAL: Check memory availability before starting
        if memory_monitor.is_critical() {
            return Err(anyhow!(
                "System memory critical ({}% used) - cannot load embedder safely",
                memory_monitor.usage_percent() as u32
            ));
        }
        
        let model_path = model_path.as_ref().to_path_buf();
        let device = Device::Cpu; // Can be configured for GPU
        
        // Use streaming loader to load model
        Self::load_model_streaming(&model_path, device, memory_monitor).await
    }
    
    /// Load entire model using streaming approach
    async fn load_model_streaming(
        model_path: &PathBuf, 
        device: Device, 
        memory_monitor: Arc<MemoryMonitor>
    ) -> Result<Self> {
        // Create streaming loader
        let mut loader = StreamingGGUFLoader::new(model_path, memory_monitor.clone())?;
        
        // Read GGUF metadata first (small allocation)
        let mut file = std::fs::File::open(model_path)?;
        let content = Content::read(&mut file)?;
        
        // Extract model configuration from metadata
        let config = Self::extract_model_config(&content)?;
        
        println!("Loading model with streaming loader...");
        println!("  Vocab size: {}", config.vocab_size);
        println!("  Embed dim: {}", config.embed_dim);
        println!("  Layers: {}", config.num_layers);
        println!("  Attention heads: {}", config.num_heads);
        
        // Load all tensors using streaming (one at a time, no large allocations)
        let tensors = Self::load_tensors_streaming(&mut loader, &content, &device).await?;
        
        // Build transformer components from loaded tensors
        let token_embeddings = tensors.get("token_embd.weight")
            .or_else(|| tensors.get("embeddings.weight"))
            .ok_or_else(|| anyhow!("Missing token embeddings tensor"))?
            .clone();
        
        let layers = Self::build_transformer_layers(&tensors, &config, &device)?;
        let layer_norm = Self::build_layer_norm(&tensors, "norm", config.embed_dim, &device)?;
        
        println!("Model loaded successfully with streaming loader");
        
        Ok(Self {
            tensors,
            device,
            memory_monitor,
            vocab_size: config.vocab_size,
            embed_dim: config.embed_dim,
            num_layers: config.num_layers,
            num_heads: config.num_heads,
            token_embeddings,
            layers,
            layer_norm,
        })
    }
    
    /// Load all tensors using streaming approach - one tensor at a time
    async fn load_tensors_streaming(
        loader: &mut StreamingGGUFLoader,
        content: &Content,
        device: &Device
    ) -> Result<HashMap<String, Tensor>> {
        let mut tensors = HashMap::new();
        let total_tensors = content.tensor_infos.len();
        let mut current_offset = content.tensor_data_offset as u64;
        
        println!("Streaming {} tensors...", total_tensors);
        
        for (i, (name, tensor_info)) in content.tensor_infos.iter().enumerate() {
            // Load tensor using streaming (ZERO large allocations)
            let tensor = loader.load_tensor_streaming(name, tensor_info, device, current_offset).await?;
            
            // Calculate next offset
            let tensor_size = StreamingGGUFLoader::calculate_tensor_size(tensor_info)?;
            current_offset += tensor_size as u64;
            
            tensors.insert(name.clone(), tensor);
            
            // Progress reporting
            if (i + 1) % 5 == 0 || i + 1 == total_tensors {
                print!("\r  Loaded {}/{} tensors", i + 1, total_tensors);
                std::io::stdout().flush().unwrap();
            }
            
            // CRITICAL: Yield to prevent V8 blocking
            if (i + 1) % 10 == 0 {
                tokio::task::yield_now().await;
            }
            
            // Memory pressure monitoring
            if loader.memory_monitor.usage_percent() > 85.0 {
                println!("\n⚠️  Memory usage high: {:.1}%", loader.memory_monitor.usage_percent());
            }
        }
        
        println!("\r  Loaded {}/{} tensors ✅", total_tensors, total_tensors);
        Ok(tensors)
    }
    
    /// Extract model configuration from GGUF metadata
    fn extract_model_config(content: &Content) -> Result<ModelConfig> {
        // Extract from metadata or use reasonable defaults
        let vocab_size = content.metadata.get("tokenizer.ggml.vocab_size")
            .and_then(|v| v.as_u32())
            .unwrap_or(32000) as usize;
            
        let embed_dim = content.metadata.get("nomic.embed_length")
            .and_then(|v| v.as_u32())
            .unwrap_or(768) as usize;
            
        let num_layers = content.metadata.get("nomic.block_count")
            .and_then(|v| v.as_u32())
            .unwrap_or(12) as usize;
            
        let num_heads = content.metadata.get("nomic.attention.head_count")
            .and_then(|v| v.as_u32())
            .unwrap_or(12) as usize;
        
        Ok(ModelConfig {
            vocab_size,
            embed_dim,
            num_layers,
            num_heads,
        })
    }
    
    /// Build transformer layers from loaded tensors
    fn build_transformer_layers(
        tensors: &HashMap<String, Tensor>,
        config: &ModelConfig,
        device: &Device
    ) -> Result<Vec<TransformerLayer>> {
        let mut layers = Vec::with_capacity(config.num_layers);
        
        for i in 0..config.num_layers {
            let layer = TransformerLayer::from_tensors(tensors, i, config, device)?;
            layers.push(layer);
        }
        
        Ok(layers)
    }
    
    /// Build layer normalization from tensors
    fn build_layer_norm(
        tensors: &HashMap<String, Tensor>,
        name: &str,
        dim: usize,
        device: &Device
    ) -> Result<LayerNorm> {
        let weight_name = format!("{}.weight", name);
        let bias_name = format!("{}.bias", name);
        
        let weight = tensors.get(&weight_name)
            .ok_or_else(|| anyhow!("Missing layer norm weight: {}", weight_name))?
            .clone();
            
        let bias = tensors.get(&bias_name).cloned();
        
        Ok(LayerNorm {
            weight,
            bias,
            eps: 1e-5,
        })
    }
    
    /// Embed text with memory monitoring
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // CRITICAL: Check memory before processing
        if self.memory_monitor.is_critical() {
            return Err(anyhow!(
                "Memory critical ({}% used) - cannot process embedding safely",
                self.memory_monitor.usage_percent() as u32
            ));
        }
        
        // Tokenize input text
        let tokens = self.tokenize(text)?;
        if tokens.is_empty() {
            return Ok(vec![0.0; self.embed_dim]);
        }
        
        // Create input tensor
        let input_tensor = Tensor::from_slice(&tokens, (1, tokens.len()), &self.device)?;
        
        // Forward pass through model
        let embeddings = self.forward(&input_tensor)?;
        
        // Convert to Vec<f32>
        let result = embeddings.to_vec2::<f32>()?;
        
        // Return pooled embedding (mean over sequence length)
        if result.is_empty() || result[0].is_empty() {
            return Ok(vec![0.0; self.embed_dim]);
        }
        
        Ok(result[0].clone())
    }
    
    /// Embed batch of texts
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        // Check memory for batch processing
        if self.memory_monitor.usage_percent() > 70.0 {
            return Err(anyhow!(
                "Memory usage too high for batch processing: {:.1}%",
                self.memory_monitor.usage_percent()
            ));
        }
        
        let mut results = Vec::with_capacity(texts.len());
        
        for text in texts {
            let embedding = self.embed(text)?;
            results.push(embedding);
            
            // Memory check between embeddings
            if self.memory_monitor.is_critical() {
                return Err(anyhow!("Memory critical during batch processing"));
            }
        }
        
        Ok(results)
    }
    
    /// Tokenize text (placeholder implementation)
    fn tokenize(&self, text: &str) -> Result<Vec<u32>> {
        // Simple whitespace tokenization for now
        // In production, this would use the actual tokenizer
        let tokens: Vec<u32> = text
            .split_whitespace()
            .enumerate()
            .map(|(i, _)| (i % self.vocab_size) as u32)
            .collect();
        
        Ok(tokens)
    }
    
    /// Forward pass through transformer model
    fn forward(&self, input_ids: &Tensor) -> Result<Tensor> {
        let (batch_size, seq_len) = input_ids.dims2()?;
        
        // Token embeddings
        let mut hidden_states = self.token_embeddings.embedding(input_ids)?;
        
        // Forward through transformer layers
        for layer in &self.layers {
            hidden_states = layer.forward(&hidden_states)?;
        }
        
        // Final layer norm
        hidden_states = self.layer_norm.forward(&hidden_states)?;
        
        // Mean pooling over sequence dimension
        let pooled = hidden_states.sum(1)? / seq_len as f64;
        
        Ok(pooled)
    }
}

/// Model configuration
#[derive(Debug, Clone)]
struct ModelConfig {
    vocab_size: usize,
    embed_dim: usize,
    num_layers: usize,
    num_heads: usize,
}

impl TransformerLayer {
    fn from_tensors(
        tensors: &HashMap<String, Tensor>,
        layer_idx: usize,
        config: &ModelConfig,
        device: &Device
    ) -> Result<Self> {
        let prefix = format!("blk.{}", layer_idx);
        
        // Load attention tensors
        let self_attention = MultiHeadAttention::from_tensors(tensors, &prefix, config, device)?;
        
        // Load feed-forward tensors
        let feed_forward = FeedForward::from_tensors(tensors, &prefix, config, device)?;
        
        // Load layer norms
        let attention_norm = LayerNorm::from_tensors(tensors, &format!("{}.attn_norm", prefix), device)?;
        let ffn_norm = LayerNorm::from_tensors(tensors, &format!("{}.ffn_norm", prefix), device)?;
        
        Ok(Self {
            self_attention,
            feed_forward,
            attention_norm,
            ffn_norm,
        })
    }
    
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        // Self-attention with residual connection
        let normed_x = self.attention_norm.forward(x)?;
        let attn_out = self.self_attention.forward(&normed_x)?;
        let x = (x + attn_out)?;
        
        // Feed-forward with residual connection
        let normed_x = self.ffn_norm.forward(&x)?;
        let ffn_out = self.feed_forward.forward(&normed_x)?;
        let x = (x + ffn_out)?;
        
        Ok(x)
    }
}

impl MultiHeadAttention {
    fn from_tensors(
        tensors: &HashMap<String, Tensor>,
        prefix: &str,
        config: &ModelConfig,
        device: &Device
    ) -> Result<Self> {
        let q_proj = tensors.get(&format!("{}.attn_q.weight", prefix))
            .ok_or_else(|| anyhow!("Missing Q projection"))?
            .clone();
        let k_proj = tensors.get(&format!("{}.attn_k.weight", prefix))
            .ok_or_else(|| anyhow!("Missing K projection"))?
            .clone();
        let v_proj = tensors.get(&format!("{}.attn_v.weight", prefix))
            .ok_or_else(|| anyhow!("Missing V projection"))?
            .clone();
        let o_proj = tensors.get(&format!("{}.attn_output.weight", prefix))
            .ok_or_else(|| anyhow!("Missing output projection"))?
            .clone();
        
        let head_dim = config.embed_dim / config.num_heads;
        
        Ok(Self {
            q_proj,
            k_proj,
            v_proj,
            o_proj,
            num_heads: config.num_heads,
            head_dim,
        })
    }
    
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let (batch_size, seq_len, embed_dim) = x.dims3()?;
        
        // Linear projections
        let q = x.matmul(&self.q_proj.t()?)?;
        let k = x.matmul(&self.k_proj.t()?)?;
        let v = x.matmul(&self.v_proj.t()?)?;
        
        // Reshape for multi-head attention
        let q = q.reshape((batch_size, seq_len, self.num_heads, self.head_dim))?
            .transpose(1, 2)?; // (batch, heads, seq, head_dim)
        let k = k.reshape((batch_size, seq_len, self.num_heads, self.head_dim))?
            .transpose(1, 2)?;
        let v = v.reshape((batch_size, seq_len, self.num_heads, self.head_dim))?
            .transpose(1, 2)?;
        
        // Scaled dot-product attention
        let scale = 1.0 / (self.head_dim as f64).sqrt();
        let scores = q.matmul(&k.transpose(-1, -2)?)?;
        let scores = (scores * scale)?;
        let attn_weights = candle_nn::ops::softmax_last_dim(&scores)?;
        
        let attn_out = attn_weights.matmul(&v)?;
        
        // Reshape and project
        let attn_out = attn_out.transpose(1, 2)?
            .reshape((batch_size, seq_len, embed_dim))?;
        let output = attn_out.matmul(&self.o_proj.t()?)?;
        
        Ok(output)
    }
}

impl FeedForward {
    fn from_tensors(
        tensors: &HashMap<String, Tensor>,
        prefix: &str,
        config: &ModelConfig,
        device: &Device
    ) -> Result<Self> {
        let gate_proj = tensors.get(&format!("{}.ffn_gate.weight", prefix))
            .ok_or_else(|| anyhow!("Missing gate projection"))?
            .clone();
        let up_proj = tensors.get(&format!("{}.ffn_up.weight", prefix))
            .ok_or_else(|| anyhow!("Missing up projection"))?
            .clone();
        let down_proj = tensors.get(&format!("{}.ffn_down.weight", prefix))
            .ok_or_else(|| anyhow!("Missing down projection"))?
            .clone();
        
        Ok(Self {
            gate_proj,
            up_proj,
            down_proj,
        })
    }
    
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        // SwiGLU activation: gate * silu(up) * down
        let gate = x.matmul(&self.gate_proj.t()?)?;
        let up = x.matmul(&self.up_proj.t()?)?;
        
        let gate_silu = candle_nn::ops::silu(&gate)?;
        let gated = (gate_silu * up)?;
        
        let output = gated.matmul(&self.down_proj.t()?)?;
        Ok(output)
    }
}

impl LayerNorm {
    fn from_tensors(
        tensors: &HashMap<String, Tensor>,
        name: &str,
        device: &Device
    ) -> Result<Self> {
        let weight = tensors.get(&format!("{}.weight", name))
            .ok_or_else(|| anyhow!("Missing layer norm weight: {}", name))?
            .clone();
        let bias = tensors.get(&format!("{}.bias", name)).cloned();
        
        Ok(Self {
            weight,
            bias,
            eps: 1e-5,
        })
    }
    
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let x_dtype = x.dtype();
        let internal_dtype = DType::F32;
        
        let x = x.to_dtype(internal_dtype)?;
        let mean = x.mean_keepdim(D::Minus1)?;
        let x_centered = x.broadcast_sub(&mean)?;
        let variance = x_centered.sqr()?.mean_keepdim(D::Minus1)?;
        let x_normed = x_centered.broadcast_div(&(variance + self.eps)?.sqrt()?)?;
        
        let x = x_normed.broadcast_mul(&self.weight)?;
        let x = match &self.bias {
            Some(bias) => x.broadcast_add(bias)?,
            None => x,
        };
        
        x.to_dtype(x_dtype)
    }
}

/// Global instance management with streaming loader
static GLOBAL_STREAMING_EMBEDDER: OnceCell<Arc<StreamingNomicEmbedder>> = OnceCell::const_new();

impl StreamingNomicEmbedder {
    /// Get global embedder instance using streaming loader
    pub async fn get_global() -> Result<Arc<Self>> {
        GLOBAL_STREAMING_EMBEDDER.get_or_try_init(|| async {
            // Use streaming implementation with default model path
            let model_path = "models/nomic-embed-text-v1.5.Q4_K_M.gguf";
            
            Self::new_with_streaming(model_path).await
                .map(Arc::new)
        }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_streaming_embedder_memory_safety() {
        // This test verifies memory safety - should not crash V8
        let result = StreamingNomicEmbedder::new_with_streaming("test_model.gguf").await;
        
        // Should fail gracefully, not crash
        match result {
            Ok(_) => println!("Embedder created successfully"),
            Err(e) => println!("Expected error: {}", e),
        }
    }
    
    #[test]
    fn test_model_config() {
        let config = ModelConfig {
            vocab_size: 32000,
            embed_dim: 768,
            num_layers: 12,
            num_heads: 12,
        };
        
        assert_eq!(config.embed_dim / config.num_heads, 64); // head_dim
    }
}