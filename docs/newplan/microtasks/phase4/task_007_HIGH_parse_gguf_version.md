# Task 007 - HIGH: Parse GGUF Version and Validate Compatibility

## Priority: HIGH
## Estimated Time: 10 minutes
## Dependencies: Task 006

## Objective
Extend GGUF parsing to handle version-specific features and ensure compatibility.

## Current Issue
- Different GGUF versions have different formats
- Need version-specific parsing logic
- Compatibility validation required

## Tasks
1. **Implement version handling** (6 min)
   ```rust
   // Extend src/ml/gguf_loader.rs
   #[derive(Debug, Clone, Copy)]
   pub enum GGUFVersion {
       V1 = 1,
       V2 = 2,
       V3 = 3,
   }
   
   impl GGUFVersion {
       pub fn from_u32(value: u32) -> Result<Self> {
           match value {
               1 => Ok(GGUFVersion::V1),
               2 => Ok(GGUFVersion::V2),
               3 => Ok(GGUFVersion::V3),
               _ => Err(anyhow!("Unsupported GGUF version: {}", value)),
           }
       }
       
       pub fn is_supported(&self) -> bool {
           matches!(self, GGUFVersion::V3)
       }
   }
   ```

2. **Add compatibility checks** (3 min)
   ```rust
   pub fn validate_compatibility(header: &GGUFHeader) -> Result<()> {
       let version = GGUFVersion::from_u32(header.version)?;
       
       if !version.is_supported() {
           return Err(anyhow!(
               "GGUF version {} not supported, need v3", 
               header.version
           ));
       }
       
       if header.tensor_count == 0 {
           return Err(anyhow!("No tensors found in model"));
       }
       
       if header.tensor_count > 10000 {
           return Err(anyhow!("Too many tensors: {}", header.tensor_count));
       }
       
       Ok(())
   }
   ```

3. **Update header parsing** (1 min)
   - Integrate version validation
   - Return structured version info

## Success Criteria
- [ ] Version parsing works correctly
- [ ] V3 compatibility confirmed
- [ ] Unsupported versions rejected cleanly
- [ ] Tensor count validation works
- [ ] Clear error messages for issues

## Files to Modify
- `src/ml/gguf_loader.rs`

## Validation
```rust
#[test]
fn test_version_validation() {
    // Test supported version
    let header = GGUFHeader { version: 3, tensor_count: 150, metadata_kv_count: 20 };
    assert!(validate_compatibility(&header).is_ok());
    
    // Test unsupported version
    let header = GGUFHeader { version: 1, tensor_count: 150, metadata_kv_count: 20 };
    assert!(validate_compatibility(&header).is_err());
}
```

## Next Task
→ Task 008: Extract tensor information from GGUF