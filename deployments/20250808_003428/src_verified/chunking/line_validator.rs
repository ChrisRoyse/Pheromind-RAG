use crate::chunking::Chunk;

/// Validates that chunks have correct line tracking
pub struct LineValidator;

impl LineValidator {
    /// Validates that chunks cover all lines without gaps or overlaps
    pub fn validate_coverage(chunks: &[Chunk], total_lines: usize) -> Result<(), ValidationError> {
        if chunks.is_empty() && total_lines > 0 {
            return Err(ValidationError::EmptyChunks);
        }
        
        if chunks.is_empty() {
            return Ok(());
        }
        
        // Check first chunk starts at 0
        if chunks[0].start_line != 0 {
            return Err(ValidationError::InvalidStart {
                expected: 0,
                actual: chunks[0].start_line,
            });
        }
        
        // Check last chunk ends at last line
        let expected_end = if total_lines > 0 { total_lines - 1 } else { 0 };
        let last_chunk = chunks.last()
            .ok_or_else(|| ValidationError::Custom {
                message: "Cannot validate last chunk: chunks collection is empty".to_string(),
            })?;
        let actual_end = last_chunk.end_line;
        if actual_end != expected_end {
            return Err(ValidationError::InvalidEnd {
                expected: expected_end,
                actual: actual_end,
            });
        }
        
        // Check continuity
        for i in 1..chunks.len() {
            let prev_end = chunks[i-1].end_line;
            let curr_start = chunks[i].start_line;
            
            if curr_start <= prev_end {
                return Err(ValidationError::Overlap {
                    chunk1: i - 1,
                    chunk2: i,
                    line: curr_start,
                });
            }
            
            if curr_start > prev_end + 1 {
                return Err(ValidationError::Gap {
                    chunk1: i - 1,
                    chunk2: i,
                    gap_start: prev_end + 1,
                    gap_end: curr_start - 1,
                });
            }
        }
        
        Ok(())
    }
    
    /// Validates that chunk content matches the original lines
    pub fn validate_content(chunks: &[Chunk], original_lines: &[&str]) -> Result<(), ValidationError> {
        for (chunk_idx, chunk) in chunks.iter().enumerate() {
            // Reconstruct expected content from original lines
            let expected_lines = &original_lines[chunk.start_line..=chunk.end_line];
            let expected_content = expected_lines.join("\n");
            
            // Compare the actual chunk content with expected content
            if chunk.content != expected_content {
                // For debugging, let's see what the actual difference is
                let chunk_lines: Vec<&str> = chunk.content.lines().collect();
                let expected_line_count = chunk.end_line - chunk.start_line + 1;
                
                if chunk_lines.len() != expected_line_count {
                    return Err(ValidationError::LineCountMismatch {
                        chunk: chunk_idx,
                        expected: expected_line_count,
                        actual: chunk_lines.len(),
                    });
                }
                
                // Check each line for content mismatch
                for (i, chunk_line) in chunk_lines.iter().enumerate() {
                    let original_idx = chunk.start_line + i;
                    if original_idx >= original_lines.len() {
                        return Err(ValidationError::LineOutOfBounds {
                            chunk: chunk_idx,
                            line: original_idx,
                        });
                    }
                    
                    if chunk_line != &original_lines[original_idx] {
                        return Err(ValidationError::ContentMismatch {
                            chunk: chunk_idx,
                            line: original_idx,
                            expected: original_lines[original_idx].to_string(),
                            actual: chunk_line.to_string(),
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get a bitmap of which lines are covered by chunks
    pub fn get_coverage_map(chunks: &[Chunk], total_lines: usize) -> Vec<bool> {
        let mut covered = vec![false; total_lines];
        
        for chunk in chunks {
            for line_num in chunk.start_line..=chunk.end_line {
                if line_num < total_lines {
                    covered[line_num] = true;
                }
            }
        }
        
        covered
    }
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    EmptyChunks,
    InvalidStart { expected: usize, actual: usize },
    InvalidEnd { expected: usize, actual: usize },
    Overlap { chunk1: usize, chunk2: usize, line: usize },
    Gap { chunk1: usize, chunk2: usize, gap_start: usize, gap_end: usize },
    LineCountMismatch { chunk: usize, expected: usize, actual: usize },
    LineOutOfBounds { chunk: usize, line: usize },
    ContentMismatch { chunk: usize, line: usize, expected: String, actual: String },
    Custom { message: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyChunks => {
                write!(f, "No chunks generated for non-empty content")
            }
            ValidationError::InvalidStart { expected, actual } => {
                write!(f, "First chunk should start at line {}, but starts at {}", expected, actual)
            }
            ValidationError::InvalidEnd { expected, actual } => {
                write!(f, "Last chunk should end at line {}, but ends at {}", expected, actual)
            }
            ValidationError::Overlap { chunk1, chunk2, line } => {
                write!(f, "Chunks {} and {} overlap at line {}", chunk1, chunk2, line)
            }
            ValidationError::Gap { chunk1, chunk2, gap_start, gap_end } => {
                write!(f, "Gap between chunks {} and {}: lines {} to {} are not covered", 
                       chunk1, chunk2, gap_start, gap_end)
            }
            ValidationError::LineCountMismatch { chunk, expected, actual } => {
                write!(f, "Chunk {} should have {} lines but has {}", chunk, expected, actual)
            }
            ValidationError::LineOutOfBounds { chunk, line } => {
                write!(f, "Chunk {} references line {} which is out of bounds", chunk, line)
            }
            ValidationError::ContentMismatch { chunk, line, expected, actual } => {
                write!(f, "Chunk {} line {} content mismatch:\nExpected: '{}'\nActual: '{}'", 
                       chunk, line, expected, actual)
            }
            ValidationError::Custom { message } => {
                write!(f, "{}", message)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_coverage_empty() {
        let chunks = vec![];
        assert!(LineValidator::validate_coverage(&chunks, 0).is_ok());
        assert!(LineValidator::validate_coverage(&chunks, 5).is_err());
    }
    
    #[test]
    fn test_validate_coverage_single_chunk() {
        let chunks = vec![Chunk {
            content: "line1\nline2".to_string(),
            start_line: 0,
            end_line: 1,
        }];
        
        assert!(LineValidator::validate_coverage(&chunks, 2).is_ok());
    }
    
    #[test]
    fn test_validate_coverage_with_gap() {
        let chunks = vec![
            Chunk {
                content: "chunk1".to_string(),
                start_line: 0,
                end_line: 1,
            },
            Chunk {
                content: "chunk2".to_string(),
                start_line: 3,
                end_line: 4,
            },
        ];
        
        match LineValidator::validate_coverage(&chunks, 5) {
            Err(ValidationError::Gap { gap_start, gap_end, .. }) => {
                assert_eq!(gap_start, 2);
                assert_eq!(gap_end, 2);
            }
            _ => panic!("Expected gap error"),
        }
    }
    
    #[test]
    fn test_coverage_map() {
        let chunks = vec![
            Chunk {
                content: "chunk1".to_string(),
                start_line: 0,
                end_line: 1,
            },
            Chunk {
                content: "chunk2".to_string(),
                start_line: 2,
                end_line: 3,
            },
        ];
        
        let map = LineValidator::get_coverage_map(&chunks, 5);
        assert_eq!(map, vec![true, true, true, true, false]);
    }
}