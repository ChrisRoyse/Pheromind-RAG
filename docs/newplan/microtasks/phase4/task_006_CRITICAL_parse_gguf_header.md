# Task 006 - CRITICAL: Parse GGUF Header Structure

## Priority: CRITICAL
## Estimated Time: 10 minutes
## Dependencies: Task 005

## Objective
Implement GGUF header parsing to extract model metadata and prepare for tensor loading.

## Current Issue
- Missing GGUF header parsing implementation
- Need to extract metadata for model construction
- Foundation for all other GGUF operations

## Tasks
1. **Implement GGUF header parser** (7 min)
   ```rust
   // In src/ml/gguf_loader.rs
   use std::io::{BufReader, Read, Seek};
   use anyhow::{Result, anyhow};
   
   #[derive(Debug)]
   pub struct GGUFHeader {
       pub version: u32,
       pub tensor_count: u64,
       pub metadata_kv_count: u64,
   }
   
   pub fn parse_header<R: Read>(reader: &mut R) -> Result<GGUFHeader> {
       // Read magic (4 bytes)
       let mut magic = [0u8; 4];
       reader.read_exact(&mut magic)?;
       if &magic != b"GGUF" {
           return Err(anyhow!("Invalid GGUF magic number"));
       }
       
       // Read version (4 bytes)
       let mut version_bytes = [0u8; 4];
       reader.read_exact(&mut version_bytes)?;
       let version = u32::from_le_bytes(version_bytes);
       
       // Read tensor count (8 bytes)
       let mut tensor_count_bytes = [0u8; 8];
       reader.read_exact(&mut tensor_count_bytes)?;
       let tensor_count = u64::from_le_bytes(tensor_count_bytes);
       
       // Read metadata KV count (8 bytes)
       let mut kv_count_bytes = [0u8; 8];
       reader.read_exact(&mut kv_count_bytes)?;
       let metadata_kv_count = u64::from_le_bytes(kv_count_bytes);
       
       Ok(GGUFHeader {
           version,
           tensor_count,
           metadata_kv_count,
       })
   }
   ```

2. **Add validation** (2 min)
   - Version compatibility check
   - Reasonable tensor count limits
   - Metadata sanity checks

3. **Create test** (1 min)
   - Test with actual model file
   - Verify parsed values

## Success Criteria
- [ ] Header parsing compiles
- [ ] Magic number validated correctly
- [ ] Version 3 detected
- [ ] Tensor count >100
- [ ] Metadata KV count >10

## Files to Create
- `src/ml/gguf_loader.rs`

## Validation
```rust
#[test]
fn test_header_parsing() {
    let file = File::open("models/nomic-embed-text-v1.5-Q4_K_M.gguf").unwrap();
    let mut reader = BufReader::new(file);
    let header = parse_header(&mut reader).unwrap();
    
    assert_eq!(header.version, 3);
    assert!(header.tensor_count > 100);
    assert!(header.metadata_kv_count > 10);
}
```

## Next Task
→ Task 007: Parse GGUF version and validate compatibility