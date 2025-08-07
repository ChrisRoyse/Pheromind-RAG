# Task 029 - CRITICAL: Implement Tokenization for Text Input

## Priority: CRITICAL
## Estimated Time: 10 minutes
## Dependencies: Task 028

## Objective
Implement text tokenization compatible with the Nomic model's vocabulary and tokenizer.

## Current Issue
- No tokenization implementation
- Need BERT-style tokenization for Nomic model
- Must handle special tokens and padding

## Tasks
1. **Add tokenizer dependencies** (2 min)
   ```toml
   # Add to Cargo.toml
   [dependencies]
   tokenizers = "0.15"
   hf-hub = "0.3"
   reqwest = { version = "0.11", features = ["json"] }
   ```

2. **Implement tokenizer wrapper** (6 min)
   ```rust
   // In src/ml/tokenizer.rs
   use tokenizers::Tokenizer;
   use anyhow::{Result, anyhow};
   use std::path::Path;
   
   pub struct EmbeddingTokenizer {
       tokenizer: Tokenizer,
       max_length: usize,
       pad_token_id: u32,
       cls_token_id: u32,
       sep_token_id: u32,
   }
   
   impl EmbeddingTokenizer {
       pub async fn new() -> Result<Self> {
           // Try to load tokenizer from local file first
           let tokenizer_path = "models/tokenizer.json";
           
           let tokenizer = if Path::new(tokenizer_path).exists() {
               Self::load_local_tokenizer(tokenizer_path)?
           } else {
               Self::download_tokenizer().await?
           };
           
           // Get special token IDs
           let vocab = tokenizer.get_vocab(false);
           
           let pad_token_id = vocab.get("[PAD]").copied().unwrap_or(0);
           let cls_token_id = vocab.get("[CLS]").copied().unwrap_or(101);
           let sep_token_id = vocab.get("[SEP]").copied().unwrap_or(102);
           
           Ok(Self {
               tokenizer,
               max_length: 8192, // Nomic model max length
               pad_token_id,
               cls_token_id,
               sep_token_id,
           })
       }
       
       fn load_local_tokenizer(path: &str) -> Result<Tokenizer> {
           Tokenizer::from_file(path)
               .map_err(|e| anyhow!("Failed to load tokenizer from {}: {}", path, e))
       }
       
       async fn download_tokenizer() -> Result<Tokenizer> {
           // Download tokenizer from Hugging Face
           let api = hf_hub::api::tokio::Api::new()?;
           let repo = api.model("nomic-ai/nomic-embed-text-v1.5".to_string());
           
           // Download tokenizer.json
           let tokenizer_path = repo.get("tokenizer.json").await
               .map_err(|e| anyhow!("Failed to download tokenizer: {}", e))?;
           
           // Load the downloaded tokenizer
           Tokenizer::from_file(tokenizer_path)
               .map_err(|e| anyhow!("Failed to load downloaded tokenizer: {}", e))
       }
       
       pub fn tokenize(&self, text: &str) -> Result<TokenizedInput> {
           // Encode the text
           let encoding = self.tokenizer
               .encode(text, false)
               .map_err(|e| anyhow!("Tokenization failed: {}", e))?;
           
           let mut token_ids = encoding.get_ids().to_vec();
           let mut attention_mask = encoding.get_attention_mask().to_vec();
           
           // Add CLS token at the beginning
           token_ids.insert(0, self.cls_token_id);
           attention_mask.insert(0, 1);
           
           // Truncate if too long (leave space for SEP token)
           if token_ids.len() > self.max_length - 1 {
               token_ids.truncate(self.max_length - 1);
               attention_mask.truncate(self.max_length - 1);
           }
           
           // Add SEP token at the end
           token_ids.push(self.sep_token_id);
           attention_mask.push(1);
           
           // Pad to max length if needed
           while token_ids.len() < self.max_length {
               token_ids.push(self.pad_token_id);
               attention_mask.push(0);
           }
           
           Ok(TokenizedInput {
               input_ids: token_ids,
               attention_mask,
               token_count: attention_mask.iter().sum::<u32>() as usize,
           })
       }
       
       pub fn tokenize_batch(&self, texts: &[&str]) -> Result<Vec<TokenizedInput>> {
           texts.iter()
               .map(|text| self.tokenize(text))
               .collect()
       }
       
       pub fn vocab_size(&self) -> usize {
           self.tokenizer.get_vocab_size(false)
       }
       
       pub fn max_length(&self) -> usize {
           self.max_length
       }
   }
   
   #[derive(Debug, Clone)]
   pub struct TokenizedInput {
       pub input_ids: Vec<u32>,
       pub attention_mask: Vec<u32>,
       pub token_count: usize,
   }
   
   impl TokenizedInput {
       pub fn to_tensor_data(&self) -> (Vec<i64>, Vec<i64>) {
           let input_ids: Vec<i64> = self.input_ids.iter().map(|&x| x as i64).collect();
           let attention_mask: Vec<i64> = self.attention_mask.iter().map(|&x| x as i64).collect();
           (input_ids, attention_mask)
       }
   }
   ```

