use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow, Context};
use tantivy::schema::{Schema, Field, TEXT, STORED, Value};
use tantivy::{Index, doc, Term, TantivyDocument, IndexSettings};
use tantivy::query::{QueryParser, FuzzyTermQuery};
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use walkdir::WalkDir;
use std::fs;
use async_trait::async_trait;

use crate::search::ExactMatch;

pub struct TantivySearcher {
    index: Index,
    index_path: Option<PathBuf>,
    project_root: Option<PathBuf>,
    file_path_field: Field,
    line_number_field: Field,
    content_field: Field,
    line_content_field: Field,
}

impl TantivySearcher {
    /// Create a new TantivySearcher with in-memory storage (for backward compatibility)
    pub async fn new() -> Result<Self> {
        // Create schema for line-by-line indexing
        let mut schema_builder = Schema::builder();
        
        let file_path_field = schema_builder.add_text_field("file_path", STORED);
        let line_number_field = schema_builder.add_text_field("line_number", STORED);
        let content_field = schema_builder.add_text_field("content", TEXT | STORED);
        let line_content_field = schema_builder.add_text_field("line_content", STORED);
        
        let schema = schema_builder.build();
        
        // Create in-memory index for backward compatibility
        let index = Index::create_in_ram(schema.clone());
        
        Ok(Self {
            index,
            index_path: None,
            project_root: None,
            file_path_field,
            line_number_field,
            content_field,
            line_content_field,
        })
    }
    
    /// Create a new TantivySearcher with persistent file-based storage
    pub async fn new_with_path<P: AsRef<Path>>(index_path: P) -> Result<Self> {
        let index_path = index_path.as_ref().to_path_buf();
        
        // Create schema for line-by-line indexing
        let mut schema_builder = Schema::builder();
        
        let file_path_field = schema_builder.add_text_field("file_path", STORED);
        let line_number_field = schema_builder.add_text_field("line_number", STORED);
        let content_field = schema_builder.add_text_field("content", TEXT | STORED);
        let line_content_field = schema_builder.add_text_field("line_content", STORED);
        
        let schema = schema_builder.build();
        
        // Create or open persistent index
        let index = Self::create_or_open_index(&index_path, schema)
            .with_context(|| format!("Failed to create/open index at {:?}", index_path))?;
        
        Ok(Self {
            index,
            index_path: Some(index_path),
            project_root: None,
            file_path_field,
            line_number_field,
            content_field,
            line_content_field,
        })
    }
    
    /// Create a new TantivySearcher with project root scoping
    pub async fn new_with_root<P: AsRef<Path>>(project_root: P) -> Result<Self> {
        let project_root = project_root.as_ref().to_path_buf();
        
        // Create an index path within the project root for scoped indexing
        let index_path = project_root.join(".tantivy_index");
        
        // Create schema for line-by-line indexing
        let mut schema_builder = Schema::builder();
        
        let file_path_field = schema_builder.add_text_field("file_path", STORED);
        let line_number_field = schema_builder.add_text_field("line_number", STORED);
        let content_field = schema_builder.add_text_field("content", TEXT | STORED);
        let line_content_field = schema_builder.add_text_field("line_content", STORED);
        
        let schema = schema_builder.build();
        
        // Create or open persistent index
        let index = Self::create_or_open_index(&index_path, schema)
            .with_context(|| format!("Failed to create/open index at {:?}", index_path))?;
        
        Ok(Self {
            index,
            index_path: Some(index_path),
            project_root: Some(project_root),
            file_path_field,
            line_number_field,
            content_field,
            line_content_field,
        })
    }
    
    /// Create or open a persistent Tantivy index
    fn create_or_open_index(index_path: &Path, schema: Schema) -> Result<Index> {
        // Ensure parent directory exists
        if let Some(parent) = index_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create index directory: {:?}", parent))?;
        }
        
