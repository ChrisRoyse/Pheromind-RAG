# Task 006: Implement Smart Chunking Algorithm

## ⏱️ Time Estimate: 10 minutes

## 🎯 Objective
Replace random chunking with intelligent documentation-centered chunking to eliminate the core issue causing documentation-code separation, boosting accuracy from 85% to 95%+.

## 📋 Context for AI Model
The current chunking system creates arbitrary boundaries that split documentation from code. This task implements smart chunking that always preserves the documentation-code relationship by centering chunks around complete logical units.

**Current Broken Chunking:**
```
Chunk 1: "/// High-performance neural processor"
Chunk 2: "/// Handles TTFS dynamics with optimization" 
Chunk 3: "pub struct SpikingCorticalColumn {" <-- Missing docs!
```

**Fixed Smart Chunking:**
```
Chunk 1: "/// High-performance neural processor\n/// Handles TTFS dynamics\npub struct SpikingCorticalColumn { ... }"
```

## 🔧 Technical Requirements

### Files to Modify
1. `python/indexer_universal.py` - Add `SmartChunkingEngine` class
2. Replace arbitrary chunking with documentation-centered approach
3. Implement adaptive chunk sizing based on content

### Smart Chunking Principles
1. **Documentation-Centered**: Chunks always start with documentation
2. **Logical Boundaries**: Respect code structure (functions, classes, etc.)
3. **Adaptive Sizing**: Adjust chunk size based on content complexity
4. **Context Preservation**: Include relevant imports and context

## 📝 Implementation Steps

### Step 1: Create Smart Chunking Engine (5 minutes)

**File: `python/indexer_universal.py`**

Add this class before the main `UniversalCodeIndexer` class:

