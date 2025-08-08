use std::collections::{BTreeMap, HashSet};
use rustc_hash::FxHashMap;
use std::path::PathBuf;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use lru::LruCache;
use std::num::NonZeroUsize;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Persistent inverted index with incremental updates
pub struct InvertedIndex {
    /// Main storage (file-backed for persistence)
    term_to_docs: BTreeMap<String, PostingList>,
    /// Document metadata
    doc_metadata: FxHashMap<String, DocumentMetadata>,
    
    /// Performance optimizations
    term_cache: LruCache<String, PostingList>,
    /// Frequently accessed terms
    frequent_terms: HashSet<String>,
    
    /// Storage configuration
    index_path: PathBuf,
    /// Whether to use compression
    use_compression: bool,
    
    /// Dirty flag for persistence
    is_dirty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostingList {
    /// Documents containing this term
    pub documents: Vec<PostingEntry>,
    /// Total frequency across all documents
    pub total_frequency: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostingEntry {
    /// Document identifier
    pub doc_id: String,
    /// Term frequency in this document
    pub term_frequency: usize,
    /// Positions where term appears
    pub positions: Vec<usize>,
    /// Importance boost for this term in this document
    pub importance_boost: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub file_path: String,
    pub chunk_index: usize,
    pub length: usize,
    pub language: Option<String>,
    pub last_modified: chrono::DateTime<chrono::Utc>,
}

impl InvertedIndex {
    /// Create a new inverted index
    pub fn new(index_path: PathBuf, cache_size: usize) -> Result<Self> {
        let cache_size = NonZeroUsize::new(cache_size)
            .ok_or_else(|| anyhow!("Cache size must be greater than 0"))?;
        
        Ok(Self {
            term_to_docs: BTreeMap::new(),
            doc_metadata: FxHashMap::default(),
            term_cache: LruCache::new(cache_size),
            frequent_terms: HashSet::new(),
            index_path,
            use_compression: true,
            is_dirty: false,
        })
    }
    
    /// Load index from disk
    pub async fn load(&mut self) -> Result<()> {
        let index_file = self.index_path.join("inverted_index.bin");
        let metadata_file = self.index_path.join("doc_metadata.bin");
        
        // Load term index
        if index_file.exists() {
            let mut file = fs::File::open(&index_file).await?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).await?;
            
            if self.use_compression {
                // Decompress if needed (using bincode for now, could add zstd later)
                self.term_to_docs = bincode::deserialize(&buffer)?;
            } else {
                self.term_to_docs = bincode::deserialize(&buffer)?;
            }
        }
        
        // Load document metadata
        if metadata_file.exists() {
            let mut file = fs::File::open(&metadata_file).await?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).await?;
            self.doc_metadata = bincode::deserialize(&buffer)?;
        }
        
        // Identify frequent terms (top 100 by document frequency)
        let mut term_frequencies: Vec<(String, usize)> = self.term_to_docs
            .iter()
            .map(|(term, posting)| (term.clone(), posting.documents.len()))
            .collect();
        term_frequencies.sort_by_key(|&(_, freq)| std::cmp::Reverse(freq));
        
        self.frequent_terms = term_frequencies
            .into_iter()
            .take(100)
            .map(|(term, _)| term)
            .collect();
        
        self.is_dirty = false;
        Ok(())
    }
    
    /// Save index to disk
    pub async fn save(&mut self) -> Result<()> {
        if !self.is_dirty {
            return Ok(());
        }
        
        // Create index directory if it doesn't exist
        fs::create_dir_all(&self.index_path).await?;
        
        let index_file = self.index_path.join("inverted_index.bin");
        let metadata_file = self.index_path.join("doc_metadata.bin");
        
        // Save term index
        let index_data = bincode::serialize(&self.term_to_docs)?;
        let mut file = fs::File::create(&index_file).await?;
        file.write_all(&index_data).await?;
        file.sync_all().await?;
        
        // Save document metadata
        let metadata_data = bincode::serialize(&self.doc_metadata)?;
        let mut file = fs::File::create(&metadata_file).await?;
        file.write_all(&metadata_data).await?;
        file.sync_all().await?;
        
        self.is_dirty = false;
        Ok(())
    }
    
