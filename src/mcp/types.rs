use serde::{Deserialize, Serialize};

/// MCP server capabilities as per the specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCapabilities {
    pub search: SearchCapabilities,
    pub indexing: IndexingCapabilities,
    pub stats: StatsCapabilities,
    pub server_info: ServerInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchCapabilities {
    pub semantic_search: bool,
    pub exact_search: bool,
    pub symbol_search: bool,
    pub statistical_search: bool,
    pub fuzzy_search: bool,
    pub max_results: u32,
    pub supported_file_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingCapabilities {
    pub batch_indexing: bool,
    pub incremental_updates: bool,
    pub file_watching: bool,
    pub symbol_extraction: bool,
    pub max_file_size_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsCapabilities {
    pub index_stats: bool,
    pub search_metrics: bool,
    pub performance_metrics: bool,
    pub cache_stats: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub features: Vec<String>,
    pub supported_backends: Vec<String>,
}

/// Search request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub max_results: Option<u32>,
    pub search_types: Option<Vec<SearchType>>,
    pub file_filters: Option<Vec<String>>,
    pub include_content: Option<bool>,
    pub context_lines: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchType {
    #[serde(rename = "semantic")]
    Semantic,
    #[serde(rename = "exact")]
    Exact,
    #[serde(rename = "symbol")]
    Symbol,
    #[serde(rename = "statistical")]
    Statistical,
    #[serde(rename = "fuzzy")]
    Fuzzy,
}

/// Search response with results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchMatch>,
    pub total_matches: u32,
    pub search_time_ms: u64,
    pub search_types_used: Vec<SearchType>,
}

/// Individual search match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    pub file_path: String,
    pub score: f64,
    pub match_type: String,
    pub line_number: Option<u32>,
    pub start_line: u32,
    pub end_line: u32,
    pub content: String,
    pub context: Option<SearchContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchContext {
    pub before: Vec<String>,
    pub after: Vec<String>,
    pub surrounding_chunks: Option<String>,
}

/// Index request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexRequest {
    pub paths: Vec<String>,
    pub incremental: Option<bool>,
    pub include_test_files: Option<bool>,
    pub file_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
}

/// Index response with statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexResponse {
    pub files_indexed: u32,
    pub chunks_created: u32,
    pub symbols_extracted: u32,
    pub errors: u32,
    pub index_time_ms: u64,
    pub index_stats: Option<IndexStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_files: u32,
    pub total_chunks: u32,
    pub total_symbols: u32,
    pub index_size_bytes: u64,
    pub last_updated: String,
}

/// Statistics request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsRequest {
    pub include_cache: Option<bool>,
    pub include_performance: Option<bool>,
    pub include_index: Option<bool>,
}

/// Statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    pub index_stats: Option<IndexStats>,
    pub cache_stats: Option<CacheStats>,
    pub performance_stats: Option<PerformanceStats>,
    pub server_stats: ServerStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub search_cache_entries: u32,
    pub search_cache_hit_rate: f64,
    pub embedding_cache_entries: u32,
    pub embedding_cache_hit_rate: f64,
    pub total_cache_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub avg_search_time_ms: f64,
    pub avg_index_time_ms: f64,
    pub total_searches: u64,
    pub total_indexes: u64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStats {
    pub active_connections: u32,
    pub total_requests: u64,
    pub error_count: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

/// Clear index request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearRequest {
    pub confirm: bool,
    pub clear_type: Option<ClearType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClearType {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "search_index")]
    SearchIndex,
    #[serde(rename = "vector_index")]
    VectorIndex,
    #[serde(rename = "symbol_index")]
    SymbolIndex,
    #[serde(rename = "cache")]
    Cache,
}

/// Clear index response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearResponse {
    pub cleared: bool,
    pub clear_type: String,
    pub items_removed: u32,
}

// Convert from internal search results to MCP format
impl From<crate::search::cache::SearchResult> for SearchMatch {
    fn from(result: crate::search::cache::SearchResult) -> Self {
        Self {
            file_path: result.file,
            score: result.score as f64,
            match_type: format!("{:?}", result.match_type),
            line_number: Some(result.three_chunk_context.target.start_line as u32),
            start_line: result.three_chunk_context.above.as_ref()
                .map(|chunk| chunk.start_line as u32)
                .unwrap_or(result.three_chunk_context.target.start_line as u32),
            end_line: result.three_chunk_context.below.as_ref()
                .map(|chunk| chunk.end_line as u32)
                .unwrap_or(result.three_chunk_context.target.end_line as u32),
            content: result.three_chunk_context.target.content.clone(),
            context: Some(SearchContext {
                before: result.three_chunk_context.above.as_ref()
                    .map(|chunk| chunk.content.lines().map(|s| s.to_string()).collect())
                    .unwrap_or_default(),
                after: result.three_chunk_context.below.as_ref()
                    .map(|chunk| chunk.content.lines().map(|s| s.to_string()).collect())
                    .unwrap_or_default(),
                surrounding_chunks: None,
            }),
        }
    }
}

impl From<crate::search::unified::IndexStats> for IndexStats {
    fn from(stats: crate::search::unified::IndexStats) -> Self {
        Self {
            total_files: stats.files_indexed as u32,
            total_chunks: stats.chunks_created as u32,
            total_symbols: 0, // Not available in current IndexStats
            index_size_bytes: 0, // Not available in current IndexStats
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }
}