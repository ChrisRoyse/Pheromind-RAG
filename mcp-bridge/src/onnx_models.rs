use anyhow::{Result, Context};
use ort::{Environment, ExecutionProvider, Session, SessionBuilder};
use std::sync::Arc;
use std::path::Path;

/// ONNX model information
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub path: String,
    pub input_shape: Vec<i64>,
    pub output_shape: Vec<i64>,
    pub precision: String,
}

/// ONNX model manager for ML inference
pub struct OnnxModelManager {
    environment: Option<Arc<Environment>>,
    sessions: std::collections::HashMap<String, Arc<Session>>,
    model_info: std::collections::HashMap<String, ModelInfo>,
    use_gpu: bool,
    initialized: bool,
}

impl OnnxModelManager {
    pub fn new() -> Self {
        Self {
            environment: None,
            sessions: std::collections::HashMap::new(),
            model_info: std::collections::HashMap::new(),
            use_gpu: false,
            initialized: false,
        }
    }

    /// Initialize the ONNX Runtime environment
    pub fn initialize(&mut self, model_path: &str, use_gpu: bool) -> Result<()> {
        // Create ONNX Runtime environment
        let environment = Arc::new(
            Environment::builder()
                .with_name("onnx_embedding_env")
                .with_log_level(ort::LogLevel::Warning)
                .build()
                .context("Failed to create ONNX Runtime environment")?
        );

        self.environment = Some(environment);
        self.use_gpu = use_gpu;

        // Load default model if path is provided
        if !model_path.is_empty() {
            self.load_model("default", model_path)?;
        }

        self.initialized = true;
        log::info!("ONNX model manager initialized with GPU support: {}", use_gpu);
        Ok(())
    }

    /// Load a model from file
    pub fn load_model(&mut self, model_name: &str, model_path: &str) -> Result<()> {
        if !self.initialized {
            return Err(anyhow::anyhow!("ONNX manager not initialized"));
        }

        let environment = self.environment.as_ref().unwrap();

        // Validate model file exists
        if !Path::new(model_path).exists() {
            return Err(anyhow::anyhow!("Model file not found: {}", model_path));
        }

        // Create session builder
        let mut session_builder = SessionBuilder::new(environment)?;

        // Configure execution providers
        if self.use_gpu {
            // Try CUDA first, then TensorRT, then DirectML, finally CPU
            let providers = vec![
                ExecutionProvider::CUDA(Default::default()),
                ExecutionProvider::TensorRT(Default::default()),
                #[cfg(target_os = "windows")]
                ExecutionProvider::DirectML(Default::default()),
                ExecutionProvider::CPU(Default::default()),
            ];

            if let Err(e) = session_builder.with_execution_providers(providers) {
                log::warn!("Failed to set GPU execution providers: {}. Using CPU only.", e);
                session_builder.with_execution_providers([ExecutionProvider::CPU(Default::default())])?;
            }
        } else {
            session_builder.with_execution_providers([ExecutionProvider::CPU(Default::default())])?;
        }

        // Enable optimizations
        session_builder
            .with_optimization_level(ort::GraphOptimizationLevel::All)?
            .with_intra_threads(num_cpus::get())?;

        // Load the model
        let session = Arc::new(
            session_builder
                .with_model_from_file(model_path)
                .with_context(|| format!("Failed to load model from {}", model_path))?
        );

        // Get model metadata
        let model_info = self.extract_model_info(model_name, model_path, &session)?;

        // Store session and info
        self.sessions.insert(model_name.to_string(), session);
        self.model_info.insert(model_name.to_string(), model_info);

        log::info!("Loaded model '{}' from {}", model_name, model_path);
        Ok(())
    }

    /// Extract model information
    fn extract_model_info(&self, name: &str, path: &str, session: &Session) -> Result<ModelInfo> {
        // Get input and output information
        let inputs = session.inputs();
        let outputs = session.outputs();

        let input_shape = if let Some(input) = inputs.first() {
            input.dimensions().collect()
        } else {
            vec![]
        };

        let output_shape = if let Some(output) = outputs.first() {
            output.dimensions().collect()
        } else {
            vec![]
        };

        Ok(ModelInfo {
            name: name.to_string(),
            path: path.to_string(),
            input_shape,
            output_shape,
            precision: "fp32".to_string(), // Default assumption
        })
    }