```python
class SmartChunkingEngine:
    """
    Smart chunking system that preserves documentation-code relationships.
    
    Core principle: Never separate documentation from the code it documents.
    """
    
    def __init__(self):
        self.min_chunk_size = 50    # Minimum characters per chunk
        self.max_chunk_size = 2000  # Maximum characters per chunk  
        self.optimal_chunk_size = 800  # Target chunk size
        self.context_lines = 3      # Lines of context to include
    
    def create_smart_chunks(self, content, language, file_path=''):
        """
        Create smart chunks that preserve documentation-code relationships.
        
        Args:
            content (str): Source code content
            language (str): Programming language
            file_path (str): Path to source file (for context)
            
        Returns:
            list: Smart chunks with preserved relationships
        """
        lines = content.split('\n')
        chunks = []
        current_pos = 0
        
        # Find all logical units (functions, classes, structs, etc.)
        logical_units = self._find_logical_units(lines, language)
        
        # Group logical units into chunks
        while current_pos < len(logical_units):
            chunk_units, next_pos = self._select_units_for_chunk(
                logical_units, current_pos, content
            )
            
            if chunk_units:
                chunk = self._create_chunk_from_units(
                    chunk_units, lines, language, file_path
                )
                chunks.append(chunk)
            
            current_pos = next_pos
        
        # Handle any remaining content
        if current_pos < len(lines):
            remaining_chunk = self._create_remaining_chunk(
                lines[current_pos:], language, file_path, current_pos
            )
            if remaining_chunk:
                chunks.append(remaining_chunk)
        
        return chunks
    
    def _find_logical_units(self, lines, language):
        """
        Find all logical units (functions, classes, etc.) with their documentation.
        
        Returns:
            list: Logical units with start/end positions and metadata
        """
        from . import LANGUAGE_PATTERNS  # Import patterns
        
        if language not in LANGUAGE_PATTERNS:
            return self._fallback_logical_units(lines)
        
        patterns = LANGUAGE_PATTERNS[language]
        units = []
        
        # Scan for declarations
        for i, line in enumerate(lines):
            for unit_type, pattern in patterns.items():
                if unit_type in ['function', 'struct', 'class', 'enum', 'impl', 'trait']:
                    try:
                        if re.match(pattern, line.strip()):
                            unit = self._extract_logical_unit(
                                lines, i, unit_type, language, patterns
                            )
                            if unit:
                                units.append(unit)
                                break
                    except re.error:
                        continue
        
        # Sort by position
        units.sort(key=lambda u: u['start_line'])
        return units
    
    def _extract_logical_unit(self, lines, decl_line, unit_type, language, patterns):
        """
        Extract a complete logical unit including documentation.
        
        Uses the enhanced documentation detection from previous tasks.
        """
        # Use multi-pass detection for documentation
        detector = MultiPassDocumentationDetector()
        detection_result = detector.detect_documentation(lines, decl_line, language)
        
        # Determine unit boundaries
        doc_start = detection_result['doc_start_idx']
        code_start = decl_line
        code_end = self._find_code_end(lines, decl_line, unit_type)
        
        # Calculate content metrics
        unit_lines = lines[doc_start:code_end + 1]
        content = '\n'.join(unit_lines)
        
        return {
            'type': unit_type,
            'start_line': doc_start,
            'end_line': code_end,
            'decl_line': decl_line,
            'lines': unit_lines,
            'content': content,
            'char_count': len(content),
            'has_documentation': detection_result['has_documentation'],
            'confidence': detection_result['confidence'],
            'detection_result': detection_result
        }
    
    def _find_code_end(self, lines, start_idx, unit_type):
        """Find the end of a code unit by analyzing structure."""
        if start_idx >= len(lines):
            return start_idx
        
        # For simple declarations without braces
        if unit_type in ['trait', 'enum'] and '{' not in lines[start_idx]:
            return start_idx
        
        # For block structures, count braces
        brace_count = 0
        found_opening = False
        
        for i in range(start_idx, len(lines)):
            line = lines[i]
            
            # Track brace counting
            if '{' in line:
                brace_count += line.count('{')
                found_opening = True
            if '}' in line:
                brace_count -= line.count('}')
            
            # End of block
            if found_opening and brace_count == 0:
                return i
            
            # Safety limit
            if i - start_idx > 500:  # Prevent runaway
                return i
        
        return min(start_idx + 50, len(lines) - 1)  # Fallback
    
    def _select_units_for_chunk(self, logical_units, start_pos, full_content):
        """
        Select logical units to include in a single chunk.
        
        Goals:
        1. Stay within size limits
        2. Keep related units together
        3. Preserve documentation-code relationships
        """
        if start_pos >= len(logical_units):
            return [], len(logical_units)
        
        selected_units = []
        total_chars = 0
        current_pos = start_pos
        
        while current_pos < len(logical_units):
            unit = logical_units[current_pos]
            unit_size = unit['char_count']
            
            # Always include first unit (even if large)
            if not selected_units:
                selected_units.append(unit)
                total_chars += unit_size
                current_pos += 1
                continue
            
            # Check if adding this unit exceeds limits
            if total_chars + unit_size > self.max_chunk_size:
                break
            
            # Check if units are related (same type, close proximity)
            last_unit = selected_units[-1]
            if self._are_units_related(last_unit, unit):
                selected_units.append(unit)
                total_chars += unit_size
                current_pos += 1
            else:
                # Stop if unrelated and we're at good size
                if total_chars >= self.min_chunk_size:
                    break
                # Include if too small
                selected_units.append(unit)
                total_chars += unit_size
                current_pos += 1
        
        return selected_units, current_pos
    
    def _are_units_related(self, unit1, unit2):
        """Check if two logical units are related and should be in same chunk."""
        # Same type units are related
        if unit1['type'] == unit2['type']:
            return True
        
        # Close proximity suggests relationship
        line_gap = unit2['start_line'] - unit1['end_line']
        if line_gap <= 5:  # Within 5 lines
            return True
        
        # impl blocks are related to their structs/traits
        if unit1['type'] == 'struct' and unit2['type'] == 'impl':
            return True
        if unit1['type'] == 'trait' and unit2['type'] == 'impl':
            return True
        
        return False
    
    def _create_chunk_from_units(self, units, all_lines, language, file_path):
        """Create a chunk from selected logical units."""
        if not units:
            return None
        
        # Determine chunk boundaries
        start_line = units[0]['start_line']
        end_line = units[-1]['end_line']
        
        # Add context lines before and after
        context_start = max(0, start_line - self.context_lines)
        context_end = min(len(all_lines), end_line + self.context_lines + 1)
        
        # Build chunk content
        content_lines = all_lines[context_start:context_end]
        content = '\n'.join(content_lines)
        
        # Gather metadata from all units
        has_any_documentation = any(unit.get('has_documentation', False) for unit in units)
        avg_confidence = sum(unit.get('confidence', 0) for unit in units) / len(units)
        unit_types = [unit['type'] for unit in units]
        
        # Extract representative name
        main_unit = max(units, key=lambda u: u.get('confidence', 0))
        chunk_name = self._extract_name_from_unit(main_unit, all_lines)
        
        return {
            'content': content,
            'type': f'{language}_smart_chunk',
            'name': chunk_name,
            'metadata': {
                'language': language,
                'file_path': file_path,
                'has_documentation': has_any_documentation,
                'confidence': avg_confidence,
                'line_start': context_start,
                'line_end': context_end - 1,
                'logical_units': len(units),
                'unit_types': list(set(unit_types)),
                'char_count': len(content),
                'chunking_method': 'smart_logical_units',
                'units_metadata': [
                    {
                        'type': u['type'],
                        'has_docs': u.get('has_documentation', False),
                        'confidence': u.get('confidence', 0)
                    } for u in units
                ]
            }
        }
    
    def _extract_name_from_unit(self, unit, all_lines):
        """Extract a meaningful name from a logical unit."""
        decl_line = all_lines[unit['decl_line']].strip()
        
        # Try to extract identifier using common patterns
        patterns = [
            r'struct\s+(\w+)',
            r'enum\s+(\w+)', 
            r'fn\s+(\w+)',
            r'class\s+(\w+)',
            r'function\s+(\w+)',
            r'def\s+(\w+)'
        ]
        
        for pattern in patterns:
            match = re.search(pattern, decl_line)
            if match:
                return match.group(1)
        
        return f"{unit['type']}_unit"
    
    def _fallback_logical_units(self, lines):
        """Fallback chunking for unsupported languages."""
        units = []
        current_start = 0
        
        # Simple paragraph-based chunking
        for i, line in enumerate(lines):
            if not line.strip():  # Empty line
                if i - current_start > 10:  # Minimum unit size
                    unit_content = '\n'.join(lines[current_start:i])
                    units.append({
                        'type': 'paragraph',
                        'start_line': current_start,
                        'end_line': i - 1,
                        'content': unit_content,
                        'char_count': len(unit_content),
                        'has_documentation': False,
                        'confidence': 0.0
                    })
                    current_start = i + 1
        
        # Handle final unit
        if current_start < len(lines):
            unit_content = '\n'.join(lines[current_start:])
            units.append({
                'type': 'paragraph',
                'start_line': current_start,
                'end_line': len(lines) - 1,
                'content': unit_content,
                'char_count': len(unit_content),
                'has_documentation': False,
                'confidence': 0.0
            })
        
        return units
```

