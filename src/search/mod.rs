pub mod ripgrep;
pub mod native_search;
pub mod fusion;
pub mod unified;
pub mod preprocessing;
pub mod cache;
pub mod symbol_index;
pub mod symbol_enhanced_searcher;

pub use ripgrep::{RipgrepSearcher, ExactMatch};
pub use native_search::{NativeSearcher, SearchMatch};
pub use fusion::{SimpleFusion, FusedResult, MatchType};
pub use unified::{UnifiedSearcher, SearchResult};
pub use preprocessing::QueryPreprocessor;
pub use cache::SearchCache;
pub use symbol_index::{SymbolIndexer, SymbolDatabase, Symbol, SymbolKind};