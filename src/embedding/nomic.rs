use once_cell::sync::OnceCell;
use std::sync::Arc;
use std::path::PathBuf;
use anyhow::{Result, anyhow};
use std::fs;
use std::io::Write;

static GLOBAL_EMBEDDER: OnceCell<Arc<NomicEmbedder>> = OnceCell::new();

pub struct NomicEmbedder {
    model_path: PathBuf,
    dimensions: usize,
    cache: Option<Arc<super::cache::EmbeddingCache>>,
}

impl NomicEmbedder {
    const MODEL_URL: &'static str = "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF/resolve/main/nomic-embed-text-v1.5.Q4_K_M.gguf";
    const MODEL_SIZE: u64 = 4_500_000_000;  // ~4.5GB
    const MODEL_FILENAME: &'static str = "nomic-embed-text-v1.5.Q4_K_M.gguf";
    
    pub async fn get_global() -> Result<Arc<Self>> {
        if let Some(embedder) = GLOBAL_EMBEDDER.get() {
            return Ok(embedder.clone());
        }
        
        let embedder = Arc::new(Self::new().await?);
        match GLOBAL_EMBEDDER.set(embedder.clone()) {
            Ok(_) => Ok(embedder),
            Err(_) => Ok(GLOBAL_EMBEDDER.get().unwrap().clone()),
        }
    }
    
    pub async fn new() -> Result<Self> {
        let model_path = Self::ensure_model_cached().await?;
        
        // Initialize cache
        let cache = Some(Arc::new(super::cache::EmbeddingCache::new(100_000)));
        
        Ok(Self {
            model_path,
            dimensions: 768,
            cache,
        })
    }
    
    async fn ensure_model_cached() -> Result<PathBuf> {
        let cache_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".nomic");
        
        fs::create_dir_all(&cache_dir)?;
        
        let model_path = cache_dir.join(Self::MODEL_FILENAME);
        
        // Check if model exists and has correct size
        if model_path.exists() {
            let metadata = fs::metadata(&model_path)?;
            if metadata.len() >= (Self::MODEL_SIZE as f64 * 0.95) as u64 {
                println!("✅ Nomic model found in cache: {:?}", model_path);
                return Ok(model_path);
            }
            println!("⚠️  Incomplete model file, re-downloading...");
            fs::remove_file(&model_path)?;
        }
        
        // Download model
        println!("📥 Downloading Nomic Embed Text v1.5 GGUF (Q4, ~4.5GB)...");
        Self::download_with_progress(Self::MODEL_URL, &model_path).await?;
        println!("✅ Model permanently cached at: {:?}", model_path);
        
        Ok(model_path)
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
        
        let total_size = response.content_length().unwrap_or(0);
        
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
    
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        if let Some(cache) = &self.cache {
            if let Some(embedding) = cache.get(text) {
                return Ok(embedding);
            }
        }
        
        // For now, return a placeholder embedding
        // TODO: Implement actual GGUF inference when we add llama-cpp-rs
        let embedding = vec![0.1; self.dimensions];
        
        // Cache the result
        if let Some(cache) = &self.cache {
            cache.put(text, embedding.clone());
        }
        
        Ok(embedding)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_singleton_pattern() {
        let embedder1 = NomicEmbedder::get_global().await.unwrap();
        let embedder2 = NomicEmbedder::get_global().await.unwrap();
        assert!(Arc::ptr_eq(&embedder1, &embedder2));
    }
    
    #[test]
    fn test_dimensions() {
        let mut embedder = NomicEmbedder {
            model_path: PathBuf::new(),
            dimensions: 768,
            cache: None,
        };
        
        assert!(embedder.set_dimensions(512).is_ok());
        assert_eq!(embedder.dimensions(), 512);
        
        assert!(embedder.set_dimensions(1024).is_err());
    }
}