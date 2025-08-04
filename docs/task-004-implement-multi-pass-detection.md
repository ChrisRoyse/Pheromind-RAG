# Task 004: Implement Multi-Pass Detection System

## ⏱️ Time Estimate: 10 minutes

## 🎯 Objective
Implement a 4-pass detection system to achieve higher accuracy by combining pattern matching, semantic analysis, context analysis, and validation.

## 📋 Context for AI Model
While the fixed extraction logic improved accuracy from 0% to ~30%, we need multi-pass detection to reach 70%+ accuracy. This task implements a sophisticated detection system that validates documentation through multiple methods.

**Current Single-Pass Logic:**
```python
# Simple pattern match
if line.startswith('///'):
    has_documentation = True
```

**New Multi-Pass Logic:**
```python
# Pass 1: Pattern matching (Rust ///, Python """, JS /**)
# Pass 2: Semantic analysis (meaningful content keywords)  
# Pass 3: Context analysis (comment blocks, proximity to code)
# Pass 4: Validation and confidence scoring
```

## 🔧 Technical Requirements

### Files to Modify
1. `python/indexer_universal.py` - Add `MultiPassDocumentationDetector` class
2. Integrate multi-pass system into extraction logic
3. Add confidence scoring and validation

### Detection Passes
1. **Pattern Pass**: Language-specific regex patterns
2. **Semantic Pass**: Content analysis for documentation keywords
3. **Context Pass**: Analyze comment blocks and positioning
4. **Validation Pass**: Cross-validate and score confidence

## 📝 Implementation Steps

### Step 1: Create Multi-Pass Detector Class (5 minutes)

**File: `python/indexer_universal.py`**

Add this class before the main `UniversalCodeIndexer` class:

