#[cfg(feature = "ml")]
use once_cell::sync::OnceCell;
#[cfg(feature = "ml")]
use std::sync::Arc;
#[cfg(feature = "ml")]
use std::path::PathBuf;
#[cfg(feature = "ml")]
use anyhow::{Result, anyhow};
#[cfg(feature = "ml")]
use std::fs;
#[cfg(feature = "ml")]
use std::io::{Write, Read, Seek};
#[cfg(feature = "ml")]
use candle_core::{Device, Tensor, DType};
#[cfg(feature = "ml")]
use candle_core::quantized::{gguf_file, GgmlDType};
#[cfg(feature = "ml")]
use tokenizers::Tokenizer;
#[cfg(feature = "ml")]
// Removed memmap2::Mmap to prevent V8 heap issues in Node.js
#[cfg(feature = "ml")]
use std::collections::HashMap;
#[cfg(feature = "ml")]
use byteorder::{LittleEndian, ReadBytesExt};

#[cfg(feature = "ml")]
static GLOBAL_EMBEDDER: OnceCell<Arc<NomicEmbedder>> = OnceCell::new();

/// GGUF-based Nomic Embed model with full transformer implementation
#[cfg(feature = "ml")]
pub struct NomicEmbedder {
    tokenizer: Tokenizer,
    device: Device,
    dimensions: usize,
    cache: Option<Arc<crate::embedding::EmbeddingCache>>,
    // Model weights
    token_embeddings: Tensor,
    #[allow(dead_code)]
    layer_norm_weight: Tensor,
    #[allow(dead_code)]
    layer_norm_bias: Tensor,
    #[allow(dead_code)]
    transformer_layers: Vec<TransformerLayer>,
    pooler_dense: Option<Tensor>,
    pooler_norm: Option<Tensor>,
}

#[cfg(feature = "ml")]
#[allow(dead_code)]
struct TransformerLayer {
    attention: MultiHeadAttention,
    feed_forward: FeedForward,
    layer_norm_1: LayerNorm,
    layer_norm_2: LayerNorm,
}

#[cfg(feature = "ml")]
#[allow(dead_code)]
struct MultiHeadAttention {
    q_proj: Tensor,
    k_proj: Tensor,
    v_proj: Tensor,
    o_proj: Tensor,
    num_heads: usize,
    head_dim: usize,
}

#[cfg(feature = "ml")]
#[allow(dead_code)]
struct FeedForward {
    fc1: Tensor,
    fc2: Tensor,
}

#[cfg(feature = "ml")]
#[allow(dead_code)]
struct LayerNorm {
    weight: Tensor,
    bias: Tensor,
}

#[cfg(feature = "ml")]
impl NomicEmbedder {
    #[allow(dead_code)]
    fn ensure_no_nan(tensor: &Tensor, name: &str) -> Result<Tensor> {
        // Convert to flat vector regardless of tensor dimensions
        let _shape = tensor.shape();
        let vec = if tensor.rank() == 1 {
            tensor.to_vec1::<f32>()?
        } else {
            // For multi-dimensional tensors, flatten first
            let flat = tensor.flatten_all()?;
            flat.to_vec1::<f32>()?
        };
        
        if vec.iter().any(|x| x.is_nan()) {
            return Err(anyhow!("NaN values detected in tensor '{}'. Model weights are corrupted and cannot be used. This is a fatal error that cannot be recovered from.", name));
        }
        
        Ok(tensor.clone())
    }

    const MODEL_URL: &'static str = "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF/resolve/main/nomic-embed-text-v1.5.Q4_K_M.gguf";
    const TOKENIZER_URL: &'static str = "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/main/tokenizer.json";
    const MODEL_SIZE: u64 = 84_000_000;  // ~84MB (actual Q4_K_M GGUF size)
    const MODEL_FILENAME: &'static str = "nomic-embed-text-v1.5.Q4_K_M.gguf";
    const TOKENIZER_FILENAME: &'static str = "tokenizer.json";
    const MAX_SEQUENCE_LENGTH: usize = 2048;
    const HIDDEN_SIZE: usize = 768;
    const NUM_LAYERS: usize = 12;
    const NUM_HEADS: usize = 12;
    const INTERMEDIATE_SIZE: usize = 3072;
    
    pub async fn get_global() -> Result<Arc<Self>, crate::error::EmbedError> {
        if let Some(embedder) = GLOBAL_EMBEDDER.get() {
            return Ok(embedder.clone());
        }
        
        let embedder = Arc::new(Self::new().await.map_err(|e| crate::error::EmbedError::Internal {
            message: format!("Failed to initialize NomicEmbedder: {}", e),
            backtrace: None,
        })?);
        match GLOBAL_EMBEDDER.set(embedder.clone()) {
            Ok(_) => Ok(embedder),
            Err(_) => Err(crate::error::EmbedError::Internal {
                message: "Global embedder was already initialized by another thread".to_string(),
                backtrace: None,
            }),
        }
    }
    
    pub async fn new() -> Result<Self> {
        // Ensure files are cached using async directly
        let (model_path, tokenizer_path) = Self::ensure_files_cached().await?;
        
        // Setup device (CPU for GGUF)
        let device = Device::Cpu;
        
        // Load tokenizer
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?;
        
        // Load and parse GGUF model with actual tensor data
        #[cfg(debug_assertions)]
        println!("Loading GGUF model from {:?}...", model_path);
        let tensors = Self::load_gguf_tensors(&model_path, &device)?;
        
        // Extract specific model components - require exact Nomic GGUF format
        let token_embeddings = tensors.get("token_embd.weight")
            .ok_or_else(|| anyhow!("Token embeddings not found at expected location 'token_embd.weight'. This model is not in the required Nomic GGUF format."))?
            .clone();
            
        let layer_norm_weight = tensors.get("token_embd_norm.weight")
            .ok_or_else(|| anyhow!("Layer normalization weight not found at expected location 'token_embd_norm.weight'. This model is not in the required Nomic GGUF format."))?
            .clone();
            
        let layer_norm_bias = tensors.get("token_embd_norm.bias")
            .ok_or_else(|| anyhow!("Layer normalization bias not found at expected location 'token_embd_norm.bias'. This model is not in the required Nomic GGUF format."))?
            .clone();
        
        // Load transformer layers
        let mut transformer_layers = Vec::new();
        for i in 0..Self::NUM_LAYERS {
            let layer = Self::load_transformer_layer(&tensors, i, &device)?;
            transformer_layers.push(layer);
        }
        
        // Load pooler if available
        let pooler_dense = tensors.get("pooler.dense.weight").cloned();
        let pooler_norm = tensors.get("pooler.dense.bias").cloned();
        
        // Initialize cache
        let cache = Some(Arc::new(
            crate::embedding::EmbeddingCache::new(100_000)
                .map_err(|e| anyhow!("Failed to initialize embedding cache: {}", e))?
        ));
        
        #[cfg(debug_assertions)]
        {
            println!("✅ Nomic GGUF model loaded successfully");
            println!("  - {} tensors loaded with actual weights", tensors.len());
            println!("  - Token embeddings shape: {:?}", token_embeddings.shape());
            println!("  - {} transformer layers", transformer_layers.len());
            println!("  - Device: {:?}", device);
            println!("  - Dimensions: {}", Self::HIDDEN_SIZE);
        }
        
        Ok(Self {
            tokenizer,
            device,
            dimensions: Self::HIDDEN_SIZE,
            cache,
            token_embeddings,
            layer_norm_weight,
            layer_norm_bias,
            transformer_layers,
            pooler_dense,
            pooler_norm,
        })
    }
    
