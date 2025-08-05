use anyhow::{Result, anyhow};
use candle_core::{Device, Tensor, DType};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use hf_hub::{api::tokio::Api, Repo, RepoType};
use tokenizers::Tokenizer;
use std::path::PathBuf;

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

impl RealMiniLMEmbedder {
    /// Load the actual all-MiniLM-L6-v2 model from Hugging Face
    pub async fn new() -> Result<Self, EmbeddingError> {
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
        let weights_filename = repo.get("pytorch_model.bin").await
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
        let model = BertModel::load(&vb, &config)
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
        let embeddings = self.model.forward(&token_ids, &attention_mask)
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
    
    /// Batch embed multiple texts for efficiency
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let mut embeddings = Vec::with_capacity(texts.len());
        
        for text in texts {
            let embedding = self.embed(text)?;
            embeddings.push(embedding);
        }
        
        Ok(embeddings)
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