```python
import re
from collections import Counter

class MultiPassDocumentationDetector:
    """
    Multi-pass documentation detection system for higher accuracy.
    
    Uses 4 passes: Pattern → Semantic → Context → Validation
    """
    
    def __init__(self):
        self.documentation_keywords = {
            'description': ['represents', 'implements', 'provides', 'handles', 'manages', 'contains'],
            'parameters': ['param', 'parameter', 'arg', 'argument', 'takes', 'accepts'],
            'returns': ['returns', 'return', 'yields', 'produces', 'outputs'],
            'examples': ['example', 'usage', 'demo', 'sample', 'illustration'],
            'notes': ['note', 'warning', 'important', 'todo', 'fixme', 'deprecated']
        }
        
        self.meaningless_words = {'the', 'a', 'an', 'and', 'or', 'but', 'in', 'on', 'at', 'to', 'for', 'of', 'with', 'by', 'this', 'that'}
    
    def detect_documentation(self, lines, start_idx, language):
        """
        Run 4-pass detection on potential documentation.
        
        Args:
            lines (list): Source code lines
            start_idx (int): Index of code declaration
            language (str): Programming language
            
        Returns:
            dict: Detection results with confidence score
        """
        result = {
            'has_documentation': False,
            'doc_lines': [],
            'doc_start_idx': start_idx,
            'confidence': 0.0,
            'pass_results': {}
        }
        
        # Pass 1: Pattern Detection
        pattern_result = self._pass1_pattern_detection(lines, start_idx, language)
        result['pass_results']['pattern'] = pattern_result
        
        if not pattern_result['found']:
            return result  # No point continuing if no patterns found
            
        # Pass 2: Semantic Analysis
        semantic_result = self._pass2_semantic_analysis(pattern_result['doc_lines'])
        result['pass_results']['semantic'] = semantic_result
        
        # Pass 3: Context Analysis
        context_result = self._pass3_context_analysis(lines, pattern_result['doc_start_idx'], start_idx)
        result['pass_results']['context'] = context_result
        
        # Pass 4: Validation and Scoring
        validation_result = self._pass4_validation(pattern_result, semantic_result, context_result)
        result['pass_results']['validation'] = validation_result
        
        # Combine results
        result['has_documentation'] = validation_result['is_documentation']
        result['doc_lines'] = pattern_result['doc_lines']
        result['doc_start_idx'] = pattern_result['doc_start_idx']
        result['confidence'] = validation_result['confidence']
        
        return result
    
    def _pass1_pattern_detection(self, lines, start_idx, language):
        """Pass 1: Find documentation patterns before declaration."""
        doc_lines = []
        doc_start_idx = start_idx
        found = False
        
        check_idx = start_idx - 1
        while check_idx >= 0:
            line = lines[check_idx].strip()
            
            if not line:  # Skip empty lines
                check_idx -= 1
                continue
                
            # Language-specific pattern matching
            is_doc_line = False
            if language == 'rust':
                is_doc_line = line.startswith('///') or line.startswith('//!')
            elif language == 'python':
                is_doc_line = '"""' in line or "'''" in line
            elif language in ['javascript', 'typescript']:
                is_doc_line = line.startswith('/**') or line.startswith('*') or '*/' in line
            
            if is_doc_line:
                doc_lines.insert(0, lines[check_idx])
                found = True
                doc_start_idx = check_idx
                check_idx -= 1
            else:
                break
        
        return {
            'found': found,
            'doc_lines': doc_lines,
            'doc_start_idx': doc_start_idx,
            'pattern_count': len(doc_lines)
        }
    
    def _pass2_semantic_analysis(self, doc_lines):
        """Pass 2: Analyze content for meaningful documentation."""
        if not doc_lines:
            return {'meaningful': False, 'keyword_score': 0, 'content_length': 0}
        
        # Extract text content (remove comment markers)
        content = []
        for line in doc_lines:
            text = line.strip()
            # Remove common comment markers
            text = re.sub(r'^(///|//!|/\*\*|\*|/\*|\*/)', '', text).strip()
            if text:
                content.append(text.lower())
        
        combined_content = ' '.join(content)
        
        # Calculate meaningful content metrics
        word_count = len(combined_content.split())
        meaningful_words = [w for w in combined_content.split() if w not in self.meaningless_words and len(w) > 2]
        
        # Check for documentation keywords
        keyword_score = 0
        for category, keywords in self.documentation_keywords.items():
            for keyword in keywords:
                if keyword in combined_content:
                    keyword_score += 1
        
        return {
            'meaningful': len(meaningful_words) >= 3 and word_count >= 5,
            'keyword_score': keyword_score,
            'content_length': len(combined_content),
            'word_count': word_count,
            'meaningful_word_ratio': len(meaningful_words) / max(word_count, 1)
        }
    
    def _pass3_context_analysis(self, lines, doc_start_idx, decl_idx):
        """Pass 3: Analyze context and positioning."""
        context_score = 0
        
        # Proximity score (closer to declaration = better)
        proximity = decl_idx - doc_start_idx
        if proximity <= 2:
            context_score += 3  # Right before declaration
        elif proximity <= 5:
            context_score += 2  # Close to declaration
        elif proximity <= 10:
            context_score += 1  # Somewhat close
        
        # Check for comment block consistency
        doc_line_count = decl_idx - doc_start_idx
        if doc_line_count >= 2:
            context_score += 2  # Multi-line documentation
        
        # Check if there are other comments nearby (might be noise)
        noise_score = 0
        for i in range(max(0, doc_start_idx - 5), min(len(lines), decl_idx + 5)):
            if i < doc_start_idx or i >= decl_idx:
                line = lines[i].strip()
                if line.startswith('//') and not (line.startswith('///') or line.startswith('//!')):
                    noise_score += 1
        
        return {
            'context_score': context_score,
            'proximity': proximity,
            'doc_line_count': doc_line_count,
            'noise_score': noise_score
        }
    
    def _pass4_validation(self, pattern_result, semantic_result, context_result):
        """Pass 4: Final validation and confidence scoring."""
        confidence = 0.0
        
        # Pattern matching weight (40%)
        if pattern_result['found']:
            confidence += 0.4
        
        # Semantic analysis weight (30%)
        if semantic_result['meaningful']:
            confidence += 0.3 * min(1.0, semantic_result['meaningful_word_ratio'] * 2)
        
        # Keyword bonus
        confidence += min(0.1, semantic_result['keyword_score'] * 0.02)
        
        # Context analysis weight (20%)
        context_normalized = min(1.0, context_result['context_score'] / 5.0)
        confidence += 0.2 * context_normalized
        
        # Noise penalty
        noise_penalty = min(0.1, context_result['noise_score'] * 0.02)
        confidence -= noise_penalty
        
        # Validation checks (10%)
        validation_bonus = 0.0
        if semantic_result['content_length'] > 20:  # Substantial content
            validation_bonus += 0.05
        if context_result['proximity'] <= 3:  # Close to declaration
            validation_bonus += 0.05
        
        confidence += validation_bonus
        
        # Final confidence bounds
        confidence = max(0.0, min(1.0, confidence))
        
        # Decision threshold
        is_documentation = confidence >= 0.5
        
        return {
            'is_documentation': is_documentation,
            'confidence': confidence,
            'threshold': 0.5
        }
```

### Step 2: Integrate Multi-Pass System (3 minutes)