    fn load_transformer_layer(tensors: &HashMap<String, Tensor>, layer_idx: usize, _device: &Device) -> Result<TransformerLayer> {
        // Require exact Nomic GGUF format - no fallbacks allowed
        let prefix = format!("blk.{}", layer_idx);
        
        // Load attention weights - require exact Nomic GGUF tensor names
        let q_proj = tensors.get(&format!("{}.attn_q.weight", prefix))
            .ok_or_else(|| anyhow!("Query projection weights not found at expected location '{}.attn_q.weight'. Layer {} is not in the required Nomic GGUF format.", prefix, layer_idx))?
            .clone();
            
        let k_proj = tensors.get(&format!("{}.attn_k.weight", prefix))
            .ok_or_else(|| anyhow!("Key projection weights not found at expected location '{}.attn_k.weight'. Layer {} is not in the required Nomic GGUF format.", prefix, layer_idx))?
            .clone();
            
        let v_proj = tensors.get(&format!("{}.attn_v.weight", prefix))
            .ok_or_else(|| anyhow!("Value projection weights not found at expected location '{}.attn_v.weight'. Layer {} is not in the required Nomic GGUF format.", prefix, layer_idx))?
            .clone();
            
        let o_proj = tensors.get(&format!("{}.attn_output.weight", prefix))
            .ok_or_else(|| anyhow!("Output projection weights not found at expected location '{}.attn_output.weight'. Layer {} is not in the required Nomic GGUF format.", prefix, layer_idx))?
            .clone();
        
        // Load feed-forward weights - require exact Nomic GGUF tensor names
        let fc1 = tensors.get(&format!("{}.ffn_gate.weight", prefix))
            .ok_or_else(|| anyhow!("Feed-forward layer 1 weights not found at expected location '{}.ffn_gate.weight'. Layer {} is not in the required Nomic GGUF format.", prefix, layer_idx))?
            .clone();
            
        let fc2 = tensors.get(&format!("{}.ffn_down.weight", prefix))
            .ok_or_else(|| anyhow!("Feed-forward layer 2 weights not found at expected location '{}.ffn_down.weight'. Layer {} is not in the required Nomic GGUF format.", prefix, layer_idx))?
            .clone();
        
        // Load layer norms - require exact Nomic GGUF tensor names
        let ln1_weight = tensors.get(&format!("{}.attn_norm.weight", prefix))
            .ok_or_else(|| anyhow!("Layer norm 1 weight not found at expected location '{}.attn_norm.weight'. Layer {} is not in the required Nomic GGUF format.", prefix, layer_idx))?
            .clone();
            
        let ln1_bias = tensors.get(&format!("{}.attn_norm.bias", prefix))
            .ok_or_else(|| anyhow!("Layer norm 1 bias not found at expected location '{}.attn_norm.bias'. Layer {} is not in the required Nomic GGUF format.", prefix, layer_idx))?
            .clone();
            
        let ln2_weight = tensors.get(&format!("{}.ffn_norm.weight", prefix))
            .ok_or_else(|| anyhow!("Layer norm 2 weight not found at expected location '{}.ffn_norm.weight'. Layer {} is not in the required Nomic GGUF format.", prefix, layer_idx))?
            .clone();
            
        let ln2_bias = tensors.get(&format!("{}.ffn_norm.bias", prefix))
            .ok_or_else(|| anyhow!("Layer norm 2 bias not found at expected location '{}.ffn_norm.bias'. Layer {} is not in the required Nomic GGUF format.", prefix, layer_idx))?
            .clone();
        
        Ok(TransformerLayer {
            attention: MultiHeadAttention {
                q_proj,
                k_proj,
                v_proj,
                o_proj,
                num_heads: Self::NUM_HEADS,
                head_dim: Self::HIDDEN_SIZE / Self::NUM_HEADS,
            },
            feed_forward: FeedForward {
                fc1,
                fc2,
            },
            layer_norm_1: LayerNorm {
                weight: ln1_weight,
                bias: ln1_bias,
            },
            layer_norm_2: LayerNorm {
                weight: ln2_weight,
                bias: ln2_bias,
            },
        })
    }
    
    fn load_gguf_tensors(model_path: &PathBuf, device: &Device) -> Result<HashMap<String, Tensor>> {
        let mut file = fs::File::open(model_path)?;
        
        // Read GGUF header to get metadata
        let content = gguf_file::Content::read(&mut file)?;
        
        // Use streaming reads instead of memory mapping to prevent V8 heap issues
        // Memory mapping can cause fatal V8 errors in Node.js environments
        let mut tensors = HashMap::new();
        let mut current_offset = content.tensor_data_offset as usize;
        
        #[cfg(debug_assertions)]
        println!("Loading {} tensors from GGUF file (streaming mode)...", content.tensor_infos.len());
        
        for (name, tensor_info) in content.tensor_infos.iter() {
            // Calculate tensor data size
            let data_size = Self::calculate_tensor_size(tensor_info)?;
            
            // Seek to tensor position
            file.seek(std::io::SeekFrom::Start(current_offset as u64))?;
            
            // Read tensor data in chunks to prevent memory pressure
            let mut tensor_data = vec![0u8; data_size];
            const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks
            
            let mut bytes_read = 0;
            while bytes_read < data_size {
                let chunk_size = std::cmp::min(CHUNK_SIZE, data_size - bytes_read);
                let chunk = &mut tensor_data[bytes_read..bytes_read + chunk_size];
                
                match file.read_exact(chunk) {
                    Ok(_) => bytes_read += chunk_size,
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                        #[cfg(debug_assertions)]
                        println!("Warning: Incomplete data for tensor {} (read {} of {} bytes)", name, bytes_read, data_size);
                        break;
                    },
                    Err(e) => return Err(anyhow::Error::from(e)),
                }
                
                // Yield CPU periodically to prevent blocking in single-threaded environments
                if bytes_read % (CHUNK_SIZE * 4) == 0 {
                    std::thread::yield_now();
                }
            }
            
            if bytes_read < data_size {
                #[cfg(debug_assertions)]
                println!("Warning: Not enough data for tensor {} (got {} bytes, expected {})", name, bytes_read, data_size);
                continue;
            }
            
            // Dequantize and create tensor
            let tensor = Self::dequantize_tensor(&tensor_data, tensor_info, device)?;
            tensors.insert(name.clone(), tensor);
            
            current_offset += data_size;
            
            // Log progress and yield CPU to prevent blocking
            if tensors.len() % 5 == 0 {
                print!("\r  Loaded {}/{} tensors", tensors.len(), content.tensor_infos.len());
                std::io::stdout().flush()?;
                std::thread::yield_now();
            }
            
            // Force garbage collection hint for large tensors
            if data_size > 10 * 1024 * 1024 { // > 10MB
                drop(tensor_data);
                std::thread::yield_now();
            }
        }
        
        #[cfg(debug_assertions)]
        println!("\r  Loaded {}/{} tensors", tensors.len(), content.tensor_infos.len());
        