### Step 2: Integrate Smart Chunking (3 minutes)

**File: `python/indexer_universal.py`**

Update the main `UniversalCodeIndexer` class to use smart chunking:

```python
def parse_content(self, content, language='python', file_path=''):
    """
    Parse content using smart chunking algorithm.
    
    This replaces the old arbitrary chunking with documentation-aware chunking.
    """
    if not content.strip():
        return []
    
    # Initialize smart chunking engine
    if not hasattr(self, 'chunking_engine'):
        self.chunking_engine = SmartChunkingEngine()
    
    # Use smart chunking instead of arbitrary splitting
    smart_chunks = self.chunking_engine.create_smart_chunks(content, language, file_path)
    
    # Convert to standard chunk format
    final_chunks = []
    for chunk in smart_chunks:
        if self._should_include_chunk(chunk):
            final_chunks.append(chunk)
    
    return final_chunks

def _should_include_chunk(self, chunk):
    """Determine if chunk should be included based on quality."""
    metadata = chunk.get('metadata', {})
    
    # Always include documented chunks
    if metadata.get('has_documentation', False):
        return True
    
    # Include substantial code chunks
    char_count = metadata.get('char_count', 0)
    if char_count > 100:  # Substantial content
        return True
    
    # Include chunks with multiple logical units
    if metadata.get('logical_units', 0) > 1:
        return True
    
    # Filter out very small chunks
    return char_count > 50
```

### Step 3: Add Chunking Quality Metrics (2 minutes)

Add methods to measure chunking effectiveness:

