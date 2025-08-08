use crate::chunking::Chunk;

/// Expands a target chunk to include surrounding context (above/target/below)
pub struct ThreeChunkExpander;

/// Context result containing target chunk with optional surrounding chunks
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChunkContext {
    /// Previous chunk (None if target is first chunk)
    pub above: Option<Chunk>,
    /// The matched/target chunk
    pub target: Chunk,
    /// Next chunk (None if target is last chunk)
    pub below: Option<Chunk>,
    /// Original index of target chunk in the chunks array
    pub target_index: usize,
}

/// Errors that can occur during chunk expansion
#[derive(Debug, PartialEq)]
pub enum ExpansionError {
    /// Empty chunks array provided
    EmptyChunks,
    /// Target index is out of bounds
    InvalidIndex { index: usize, max: usize },
}

impl std::fmt::Display for ExpansionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpansionError::EmptyChunks => {
                write!(f, "Cannot expand context: no chunks provided")
            }
            ExpansionError::InvalidIndex { index, max } => {
                write!(f, "Invalid chunk index {}: valid range is 0..{}", index, max)
            }
        }
    }
}

impl std::error::Error for ExpansionError {}

impl ThreeChunkExpander {
    /// Expands a target chunk to include surrounding context
    /// 
    /// # Arguments
    /// * `chunks` - Array of chunks to expand from
    /// * `target_index` - Index of the chunk to expand
    /// 
    /// # Returns
    /// * `Ok(ChunkContext)` - Context with above/target/below chunks
    /// * `Err(ExpansionError)` - If chunks is empty or index is invalid
    /// 
    /// # Examples
    /// ```
    /// use embed_search::chunking::{ThreeChunkExpander, Chunk};
    /// 
    /// let chunks = vec![
    ///     Chunk { content: "chunk1".to_string(), start_line: 0, end_line: 0 },
    ///     Chunk { content: "chunk2".to_string(), start_line: 1, end_line: 1 },
    ///     Chunk { content: "chunk3".to_string(), start_line: 2, end_line: 2 },
    /// ];
    /// 
    /// let context = ThreeChunkExpander::expand(&chunks, 1).unwrap();
    /// assert!(context.above.is_some());
    /// assert_eq!(context.target.content, "chunk2");
    /// assert!(context.below.is_some());
    /// ```
    pub fn expand(chunks: &[Chunk], target_index: usize) -> Result<ChunkContext, ExpansionError> {
        // Validate inputs
        if chunks.is_empty() {
            return Err(ExpansionError::EmptyChunks);
        }
        
        if target_index >= chunks.len() {
            return Err(ExpansionError::InvalidIndex {
                index: target_index,
                max: chunks.len() - 1,
            });
        }
        
        // Extract chunks safely
        let above = if target_index > 0 {
            Some(chunks[target_index - 1].clone())
        } else {
            None
        };
        
        let target = chunks[target_index].clone();
        
        let below = if target_index < chunks.len() - 1 {
            Some(chunks[target_index + 1].clone())
        } else {
            None
        };
        
        Ok(ChunkContext {
            above,
            target,
            below,
            target_index,
        })
    }
    
    /// Get total line range covered by the context
    pub fn get_line_range(context: &ChunkContext) -> Result<(usize, usize), crate::error::EmbedError> {
        let start = context.above
            .as_ref()
            .map(|c| c.start_line)
            .ok_or_else(|| crate::error::EmbedError::ChunkingError {
                message: "Above context chunk is missing. Cannot determine line range without complete context.".to_string(),
            })?;
            
        let end = context.below
            .as_ref()
            .map(|c| c.end_line)
            .ok_or_else(|| crate::error::EmbedError::ChunkingError {
                message: "Below context chunk is missing. Cannot determine line range without complete context.".to_string(),
            })?;
            
        Ok((start, end))
    }
    
    /// Count total lines in the context
    pub fn count_lines(context: &ChunkContext) -> usize {
        let mut count = context.target.end_line - context.target.start_line + 1;
        
        if let Some(above) = &context.above {
            count += above.end_line - above.start_line + 1;
        }
        
        if let Some(below) = &context.below {
            count += below.end_line - below.start_line + 1;
        }
        
        count
    }
}