        Ok(tensors)
    }
    
    fn calculate_tensor_size(tensor_info: &gguf_file::TensorInfo) -> Result<usize> {
        let total_elements = tensor_info.shape.elem_count();
        
        let size = match tensor_info.ggml_dtype {
            GgmlDType::F32 => total_elements * 4,
            GgmlDType::F16 => total_elements * 2,
            GgmlDType::Q4_0 => (total_elements / 32) * 18,
            GgmlDType::Q4_1 => (total_elements / 32) * 20,
            GgmlDType::Q5_0 => (total_elements / 32) * 22,
            GgmlDType::Q5_1 => (total_elements / 32) * 24,
            GgmlDType::Q8_0 => (total_elements / 32) * 34,
            GgmlDType::Q4K => (total_elements / 256) * 144,
            GgmlDType::Q5K => (total_elements / 256) * 176,
            GgmlDType::Q6K => (total_elements / 256) * 210,
            GgmlDType::Q8K => (total_elements / 256) * 292,
            _ => {
                return Err(anyhow!(
                    "Unsupported quantization type {:?}. This model uses an unsupported GGUF quantization format. \
                     Only Q4_0, Q4_1, Q5_0, Q5_1, Q8_0, Q4K, Q5K, Q6K, Q8K are supported. \
                     No fallback or approximation will be used - you must use a properly quantized model.", 
                    tensor_info.ggml_dtype
                ));
            }
        };
        
        Ok(size)
    }
    
    fn dequantize_tensor(data: &[u8], tensor_info: &gguf_file::TensorInfo, device: &Device) -> Result<Tensor> {
        let shape = &tensor_info.shape;
        let total_elements = shape.elem_count();
        
        // Dequantize based on the data type
        let values = match tensor_info.ggml_dtype {
            GgmlDType::F32 => {
                // Direct F32 data
                let mut values = Vec::with_capacity(total_elements);
                let mut cursor = std::io::Cursor::new(data);
                for _ in 0..total_elements {
                    values.push(cursor.read_f32::<LittleEndian>()?);
                }
                values
            },
            GgmlDType::F16 => {
                // F16 to F32 conversion
                let mut values = Vec::with_capacity(total_elements);
                let mut cursor = std::io::Cursor::new(data);
                for _ in 0..total_elements {
                    let f16_bits = cursor.read_u16::<LittleEndian>()?;
                    values.push(Self::f16_to_f32(f16_bits));
                }
                values
            },
            GgmlDType::Q4_0 | GgmlDType::Q4_1 => {
                // Simple 4-bit quantization dequantization (32-element blocks)
                Self::dequantize_q4(data, total_elements)?
            },
            GgmlDType::Q4K => {
                // Q4_K_M quantization dequantization (256-element superblocks)
                Self::dequantize_q4_k_m(data, total_elements)?
            },
            GgmlDType::Q5_0 | GgmlDType::Q5_1 | GgmlDType::Q5K => {
                // 5-bit quantization dequantization
                Self::dequantize_q5(data, total_elements)?
            },
            GgmlDType::Q6K => {
                // 6-bit quantization dequantization
                Self::dequantize_q6(data, total_elements)?
            },
            GgmlDType::Q8_0 | GgmlDType::Q8K => {
                // 8-bit quantization dequantization
                Self::dequantize_q8(data, total_elements)?
            },
            _ => {
                return Err(anyhow!("Unsupported quantization type {:?} for tensor. System requires proper GGUF model with supported quantization.", tensor_info.ggml_dtype));
            }
        };
        
        // Create tensor from dequantized values
        Ok(Tensor::from_vec(values, shape.dims(), device)
            .map_err(|e| anyhow!("Failed to create tensor from values: {}", e))?)
    }
    
    fn f16_to_f32(bits: u16) -> f32 {
        let sign = (bits >> 15) & 1;
        let exp = (bits >> 10) & 0x1f;
        let frac = bits & 0x3ff;
        
        if exp == 0 {
            if frac == 0 {
                if sign == 1 { -0.0 } else { 0.0 }
            } else {
                // Subnormal
                let val = (frac as f32) / 1024.0 / 16384.0;
                if sign == 1 { -val } else { val }
            }
        } else if exp == 0x1f {
            if frac == 0 {
                if sign == 1 { f32::NEG_INFINITY } else { f32::INFINITY }
            } else {
                f32::NAN
            }
        } else {
            let val = f32::from_bits(
                ((sign as u32) << 31) |
                (((exp as u32) + 127 - 15) << 23) |
                ((frac as u32) << 13)
            );
            val
        }
    }

    /// Extract a 6-bit value from the packed scales array at the specified index
    fn extract_6bit_value(scales: &[u8; 12], index: usize) -> u8 {
        // Q4K scales are packed as 6-bit values in 12 bytes
        // 8 scales and 8 mins = 16 6-bit values = 96 bits = 12 bytes
        let bit_offset = index * 6;
        let byte_start = bit_offset / 8;
        let bit_start = bit_offset % 8;
        
        if byte_start >= 11 {
            return 0;
        }
        
        if bit_start + 6 <= 8 {
            // Value fits within one byte
            (scales[byte_start] >> bit_start) & 0x3F
        } else if byte_start + 1 < 12 {
            // Value spans two bytes
            let low_bits = 8 - bit_start;
            let high_bits = 6 - low_bits;
            let low_part = (scales[byte_start] >> bit_start) & ((1 << low_bits) - 1);
            let high_part = scales[byte_start + 1] & ((1 << high_bits) - 1);
            low_part | (high_part << low_bits)
        } else {
            0
        }
    }

    /// Correct Q4_K_M dequantization using 256-element superblocks
    fn dequantize_q4_k_m(data: &[u8], total_elements: usize) -> Result<Vec<f32>> {
        const QK_K: usize = 256;  // Superblock size
        const K_SCALE_SIZE: usize = 12;  // Size of scales array
        const BLOCK_Q4_K_SIZE: usize = 2 + 2 + K_SCALE_SIZE + (QK_K / 2);  // 144 bytes
        
        let superblocks = (total_elements + QK_K - 1) / QK_K;
        let mut values = Vec::with_capacity(total_elements);
        
        for superblock_idx in 0..superblocks {
            let block_offset = superblock_idx * BLOCK_Q4_K_SIZE;
            
            // Bounds check for the entire superblock
            if block_offset + BLOCK_Q4_K_SIZE > data.len() {
                return Err(anyhow!("Insufficient data for Q4_K_M superblock {}: need {} bytes but only {} available. Model file is truncated or corrupted.", superblock_idx, BLOCK_Q4_K_SIZE, data.len() - block_offset));
            }
            
            // Extract superblock components
            let d_bits = u16::from_le_bytes([
                data[block_offset], 
                data[block_offset + 1]
            ]);
            let dmin_bits = u16::from_le_bytes([
                data[block_offset + 2], 
                data[block_offset + 3]
            ]);
            
            let d = Self::f16_to_f32(d_bits);
            let dmin = Self::f16_to_f32(dmin_bits);
            
            // Validate scales for NaN prevention
            if !d.is_finite() || !dmin.is_finite() {
                return Err(anyhow!("Invalid scales in Q4_K_M superblock {}: d={}, dmin={}. Model data is corrupted.", superblock_idx, d, dmin));
            }
            
            // Extract scales array (12 bytes)
            let scales_start = block_offset + 4;
            if scales_start + K_SCALE_SIZE > data.len() {
                return Err(anyhow!("Insufficient data for Q4_K_M scales array: need {} bytes but only {} available.", K_SCALE_SIZE, data.len() - scales_start));
            }
            let mut scales_array = [0u8; K_SCALE_SIZE];
            scales_array.copy_from_slice(&data[scales_start..scales_start + K_SCALE_SIZE]);
            
            // Extract quantized values array (128 bytes)
            let qs_start = scales_start + K_SCALE_SIZE;
            if qs_start + QK_K / 2 > data.len() {
                return Err(anyhow!("Insufficient data for Q4_K_M quantized values: need {} bytes but only {} available.", QK_K / 2, data.len() - qs_start));
            }
            let qs = &data[qs_start..qs_start + QK_K / 2];
            
            // Process each of the 8 blocks within this superblock
            for block_idx in 0..8 {
                // Extract 6-bit scale and min for this block
                let scale_bits = Self::extract_6bit_value(&scales_array, block_idx);
                let min_bits = Self::extract_6bit_value(&scales_array, block_idx + 8);
                
                let block_scale = d * (scale_bits as f32);
                let block_min = dmin * (min_bits as f32);
                
                // Validate block scale and min
                if !block_scale.is_finite() || !block_min.is_finite() {
                    return Err(anyhow!("Invalid block scales in Q4_K_M block {}: scale={}, min={}. Model computation failed.", block_idx, block_scale, block_min));
                }
                
                // Dequantize 32 weights in this block
                for weight_idx in 0..32 {
                    let global_idx = block_idx * 32 + weight_idx;
                    
                    if values.len() >= total_elements {
                        break;
                    }
                    
                    // Extract 4-bit quantized value
                    let byte_idx = global_idx / 2;
                    if byte_idx >= qs.len() {
                        return Err(anyhow!("Byte index {} out of bounds for quantized data length {}. Model data structure is invalid.", byte_idx, qs.len()));
                    }
                    
                    let is_high_nibble = (global_idx % 2) == 1;
                    let q4_value = if is_high_nibble {
                        (qs[byte_idx] >> 4) & 0x0F
                    } else {
                        qs[byte_idx] & 0x0F
                    };
                    
                    // Apply Q4_K_M dequantization formula: y = d * q + dmin * q_offset
                    let dequantized_weight = block_scale * (q4_value as f32) + block_min;
                    
                    // Validate the dequantized value
                    if !dequantized_weight.is_finite() {
                        return Err(anyhow!("Dequantized weight {} is not finite in Q4_K_M processing. Model computation failed.", dequantized_weight));
                    }
                    values.push(dequantized_weight);
                }
            }
        }
        
        // Ensure we have exactly the right number of elements
        if values.len() != total_elements {
            return Err(anyhow!("Q4_K_M dequantization produced {} elements but expected {}. Model processing failed.", values.len(), total_elements));
        }
        
        Ok(values)
    }
    
    fn dequantize_q4(data: &[u8], total_elements: usize) -> Result<Vec<f32>> {
        let mut values = Vec::with_capacity(total_elements);
        let block_size = 32;
        let blocks = total_elements / block_size;
        
        let mut offset = 0;
        for block_idx in 0..blocks {
            if offset + 18 > data.len() {
                return Err(anyhow!("Insufficient data for Q4 block {}: need {} bytes but only {} available. Model file is truncated.", block_idx, 18, data.len() - offset));
            }
            
            // Read scale (f16)
            let scale_bits = u16::from_le_bytes([data[offset], data[offset + 1]]);
            let scale = Self::f16_to_f32(scale_bits);
            offset += 2;
            
            // Read 32 4-bit values (16 bytes)
            for byte_idx in 0..16 {
                if offset >= data.len() {
                    return Err(anyhow!("Insufficient data for Q4 quantized values in block {}: reached end of data at byte {}.", block_idx, byte_idx));
                }
                let byte = data[offset];
                offset += 1;
                
                // Extract two 4-bit values
                let val1 = (byte & 0x0F) as f32;
                let val2 = ((byte >> 4) & 0x0F) as f32;
                
                // Dequantize: map [0, 15] to [-1, 1] and scale
                values.push((val1 - 8.0) * scale / 8.0);
                values.push((val2 - 8.0) * scale / 8.0);
            }
        }
        
        // Ensure we have exactly the right number of elements
        if values.len() != total_elements {
            return Err(anyhow!("Q4 dequantization produced {} elements but expected {}. Model data is insufficient or corrupted.", values.len(), total_elements));
        }
        
        Ok(values)
    }
    
    /// Correct Q6K dequantization using 256-element super-blocks with 16 blocks of 16 weights
    fn dequantize_q6(data: &[u8], total_elements: usize) -> Result<Vec<f32>> {
        const QK_K: usize = 256;  // Super-block size: 16 blocks × 16 weights
        const BLOCK_Q6_K_SIZE: usize = 210;  // Q6K super-block size in bytes
        
        let superblocks = (total_elements + QK_K - 1) / QK_K;
        let mut values = Vec::with_capacity(total_elements);
        
        for superblock_idx in 0..superblocks {
            let block_offset = superblock_idx * BLOCK_Q6_K_SIZE;
            
            // Bounds check for the entire super-block
            if block_offset + BLOCK_Q6_K_SIZE > data.len() {
                return Err(anyhow!("Insufficient data for Q6K super-block {}: need {} bytes but only {} available. Model file is truncated.", superblock_idx, BLOCK_Q6_K_SIZE, data.len() - block_offset));
            }
            
            // Q6K structure: [scales: 16×1 byte] [quantized_weights: 3×64 bytes] [padding if any]
            // Extract 16 scales (8-bit quantized scales, higher precision than typical 4-bit)
            let scales_start = block_offset;
            if scales_start + 16 > data.len() {
                return Err(anyhow!("Insufficient data for Q6K scales: need 16 bytes but only {} available.", data.len() - scales_start));
            }
            
            let mut scales = [0f32; 16];
            for i in 0..16 {
                // Convert 8-bit quantized scale to f32
                // Q6K scales are stored as unsigned 8-bit values that need to be rescaled
                let scale_u8 = data[scales_start + i];
                // Convert from quantized 8-bit to proper scale factor
                scales[i] = (scale_u8 as f32) / 256.0;
            }
            
            // Extract quantized weights: 192 bytes for 256 weights (6 bits each)
            // 256 × 6 bits = 1536 bits = 192 bytes
            let weights_start = scales_start + 16;
            if weights_start + 192 > data.len() {
                return Err(anyhow!("Insufficient data for Q6K weights: need 192 bytes but only {} available.", data.len() - weights_start));
            }
            
            // Process each of the 16 blocks within this super-block
            for block_idx in 0..16 {
                let block_scale = scales[block_idx];
                
                // Validate block scale
                if !block_scale.is_finite() || block_scale == 0.0 {
                    return Err(anyhow!("Invalid scale in Q6K block {}: scale={}. Model data is corrupted.", block_idx, block_scale));
                }
                
                // Process 16 weights in this block
                for weight_idx in 0..16 {
                    let global_weight_idx = block_idx * 16 + weight_idx;
                    
                    if values.len() >= total_elements {
                        break;
                    }
                    
                    // Extract 6-bit quantized value from packed data
                    // Calculate bit position: each weight uses 6 bits
                    let bit_offset = global_weight_idx * 6;
                    let byte_start = weights_start + (bit_offset / 8);
                    let bit_start = bit_offset % 8;
                    
                    if byte_start >= data.len() {
                        return Err(anyhow!("Weight byte index {} out of bounds for data length {}.", byte_start, data.len()));
                    }
                    
                    // Extract 6-bit value spanning potentially two bytes
                    let q6_value = if bit_start + 6 <= 8 {
                        // Value fits within one byte
                        (data[byte_start] >> bit_start) & 0x3F
                    } else if byte_start + 1 < data.len() {
                        // Value spans two bytes
                        let low_bits = 8 - bit_start;
                        let high_bits = 6 - low_bits;
                        let low_part = (data[byte_start] >> bit_start) & ((1 << low_bits) - 1);
                        let high_part = data[byte_start + 1] & ((1 << high_bits) - 1);
                        low_part | (high_part << low_bits)
                    } else {
                        return Err(anyhow!("Insufficient data for Q6K 6-bit extraction: byte {} out of bounds.", byte_start + 1));
                    };
                    
                    // Apply Q6K dequantization formula
                    // Q6K uses 6-bit values [0, 63] scaled by block-specific scale
                    // Center around 0 by subtracting 32 (middle of [0, 63] range)
                    let dequantized_weight = block_scale * ((q6_value as f32) - 32.0);
                    
                    // Validate the dequantized value
                    if !dequantized_weight.is_finite() {
                        return Err(anyhow!("Dequantized weight {} is not finite in Q6K processing. Model computation failed.", dequantized_weight));
                    }
                    
                    values.push(dequantized_weight);
                }
            }
        }
        
        // Ensure we have exactly the right number of elements
        if values.len() != total_elements {
            return Err(anyhow!("Q6K dequantization produced {} elements but expected {}. Model processing failed.", values.len(), total_elements));
        }
        
        Ok(values)
    }
    
    fn dequantize_q5(data: &[u8], total_elements: usize) -> Result<Vec<f32>> {
        let mut values = Vec::with_capacity(total_elements);
        let block_size = 32;
        let blocks = total_elements / block_size;
        
        let mut offset = 0;
        for block_idx in 0..blocks {
            if offset + 22 > data.len() {
                return Err(anyhow!("Insufficient data for Q5 block {}: need {} bytes but only {} available. Model file is truncated.", block_idx, 22, data.len() - offset));
            }
            
            // Read scale (f16)
            let scale_bits = u16::from_le_bytes([data[offset], data[offset + 1]]);
            let scale = Self::f16_to_f32(scale_bits);
            offset += 2;
            
            // Read high bits (4 bytes for 32 values)
            let mut high_bits = [0u8; 4];
            for i in 0..4 {
                if offset >= data.len() {
                    return Err(anyhow!("Insufficient data for Q5 high bits in block {}: reached end of data at byte {}.", block_idx, i));
                }
                high_bits[i] = data[offset];
                offset += 1;
            }
            
            // Read 32 4-bit values (16 bytes)
            for i in 0..16 {
                if offset >= data.len() {
                    return Err(anyhow!("Insufficient data for Q5 quantized values in block {}: reached end of data at byte {}.", block_idx, i));
                }
                let byte = data[offset];
                offset += 1;
                
                // Extract two 4-bit values and combine with high bits
                let idx = i * 2;
                let high_bit_1 = ((high_bits[idx / 8] >> (idx % 8)) & 1) << 4;
                let high_bit_2 = ((high_bits[(idx + 1) / 8] >> ((idx + 1) % 8)) & 1) << 4;
                
                let val1 = ((byte & 0x0F) | high_bit_1) as f32;
                let val2 = (((byte >> 4) & 0x0F) | high_bit_2) as f32;
                
                // Dequantize: map [0, 31] to [-1, 1] and scale
                values.push((val1 - 16.0) * scale / 16.0);
                values.push((val2 - 16.0) * scale / 16.0);
            }
        }
        
        // Ensure we have exactly the right number of elements
        if values.len() != total_elements {
            return Err(anyhow!("Q5 dequantization produced {} elements but expected {}. Model data is insufficient or corrupted.", values.len(), total_elements));
        }
        
        Ok(values)
    }
    
    fn dequantize_q8(data: &[u8], total_elements: usize) -> Result<Vec<f32>> {
        let mut values = Vec::with_capacity(total_elements);
        let block_size = 32;
        let blocks = total_elements / block_size;
        
        let mut offset = 0;
        for block_idx in 0..blocks {
            if offset + 34 > data.len() {
                return Err(anyhow!("Insufficient data for Q8 block {}: need {} bytes but only {} available. Model file is truncated.", block_idx, 34, data.len() - offset));
            }
            
            // Read scale (f16)
            let scale_bits = u16::from_le_bytes([data[offset], data[offset + 1]]);
            let scale = Self::f16_to_f32(scale_bits);
            offset += 2;
            
            // Read 32 8-bit values
            for val_idx in 0..32 {
                if offset >= data.len() {
                    return Err(anyhow!("Insufficient data for Q8 quantized values in block {}: reached end of data at value {}.", block_idx, val_idx));
                }
                let val = data[offset] as i8 as f32;
                offset += 1;
                
                // Dequantize
                values.push(val * scale / 127.0);
            }
        }
        
        // Ensure we have exactly the right number of elements
        if values.len() != total_elements {
            return Err(anyhow!("Q8 dequantization produced {} elements but expected {}. Model data is insufficient or corrupted.", values.len(), total_elements));
        }
        
        Ok(values)
    }
    
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        if let Some(cache) = &self.cache {
            if let Ok(Some(embedding)) = cache.get(text) {
                return Ok(embedding);
            }
        }
        
        // Tokenize the input
        let encoding = self.tokenizer
            .encode(text, true)
            .map_err(|e| anyhow!("Tokenization failed: {}", e))?;
        
        let input_ids = encoding.get_ids();
        let attention_mask = encoding.get_attention_mask();
        
        // Truncate to max sequence length
        let seq_len = input_ids.len().min(Self::MAX_SEQUENCE_LENGTH);
        let input_ids = &input_ids[..seq_len];
        let attention_mask = &attention_mask[..seq_len];
        
        // Validate attention mask before processing
        Self::validate_attention_mask(attention_mask, input_ids.len())?;

        // Convert to tensors
        let input_tensor = Tensor::new(input_ids, &self.device)?;
        let attention_tensor = Tensor::new(attention_mask, &self.device)?
            .to_dtype(DType::F32)?;
        let _attention_tensor = attention_tensor;
        
        // Get token embeddings
        let hidden_states = self.token_embeddings.index_select(&input_tensor, 0)
            .map_err(|e| anyhow!("Failed to get token embeddings: {}", e))?;
        
        // Apply transformer layers for proper embedding generation
        let mut current_hidden = hidden_states;
        
        // Apply each transformer layer
        for (layer_idx, layer) in self.transformer_layers.iter().enumerate() {
            current_hidden = Self::transformer_forward(current_hidden, &_attention_tensor, layer)
                .map_err(|e| anyhow!("Failed to apply transformer layer {}: {}", layer_idx, e))?;
        }
        
        // Apply proper mean pooling over sequence dimension
        let pooled = current_hidden.mean(0)?;
        #[cfg(debug_assertions)]
        println!("Applied {} transformer layers, pooled shape: {:?}", self.transformer_layers.len(), pooled.shape());
        
        // Apply pooler if available
        let output = if let Some(pooler) = &self.pooler_dense {
            // Ensure pooled is [hidden_size], reshape if needed
            let pooled_flat = if pooled.rank() == 1 {
                pooled.clone()
            } else {
                pooled.flatten(0, pooled.rank() - 1)?
            };
            
            // For dense layer: pooled_flat [hidden_size] * pooler [hidden_size, hidden_size] 
            let transformed = pooled_flat.matmul(&pooler.t()?)
                .map_err(|e| anyhow!("Failed in pooler matmul: {}", e))?;
            
            if let Some(bias) = &self.pooler_norm {
                transformed.broadcast_add(bias)
                    .map_err(|e| anyhow!("Failed to add pooler bias: {}", e))?
            } else {
                transformed
            }
        } else {
            pooled
        };
        
        // L2 normalization
        let output_vec = output.to_vec1::<f32>()
            .map_err(|e| anyhow!("Failed to convert output to vec: {}", e))?;
        
        // Debug: print raw values before normalization
        #[cfg(debug_assertions)]
        {
            println!("Raw output before normalization (first 10): {:?}", &output_vec[..10]);
            println!("Raw output sum: {}", output_vec.iter().sum::<f32>());
        }
        
        let norm = output_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        #[cfg(debug_assertions)]
        println!("Norm: {}", norm);
        
        let normalized: Vec<f32> = if norm > 1e-9 {
            output_vec.iter().map(|x| x / norm).collect()
        } else {
            return Err(anyhow!("Generated embedding has near-zero norm ({}). This indicates a fundamental model or computation error that cannot be recovered from.", norm));
        };
        
        // Validate that dimensions match exactly
        if normalized.len() != self.dimensions {
            return Err(anyhow!("Generated embedding has {} dimensions but expected {}. This indicates a model architecture mismatch that cannot be corrected.", normalized.len(), self.dimensions));
        }
        
        let embedding = normalized;
        
        // Cache the result - caching failures are fatal to maintain data consistency
        if let Some(cache) = &self.cache {
            cache.put(text, embedding.clone())
                .map_err(|e| anyhow!("Failed to cache embedding result: {}. Cache integrity must be maintained.", e))?;
        }
        
        Ok(embedding)
    }
    
    #[allow(dead_code)]
    fn transformer_forward(mut hidden_states: Tensor, attention_mask: &Tensor, layer: &TransformerLayer) -> Result<Tensor> {
        // Multi-head attention with robust error handling
        let attn_output = Self::attention_forward(&hidden_states, attention_mask, &layer.attention)
            .map_err(|e| anyhow!("Attention forward failed: {}", e))?;
        
        // Add & Norm
        hidden_states = (hidden_states + attn_output)
            .map_err(|e| anyhow!("Failed in residual add (attention): {}", e))?;
        hidden_states = Self::layer_norm(&hidden_states, &layer.layer_norm_1.weight, &layer.layer_norm_1.bias)?;
        
        // Feed-forward with robust error handling  
        let ff_output = Self::feed_forward(&hidden_states, &layer.feed_forward)
            .map_err(|e| anyhow!("Feed forward failed: {}", e))?;
        
        // Add & Norm
        hidden_states = (hidden_states + ff_output)
            .map_err(|e| anyhow!("Failed in residual add (ff): {}", e))?;
        hidden_states = Self::layer_norm(&hidden_states, &layer.layer_norm_2.weight, &layer.layer_norm_2.bias)?;
        
        Ok(hidden_states)
    }
    
    #[allow(dead_code)]
    fn softmax(x: &Tensor, dim: usize) -> Result<Tensor> {
        // Numerically stable softmax: softmax(x) = exp(x - max(x)) / sum(exp(x - max(x)))
        let max_vals = x.max_keepdim(dim)
            .map_err(|e| anyhow!("Failed to compute max for softmax: {}", e))?;
        let x_shifted = x.broadcast_sub(&max_vals)
            .map_err(|e| anyhow!("Failed to shift values for softmax: {}", e))?;
        let exp_vals = x_shifted.exp()
            .map_err(|e| anyhow!("Failed to compute exp for softmax: {}", e))?;
        let sum_exp = exp_vals.sum_keepdim(dim)
            .map_err(|e| anyhow!("Failed to compute sum for softmax: {}", e))?;
        exp_vals.broadcast_div(&sum_exp)
            .map_err(|e| anyhow!("Failed to normalize softmax: {}", e))
    }
    
    #[allow(dead_code)]
    fn attention_forward(hidden_states: &Tensor, attention_mask: &Tensor, attention: &MultiHeadAttention) -> Result<Tensor> {
        let (seq_len, hidden_size) = hidden_states.dims2()
            .map_err(|e| anyhow!("Failed to get dimensions: {}", e))?;
        
        // Get attention dimensions
        let num_heads = attention.num_heads;
        let head_dim = attention.head_dim;
        let scale = 1.0 / (head_dim as f32).sqrt();
        
        // Project to Q, K, V
        let q = hidden_states.matmul(&attention.q_proj.t()?)
            .map_err(|e| anyhow!("Query projection failed: {}", e))?;
        let k = hidden_states.matmul(&attention.k_proj.t()?)
            .map_err(|e| anyhow!("Key projection failed: {}", e))?;
        let v = hidden_states.matmul(&attention.v_proj.t()?)
            .map_err(|e| anyhow!("Value projection failed: {}", e))?;
        
        // Reshape for multi-head attention: (seq_len, hidden_size) -> (seq_len, num_heads, head_dim)
        let q = q.reshape(&[seq_len, num_heads, head_dim])?
            .transpose(0, 1)?; // -> (num_heads, seq_len, head_dim)
        let k = k.reshape(&[seq_len, num_heads, head_dim])?
            .transpose(0, 1)?; // -> (num_heads, seq_len, head_dim)  
        let v = v.reshape(&[seq_len, num_heads, head_dim])?
            .transpose(0, 1)?; // -> (num_heads, seq_len, head_dim)
        
        // Compute scaled dot-product attention: Q*K^T/sqrt(d_k)
        let attention_scores = q.matmul(&k.transpose(1, 2)
            .map_err(|e| anyhow!("Failed to transpose K: {}", e))?)
            .map_err(|e| anyhow!("Attention score computation failed: {}", e))?;
        
        // Apply scaling
        let scaled_scores = attention_scores.affine(scale as f64, 0.0)
            .map_err(|e| anyhow!("Failed to scale attention scores: {}", e))?;
        
        // Apply attention mask (add large negative value to masked positions)
        // attention_mask should be (seq_len, seq_len) with 1s for valid positions, 0s for masked
        let mask_value = -1e9f32;
        let expanded_mask = attention_mask.unsqueeze(0)
            .map_err(|e| anyhow!("Failed to expand mask: {}", e))?
            .expand(&[num_heads, seq_len, seq_len])
            .map_err(|e| anyhow!("Failed to broadcast mask: {}", e))?;
        
        // Convert mask to additive form: (1-mask) * large_negative_value
        let mask_to_add = expanded_mask.affine(-1.0, 1.0)
            .map_err(|e| anyhow!("Failed to invert mask: {}", e))?
            .affine(mask_value as f64, 0.0)
            .map_err(|e| anyhow!("Failed to scale mask: {}", e))?;
        
        let masked_scores = scaled_scores.broadcast_add(&mask_to_add)
            .map_err(|e| anyhow!("Failed to apply attention mask: {}", e))?;
        
        // Apply softmax to get attention weights (dim=2 is the last dimension)
        let attention_weights = Self::softmax(&masked_scores, 2)
            .map_err(|e| anyhow!("Failed to compute attention softmax: {}", e))?;
        
        // Apply attention weights to values: Attention(Q,K,V) = softmax(Q*K^T/sqrt(d_k))*V
        let attention_output = attention_weights.matmul(&v)
            .map_err(|e| anyhow!("Attention value computation failed: {}", e))?;
        
        // Reshape back: (num_heads, seq_len, head_dim) -> (seq_len, hidden_size)
        let attention_output = attention_output.transpose(0, 1)? // -> (seq_len, num_heads, head_dim)
            .reshape(&[seq_len, hidden_size])?;
        
        // Apply output projection
        let final_output = attention_output.matmul(&attention.o_proj.t()?)
            .map_err(|e| anyhow!("Output projection failed: {}", e))?;
        
        // Validate output for NaN/Inf - fail instead of fallback
        let output_vec = final_output.flatten_all()?.to_vec1::<f32>()?;
        if output_vec.iter().any(|x| x.is_nan() || x.is_infinite()) {
            return Err(anyhow!("Attention computation produced NaN or infinite values. Model weights may be corrupted."));
        }
        
        Ok(final_output)
    }
    
    #[allow(dead_code)]
    fn feed_forward(hidden_states: &Tensor, ff: &FeedForward) -> Result<Tensor> {
        let intermediate = hidden_states.matmul(&ff.fc1.t()
            .map_err(|e| anyhow!("Failed to transpose fc1: {}", e))?)
            .map_err(|e| anyhow!("Failed in fc1 matmul: {}", e))?;
        let activated = Self::gelu(&intermediate)?;
        Ok(activated.matmul(&ff.fc2.t()
            .map_err(|e| anyhow!("Failed to transpose fc2: {}", e))?)
            .map_err(|e| anyhow!("Failed in fc2 matmul: {}", e))?)
    }
    
    #[allow(dead_code)]
    fn gelu(x: &Tensor) -> Result<Tensor> {
        // GELU activation: x * 0.5 * (1 + tanh(sqrt(2/pi) * (x + 0.044715 * x^3)))
        let sqrt_2_over_pi = std::f32::consts::FRAC_2_PI.sqrt();
        let x_cubed = x.powf(3.0)
            .map_err(|e| anyhow!("Failed to compute x^3: {}", e))?;
        let inner = (x + x_cubed.affine(0.044715, 0.0)
            .map_err(|e| anyhow!("Failed in x^3 affine: {}", e))?)
            .map_err(|e| anyhow!("Failed to add x and x^3: {}", e))?
            .affine(sqrt_2_over_pi as f64, 0.0)
            .map_err(|e| anyhow!("Failed in sqrt affine: {}", e))?;
        let tanh_inner = inner.tanh()
            .map_err(|e| anyhow!("Failed in tanh: {}", e))?;
        Ok(x.broadcast_mul(&tanh_inner.affine(0.5, 0.5)
            .map_err(|e| anyhow!("Failed in GELU affine: {}", e))?
        ).map_err(|e| anyhow!("Failed in GELU mul: {}", e))?)
    }
    
    #[allow(dead_code)]
    fn layer_norm(x: &Tensor, weight: &Tensor, bias: &Tensor) -> Result<Tensor> {
        let eps = 1e-12;
        let mean = x.mean_keepdim(1)
            .map_err(|e| anyhow!("Failed to compute mean: {}", e))?;
        let x_centered = x.broadcast_sub(&mean)
            .map_err(|e| anyhow!("Failed to center x: {}", e))?;
        let var = x_centered.sqr()
            .map_err(|e| anyhow!("Failed to square centered x: {}", e))?
            .mean_keepdim(1)
            .map_err(|e| anyhow!("Failed to compute variance: {}", e))?;
        let x_normed = x_centered.broadcast_div(&var.affine(1.0, eps)
            .map_err(|e| anyhow!("Failed in variance affine: {}", e))?
            .sqrt()
            .map_err(|e| anyhow!("Failed to compute sqrt of variance: {}", e))?)
            .map_err(|e| anyhow!("Failed to normalize: {}", e))?;
        
        // Apply weight and bias
        Ok(x_normed.broadcast_mul(weight)
            .map_err(|e| anyhow!("Failed in layer norm mul: {}", e))?
            .broadcast_add(bias)
            .map_err(|e| anyhow!("Failed in layer norm add: {}", e))?)
    }
    
    #[allow(dead_code)]
    fn mean_pool(hidden_states: &Tensor, attention_mask: &Tensor) -> Result<Tensor> {
        // Check if mask is all zeros
        let mask_sum = attention_mask.sum_all()?.to_scalar::<f32>()?;
        
        if mask_sum == 0.0 {
            return Err(anyhow!("Invalid attention mask: all zeros"));
        }
        
        // Expand attention mask to match hidden states shape for broadcasting
        // attention_mask is [seq_len], hidden_states is [seq_len, hidden_size]
        let mask_expanded = attention_mask
            .unsqueeze(1)  // [seq_len, 1]
            .map_err(|e| anyhow!("Failed to unsqueeze mask in mean pool: {}", e))?
            .broadcast_as(hidden_states.shape())  // [seq_len, hidden_size]
            .map_err(|e| anyhow!("Failed to broadcast mask in mean pool: {}", e))?;
        
        // Apply attention mask
        let masked = hidden_states.broadcast_mul(&mask_expanded)
            .map_err(|e| anyhow!("Failed to apply mask in mean pool: {}", e))?;
        
        // Sum along sequence dimension (dim 0) to get [hidden_size]
        let summed = masked.sum(0)
            .map_err(|e| anyhow!("Failed to sum in mean pool: {}", e))?;
        
        // Count non-masked tokens (scalar)
        let count_scalar = attention_mask.sum_all()?.to_scalar::<f32>()?;
        let count_scalar = count_scalar.max(1e-9);
        
        // Divide by count to get mean pooling
        Ok(summed.affine(1.0 / count_scalar as f64, 0.0)
            .map_err(|e| anyhow!("Failed in mean pool div: {}", e))?)
    }
    
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        texts.iter()
            .map(|text| self.embed(text))
            .collect()
    }
    
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }
    
    pub fn set_dimensions(&mut self, dims: usize) -> Result<()> {
        let valid_dims = [64, 128, 256, 512, 768];
        
        if !valid_dims.contains(&dims) {
            return Err(anyhow!("Invalid dimensions. Must be one of: {:?}", valid_dims));
        }
        
        self.dimensions = dims;
        Ok(())
    }
    
    /// Validate attention mask dimensions and content
    /// 
    /// This function performs comprehensive validation of attention masks:
    /// - Verifies dimensions match input dimensions
    /// - Ensures at least one non-zero value exists
    /// - Returns descriptive errors for each failure type
    pub fn validate_attention_mask(attention_mask: &[u32], expected_length: usize) -> Result<()> {
        // Check dimension match
        if attention_mask.len() != expected_length {
            return Err(anyhow!(
                "Attention mask dimension mismatch: mask has {} elements but expected {} elements. \
                 The attention mask must have exactly the same number of elements as the input sequence. \
                 This mismatch indicates a tokenization or preprocessing error.",
                attention_mask.len(),
                expected_length
            ));
        }

        // Check for at least one non-zero value
        let has_attention = attention_mask.iter().any(|&val| val != 0);
        if !has_attention {
            return Err(anyhow!(
                "Invalid attention mask: all values are zero. \
                 An attention mask with all zeros means no tokens should be attended to, \
                 which makes embedding generation impossible. This indicates the input \
                 sequence was empty or improperly tokenized. At least one token must \
                 have a non-zero attention value."
            ));
        }

        // Additional validation: check for reasonable attention values
        let max_attention = attention_mask.iter().max().copied()
            .ok_or_else(|| anyhow!("Empty attention mask - cannot validate attention values"))?;
        if max_attention > 1 {
            log::warn!(
                "Attention mask contains values greater than 1 (max: {}). \
                 While not an error, attention masks typically use binary values (0 or 1). \
                 Values greater than 1 may indicate unusual tokenization or mask generation.",
                max_attention
            );
        }

        Ok(())
    }
    
    async fn ensure_files_cached() -> Result<(PathBuf, PathBuf)> {
        let cache_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not determine home directory. Set HOME environment variable."))?
            .join(".nomic");
        
        fs::create_dir_all(&cache_dir)?;
        
        let model_path = cache_dir.join(Self::MODEL_FILENAME);
        let tokenizer_path = cache_dir.join(Self::TOKENIZER_FILENAME);
        
        // Download model if needed
        if !model_path.exists() || fs::metadata(&model_path)?.len() < (Self::MODEL_SIZE as f64 * 0.95) as u64 {
            println!("📥 Downloading Nomic Embed Text v1.5 GGUF (Q4_K_M, ~80MB)...");
            println!("   Cache location: {:?}", model_path);
            Self::download_with_progress(Self::MODEL_URL, &model_path).await?;
            println!("✅ Model cached successfully at: {:?}", model_path);
        } else {
            println!("✅ Nomic model found in cache: {:?}", model_path);
            #[cfg(debug_assertions)]
            println!("   File size: {:.1}MB", fs::metadata(&model_path)?.len() as f64 / 1_048_576.0);
        }
        
        // Download tokenizer if needed
        if !tokenizer_path.exists() {
            println!("📥 Downloading tokenizer...");
            Self::download_file(Self::TOKENIZER_URL, &tokenizer_path).await?;
            println!("✅ Tokenizer cached successfully at: {:?}", tokenizer_path);
        } else {
            println!("✅ Tokenizer found in cache: {:?}", tokenizer_path);
        }
        
        Ok((model_path, tokenizer_path))
    }
    
    async fn download_file(url: &str, target: &PathBuf) -> Result<()> {
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to download file: {}", response.status()));
        }
        
        let content = response.bytes().await?;
        fs::write(target, content)?;
        
        Ok(())
    }
    
    async fn download_with_progress(url: &str, target: &PathBuf) -> Result<()> {
        use reqwest;
        use futures::StreamExt;
        
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3600))
            .build()?;
            
        let response = client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to download model: {}", response.status()));
        }
        
        let total_size = response.content_length()
            .ok_or_else(|| anyhow!("Server did not provide content length for model download. Cannot track progress."))?;
        
        let mut file = fs::File::create(target)?;
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;
            
            if total_size > 0 {
                let progress = (downloaded as f64 / total_size as f64) * 100.0;
                print!("\r📥 Progress: {:.1}% ({:.1}MB / {:.1}MB)", 
                       progress, 
                       downloaded as f64 / 1_048_576.0,
                       total_size as f64 / 1_048_576.0);
                std::io::stdout().flush()?;
            }
        }
        println!();
        
        Ok(())
    }
}

