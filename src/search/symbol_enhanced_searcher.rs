use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::Result;
use tokio::sync::RwLock;

use crate::search::symbol_index::{SymbolIndexer, SymbolDatabase, Symbol, SymbolKind};
use crate::search::unified::UnifiedSearcher;
use crate::search::MatchType;
use crate::search::unified::SearchResult;
use crate::chunking::ChunkContext;
use crate::config::Config;

/// Enhanced searcher that combines the existing hybrid search with symbol indexing
pub struct SymbolEnhancedSearcher {
    base_searcher: UnifiedSearcher,
    symbol_indexer: Arc<RwLock<SymbolIndexer>>,
    symbol_db: Arc<RwLock<SymbolDatabase>>,
}

impl SymbolEnhancedSearcher {
    pub async fn new(project_path: PathBuf, db_path: PathBuf) -> Result<Self> {
        let _ = Config::load();
        let base_searcher = UnifiedSearcher::new(project_path, db_path).await?;
        let symbol_indexer = Arc::new(RwLock::new(SymbolIndexer::new()?));
        let symbol_db = Arc::new(RwLock::new(SymbolDatabase::new()));
        
        Ok(Self {
            base_searcher,
            symbol_indexer,
            symbol_db,
        })
    }
    
    /// Enhanced search that includes symbol lookup
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        // Check if this is a symbol lookup query
        if let Some(symbol_results) = self.try_symbol_search(query).await? {
            return Ok(symbol_results);
        }
        
        // If query looks like a symbol search but no results found, return error
        let query_lower = query.to_lowercase();
        if self.looks_like_symbol_query(&query_lower) {
            return Err(anyhow::anyhow!(
                "No symbols found matching '{}'. Symbol search requires exact matches.",
                query
            ));
        }
        