        // Try to open existing index first
        if index_path.exists() && index_path.is_dir() {
            // Check if index files exist
            let has_index_files = fs::read_dir(index_path)
                .map(|entries| entries.count() > 0)
                .unwrap_or(false);
                
            if has_index_files {
                match MmapDirectory::open(index_path) {
                    Ok(directory) => {
                        match Index::open(directory) {
                            Ok(index) => {
                                // Verify schema compatibility
                                if Self::is_schema_compatible(&index, &schema) {
                                    return Ok(index);
                                } else {
                                    eprintln!("⚠️  Schema mismatch detected, rebuilding index...");
                                    // Schema mismatch, rebuild the index
                                    Self::remove_index_files(index_path)?;
                                }
                            }
                            Err(e) => {
                                eprintln!("⚠️  Failed to open existing index ({}), rebuilding...", e);
                                Self::remove_index_files(index_path)?;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to open directory ({}), rebuilding...", e);
                        Self::remove_index_files(index_path)?;
                    }
                }
            }
        }
        
        // Create new index
        fs::create_dir_all(index_path)
            .with_context(|| format!("Failed to create index directory: {:?}", index_path))?;
            
        let directory = MmapDirectory::open(index_path)
            .with_context(|| format!("Failed to open directory for new index: {:?}", index_path))?;
            
        let index = Index::create(directory, schema, IndexSettings::default())
            .with_context(|| format!("Failed to create new index: {:?}", index_path))?;
            
        Ok(index)
    }
    
    /// Check if the existing index schema is compatible with the expected schema
    fn is_schema_compatible(index: &Index, expected_schema: &Schema) -> bool {
        let index_schema = index.schema();
        
        // Simply check if required field names exist
        let required_field_names = [
            "file_path",
            "line_number",
            "content",
            "line_content",
        ];
        
        for field_name in &required_field_names {
            // Check if field exists in both schemas
            match (expected_schema.get_field(field_name), index_schema.get_field(field_name)) {
                (Ok(_expected_field), Ok(_index_field)) => {
                    // For now, just check existence. More detailed type checking could be added later.
                    continue;
                }
                _ => {
                    return false;
                }
            }
        }
        
        true
    }
    
    /// Remove all index files from the directory
    fn remove_index_files(index_path: &Path) -> Result<()> {
        if index_path.exists() && index_path.is_dir() {
            fs::remove_dir_all(index_path)
                .with_context(|| format!("Failed to remove corrupt index: {:?}", index_path))?;
        }
        Ok(())
    }
    
    pub async fn index_directory(&mut self, path: &Path) -> Result<()> {
        let mut writer: tantivy::IndexWriter<TantivyDocument> = self.index.writer(15_000_000)?;
        
        // Walk directory and index files line by line
        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let file_path = entry.path();
            
            // Skip directories and non-code files
            if !file_path.is_file() || !self.is_code_file(file_path) {
                continue;
            }
            
            // Skip files outside project root if project scoping is enabled
            if let Some(project_root) = &self.project_root {
                if !file_path.starts_with(project_root) {
                    continue;
                }
            }
            
            // Read file and index line by line
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    for (line_num, line) in content.lines().enumerate() {
                        if line.trim().is_empty() {
                            continue;
                        }
                        
                        let doc = doc!(
                            self.file_path_field => file_path.to_string_lossy().to_string(),
                            self.line_number_field => (line_num + 1).to_string(),
                            self.content_field => line,
                            self.line_content_field => line
                        );
                        
                        writer.add_document(doc)?;
                    }
                }
                Err(_) => {
                    // Skip files that can't be read (binary files, etc.)
                    continue;
                }
            }
        }
        
        writer.commit()?;
        
        // Reload the searcher to make documents available
        self.index.reader()?.reload()?;
        
        Ok(())
    }
    
    pub async fn search(&self, query: &str) -> Result<Vec<ExactMatch>> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        
        // Create query parser for content field
        let query_parser = QueryParser::for_index(&self.index, vec![self.content_field]);
        
        // Parse the query - escape special characters for exact matching
        let escaped_query = format!("\"{}\"", query.replace("\"", "\\\""));
        let parsed_query = query_parser.parse_query(&escaped_query)?;
        
        // Search with limit of 100 results
        let top_docs = searcher.search(&parsed_query, &TopDocs::with_limit(100))?;
        
        let mut matches = Vec::new();
        
        for (_score, doc_address) in top_docs {
            // Retrieve the actual document from Tantivy
            let doc: TantivyDocument = searcher.doc(doc_address)?;
            
            // Extract real field values from the document
            let file_path = doc.get_first(self.file_path_field)
                .ok_or_else(|| anyhow!("Missing file_path field"))?
                .as_str()
                .ok_or_else(|| anyhow!("file_path field is not string"))?
                .to_string();
            
            // Skip files outside project root if project scoping is enabled
            if let Some(project_root) = &self.project_root {
                let file_path_buf = PathBuf::from(&file_path);
                if !file_path_buf.starts_with(project_root) {
                    continue;
                }
            }
                
            let line_number: usize = doc.get_first(self.line_number_field)
                .ok_or_else(|| anyhow!("Missing line_number field"))?
                .as_str()
                .ok_or_else(|| anyhow!("line_number field is not string"))?
                .parse()
                .map_err(|e| anyhow!("Failed to parse line_number: {}", e))?;
                
            let content = doc.get_first(self.content_field)
                .ok_or_else(|| anyhow!("Missing content field"))?
                .as_str()
                .ok_or_else(|| anyhow!("content field is not string"))?
                .to_string();
                
            let line_content = doc.get_first(self.line_content_field)
                .ok_or_else(|| anyhow!("Missing line_content field"))?
                .as_str()
                .ok_or_else(|| anyhow!("line_content field is not string"))?
                .to_string();
            
            matches.push(ExactMatch {
                file_path,
                line_number,
                content,
                line_content,
            });
        }
        
        Ok(matches)
    }
    
    pub async fn search_fuzzy(&self, query: &str, max_distance: u8) -> Result<Vec<ExactMatch>> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        
        // Create a fuzzy term query for the content field
        let term = Term::from_field_text(self.content_field, query);
        let fuzzy_query = FuzzyTermQuery::new(term, max_distance, true);
        
        // Search with limit of 100 results
        let top_docs = searcher.search(&fuzzy_query, &TopDocs::with_limit(100))?;
        
        let mut matches = Vec::new();
        
        for (_score, doc_address) in top_docs {
            // Retrieve the actual document from Tantivy
            let doc: TantivyDocument = searcher.doc(doc_address)?;
            
            // Extract real field values from the document
            let file_path = doc.get_first(self.file_path_field)
                .ok_or_else(|| anyhow!("Missing file_path field"))?
                .as_str()
                .ok_or_else(|| anyhow!("file_path field is not string"))?
                .to_string();
            
            // Skip files outside project root if project scoping is enabled
            if let Some(project_root) = &self.project_root {
                let file_path_buf = PathBuf::from(&file_path);
                if !file_path_buf.starts_with(project_root) {
                    continue;
                }
            }
                
            let line_number: usize = doc.get_first(self.line_number_field)
                .ok_or_else(|| anyhow!("Missing line_number field"))?
                .as_str()
                .ok_or_else(|| anyhow!("line_number field is not string"))?
                .parse()
                .map_err(|e| anyhow!("Failed to parse line_number: {}", e))?;
                
            let content = doc.get_first(self.content_field)
                .ok_or_else(|| anyhow!("Missing content field"))?
                .as_str()
                .ok_or_else(|| anyhow!("content field is not string"))?
                .to_string();
                
            let line_content = doc.get_first(self.line_content_field)
                .ok_or_else(|| anyhow!("Missing line_content field"))?
                .as_str()
                .ok_or_else(|| anyhow!("line_content field is not string"))?
                .to_string();
            
            matches.push(ExactMatch {
                file_path,
                line_number,
                content,
                line_content,
            });
        }
        
        Ok(matches)
    }
    
    pub async fn index_file(&mut self, file_path: &Path) -> Result<()> {
        let mut writer: tantivy::IndexWriter<TantivyDocument> = self.index.writer(15_000_000)?;
        
        // Skip if not a code file
        if !self.is_code_file(file_path) {
            return Ok(());
        }
        
        // Skip files outside project root if project scoping is enabled
        if let Some(project_root) = &self.project_root {
            if !file_path.starts_with(project_root) {
                return Ok(());
            }
        }
        
        // Read file and index line by line
        match fs::read_to_string(file_path) {
            Ok(content) => {
                for (line_num, line) in content.lines().enumerate() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    let doc = doc!(
                        self.file_path_field => file_path.to_string_lossy().to_string(),
                        self.line_number_field => (line_num + 1).to_string(),
                        self.content_field => line,
                        self.line_content_field => line
                    );
                    
                    writer.add_document(doc)?;
                }
            }
            Err(_) => {
                // Skip files that can't be read (binary files, etc.)
                return Ok(());
            }
        }
        
        writer.commit()?;
        
        // Reload the searcher to make documents available
        self.index.reader()?.reload()?;
        
        Ok(())
    }
    
    pub async fn clear_index(&mut self) -> Result<()> {
        if let Some(index_path) = &self.index_path {
            // For persistent storage, rebuild the index from scratch
            let schema = self.index.schema();
            self.index = Self::create_or_open_index(index_path, schema)
                .with_context(|| format!("Failed to rebuild index at {:?}", index_path))?;
        } else {
            // For in-memory storage, delete all documents
            let mut writer: tantivy::IndexWriter<TantivyDocument> = self.index.writer(15_000_000)?;
            writer.delete_all_documents()?;
            writer.commit()?;
            
            // Reload the searcher to reflect the changes
            self.index.reader()?.reload()?;
        }
        
        Ok(())
    }
    
    /// Get the path where this index is stored (if persistent)
    pub fn index_path(&self) -> Option<&Path> {
        self.index_path.as_deref()
    }
    
    /// Get the project root path (if project scoping is enabled)
    pub fn project_root(&self) -> Option<&Path> {
        self.project_root.as_deref()
    }
    
    /// Check if this searcher uses persistent storage
    pub fn is_persistent(&self) -> bool {
        self.index_path.is_some()
    }
    
    /// Force a rebuild of the persistent index
    pub async fn rebuild_index(&mut self) -> Result<()> {
        if let Some(index_path) = &self.index_path {
            let schema = self.index.schema();
            
            // Remove existing index files
            Self::remove_index_files(index_path)?;
            
            // Create new index
            self.index = Self::create_or_open_index(index_path, schema)
                .with_context(|| format!("Failed to rebuild index at {:?}", index_path))?;
            
            println!("✨ Index rebuilt at {:?}", index_path);
        } else {
            // For in-memory, just clear
            self.clear_index().await?;
        }
        
        Ok(())
    }
    
    /// Get index statistics
    pub fn get_index_stats(&self) -> Result<IndexStats> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        
        let num_docs = searcher.num_docs() as usize;
        let num_segments = searcher.segment_readers().len();
        
        let index_size = if let Some(index_path) = &self.index_path {
            Self::calculate_directory_size(index_path)?
        } else {
            0
        };
        
        Ok(IndexStats {
            num_documents: num_docs,
            num_segments,
            index_size_bytes: index_size,
            is_persistent: self.is_persistent(),
        })
    }
    
    /// Calculate the total size of an index directory
    fn calculate_directory_size(path: &Path) -> Result<u64> {
        if !path.exists() {
            return Ok(0);
        }
        
        let mut total_size = 0u64;
        
        for entry in WalkDir::new(path) {
            let entry = entry.map_err(|e| anyhow!("Failed to read directory entry: {}", e))?;
            if entry.file_type().is_file() {
                let metadata = entry.metadata().map_err(|e| anyhow!("Failed to get file metadata: {}", e))?;
                total_size += metadata.len();
            }
        }
        
        Ok(total_size)
    }
    
    fn is_code_file(&self, path: &Path) -> bool {
        match path.extension().and_then(|s| s.to_str()) {
            Some(ext) => matches!(
                ext,
                "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | 
                "go" | "java" | "cpp" | "c" | "h" | "hpp" |
                "rb" | "php" | "swift" | "kt" | "scala" | "cs" |
                "sql" | "md" | "json" | "yaml" | "yml" | "toml"
            ),
            None => false,
        }
    }
}

#[derive(Debug)]
pub struct IndexStats {
    pub num_documents: usize,
    pub num_segments: usize,
    pub index_size_bytes: u64,
    pub is_persistent: bool,
}

impl std::fmt::Display for IndexStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Index Stats: {} documents, {} segments, {:.2} MB, {} storage",
            self.num_documents,
            self.num_segments,
            self.index_size_bytes as f64 / 1024.0 / 1024.0,
            if self.is_persistent { "persistent" } else { "in-memory" }
        )
    }
}

#[async_trait]
impl crate::search::search_adapter::TextSearcher for TantivySearcher {
    async fn search(&self, query: &str) -> Result<Vec<ExactMatch>> {
        self.search(query).await
    }
    
    async fn index_file(&mut self, file_path: &Path) -> Result<()> {
        self.index_file(file_path).await
    }
    
    async fn clear_index(&mut self) -> Result<()> {
        self.clear_index().await
    }
}