    /// Index a document
    pub fn index_document(&mut self, doc_id: String, tokens: Vec<crate::search::bm25::Token>, metadata: DocumentMetadata) -> Result<()> {
        // Store document metadata
        self.doc_metadata.insert(doc_id.clone(), metadata);
        
        // Process tokens and build posting lists
        let mut term_positions: FxHashMap<String, Vec<usize>> = FxHashMap::default();
        let mut term_counts: FxHashMap<String, usize> = FxHashMap::default();
        
        for (pos, token) in tokens.iter().enumerate() {
            let term = token.text.to_lowercase();
            
            term_positions.entry(term.clone())
                .or_insert_with(Vec::new)
                .push(pos);
            
            *term_counts.entry(term).or_insert(0) += 1;
        }
        
        // Update posting lists
        for (term, positions) in term_positions {
            let freq = term_counts[&term];
            
            let posting_entry = PostingEntry {
                doc_id: doc_id.clone(),
                term_frequency: freq,
                positions,
                importance_boost: 1.0, // Can be adjusted based on token importance
            };
            
            // Get or create posting list
            let posting_list = self.term_to_docs.entry(term.clone())
                .or_insert_with(|| PostingList {
                    documents: Vec::new(),
                    total_frequency: 0,
                });
            
            // Remove old entry for this document if it exists (for updates)
            posting_list.documents.retain(|entry| entry.doc_id != doc_id);
            
            // Add new entry
            posting_list.documents.push(posting_entry);
            posting_list.total_frequency += freq;
            
            // Invalidate cache for this term
            self.term_cache.pop(&term);
        }
        
        self.is_dirty = true;
        Ok(())
    }
    
    /// Get posting list for a term
    pub fn get_posting_list(&mut self, term: &str) -> Option<PostingList> {
        let term_lower = term.to_lowercase();
        
        // Check cache first
        if let Some(cached) = self.term_cache.get(&term_lower) {
            return Some(cached.clone());
        }
        
        // Get from main storage
        if let Some(posting_list) = self.term_to_docs.get(&term_lower) {
            let posting_list = posting_list.clone();
            
            // Add to cache if it's a frequent term
            if self.frequent_terms.contains(&term_lower) || posting_list.documents.len() > 10 {
                self.term_cache.put(term_lower.clone(), posting_list.clone());
            }
            
            return Some(posting_list);
        }
        
        None
    }
    
    /// Remove a document from the index
    pub fn remove_document(&mut self, doc_id: &str) -> Result<()> {
        // Remove from metadata
        self.doc_metadata.remove(doc_id);
        
        // Remove from all posting lists
        let mut empty_terms = Vec::new();
        
        for (term, posting_list) in self.term_to_docs.iter_mut() {
            // Remove document entries
            let original_len = posting_list.documents.len();
            posting_list.documents.retain(|entry| entry.doc_id != doc_id);
            
            // Update total frequency
            if posting_list.documents.len() < original_len {
                // Recalculate total frequency
                posting_list.total_frequency = posting_list.documents
                    .iter()
                    .map(|entry| entry.term_frequency)
                    .sum();
                
                // Invalidate cache
                self.term_cache.pop(term);
            }
            
            // Mark term for removal if no documents left
            if posting_list.documents.is_empty() {
                empty_terms.push(term.clone());
            }
        }
        
        // Remove empty terms
        for term in empty_terms {
            self.term_to_docs.remove(&term);
            self.frequent_terms.remove(&term);
        }
        
        self.is_dirty = true;
        Ok(())
    }
    
    /// Get all documents containing a term
    pub fn get_documents_for_term(&mut self, term: &str) -> Vec<String> {
        match self.get_posting_list(term) {
            Some(posting) => posting.documents
                .into_iter()
                .map(|entry| entry.doc_id)
                .collect(),
            None => {
                // Term not found in index - legitimate empty result for search
                Vec::new()
            }
        }
    }
    
    /// Get term frequency in a specific document
    pub fn get_term_frequency(&mut self, term: &str, doc_id: &str) -> usize {
        match self.get_posting_list(term) {
            Some(posting) => {
                // Term exists in index, check if document contains it
                match posting.documents.iter().find(|entry| entry.doc_id == doc_id) {
                    Some(entry) => entry.term_frequency,
                    None => 0, // Document doesn't contain this term - legitimate 0 frequency
                }
            }
            None => {
                // Term not found in any document - legitimate 0 frequency
                0
            }
        }
    }
    
