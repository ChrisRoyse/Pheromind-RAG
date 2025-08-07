# Task 004 - HIGH: Check Model Format Compatibility with Candle

## Priority: HIGH
## Estimated Time: 10 minutes
## Dependencies: Task 003

## Objective
Verify that the GGUF model format is compatible with Candle library and identify any integration issues.

## Current Issue
- Candle GGUF support may be limited
- Need to verify tensor format compatibility
- Check quantization support

## Tasks
1. **Test basic GGUF loading** (5 min)
   ```rust
   // Create test in src/ml/tests/
   use candle_core::{Device, Result};
   use candle_transformers::models::bert::BertModel;
   
   #[test]
   fn test_gguf_basic_loading() -> Result<()> {
       let device = Device::Cpu;
       let model_path = "models/nomic-embed-text-v1.5-Q4_K_M.gguf";
       
       // Try to load with candle-gguf
       let file = std::fs::File::open(model_path)?;
       let model = candle_gguf::gguf_file::Content::read(file)?;
       
       println!("Tensors count: {}", model.tensor_infos.len());
       Ok(())
   }
   ```

2. **Check tensor shapes** (3 min)
   - Verify embedding layer dimensions
   - Check attention layer compatibility
   - Validate tensor naming conventions

3. **Test quantization support** (2 min)
   - Verify Q4_K_M quantization is supported
   - Check performance implications

## Success Criteria
- [ ] GGUF file loads without errors
- [ ] Tensor count matches expected (>100)
- [ ] Embedding tensors found and correctly shaped
- [ ] Quantization format supported
- [ ] No critical compatibility issues

## Files to Create
- `src/ml/tests/gguf_compatibility.rs`

## Expected Issues
- May need to implement custom GGUF loader
- Quantization might require specific handling
- Tensor format conversion needed

## Validation
```bash
cargo test ml::tests::gguf_compatibility

# Expected: Either success or clear error messages
# about what needs to be implemented
```

## Next Task
→ Task 005: Test basic model loading pipeline