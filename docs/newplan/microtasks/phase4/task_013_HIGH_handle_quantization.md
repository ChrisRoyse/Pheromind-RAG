# Task 013 - HIGH: Handle Quantization Formats Properly

## Priority: HIGH
## Estimated Time: 10 minutes
## Dependencies: Task 012

## Objective
Implement proper quantization handling, especially Q4_K_M format used by the Nomic model.

## Current Issue
- Q4_K_M quantization not fully implemented
- Dequantization affects model performance
- Need efficient quantization support

## Tasks
1. **Implement Q4_K_M dequantization** (7 min)
   ```rust
   // In src/ml/quantization.rs
   use anyhow::Result;
   
   const QK_K: usize = 256;
   const K_SCALE_SIZE: usize = 12;
   
   #[repr(C)]
   #[derive(Clone, Copy)]
   pub struct BlockQ4KM {
       pub scales: [u8; K_SCALE_SIZE],
       pub qs: [u8; QK_K / 2],
   }
   
   pub fn dequantize_q4_k_m(data: &[u8], shape: &[u64]) -> Result<Vec<f32>> {
       let element_count = shape.iter().product::<u64>() as usize;
       let mut result = Vec::with_capacity(element_count);
       
       let block_size = std::mem::size_of::<BlockQ4KM>();
       let blocks = data.len() / block_size;
       
       for block_idx in 0..blocks {
           let block_data = &data[block_idx * block_size..(block_idx + 1) * block_size];
           let block = unsafe { &*(block_data.as_ptr() as *const BlockQ4KM) };
           
           // Extract scales
           let mut scales = [0f32; 8];
           for i in 0..6 {
               let scale_bytes = [block.scales[2 * i], block.scales[2 * i + 1]];
               let scale_u16 = u16::from_le_bytes(scale_bytes);
               scales[i] = f16::from_bits(scale_u16).to_f32();
           }
           
           // Dequantize quantized values
           for i in 0..QK_K / 2 {
               let q_byte = block.qs[i];
               
               // Extract two 4-bit values
               let q0 = (q_byte & 0x0F) as i8 - 8;
               let q1 = ((q_byte >> 4) & 0x0F) as i8 - 8;
               
               // Apply scales
               let scale_idx = i / 32;
               let scale = scales[scale_idx];
               
               result.push(q0 as f32 * scale);
               result.push(q1 as f32 * scale);
           }
       }
       
       // Trim to exact size
       result.truncate(element_count);
       Ok(result)
   }
   ```

2. **Add other quantization formats** (2 min)
   ```rust
   pub fn dequantize_q4_0(data: &[u8], shape: &[u64]) -> Result<Vec<f32>> {
       const QK4_0: usize = 32;
       
       #[repr(C)]
       struct BlockQ4_0 {
           d: f16,
           qs: [u8; QK4_0 / 2],
       }
       
       let element_count = shape.iter().product::<u64>() as usize;
       let mut result = Vec::with_capacity(element_count);
       
       let block_size = std::mem::size_of::<BlockQ4_0>();
       let blocks = data.len() / block_size;
       
       for block_idx in 0..blocks {
           let block_data = &data[block_idx * block_size..(block_idx + 1) * block_size];
           let block = unsafe { &*(block_data.as_ptr() as *const BlockQ4_0) };
           
           let scale = block.d.to_f32();
           
           for &q_byte in &block.qs {
               let q0 = (q_byte & 0x0F) as i8 - 8;
               let q1 = ((q_byte >> 4) & 0x0F) as i8 - 8;
               
               result.push(q0 as f32 * scale);
               result.push(q1 as f32 * scale);
           }
       }
       
       result.truncate(element_count);
       Ok(result)
   }
   ```

3. **Update tensor loader** (1 min)
   ```rust
   // Update load_q4_k_m_tensor in tensor_loader.rs
   fn load_q4_k_m_tensor<R: Read>(
       reader: &mut R,
       tensor_info: &TensorInfo,
       device: &Device,
   ) -> Result<Tensor> {
       let mut quant_data = vec![0u8; tensor_info.size as usize];
       reader.read_exact(&mut quant_data)?;
       
       let dequantized = crate::ml::quantization::dequantize_q4_k_m(
           &quant_data, 
           &tensor_info.dimensions
       )?;
       
       let shape: Vec<usize> = tensor_info.dimensions.iter().map(|&d| d as usize).collect();
       Tensor::from_vec(dequantized, &shape, device)
   }
   ```

## Success Criteria
- [ ] Q4_K_M dequantization implemented
- [ ] Q4_0 dequantization works
- [ ] Dequantized values are reasonable
- [ ] Performance is acceptable
- [ ] Memory usage is efficient

## Files to Create
- `src/ml/quantization.rs`

## Files to Modify
- `src/ml/tensor_loader.rs`
- `src/ml/mod.rs` (add quantization module)

## Validation
```rust
#[test]
fn test_quantization() {
    // Test with sample quantized data
    let test_data = vec![/* sample Q4_K_M block */];
    let shape = vec![32];
    let result = dequantize_q4_k_m(&test_data, &shape).unwrap();
    
    assert_eq!(result.len(), 32);
    // Values should be in reasonable range for embeddings
    assert!(result.iter().all(|&x| x.abs() < 10.0));
}
```

## Next Task
→ Task 014: Implement CPU/GPU device selection