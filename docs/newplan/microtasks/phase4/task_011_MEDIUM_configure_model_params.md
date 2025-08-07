# Task 011 - MEDIUM: Configure Model Parameters from Metadata

## Priority: MEDIUM
## Estimated Time: 10 minutes
## Dependencies: Task 010

## Objective
Extract and configure model parameters from GGUF metadata to properly initialize the transformer model.

## Current Issue
- Model parameters hardcoded
- Need to read from GGUF metadata
- Configuration must match model architecture

## Tasks
1. **Parse GGUF metadata** (5 min)
   ```rust
   // In src/ml/gguf_loader.rs
   #[derive(Debug, Clone)]
   pub struct ModelConfig {
       pub embedding_length: u32,
       pub context_length: u32,
       pub vocab_size: u32,
       pub layer_count: u32,
       pub attention_head_count: u32,
       pub attention_head_count_kv: u32,
       pub feed_forward_length: u32,
       pub rope_dimension_count: u32,
       pub rope_freq_base: f32,
   }
   
   pub fn parse_metadata<R: Read>(
       reader: &mut R, 
       header: &GGUFHeader
   ) -> Result<ModelConfig> {
       let mut config = ModelConfig::default();
       
       for _ in 0..header.metadata_kv_count {
           // Read key length and key
           let key = read_string(reader)?;
           
           // Read value type and value
           let mut value_type_bytes = [0u8; 4];
           reader.read_exact(&mut value_type_bytes)?;
           let value_type = u32::from_le_bytes(value_type_bytes);
           
           match (key.as_str(), value_type) {
               ("nomic.embedding_length", 4) => { // u32
                   let mut value_bytes = [0u8; 4];
                   reader.read_exact(&mut value_bytes)?;
                   config.embedding_length = u32::from_le_bytes(value_bytes);
               },
               ("nomic.context_length", 4) => {
                   let mut value_bytes = [0u8; 4];
                   reader.read_exact(&mut value_bytes)?;
                   config.context_length = u32::from_le_bytes(value_bytes);
               },
               ("nomic.vocab_size", 4) => {
                   let mut value_bytes = [0u8; 4];
                   reader.read_exact(&mut value_bytes)?;
                   config.vocab_size = u32::from_le_bytes(value_bytes);
               },
               ("nomic.block_count", 4) => {
                   let mut value_bytes = [0u8; 4];
                   reader.read_exact(&mut value_bytes)?;
                   config.layer_count = u32::from_le_bytes(value_bytes);
               },
               ("nomic.attention.head_count", 4) => {
                   let mut value_bytes = [0u8; 4];
                   reader.read_exact(&mut value_bytes)?;
                   config.attention_head_count = u32::from_le_bytes(value_bytes);
               },
               _ => {
                   // Skip unknown metadata
                   skip_value(reader, value_type)?;
               }
           }
       }
       
       Ok(config)
   }
   ```

2. **Add parameter validation** (3 min)
   ```rust
   impl ModelConfig {
       pub fn validate(&self) -> Result<()> {
           if self.embedding_length == 0 {
               return Err(anyhow!("Invalid embedding length: 0"));
           }
           if self.layer_count == 0 {
               return Err(anyhow!("Invalid layer count: 0"));
           }
           if self.attention_head_count == 0 {
               return Err(anyhow!("Invalid attention head count: 0"));
           }
           if self.vocab_size == 0 {
               return Err(anyhow!("Invalid vocab size: 0"));
           }
           Ok(())
       }
       
       pub fn head_dim(&self) -> u32 {
           self.embedding_length / self.attention_head_count
       }
   }
   ```

3. **Add configuration defaults** (2 min)
   - Fallback values for missing metadata
   - Model-specific overrides

## Success Criteria
- [ ] Metadata parsing compiles
- [ ] All required parameters extracted
- [ ] Configuration validation passes
- [ ] Sensible defaults provided
- [ ] Model dimensions match tensors

## Files to Modify
- `src/ml/gguf_loader.rs`

## Expected Values
- `embedding_length`: 768
- `context_length`: 8192
- `vocab_size`: 30522
- `layer_count`: 12
- `attention_head_count`: 12

## Next Task
→ Task 012: Test GGUF parsing pipeline