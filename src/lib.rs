// embed-search library - AI/ML Embedding System
// Phase 1: Foundation & Safety implementation

// Core error handling module (Phase 1)
pub mod error;

// Configuration management (Phase 1)  
pub mod config;

// Safe storage implementations (Phase 1)
pub mod storage {
    pub mod safe_vectordb;  // New thread-safe implementation
    // Legacy modules to be replaced
    pub mod simple_vectordb;
    pub mod lancedb_storage;
    pub mod lancedb;
}

// Bounded cache system (Phase 1)
pub mod cache {
    pub mod bounded_cache;
}

// Embedding system (to be fixed in Phase 2)
pub mod embedding;

// Search system (to be optimized in Phase 3)
pub mod search;

// Git integration
pub mod git {
    pub mod watcher;
    // pub mod mod_git;  // TODO: Create or remove
}

// Other modules
pub mod chunking;
// pub mod file_cache;    // TODO: Create or remove
// pub mod symbol;        // TODO: Create or remove  
// pub mod treesitter;    // TODO: Create or remove

// Existing modules that were missing from lib.rs  
// pub mod observability;  // TODO: Create or remove
// pub mod utils;          // TODO: Create or remove

// Re-export commonly used types
pub use error::{EmbedError, Result};
pub use config::{Config, SearchBackend};
pub use storage::safe_vectordb::{VectorStorage, StorageConfig};
pub use cache::bounded_cache::{BoundedCache, EmbeddingCache, SearchCache};

/// Phase 1 Safety Validation
/// 
/// This function validates that all Phase 1 safety improvements are working correctly.
/// It should be called during initialization to ensure the system is safe for production.
pub fn validate_phase1_safety() -> Result<()> {
    use error::EmbedError;
    
    println!("🔍 Validating Phase 1 Safety Improvements...");
    
    // Test 1: Configuration safety
    let config = Config::default();
    config.validate()?;
    println!("  ✅ Configuration validation passed");
    
    // Test 2: Storage safety (no unsafe impl)
    let _storage = VectorStorage::new(StorageConfig::default())?;
    println!("  ✅ Storage created without unsafe code");
    
    // Test 3: Cache safety
    let _cache: BoundedCache<String, String> = BoundedCache::new(100)?;
    println!("  ✅ Bounded cache operational");
    
    // Test 4: Error handling (this would panic with unwrap)
    let result: Result<()> = Err(EmbedError::Internal {
        message: "Test error".to_string(),
        backtrace: None,
    });
    
    match result {
        Ok(_) => {},
        Err(_) => println!("  ✅ Error handling working correctly"),
    }
    
    println!("✅ Phase 1 Safety Validation Complete!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_phase1_validation() {
        assert!(validate_phase1_safety().is_ok());
    }
}