        // For non-symbol queries, explicitly delegate to base searcher
        self.base_searcher.search(query).await
    }
    
    /// Try to perform a symbol-based search
    async fn try_symbol_search(&self, query: &str) -> Result<Option<Vec<SearchResult>>> {
        let query_lower = query.to_lowercase();
        
        // Detect symbol lookup patterns
        let is_definition_query = query_lower.starts_with("def ") || 
                                 query_lower.starts_with("definition ") ||
                                 query_lower.starts_with("find def") ||
                                 query_lower.starts_with("go to def");
        
        let is_reference_query = query_lower.starts_with("ref ") ||
                                query_lower.starts_with("references ") ||
                                query_lower.starts_with("find ref") ||
                                query_lower.starts_with("uses of");
        
        let is_symbol_type_query = query_lower.starts_with("class ") ||
                                   query_lower.starts_with("function ") ||
                                   query_lower.starts_with("struct ") ||
                                   query_lower.starts_with("interface ");
        
        if !is_definition_query && !is_reference_query && !is_symbol_type_query {
            // Also check if query looks like a code identifier (camelCase, snake_case, etc.)
            if !self.looks_like_identifier(query) {
                return Ok(None);
            }
        }
        
        // Extract the actual symbol name from the query
        let symbol_name = self.extract_symbol_name(query);
        
        // Look up in symbol database
        let db = self.symbol_db.read().await;
        
        let symbols = if is_definition_query {
            vec![db.find_definition(&symbol_name)].into_iter().flatten().collect()
        } else if is_reference_query {
            db.find_all_references(&symbol_name)
        } else if query_lower.starts_with("class ") {
            db.find_by_kind(SymbolKind::Class)
        } else if query_lower.starts_with("function ") {
            db.find_by_kind(SymbolKind::Function)
        } else {
            // Direct identifier lookup - find all matches
            db.find_all_references(&symbol_name)
        };
        
        if symbols.is_empty() {
            return Ok(None);
        }
        
        // Convert symbols to SearchResults with 3-chunk context
        let mut results = Vec::new();
        for symbol in symbols.iter().take(20) {
            match self.symbol_to_search_result(symbol).await {
                Ok(result) => results.push(result),
                Err(e) => eprintln!("Failed to convert symbol to result: {}", e),
            }
        }
        
        Ok(Some(results))
    }
    
    /// Convert a symbol to a SearchResult with 3-chunk context
    async fn symbol_to_search_result(&self, symbol: &Symbol) -> Result<SearchResult> {
        let file_path = PathBuf::from(&symbol.file_path);
        let content = tokio::fs::read_to_string(&file_path).await?;
        
        // Get the lines around the symbol
        let lines: Vec<&str> = content.lines().collect();
        let start_idx = symbol.line_start.saturating_sub(1);
        let end_idx = symbol.line_end.min(lines.len());
        
        // Build 3-chunk context manually
        let above_start = start_idx.saturating_sub(10);
        let above_content = lines[above_start..start_idx].join("\n");
        
        let target_content = lines[start_idx..end_idx].join("\n");
        
        let below_end = (end_idx + 10).min(lines.len());
        let below_content = lines[end_idx..below_end].join("\n");
        
        let context = ChunkContext {
            above: if above_start < start_idx {
                Some(crate::chunking::Chunk {
                    content: above_content,
                    start_line: above_start,
                    end_line: start_idx,
                })
            } else {
                None
            },
            target: crate::chunking::Chunk {
                content: target_content,
                start_line: start_idx,
                end_line: end_idx,
            },
            below: if end_idx < below_end {
                Some(crate::chunking::Chunk {
                    content: below_content,
                    start_line: end_idx,
                    end_line: below_end,
                })
            } else {
                None
            },
            target_index: start_idx, // Use line number as pseudo-index
        };
        
        Ok(SearchResult::new(
            symbol.file_path.clone(),
            context,
            2.0, // Symbol matches get highest score
            MatchType::Exact, // Treat as exact match
        ))
    }
    
    /// Index a file's symbols
    pub async fn index_file_symbols(&self, file_path: &Path) -> Result<()> {
        let language = match SymbolIndexer::detect_language(file_path) {
            Some(lang) => lang,
            None => return Ok(()), // Skip unsupported files
        };
        
        let content = tokio::fs::read_to_string(file_path).await?;
        
        let mut indexer = self.symbol_indexer.write().await;
        let symbols = indexer.extract_symbols(
            &content,
            language,
            &file_path.to_string_lossy()
        )?;
        
        let mut db = self.symbol_db.write().await;
        db.add_symbols(symbols);
        
        Ok(())
    }
    
    /// Index all files in a directory
    pub async fn index_directory_symbols(&self, dir_path: &Path) -> Result<usize> {
        let mut count = 0;
        let mut entries = tokio::fs::read_dir(dir_path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_dir() {
                // Skip common non-code directories
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if matches!(name_str.as_ref(), "target" | "node_modules" | ".git" | "dist" | "build") {
                        continue;
                    }
                }
                
                // Recursively index subdirectory
                count += Box::pin(self.index_directory_symbols(&path)).await?;
            } else if SymbolIndexer::detect_language(&path).is_some() {
                match self.index_file_symbols(&path).await {
                    Ok(_) => count += 1,
                    Err(e) => eprintln!("Failed to index symbols in {:?}: {}", path, e),
                }
            }
        }
        
        Ok(count)
    }
    
    /// Check if query appears to be a symbol search
    fn looks_like_symbol_query(&self, query_lower: &str) -> bool {
        query_lower.starts_with("def ") || 
        query_lower.starts_with("definition ") ||
        query_lower.starts_with("find def") ||
        query_lower.starts_with("go to def") ||
        query_lower.starts_with("ref ") ||
        query_lower.starts_with("references ") ||
        query_lower.starts_with("find ref") ||
        query_lower.starts_with("uses of") ||
        query_lower.starts_with("class ") ||
        query_lower.starts_with("function ") ||
        query_lower.starts_with("struct ") ||
        query_lower.starts_with("interface ") ||
        self.looks_like_identifier(query_lower)
    }
    
    /// Check if a string looks like a code identifier
    fn looks_like_identifier(&self, s: &str) -> bool {
        // Check for camelCase, PascalCase, snake_case, CONSTANT_CASE
        let has_camel = s.chars().any(|c| c.is_uppercase()) && s.chars().any(|c| c.is_lowercase());
        let has_snake = s.contains('_');
        let is_constant = s.chars().all(|c| c.is_uppercase() || c == '_');
        // Handle empty strings explicitly - they are not valid identifiers
        if s.is_empty() {
            return false; // Empty strings are explicitly not identifiers
        }
        let first_char = s.chars().next().unwrap(); // Safe after empty check
        let starts_with_letter = first_char.is_alphabetic();
        
        starts_with_letter && (has_camel || has_snake || is_constant)
    }
    
    /// Extract symbol name from query - no fallbacks or prefix guessing
    fn extract_symbol_name(&self, query: &str) -> String {
        // Use query as-is after trimming whitespace
        // No prefix processing to avoid creating illusions of natural language support
        query.trim().to_string()
    }
    
    /// Get statistics about the symbol index
    pub async fn symbol_stats(&self) -> Result<SymbolStats> {
        let db = self.symbol_db.read().await;
        
        Ok(SymbolStats {
            total_symbols: db.symbols_by_name.values().map(|v| v.len()).sum(),
            unique_names: db.symbols_by_name.len(),
            files_indexed: db.symbols_by_file.len(),
            functions: db.find_by_kind(SymbolKind::Function).len(),
            classes: db.find_by_kind(SymbolKind::Class).len(),
            structs: db.find_by_kind(SymbolKind::Struct).len(),
        })
    }
}

#[derive(Debug)]
pub struct SymbolStats {
    pub total_symbols: usize,
    pub unique_names: usize,
    pub files_indexed: usize,
    pub functions: usize,
    pub classes: usize,
    pub structs: usize,
}

impl std::fmt::Display for SymbolStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Symbols: {} total, {} unique | Files: {} | Functions: {}, Classes: {}, Structs: {}",
            self.total_symbols,
            self.unique_names,
            self.files_indexed,
            self.functions,
            self.classes,
            self.structs
        )
    }
}