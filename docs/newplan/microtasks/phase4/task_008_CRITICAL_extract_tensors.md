# Task 008 - CRITICAL: Extract Tensor Information from GGUF

## Priority: CRITICAL
## Estimated Time: 10 minutes
## Dependencies: Task 007

## Objective
Parse tensor metadata from GGUF file to understand model structure and prepare for tensor loading.

## Current Issue
- Tensor information needed for model construction
- Must handle quantized tensor formats
- Need tensor names, shapes, and data locations

## Tasks
1. **Define tensor info structure** (3 min)
   ```rust
   // In src/ml/gguf_loader.rs
   #[derive(Debug, Clone)]
   pub struct TensorInfo {
       pub name: String,
       pub dimensions: Vec<u64>,
       pub data_type: GGUFDataType,
       pub offset: u64,
       pub size: u64,
   }
   
   #[derive(Debug, Clone, Copy)]
   pub enum GGUFDataType {
       F32 = 0,
       F16 = 1,
       Q4_0 = 2,
       Q4_1 = 3,
       Q4_K_M = 12,
       // Add others as needed
   }
   ```

2. **Implement tensor parsing** (6 min)
   ```rust
   pub fn parse_tensors<R: Read + Seek>(reader: &mut R, header: &GGUFHeader) -> Result<Vec<TensorInfo>> {
       let mut tensors = Vec::with_capacity(header.tensor_count as usize);
       
       for _ in 0..header.tensor_count {
           // Read tensor name length
           let mut name_len_bytes = [0u8; 8];
           reader.read_exact(&mut name_len_bytes)?;
           let name_len = u64::from_le_bytes(name_len_bytes);
           
           // Read tensor name
           let mut name_bytes = vec![0u8; name_len as usize];
           reader.read_exact(&mut name_bytes)?;
           let name = String::from_utf8(name_bytes)?;
           
           // Read number of dimensions
           let mut n_dims_bytes = [0u8; 4];
           reader.read_exact(&mut n_dims_bytes)?;
           let n_dims = u32::from_le_bytes(n_dims_bytes);
           
           // Read dimensions
           let mut dimensions = Vec::with_capacity(n_dims as usize);
           for _ in 0..n_dims {
               let mut dim_bytes = [0u8; 8];
               reader.read_exact(&mut dim_bytes)?;
               dimensions.push(u64::from_le_bytes(dim_bytes));
           }
           
           // Read data type
           let mut dtype_bytes = [0u8; 4];
           reader.read_exact(&mut dtype_bytes)?;
           let data_type = GGUFDataType::from_u32(u32::from_le_bytes(dtype_bytes))?;
           
           // Read offset
           let mut offset_bytes = [0u8; 8];
           reader.read_exact(&mut offset_bytes)?;
           let offset = u64::from_le_bytes(offset_bytes);
           
           tensors.push(TensorInfo {
               name,
               dimensions,
               data_type,
               offset,
               size: calculate_tensor_size(&dimensions, data_type)?,
           });
       }
       
       Ok(tensors)
   }
   ```

3. **Add utility functions** (1 min)
   - Tensor size calculation
   - Data type conversion

## Success Criteria
- [ ] Tensor parsing compiles successfully
- [ ] All tensors extracted from model
- [ ] Tensor names match expected patterns
- [ ] Dimensions are reasonable
- [ ] Data types are recognized

## Files to Modify
- `src/ml/gguf_loader.rs`

## Expected Tensor Names
- `token_embd.weight`
- `blk.*.attn_norm.weight`
- `blk.*.attn_q.weight`
- `blk.*.attn_k.weight`
- `blk.*.attn_v.weight`
- `blk.*.attn_o.weight`
- `blk.*.ffn_norm.weight`
- `blk.*.ffn_up.weight`
- `blk.*.ffn_down.weight`
- `output_norm.weight`

## Next Task
→ Task 009: Build transformer layer structure