3. **Add text preprocessing** (2 min)
   ```rust
   impl EmbeddingTokenizer {
       pub fn preprocess_text(&self, text: &str) -> String {
           // Basic text preprocessing
           text.trim()
               .replace("\n", " ")
               .replace("\t", " ")
               .replace("\r", " ")
               // Normalize multiple spaces
               .split_whitespace()
               .collect::<Vec<_>>()
               .join(" ")
       }
       
       pub fn tokenize_with_preprocessing(&self, text: &str) -> Result<TokenizedInput> {
           let processed_text = self.preprocess_text(text);
           self.tokenize(&processed_text)
       }
       
       pub fn estimate_token_count(&self, text: &str) -> usize {
           // Quick estimation without full tokenization
           // Rough estimate: ~1.3 tokens per word for English
           let word_count = text.split_whitespace().count();
           ((word_count as f64) * 1.3) as usize
       }
       
       pub fn chunk_text(
           &self,
           text: &str,
           chunk_size: usize,
           overlap: usize,
       ) -> Result<Vec<String>> {
           let words: Vec<&str> = text.split_whitespace().collect();
           if words.is_empty() {
               return Ok(vec![]);
           }
           
           let mut chunks = Vec::new();
           let mut start = 0;
           
           while start < words.len() {
               let end = (start + chunk_size).min(words.len());
               let chunk = words[start..end].join(" ");
               
               // Verify chunk fits in token limit
               let estimated_tokens = self.estimate_token_count(&chunk);
               if estimated_tokens > self.max_length - 10 { // Leave room for special tokens
                   // If single chunk is too large, split it further
                   let smaller_chunks = self.split_large_chunk(&chunk)?;
                   chunks.extend(smaller_chunks);
               } else {
                   chunks.push(chunk);
               }
               
               if end == words.len() {
                   break;
               }
               
               start = end - overlap.min(end);
           }
           
           Ok(chunks)
       }
       
       fn split_large_chunk(&self, text: &str) -> Result<Vec<String>> {
           // Split by sentences if possible
           let sentences: Vec<&str> = text.split(&['.', '!', '?'][..]).collect();
           let mut result = Vec::new();
           let mut current_chunk = String::new();
           
           for sentence in sentences {
               let sentence = sentence.trim();
               if sentence.is_empty() {
                   continue;
               }
               
               let test_chunk = if current_chunk.is_empty() {
                   sentence.to_string()
               } else {
                   format!("{} {}", current_chunk, sentence)
               };
               
               if self.estimate_token_count(&test_chunk) > self.max_length - 10 {
                   if !current_chunk.is_empty() {
                       result.push(current_chunk);
                       current_chunk = sentence.to_string();
                   } else {
                       // Single sentence is too long, split by words
                       let words: Vec<&str> = sentence.split_whitespace().collect();
                       let chunks_per_sentence = words.len() / (self.max_length / 2);
                       
                       for chunk in words.chunks(self.max_length / 2) {
                           result.push(chunk.join(" "));
                       }
                   }
               } else {
                   current_chunk = test_chunk;
               }
           }
           
           if !current_chunk.is_empty() {
               result.push(current_chunk);
           }
           
           Ok(result)
       }
   }
   ```

## Success Criteria
- [ ] Tokenizer loads successfully
- [ ] Text tokenization works correctly
- [ ] Special tokens handled properly
- [ ] Padding and truncation work
- [ ] Batch processing implemented
- [ ] Text chunking functions correctly

## Files to Create
- `src/ml/tokenizer.rs`

## Files to Modify
- `src/ml/mod.rs`
- `Cargo.toml`

## Validation
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tokenization() {
        let tokenizer = EmbeddingTokenizer::new().await.unwrap();
        
        let text = "Hello, this is a test sentence for tokenization.";
        let result = tokenizer.tokenize(text).unwrap();
        
        assert!(result.input_ids.len() <= tokenizer.max_length());
        assert_eq!(result.input_ids.len(), result.attention_mask.len());
        assert!(result.token_count > 0);
        
        // Should start with CLS token
        assert_eq!(result.input_ids[0], tokenizer.cls_token_id);
    }
    
    #[tokio::test]
    async fn test_text_chunking() {
        let tokenizer = EmbeddingTokenizer::new().await.unwrap();
        
        let long_text = "word ".repeat(10000);
        let chunks = tokenizer.chunk_text(&long_text, 1000, 100).unwrap();
        
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            let tokens = tokenizer.tokenize(chunk).unwrap();
            assert!(tokens.token_count <= tokenizer.max_length());
        }
    }
}
```

## Next Task
→ Task 030: Convert tokens to tensor format for model input