# Task 014 - MEDIUM: Implement CPU/GPU Device Selection

## Priority: MEDIUM
## Estimated Time: 10 minutes
## Dependencies: Task 013

## Objective
Implement smart device selection for model loading, preferring GPU when available but falling back to CPU.

## Current Issue
- Hardcoded CPU device usage
- No GPU detection or utilization
- Performance could be improved with GPU

## Tasks
1. **Implement device detection** (4 min)
   ```rust
   // In src/ml/device_manager.rs
   use candle_core::Device;
   use anyhow::{Result, anyhow};
   
   #[derive(Debug, Clone)]
   pub enum DeviceType {
       Cpu,
       Cuda(usize),
       Metal,
   }
   
   pub struct DeviceManager {
       device: Device,
       device_type: DeviceType,
   }
   
   impl DeviceManager {
       pub fn new() -> Result<Self> {
           let (device, device_type) = Self::select_best_device()?;
           Ok(Self { device, device_type })
       }
       
       pub fn with_preference(preference: DeviceType) -> Result<Self> {
           let device = match preference {
               DeviceType::Cpu => Device::Cpu,
               DeviceType::Cuda(id) => {
                   if candle_core::utils::cuda_is_available() {
                       Device::new_cuda(id)?
                   } else {
                       println!("CUDA not available, falling back to CPU");
                       Device::Cpu
                   }
               },
               DeviceType::Metal => {
                   if candle_core::utils::metal_is_available() {
                       Device::new_metal(0)?
                   } else {
                       println!("Metal not available, falling back to CPU");
                       Device::Cpu
                   }
               },
           };
           
           let actual_type = match device {
               Device::Cpu => DeviceType::Cpu,
               Device::Cuda(_) => DeviceType::Cuda(0),
               Device::Metal(_) => DeviceType::Metal,
           };
           
           Ok(Self { device, device_type: actual_type })
       }
       
       fn select_best_device() -> Result<(Device, DeviceType)> {
           // Try CUDA first
           if candle_core::utils::cuda_is_available() {
               match Device::new_cuda(0) {
                   Ok(device) => {
                       println!("Using CUDA device 0");
                       return Ok((device, DeviceType::Cuda(0)));
                   }
                   Err(e) => println!("CUDA available but failed to initialize: {}", e),
               }
           }
           
           // Try Metal on macOS
           if candle_core::utils::metal_is_available() {
               match Device::new_metal(0) {
                   Ok(device) => {
                       println!("Using Metal device");
                       return Ok((device, DeviceType::Metal));
                   }
                   Err(e) => println!("Metal available but failed to initialize: {}", e),
               }
           }
           
           // Fall back to CPU
           println!("Using CPU device");
           Ok((Device::Cpu, DeviceType::Cpu))
       }
       
       pub fn device(&self) -> &Device {
           &self.device
       }
       
       pub fn device_type(&self) -> &DeviceType {
           &self.device_type
       }
   }
   ```

2. **Add memory management** (4 min)
   ```rust
   impl DeviceManager {
       pub fn get_memory_info(&self) -> Result<MemoryInfo> {
           match &self.device {
               Device::Cuda(_) => {
                   // Get CUDA memory info if available
                   Ok(MemoryInfo {
                       total: 0, // Would need CUDA API calls
                       free: 0,
                       used: 0,
                   })
               },
               Device::Metal(_) => {
                   // Get Metal memory info if available
                   Ok(MemoryInfo {
                       total: 0, // Would need Metal API calls
                       free: 0,
                       used: 0,
                   })
               },
               Device::Cpu => {
                   // Get system memory info
                   Ok(MemoryInfo {
                       total: Self::get_system_memory()?,
                       free: Self::get_available_memory()?,
                       used: 0,
                   })
               },
           }
       }
       
       fn get_system_memory() -> Result<u64> {
           // Platform-specific system memory detection
           #[cfg(target_os = "linux")]
           {
               let meminfo = std::fs::read_to_string("/proc/meminfo")?;
               // Parse MemTotal line
               Ok(16 * 1024 * 1024 * 1024) // Default 16GB
           }
           
           #[cfg(not(target_os = "linux"))]
           {
               Ok(16 * 1024 * 1024 * 1024) // Default 16GB
           }
       }
       
       fn get_available_memory() -> Result<u64> {
           Ok(8 * 1024 * 1024 * 1024) // Default 8GB available
       }
   }
   
   #[derive(Debug)]
   pub struct MemoryInfo {
       pub total: u64,
       pub free: u64,
       pub used: u64,
   }
   ```

3. **Integrate with embedding service** (2 min)
   ```rust
   // Update EmbeddingService to use DeviceManager
   impl EmbeddingService {
       pub async fn new() -> Result<Self> {
           let device_manager = DeviceManager::new()?;
           println!("Selected device: {:?}", device_manager.device_type());
           
           // Use device_manager.device() for tensor operations
           Ok(Self {
               device_manager,
               model: None,
               tokenizer: None,
           })
       }
   }
   ```

## Success Criteria
- [ ] Device detection works correctly
- [ ] GPU used when available
- [ ] CPU fallback works
- [ ] Memory info accessible
- [ ] Integration with existing code

## Files to Create
- `src/ml/device_manager.rs`

## Files to Modify
- `src/ml/embedding_service.rs`
- `src/ml/mod.rs`

## Next Task
→ Task 015: Optimize memory management for large models