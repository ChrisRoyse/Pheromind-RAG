// Search module with balanced sophistication

pub mod bm25_fixed;
pub mod fusion;
pub mod preprocessing;
pub mod text_processor;

// Re-export key types
pub use bm25_fixed::{BM25Engine, BM25Match};
pub use fusion::{FusionConfig, MatchType};
pub use text_processor::CodeTextProcessor;