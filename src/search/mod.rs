pub mod ripgrep;
pub mod native_search;
pub mod fusion;
pub mod unified;
pub mod preprocessing;
pub mod cache;

pub use ripgrep::{RipgrepSearcher, ExactMatch};
pub use native_search::{NativeSearcher, SearchMatch};
pub use fusion::{SimpleFusion, FusedResult, MatchType};
pub use unified::{UnifiedSearcher, SearchResult};
pub use preprocessing::QueryPreprocessor;
pub use cache::SearchCache;