Update the `_extract_block_with_docs` method to use the multi-pass detector:

```python
def _extract_block_with_docs(self, lines, start_idx, patterns, block_type, language):
    """Extract a code block WITH multi-pass documentation detection."""
    
    # Initialize multi-pass detector
    if not hasattr(self, 'doc_detector'):
        self.doc_detector = MultiPassDocumentationDetector()
    
    # Run multi-pass detection
    detection_result = self.doc_detector.detect_documentation(lines, start_idx, language)
    
    # Use detection results
    has_documentation = detection_result['has_documentation']
    doc_lines = detection_result['doc_lines']
    doc_start_idx = detection_result['doc_start_idx']
    confidence = detection_result['confidence']
    
    # Build block with documentation
    block_lines = []
    if has_documentation and doc_lines:
        block_lines.extend(doc_lines)
    block_lines.append(lines[start_idx])
    
    # ... rest of extraction logic (brace counting, etc.) ...
    # (Keep existing code from task 003)
    
    return {
        'content': '\n'.join(block_lines),
        'type': f'{language}_{block_type}',
        'name': name,
        'line_start': doc_start_idx,
        'line_end': i,
        'has_documentation': has_documentation,
        'confidence': confidence,  # ✅ NEW: Confidence score
        'metadata': {
            'language': language,
            'block_type': block_type,
            'has_documentation': has_documentation,
            'confidence': confidence,
            'detection_passes': detection_result['pass_results'],  # ✅ NEW: Debug info
            'doc_lines_count': len(doc_lines),
            'total_lines': len(block_lines)
        }
    }
```

### Step 3: Add Confidence-Based Filtering (2 minutes)

Add option to filter low-confidence detections:

```python
def should_include_chunk(self, chunk, min_confidence=0.3):
    """
    Determine if chunk should be included based on confidence.
    
    Args:
        chunk (dict): Chunk with confidence score
        min_confidence (float): Minimum confidence threshold
        
    Returns:
        bool: True if chunk should be included
    """
    if not chunk.get('has_documentation'):
        return True  # Always include undocumented chunks
    
    confidence = chunk.get('confidence', 0.0)
    return confidence >= min_confidence
```

## ✅ Success Criteria

1. **Multi-pass detection works**
   - All 4 passes execute correctly
   - Confidence scores are reasonable (0.0-1.0)
   - Detection accuracy improves significantly

2. **Semantic analysis effective**
   - Detects meaningful documentation keywords
   - Filters out noise and false positives
   - Handles different documentation styles

3. **Context analysis works**
   - Proximity scoring functions correctly
   - Block consistency detection works
   - Noise detection reduces false positives

4. **Tests pass with higher accuracy**
   - Documentation detection rate improves to 70%+
   - False positive rate decreases
   - Confidence scores align with manual validation

## 🔍 Validation Commands

```bash
# Test multi-pass detection
npm test -- test/vector_system.test.js

# Test confidence scoring
cd python && python -c "
from indexer_universal import UniversalCodeIndexer
indexer = UniversalCodeIndexer()

# Test high-quality documentation
good_rust = '''
/// A spiking cortical column with TTFS (Time-to-First-Spike) dynamics.
/// This struct represents a biologically-inspired cortical column that can:
/// - Transition through states (Available → Activated → Competing)
/// - Maintain activation levels with exponential decay
pub struct SpikingCorticalColumn {
    activation_level: f64,
}
'''

chunks = indexer.parse_content(good_rust, 'rust')
print(f'Confidence: {chunks[0].get(\"confidence\", 0):.2f}')
print(f'Has docs: {chunks[0].get(\"has_documentation\", False)}')
"
```

## 📊 Expected Results

- **Accuracy Improvement**: 30% → 70%+ documentation detection
- **Confidence Scores**: Meaningful distribution across 0.0-1.0 range
- **False Positive Reduction**: Better filtering of noise comments

## 🚨 Quality Assurance (10 iterations)

Validate these aspects:

1. **All Passes Work**: Each pass produces reasonable results?
2. **Confidence Calibration**: Scores align with manual assessment?
3. **Performance Impact**: Multi-pass doesn't slow system significantly?
4. **Edge Case Handling**: Works with malformed/empty documentation?
5. **Language Coverage**: Effective across Rust, Python, JavaScript?

## 📁 Files Modified

1. `python/indexer_universal.py` - Added `MultiPassDocumentationDetector` class
2. Updated `_extract_block_with_docs` to use multi-pass system
3. Enhanced metadata with confidence scores and debug info

## ➡️ Next Task
Task 005: Implement Semantic Analysis Enhancement