    /// Get all document IDs and their metadata
    pub fn get_all_documents(&self) -> impl Iterator<Item = (&String, &DocumentMetadata)> {
        self.doc_metadata.iter()
    }
    
    /// Get all document IDs matching a file path pattern
    pub fn get_documents_by_file_pattern(&self, file_path_pattern: &str) -> Vec<String> {
        self.doc_metadata.iter()
            .filter(|(doc_id, metadata)| {
                doc_id.starts_with(file_path_pattern) || metadata.file_path.contains(file_path_pattern)
            })
            .map(|(doc_id, _)| doc_id.clone())
            .collect()
    }
    
    /// Get statistics about the index
    pub fn get_document_metadata(&self, doc_id: &str) -> Option<&DocumentMetadata> {
        self.doc_metadata.get(doc_id)
    }
    
    pub fn get_stats(&self) -> IndexStats {
        let total_terms = self.term_to_docs.len();
        let total_documents = self.doc_metadata.len();
        let avg_terms_per_doc = if total_documents > 0 {
            self.term_to_docs.values()
                .map(|posting| posting.documents.len())
                .sum::<usize>() as f32 / total_documents as f32
        } else {
            0.0
        };
        
        IndexStats {
            total_terms,
            total_documents,
            avg_terms_per_doc,
            cache_size: self.term_cache.len(),
            frequent_terms_count: self.frequent_terms.len(),
        }
    }
    
    /// Clear the entire index
    pub fn clear(&mut self) {
        self.term_to_docs.clear();
        self.doc_metadata.clear();
        self.term_cache.clear();
        self.frequent_terms.clear();
        self.is_dirty = true;
    }
}

#[derive(Debug, Clone)]
pub struct IndexStats {
    pub total_terms: usize,
    pub total_documents: usize,
    pub avg_terms_per_doc: f32,
    pub cache_size: usize,
    pub frequent_terms_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_inverted_index_basic() {
        let temp_dir = TempDir::new().unwrap();
        let mut index = InvertedIndex::new(temp_dir.path().to_path_buf(), 100).unwrap();
        
        // Index a document
        let tokens = vec![
            crate::search::bm25::Token {
                text: "hello".to_string(),
                position: 0,
                importance_weight: 1.0,
            },
            crate::search::bm25::Token {
                text: "world".to_string(),
                position: 1,
                importance_weight: 1.0,
            },
        ];
        
        let metadata = DocumentMetadata {
            file_path: "test.txt".to_string(),
            chunk_index: 0,
            length: 2,
            language: None,
            last_modified: chrono::Utc::now(),
        };
        
        index.index_document("doc1".to_string(), tokens, metadata).unwrap();
        
        // Check posting list
        let posting = index.get_posting_list("hello").unwrap();
        assert_eq!(posting.documents.len(), 1);
        assert_eq!(posting.documents[0].doc_id, "doc1");
        assert_eq!(posting.documents[0].term_frequency, 1);
        
        // Check document list for term
        let docs = index.get_documents_for_term("hello");
        assert_eq!(docs, vec!["doc1"]);
        
        // Check term frequency
        let freq = index.get_term_frequency("hello", "doc1");
        assert_eq!(freq, 1);
    }
    
    #[tokio::test]
    async fn test_inverted_index_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().to_path_buf();
        
        // Create and populate index
        {
            let mut index = InvertedIndex::new(index_path.clone(), 100).unwrap();
            
            let tokens = vec![
                crate::search::bm25::Token {
                    text: "persistent".to_string(),
                    position: 0,
                    importance_weight: 1.0,
                },
            ];
            
            let metadata = DocumentMetadata {
                file_path: "test.txt".to_string(),
                chunk_index: 0,
                length: 1,
                language: None,
                last_modified: chrono::Utc::now(),
            };
            
            index.index_document("doc1".to_string(), tokens, metadata).unwrap();
            index.save().await.unwrap();
        }
        
        // Load index and verify
        {
            let mut index = InvertedIndex::new(index_path, 100).unwrap();
            index.load().await.unwrap();
            
            let docs = index.get_documents_for_term("persistent");
            assert_eq!(docs, vec!["doc1"]);
        }
    }
}