```python
def calculate_chunking_metrics(self, chunks):
    """Calculate metrics to assess chunking quality."""
    if not chunks:
        return {}
    
    total_chunks = len(chunks)
    documented_chunks = sum(1 for c in chunks if c.get('metadata', {}).get('has_documentation', False))
    
    # Size distribution
    chunk_sizes = [c.get('metadata', {}).get('char_count', 0) for c in chunks]
    avg_size = sum(chunk_sizes) / len(chunk_sizes)
    
    # Confidence distribution  
    confidences = [c.get('metadata', {}).get('confidence', 0) for c in chunks]
    avg_confidence = sum(confidences) / len(confidences)
    
    return {
        'total_chunks': total_chunks,
        'documented_chunks': documented_chunks,
        'documentation_coverage': documented_chunks / total_chunks if total_chunks > 0 else 0,
        'avg_chunk_size': int(avg_size),
        'avg_confidence': round(avg_confidence, 3),
        'size_distribution': {
            'min': min(chunk_sizes) if chunk_sizes else 0,
            'max': max(chunk_sizes) if chunk_sizes else 0,
            'median': sorted(chunk_sizes)[len(chunk_sizes)//2] if chunk_sizes else 0
        }
    }
```

## ✅ Success Criteria

1. **Smart chunking works correctly**
   - Documentation always included with related code
   - Logical units preserved (no mid-function splits)
   - Adaptive sizing based on content complexity

2. **Dramatic accuracy improvement**
   - Documentation detection jumps from 85% to 95%+
   - No more documentation-code separation issues
   - Confidence scores more accurately reflect quality

3. **Performance maintained**
   - Chunking speed remains acceptable
   - Memory usage doesn't spike significantly
   - Works across all supported languages

## 🔍 Validation Commands

```bash
# Test smart chunking
npm test -- test/chunking_algorithm.test.js

# Test chunking quality
cd python && python -c "
from indexer_universal import UniversalCodeIndexer
indexer = UniversalCodeIndexer()

# Test with complex Rust code
rust_code = '''
use std::collections::HashMap;

/// A high-performance spiking cortical column implementation.
/// 
/// This struct provides a biologically-inspired neural processing unit.
pub struct SpikingCorticalColumn {
    activation_level: f64,
    connections: HashMap<u32, f64>,
}

/// Implementation block for the cortical column.
impl SpikingCorticalColumn {
    /// Creates a new cortical column with default parameters.
    pub fn new() -> Self {
        Self {
            activation_level: 0.0,
            connections: HashMap::new(),
        }
    }
    
    /// Processes input spike and updates activation level.
    pub fn process_spike(&mut self, input: f64) -> f64 {
        self.activation_level += input * 0.1;
        self.activation_level.min(1.0)
    }
}
'''

chunks = indexer.parse_content(rust_code, 'rust')
print(f'Generated {len(chunks)} smart chunks')

for i, chunk in enumerate(chunks):
    meta = chunk.get('metadata', {})
    print(f'Chunk {i+1}:')
    print(f'  - Has docs: {meta.get(\"has_documentation\", False)}')
    print(f'  - Confidence: {meta.get(\"confidence\", 0):.3f}')
    print(f'  - Units: {meta.get(\"logical_units\", 0)}')
    print(f'  - Size: {meta.get(\"char_count\", 0)} chars')

metrics = indexer.calculate_chunking_metrics(chunks)
print(f'\\nChunking metrics: {metrics}')
"
```

## 📊 Expected Results

- **Accuracy Boost**: 85% → 95%+ documentation detection
- **Zero Documentation Loss**: No more doc-code separation
- **Improved Chunk Quality**: Better logical boundaries
- **Higher Confidence Scores**: More accurate confidence calibration

## 🚨 Quality Assurance (10 iterations)

Validate these scenarios:

1. **Complex Files**: Multi-struct files with mixed documentation
2. **Edge Cases**: Very large functions, nested structures
3. **Performance**: Large files (>10,000 lines) process efficiently
4. **Memory Usage**: No memory leaks with many chunks
5. **Cross-Language**: Works consistently across Rust, Python, JS

## 📁 Files Modified

1. `python/indexer_universal.py` - Added `SmartChunkingEngine` class
2. Updated main parsing logic to use smart chunking
3. Added chunking quality metrics and validation

## ➡️ Next Task
Task 007: Implement Confidence Scoring System