impl ChunkContext {
    /// Format the context for display with clear chunk boundaries
    pub fn format_for_display(&self) -> String {
        let mut result = String::new();
        
        // Above chunk (if exists)
        if let Some(above) = &self.above {
            result.push_str(&format!("┌─ Context Above (lines {}-{}) ─\n", 
                                   above.start_line + 1, above.end_line + 1));
            result.push_str(&above.content);
            if !above.content.ends_with('\n') {
                result.push('\n');
            }
        }
        
        // Target chunk (highlighted)
        result.push_str(&format!("┏━ TARGET MATCH (lines {}-{}) ━\n", 
                               self.target.start_line + 1, self.target.end_line + 1));
        result.push_str(&self.target.content);
        if !self.target.content.ends_with('\n') {
            result.push('\n');
        }
        result.push_str("┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        
        // Below chunk (if exists)
        if let Some(below) = &self.below {
            result.push_str(&format!("└─ Context Below (lines {}-{}) ─\n", 
                                   below.start_line + 1, below.end_line + 1));
            result.push_str(&below.content);
            if !below.content.ends_with('\n') {
                result.push('\n');
            }
        }
        
        result
    }
    
    /// Format as compact single-line summary
    pub fn format_summary(&self) -> Result<String, crate::error::EmbedError> {
        let (start, end) = ThreeChunkExpander::get_line_range(self)?;
        let total_lines = ThreeChunkExpander::count_lines(self);
        let context_info = match (self.above.is_some(), self.below.is_some()) {
            (true, true) => "full context",
            (true, false) => "context above",
            (false, true) => "context below",
            (false, false) => "single chunk",
        };
        
        Ok(format!("Match at chunk {} (lines {}-{}, {} lines, {})", 
                   self.target_index, start + 1, end + 1, total_lines, context_info))
    }
    
    /// Get all content as a single string
    pub fn get_full_content(&self) -> String {
        let mut content = String::new();
        
        if let Some(above) = &self.above {
            content.push_str(&above.content);
            content.push('\n');
        }
        
        content.push_str(&self.target.content);
        
        if let Some(below) = &self.below {
            content.push('\n');
            content.push_str(&below.content);
        }
        
        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_chunks() -> Vec<Chunk> {
        vec![
            Chunk { content: "chunk0".to_string(), start_line: 0, end_line: 2 },
            Chunk { content: "chunk1".to_string(), start_line: 3, end_line: 5 },
            Chunk { content: "chunk2".to_string(), start_line: 6, end_line: 8 },
            Chunk { content: "chunk3".to_string(), start_line: 9, end_line: 11 },
        ]
    }
    
    #[test]
    fn test_empty_chunks() {
        let chunks = vec![];
        let result = ThreeChunkExpander::expand(&chunks, 0);
        assert_eq!(result, Err(ExpansionError::EmptyChunks));
    }
    
    #[test]
    fn test_invalid_index() {
        let chunks = create_test_chunks();
        let result = ThreeChunkExpander::expand(&chunks, 10);
        assert_eq!(result, Err(ExpansionError::InvalidIndex { index: 10, max: 3 }));
    }
    
    #[test]
    fn test_single_chunk() {
        let chunks = vec![
            Chunk { content: "only".to_string(), start_line: 0, end_line: 2 }
        ];
        
        let context = ThreeChunkExpander::expand(&chunks, 0).unwrap();
        assert!(context.above.is_none());
        assert_eq!(context.target.content, "only");
        assert!(context.below.is_none());
        assert_eq!(context.target_index, 0);
    }
    
    #[test]
    fn test_first_chunk() {
        let chunks = create_test_chunks();
        let context = ThreeChunkExpander::expand(&chunks, 0).unwrap();
        
        assert!(context.above.is_none());
        assert_eq!(context.target.content, "chunk0");
        assert!(context.below.is_some());
        assert_eq!(context.below.unwrap().content, "chunk1");
        assert_eq!(context.target_index, 0);
    }
    
    #[test]
    fn test_last_chunk() {
        let chunks = create_test_chunks();
        let context = ThreeChunkExpander::expand(&chunks, 3).unwrap();
        
        assert!(context.above.is_some());
        assert_eq!(context.above.unwrap().content, "chunk2");
        assert_eq!(context.target.content, "chunk3");
        assert!(context.below.is_none());
        assert_eq!(context.target_index, 3);
    }
    
    #[test]
    fn test_middle_chunk() {
        let chunks = create_test_chunks();
        let context = ThreeChunkExpander::expand(&chunks, 1).unwrap();
        
        assert!(context.above.is_some());
        assert_eq!(context.above.unwrap().content, "chunk0");
        assert_eq!(context.target.content, "chunk1");
        assert!(context.below.is_some());
        assert_eq!(context.below.unwrap().content, "chunk2");
        assert_eq!(context.target_index, 1);
    }
    
    #[test]
    fn test_line_range() {
        let chunks = create_test_chunks();
        let context = ThreeChunkExpander::expand(&chunks, 1).unwrap();
        let (start, end) = ThreeChunkExpander::get_line_range(&context)
            .expect("Line range calculation must succeed with complete context");
        assert_eq!(start, 0);  // Start of above chunk
        assert_eq!(end, 8);    // End of below chunk
    }
    
    #[test]
    fn test_count_lines() {
        let chunks = create_test_chunks();
        let context = ThreeChunkExpander::expand(&chunks, 1).unwrap();
        let count = ThreeChunkExpander::count_lines(&context);
        assert_eq!(count, 9); // 3 + 3 + 3 lines
    }
    
    #[test]
    fn test_format_summary() {
        let chunks = create_test_chunks();
        let context = ThreeChunkExpander::expand(&chunks, 1).unwrap();
        let summary = context.format_summary()
            .expect("Format summary must succeed with complete context");
        assert_eq!(summary, "Match at chunk 1 (lines 1-9, 9 lines, full context)");
    }
    
    #[test]
    fn test_get_full_content() {
        let chunks = vec![
            Chunk { content: "line1".to_string(), start_line: 0, end_line: 0 },
            Chunk { content: "line2".to_string(), start_line: 1, end_line: 1 },
            Chunk { content: "line3".to_string(), start_line: 2, end_line: 2 },
        ];
        
        let context = ThreeChunkExpander::expand(&chunks, 1).unwrap();
        let content = context.get_full_content();
        assert_eq!(content, "line1\nline2\nline3");
    }
}