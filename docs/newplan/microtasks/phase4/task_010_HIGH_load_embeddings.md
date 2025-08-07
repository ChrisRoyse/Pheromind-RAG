# Task 010 - HIGH: Load Embedding Layer Tensors

## Priority: HIGH
## Estimated Time: 10 minutes
## Dependencies: Task 009

## Objective
Implement tensor loading from GGUF data into Candle tensors for the embedding layer.

## Current Issue
- Need to convert GGUF tensor data to Candle format
- Handle quantized weights properly
- Manage memory efficiently

## Tasks
1. **Implement tensor loader** (6 min)
   ```rust
   // In src/ml/tensor_loader.rs
   use candle_core::{Tensor, Device, DType};
   use std::io::{Read, Seek, SeekFrom};
   use anyhow::Result;
   
   pub fn load_tensor<R: Read + Seek>(
       reader: &mut R,
       tensor_info: &TensorInfo,
       device: &Device,
   ) -> Result<Tensor> {
       // Seek to tensor data
       reader.seek(SeekFrom::Start(tensor_info.offset))?;
       
       match tensor_info.data_type {
           GGUFDataType::F32 => load_f32_tensor(reader, tensor_info, device),
           GGUFDataType::F16 => load_f16_tensor(reader, tensor_info, device),
           GGUFDataType::Q4_K_M => load_q4_k_m_tensor(reader, tensor_info, device),
           _ => Err(anyhow!("Unsupported data type: {:?}", tensor_info.data_type)),
       }
   }
   
   fn load_f32_tensor<R: Read>(
       reader: &mut R,
       tensor_info: &TensorInfo,
       device: &Device,
   ) -> Result<Tensor> {
       let element_count = tensor_info.dimensions.iter().product::<u64>() as usize;
       let mut data = vec![0f32; element_count];
       
       // Read raw bytes
       let byte_data = unsafe {
           std::slice::from_raw_parts_mut(
               data.as_mut_ptr() as *mut u8,
               element_count * 4
           )
       };
       reader.read_exact(byte_data)?;
       
       // Create tensor
       let shape: Vec<usize> = tensor_info.dimensions.iter().map(|&d| d as usize).collect();
       Tensor::from_vec(data, &shape, device)
   }
   ```

2. **Add quantization support** (3 min)
   ```rust
   fn load_q4_k_m_tensor<R: Read>(
       reader: &mut R,
       tensor_info: &TensorInfo,
       device: &Device,
   ) -> Result<Tensor> {
       // Read quantized data
       let mut quant_data = vec![0u8; tensor_info.size as usize];
       reader.read_exact(&mut quant_data)?;
       
       // Dequantize to f32
       let dequantized = dequantize_q4_k_m(&quant_data, &tensor_info.dimensions)?;
       
       let shape: Vec<usize> = tensor_info.dimensions.iter().map(|&d| d as usize).collect();
       Tensor::from_vec(dequantized, &shape, device)
   }
   
   fn dequantize_q4_k_m(data: &[u8], dimensions: &[u64]) -> Result<Vec<f32>> {
       // Implement Q4_K_M dequantization
       // This is a simplified version - full implementation needed
       let element_count = dimensions.iter().product::<u64>() as usize;
       let mut result = Vec::with_capacity(element_count);
       
       // Process quantized blocks
       for chunk in data.chunks(32) { // Q4_K_M block size
           // Extract scale and quantized values
           // Dequantize and add to result
           // This needs proper Q4_K_M implementation
       }
       
       Ok(result)
   }
   ```

3. **Add validation** (1 min)
   - Tensor shape verification
   - Data range checks

## Success Criteria
- [ ] F32 tensor loading works
- [ ] F16 tensor loading works
- [ ] Q4_K_M dequantization implemented
- [ ] Tensor shapes are correct
- [ ] Memory usage is reasonable

## Files to Create
- `src/ml/tensor_loader.rs`

## Known Issues
- Q4_K_M dequantization is complex
- May need external quantization library
- Performance will be slower than native

## Next Task
→ Task 011: Configure model parameters from metadata