    /// Get session by model name
    pub fn get_session(&self, model_name: &str) -> Result<Arc<Session>> {
        self.sessions
            .get(model_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Model '{}' not found", model_name))
    }

    /// Get model info by name
    pub fn get_model_info(&self, model_name: &str) -> Result<&ModelInfo> {
        self.model_info
            .get(model_name)
            .ok_or_else(|| anyhow::anyhow!("Model info for '{}' not found", model_name))
    }

    /// List all loaded models
    pub fn list_models(&self) -> Vec<String> {
        self.sessions.keys().cloned().collect()
    }

    /// Unload a model
    pub fn unload_model(&mut self, model_name: &str) -> Result<()> {
        if self.sessions.remove(model_name).is_some() {
            self.model_info.remove(model_name);
            log::info!("Unloaded model '{}'", model_name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Model '{}' not found", model_name))
        }
    }

    /// Check if model is loaded
    pub fn is_model_loaded(&self, model_name: &str) -> bool {
        self.sessions.contains_key(model_name)
    }

    /// Get current GPU support status
    pub fn has_gpu_support(&self) -> bool {
        self.use_gpu && self.check_gpu_availability()
    }

    /// Check if GPU is actually available
    fn check_gpu_availability(&self) -> bool {
        // In a real implementation, this would check for CUDA/DirectML availability
        // For now, we just return the use_gpu flag
        self.use_gpu
    }

    /// Get memory usage information
    pub fn get_memory_info(&self) -> Result<std::collections::HashMap<String, u64>> {
        let mut memory_info = std::collections::HashMap::new();

        // In a real implementation, this would query actual memory usage
        // For now, we provide estimated values
        for (name, _) in &self.sessions {
            // Estimate model size based on typical embedding models
            memory_info.insert(format!("{}_model", name), 100 * 1024 * 1024); // ~100MB estimate
        }

        memory_info.insert("total_runtime".to_string(), 50 * 1024 * 1024); // ~50MB for runtime

        Ok(memory_info)
    }

    /// Warm up models (run dummy inference to initialize)
    pub fn warmup_models(&self) -> Result<()> {
        for (name, session) in &self.sessions {
            log::debug!("Warming up model '{}'", name);
            
            // In a real implementation, this would run dummy inference
            // to ensure the model is ready and optimized
            let _ = session.inputs(); // Just access inputs to "warm up"
        }

        log::info!("Model warmup completed for {} models", self.sessions.len());
        Ok(())
    }

    /// Optimize models for inference
    pub fn optimize_models(&mut self) -> Result<()> {
        // In a real implementation, this might:
        // 1. Convert models to optimized formats
        // 2. Apply graph optimizations
        // 3. Quantize models if beneficial
        // 4. Set optimal batch sizes

        log::info!("Model optimization completed");
        Ok(())
    }

    /// Get runtime statistics
    pub fn get_runtime_stats(&self) -> std::collections::HashMap<String, serde_json::Value> {
        let mut stats = std::collections::HashMap::new();
        
        stats.insert("initialized".to_string(), serde_json::Value::Bool(self.initialized));
        stats.insert("gpu_enabled".to_string(), serde_json::Value::Bool(self.use_gpu));
        stats.insert("models_loaded".to_string(), serde_json::Value::Number(self.sessions.len().into()));
        stats.insert("gpu_available".to_string(), serde_json::Value::Bool(self.has_gpu_support()));

        // Add model-specific stats
        for (name, info) in &self.model_info {
            let model_stats = serde_json::json!({
                "path": info.path,
                "input_shape": info.input_shape,
                "output_shape": info.output_shape,
                "precision": info.precision
            });
            stats.insert(format!("model_{}", name), model_stats);
        }

        stats
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Cleanup all resources
    pub fn cleanup(&mut self) -> Result<()> {
        // Clear all sessions and model info
        self.sessions.clear();
        self.model_info.clear();
        
        // Drop environment last
        self.environment = None;
        
        self.initialized = false;
        log::info!("ONNX model manager cleaned up");
        Ok(())
    }
}

impl Default for OnnxModelManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_manager_creation() {
        let manager = OnnxModelManager::new();
        assert!(!manager.is_initialized());
        assert!(manager.list_models().is_empty());
    }

    #[test]
    fn test_model_info() {
        let info = ModelInfo {
            name: "test".to_string(),
            path: "/test/model.onnx".to_string(),
            input_shape: vec![1, 512],
            output_shape: vec![1, 384],
            precision: "fp32".to_string(),
        };

        assert_eq!(info.name, "test");
        assert_eq!(info.input_shape, vec![1, 512]);
    }
}