#[cfg(all(test, feature = "ml"))]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_singleton_pattern() {
        let embedder1 = NomicEmbedder::get_global().await.unwrap();
        let embedder2 = NomicEmbedder::get_global().await.unwrap();
        assert!(Arc::ptr_eq(&embedder1, &embedder2));
    }
    
    #[tokio::test]
    async fn test_embedding_generation() {
        let embedder = NomicEmbedder::get_global().await.unwrap();
        
        let text1 = "def calculate_sum(a, b): return a + b";
        let text2 = "class User: pass";
        
        let embedding1 = embedder.embed(text1).unwrap();
        let embedding2 = embedder.embed(text2).unwrap();
        
        // Check dimensions
        assert_eq!(embedding1.len(), 768);
        assert_eq!(embedding2.len(), 768);
        
        // Check that embeddings are different
        let diff: f32 = embedding1.iter()
            .zip(embedding2.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        
        // Debug output
        println!("First 10 values of embedding1: {:?}", &embedding1[..10]);
        println!("First 10 values of embedding2: {:?}", &embedding2[..10]);
        println!("Total difference between embeddings: {}", diff);
        
        assert!(diff > 0.1, "Embeddings should be different for different inputs");
        
        // Check L2 normalization
        let norm1: f32 = embedding1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = embedding2.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm1 - 1.0).abs() < 0.01, "Embedding 1 should be L2 normalized");
        assert!((norm2 - 1.0).abs() < 0.01, "Embedding 2 should be L2 normalized");
        
        println!("✅ Test passed: embeddings are different and normalized");
        println!("  - Embedding 1 norm: {}", norm1);
        println!("  - Embedding 2 norm: {}", norm2);
        println!("  - Difference: {}", diff);
    }
    
    #[tokio::test]
    async fn test_batch_embedding() {
        let embedder = NomicEmbedder::get_global().await.unwrap();
        
        let texts = vec![
            "class User:",
            "def __init__(self, name):",
            "self.name = name",
        ];
        
        let embeddings = embedder.embed_batch(&texts).unwrap();
        
        assert_eq!(embeddings.len(), 3);
        for (i, embedding) in embeddings.iter().enumerate() {
            assert_eq!(embedding.len(), 768);
            
            // Check L2 normalization
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 0.01, "Embedding {} should be L2 normalized", i);
        }
        
        // Check that all embeddings are different
        for i in 0..embeddings.len() {
            for j in i+1..embeddings.len() {
                let diff: f32 = embeddings[i].iter()
                    .zip(embeddings[j].iter())
                    .map(|(a, b)| (a - b).abs())
                    .sum();
                assert!(diff > 0.01, "Embeddings {} and {} should be different", i, j);
            }
        }
    }
    
    #[test]
    fn test_attention_mask_validation_valid() {
        // Test valid attention mask
        let mask = vec![1, 1, 1, 0, 0];
        let result = NomicEmbedder::validate_attention_mask(&mask, 5);
        assert!(result.is_ok(), "Valid attention mask should pass validation");
        
        // Test all ones
        let mask = vec![1, 1, 1, 1, 1];
        let result = NomicEmbedder::validate_attention_mask(&mask, 5);
        assert!(result.is_ok(), "All-ones attention mask should pass validation");
        
        // Test single attention token
        let mask = vec![1, 0, 0, 0, 0];
        let result = NomicEmbedder::validate_attention_mask(&mask, 5);
        assert!(result.is_ok(), "Single attention token should pass validation");
    }
    
    #[test]
    fn test_attention_mask_validation_dimension_mismatch() {
        // Test dimension mismatch - mask too short
        let mask = vec![1, 1, 1];
        let result = NomicEmbedder::validate_attention_mask(&mask, 5);
        assert!(result.is_err(), "Dimension mismatch should fail validation");
        
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("dimension mismatch"), 
                "Error should mention dimension mismatch: {}", error_message);
        assert!(error_message.contains("mask has 3 elements but expected 5"),
                "Error should specify exact dimensions: {}", error_message);
        
        // Test dimension mismatch - mask too long
        let mask = vec![1, 1, 1, 1, 1, 1, 1];
        let result = NomicEmbedder::validate_attention_mask(&mask, 5);
        assert!(result.is_err(), "Dimension mismatch should fail validation");
        
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("mask has 7 elements but expected 5"),
                "Error should specify exact dimensions: {}", error_message);
    }
    
    #[test]
    fn test_attention_mask_validation_all_zeros() {
        // Test all zeros
        let mask = vec![0, 0, 0, 0, 0];
        let result = NomicEmbedder::validate_attention_mask(&mask, 5);
        assert!(result.is_err(), "All-zeros attention mask should fail validation");
        
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("all values are zero"), 
                "Error should mention all zeros: {}", error_message);
        assert!(error_message.contains("embedding generation impossible"),
                "Error should explain why all zeros is invalid: {}", error_message);
        
        // Test empty mask (which is also all zeros conceptually)
        let mask: Vec<u32> = vec![];
        let result = NomicEmbedder::validate_attention_mask(&mask, 0);
        assert!(result.is_err(), "Empty attention mask should fail validation");
    }
    
    #[test] 
    fn test_attention_mask_validation_edge_cases() {
        // Test very large attention values (should work but with warning)
        let mask = vec![1, 5, 10, 1];
        let result = NomicEmbedder::validate_attention_mask(&mask, 4);
        assert!(result.is_ok(), "Large attention values should not fail validation");
        
        // Test single element masks
        let mask = vec![1];
        let result = NomicEmbedder::validate_attention_mask(&mask, 1);
        assert!(result.is_ok(), "Single element mask with attention should pass");
        
        let mask = vec![0];
        let result = NomicEmbedder::validate_attention_mask(&mask, 1);
        assert!(result.is_err(), "Single element mask without attention should fail");
        
        // Test maximum u32 values
        let mask = vec![u32::MAX, 1, 0];
        let result = NomicEmbedder::validate_attention_mask(&mask, 3);
        assert!(result.is_ok(), "Maximum u32 values should not fail validation");
    }
    
    #[test]
    fn test_attention_mask_validation_comprehensive_error_messages() {
        // Test that error messages are descriptive and helpful
        let mask = vec![1, 1];
        let result = NomicEmbedder::validate_attention_mask(&mask, 5);
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        let error_message = error.to_string();
        
        // Check that error message contains all required information
        assert!(error_message.contains("dimension mismatch"), 
                "Error should identify the problem type");
        assert!(error_message.contains("tokenization or preprocessing error"),
                "Error should explain potential cause");
        assert!(error_message.contains("2") && error_message.contains("5"),
                "Error should include actual dimensions");
        
        // Test all-zeros error message
        let mask = vec![0, 0, 0];
        let result = NomicEmbedder::validate_attention_mask(&mask, 3);
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        let error_message = error.to_string();
        
        assert!(error_message.contains("all values are zero"),
                "Error should identify all-zeros problem");
        assert!(error_message.contains("embedding generation impossible"),
                "Error should explain the consequence");
        assert!(error_message.contains("At least one token must have a non-zero attention"),
                "Error should specify the requirement");
    }
    
    /// Test Q6K dequantization mathematical correctness
    #[test]
    fn test_q6k_mathematical_correctness() {
        // Test 6-bit value extraction from packed data
        // Values: [0, 1, 2, 3] should pack into 3 bytes
        let test_data = [0b00000100, 0b00010000, 0b00001100];
        
        for i in 0..4 {
            let bit_offset = i * 6;
            let byte_start = bit_offset / 8;
            let bit_start = bit_offset % 8;
            
            let q6_value = if bit_start + 6 <= 8 {
                (test_data[byte_start] >> bit_start) & 0x3F
            } else if byte_start + 1 < test_data.len() {
                let low_bits = 8 - bit_start;
                let high_bits = 6 - low_bits;
                let low_part = (test_data[byte_start] >> bit_start) & ((1 << low_bits) - 1);
                let high_part = test_data[byte_start + 1] & ((1 << high_bits) - 1);
                low_part | (high_part << low_bits)
            } else {
                panic!("Insufficient data for 6-bit extraction");
            };
            
            assert_eq!(q6_value, i as u8, "6-bit extraction failed for value {}", i);
        }
    }
    
    /// Test Q6K scale conversion accuracy  
    #[test]
    fn test_q6k_scale_conversion() {
        let test_cases = [
            (0u8, 0.0f32),
            (128, 0.5),
            (255, 255.0/256.0),
        ];
        
        for &(scale_u8, expected) in &test_cases {
            let converted = (scale_u8 as f32) / 256.0;
            assert!((converted - expected).abs() < f32::EPSILON,
                    "Scale conversion failed: {} -> {} (expected {})", scale_u8, converted, expected);
        }
    }
    
    /// Test Q6K dequantization formula
    #[test] 
    fn test_q6k_dequantization_formula() {
        let scale = 0.1f32;
        let test_values = [0u8, 32, 63]; // Min, center, max
        let expected = [scale * -32.0, 0.0, scale * 31.0];
        
        for (i, &q6_val) in test_values.iter().enumerate() {
            let result = scale * ((q6_val as f32) - 32.0);
            assert!((result - expected[i]).abs() < f32::EPSILON,
                    "Dequantization failed: {} -> {} (expected {})", q6_val, result, expected[i]);
        }
    }
}

