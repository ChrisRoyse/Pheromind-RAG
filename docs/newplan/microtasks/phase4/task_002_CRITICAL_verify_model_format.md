# Task 002 - CRITICAL: Verify Model File Format and Metadata

## Priority: CRITICAL
## Estimated Time: 10 minutes
## Dependencies: Task 001

## Objective
Verify the downloaded GGUF model file format and extract metadata to ensure compatibility.

## Current Issue
- Need to validate GGUF format
- Extract model parameters and architecture
- Ensure compatibility with Candle library

## Tasks
1. **Inspect GGUF header** (4 min)
   ```rust
   // Create simple GGUF inspector
   use std::fs::File;
   use std::io::{BufReader, Read};
   
   fn inspect_gguf(path: &str) -> Result<()> {
       let file = File::open(path)?;
       let mut reader = BufReader::new(file);
       
       // Read magic (4 bytes)
       let mut magic = [0u8; 4];
       reader.read_exact(&mut magic)?;
       println!("Magic: {:?}", std::str::from_utf8(&magic));
       
       // Read version (4 bytes)
       let mut version = [0u8; 4];
       reader.read_exact(&mut version)?;
       let version_num = u32::from_le_bytes(version);
       println!("Version: {}", version_num);
       
       Ok(())
   }
   ```

2. **Extract model metadata** (4 min)
   - Model name and architecture
   - Embedding dimensions
   - Vocabulary size
   - Quantization type

3. **Document findings** (2 min)
   - Create model info file
   - Update configuration constants

## Success Criteria
- [ ] GGUF magic number verified ("GGUF")
- [ ] Version extracted (should be 3)
- [ ] Embedding dimension confirmed (768)
- [ ] Architecture validated (nomic-bert)
- [ ] Quantization type identified (Q4_K_M)

## Files to Create
- `src/ml/model_inspector.rs` (temporary tool)
- `models/model_info.json` (metadata)

## Validation
```bash
# Run inspector
cargo run --bin model_inspector models/nomic-embed-text-v1.5-Q4_K_M.gguf

# Expected output:
# Magic: "GGUF"
# Version: 3
# Embedding dim: 768
# Architecture: nomic-bert
```

## Next Task
→ Task 003: Update model paths in configuration