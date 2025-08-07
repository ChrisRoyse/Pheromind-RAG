# Task 012 - HIGH: Test GGUF Parsing Pipeline

## Priority: HIGH
## Estimated Time: 10 minutes
## Dependencies: Task 011

## Objective
Create comprehensive tests for the GGUF parsing pipeline to verify all components work together.

## Current Issue
- Need integration test for complete parsing
- Verify each parsing stage works
- Identify any remaining issues

## Tasks
1. **Create integration test** (6 min)
   ```rust
   // In src/ml/tests/gguf_parsing.rs
   use super::*;
   use std::fs::File;
   use std::io::BufReader;
   use anyhow::Result;
   
   #[test]
   fn test_complete_gguf_parsing() -> Result<()> {
       let model_path = "models/nomic-embed-text-v1.5-Q4_K_M.gguf";
       let file = File::open(model_path)?;
       let mut reader = BufReader::new(file);
       
       // Test header parsing
       let header = parse_header(&mut reader)?;
       println!("Header: {:?}", header);
       assert_eq!(header.version, 3);
       assert!(header.tensor_count > 0);
       
       // Test compatibility validation
       validate_compatibility(&header)?;
       
       // Test metadata parsing
       let config = parse_metadata(&mut reader, &header)?;
       println!("Config: {:?}", config);
       config.validate()?;
       
       // Test tensor info parsing
       let tensors = parse_tensors(&mut reader, &header)?;
       println!("Found {} tensors", tensors.len());
       assert_eq!(tensors.len(), header.tensor_count as usize);
       
       // Verify key tensors exist
       let tensor_names: Vec<&String> = tensors.iter().map(|t| &t.name).collect();
       assert!(tensor_names.iter().any(|name| name == &"token_embd.weight"));
       assert!(tensor_names.iter().any(|name| name.contains("blk.0.attn_norm.weight")));
       assert!(tensor_names.iter().any(|name| name == &"output_norm.weight"));
       
       Ok(())
   }
   ```

2. **Test tensor loading** (3 min)
   ```rust
   #[test]
   fn test_tensor_loading() -> Result<()> {
       let model_path = "models/nomic-embed-text-v1.5-Q4_K_M.gguf";
       let file = File::open(model_path)?;
       let mut reader = BufReader::new(file);
       
       let header = parse_header(&mut reader)?;
       let _config = parse_metadata(&mut reader, &header)?;
       let tensors = parse_tensors(&mut reader, &header)?;
       
       // Find and load token embedding tensor
       let token_embd = tensors.iter()
           .find(|t| t.name == "token_embd.weight")
           .ok_or_else(|| anyhow!("Token embedding tensor not found"))?;
       
       println!("Token embedding shape: {:?}", token_embd.dimensions);
       assert_eq!(token_embd.dimensions.len(), 2); // [vocab_size, embedding_dim]
       assert_eq!(token_embd.dimensions[1], 768); // embedding_dim
       
       // Test loading a small tensor (just verify it doesn't crash)
       let device = candle_core::Device::Cpu;
       file.seek(std::io::SeekFrom::Start(0))?;
       let tensor = load_tensor(&mut reader, token_embd, &device)?;
       println!("Loaded tensor shape: {:?}", tensor.shape());
       
       Ok(())
   }
   ```

3. **Add performance benchmark** (1 min)
   ```rust
   #[test]
   fn test_parsing_performance() {
       let start = std::time::Instant::now();
       let result = test_complete_gguf_parsing();
       let duration = start.elapsed();
       
       println!("GGUF parsing took: {:?}", duration);
       assert!(result.is_ok());
       assert!(duration.as_secs() < 10); // Should complete in under 10 seconds
   }
   ```

## Success Criteria
- [ ] Integration test passes completely
- [ ] All parsing stages work sequentially
- [ ] Key tensors are identified correctly
- [ ] Configuration is extracted properly
- [ ] Performance is acceptable (<10s)

## Files to Create
- `src/ml/tests/gguf_parsing.rs`

## Expected Test Results
- Header version: 3
- Tensor count: ~150-200
- Config embedding_length: 768
- Token embedding shape: [30522, 768]
- All key transformer tensors present

## Next Task
→ Task 013: Handle quantization formats properly