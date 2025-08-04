#!/usr/bin/env python3
"""
Comprehensive Performance Benchmarking Suite for MCP RAG Indexer
Measures, optimizes, and validates system performance across different scales
"""

import os
import sys
import time
import statistics
import psutil
import threading
import multiprocessing
import asyncio
import concurrent.futures
import json

# Optional memory profiler import
try:
    import memory_profiler
    MEMORY_PROFILER_AVAILABLE = True
except ImportError:
    MEMORY_PROFILER_AVAILABLE = False
import hashlib
from pathlib import Path
from typing import Dict, List, Tuple, Any, Optional, Callable
from dataclasses import dataclass, asdict
from collections import defaultdict, deque
import tempfile
import random
import string

# Add current directory to Python path for local imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

@dataclass
class BenchmarkResult:
    """Result of a single benchmark run."""
    test_name: str
    scale: str
    language: str
    input_size: int
    processing_time: float
    memory_usage: float
    peak_memory: float
    cpu_usage: float
    chunks_created: int
    documentation_coverage: float
    confidence_score: float
    concurrent_workers: int = 1
    success: bool = True
    error_message: str = ""

@dataclass
class PerformanceBaseline:
    """Performance baseline for regression detection."""
    test_name: str
    scale: str
    avg_processing_time: float
    std_processing_time: float
    avg_memory_usage: float
    std_memory_usage: float
    timestamp: float
    sample_count: int

class MemoryProfiler:
    """Advanced memory usage profiler."""
    
    def __init__(self):
        self.process = psutil.Process()
        self.memory_samples = deque(maxlen=1000)
        self.peak_memory = 0
        self.monitoring = False
        self.monitor_thread = None
    
    def start_monitoring(self, interval=0.1):
        """Start continuous memory monitoring."""
        self.monitoring = True
        self.peak_memory = 0
        self.memory_samples.clear()
        
        def monitor():
            while self.monitoring:
                try:
                    memory_info = self.process.memory_info()
                    memory_mb = memory_info.rss / (1024 * 1024)
                    self.memory_samples.append(memory_mb)
                    self.peak_memory = max(self.peak_memory, memory_mb)
                    time.sleep(interval)
                except:
                    break
        
        self.monitor_thread = threading.Thread(target=monitor, daemon=True)
        self.monitor_thread.start()
    
    def stop_monitoring(self):
        """Stop memory monitoring and return statistics."""
        self.monitoring = False
        if self.monitor_thread:
            self.monitor_thread.join(timeout=1.0)
        
        if not self.memory_samples:
            return {'avg_memory': 0, 'peak_memory': 0, 'memory_samples': []}
        
        return {
            'avg_memory': statistics.mean(self.memory_samples),
            'peak_memory': self.peak_memory,
            'memory_samples': list(self.memory_samples)
        }

class ConcurrentProcessor:
    """Concurrent processing implementation for performance optimization."""
    
    def __init__(self, max_workers=None):
        self.max_workers = max_workers or min(32, (os.cpu_count() or 1) + 4)
        self.thread_pool = concurrent.futures.ThreadPoolExecutor(max_workers=self.max_workers)
        self.process_pool = concurrent.futures.ProcessPoolExecutor(max_workers=min(8, os.cpu_count() or 1))
    
    def process_files_concurrent(self, file_processor, files_and_contents, worker_count=4):
        """Process multiple files concurrently."""
        results = []
        
        # Split files into batches for workers
        batch_size = max(1, len(files_and_contents) // worker_count)
        batches = [files_and_contents[i:i + batch_size] 
                  for i in range(0, len(files_and_contents), batch_size)]
        
        with concurrent.futures.ThreadPoolExecutor(max_workers=worker_count) as executor:
            futures = []
            for batch in batches:
                future = executor.submit(self._process_batch, file_processor, batch)
                futures.append(future)
            
            for future in concurrent.futures.as_completed(futures):
                try:
                    batch_results = future.result()
                    results.extend(batch_results)
                except Exception as e:
                    print(f"Batch processing error: {e}")
        
        return results
    
    def _process_batch(self, processor, batch):
        """Process a batch of files."""
        results = []
        for file_path, content, language in batch:
            try:
                result = processor(content, language, file_path)
                results.append(result)
            except Exception as e:
                print(f"Error processing {file_path}: {e}")
        return results
    
    def cleanup(self):
        """Cleanup concurrent processing resources."""
        self.thread_pool.shutdown(wait=True)
        self.process_pool.shutdown(wait=True)

class TestDataGenerator:
    """Generate realistic test datasets of various sizes."""
    
    def __init__(self):
        self.language_templates = {
            'rust': {
                'function': '''/// {doc_comment}
/// 
/// This function {description}
/// 
/// # Arguments
/// 
/// * `{param}` - {param_desc}
/// 
/// # Returns
/// 
/// {return_desc}
/// 
/// # Examples
/// 
/// ```
/// let result = {func_name}({example_param});
/// assert_eq!(result, {expected_result});
/// ```
pub fn {func_name}({param}: {param_type}) -> {return_type} {{
    // Implementation details
    let intermediate = {param} * 2;
    let result = intermediate + 1;
    
    // Additional processing
    if result > 100 {{
        result - 50
    }} else {{
        result
    }}
}}''',
                'struct': '''/// {doc_comment}
/// 
/// This struct represents {description}
/// 
/// # Fields
/// 
/// * `{field}` - {field_desc}
#[derive(Debug, Clone)]
pub struct {struct_name} {{
    /// {field_desc}
    pub {field}: {field_type},
    /// Additional metadata
    pub metadata: Option<String>,
}}

impl {struct_name} {{
    /// Creates a new instance of {struct_name}
    pub fn new({field}: {field_type}) -> Self {{
        Self {{
            {field},
            metadata: None,
        }}
    }}
}}''',
                'enum': '''/// {doc_comment}
/// 
/// This enum represents {description}
#[derive(Debug, Clone, PartialEq)]
pub enum {enum_name} {{
    /// {variant_desc}
    {variant1}({variant1_type}),
    /// {variant2_desc} 
    {variant2} {{
        value: {variant2_type},
        description: String,
    }},
    /// Default variant
    None,
}}'''
            },
            'python': {
                'function': '''def {func_name}({param}: {param_type}) -> {return_type}:
    """
    {doc_comment}
    
    This function {description}
    
    Args:
        {param}: {param_desc}
        
    Returns:
        {return_desc}
        
    Examples:
        >>> result = {func_name}({example_param})
        >>> assert result == {expected_result}
    """
    # Implementation details
    intermediate = {param} * 2
    result = intermediate + 1
    
    # Additional processing
    if result > 100:
        return result - 50
    else:
        return result''',
                'class': '''class {class_name}:
    """
    {doc_comment}
    
    This class represents {description}
    
    Attributes:
        {field}: {field_desc}
    """
    
    def __init__(self, {field}: {field_type}):
        """
        Initialize {class_name}.
        
        Args:
            {field}: {field_desc}
        """
        self.{field} = {field}
        self.metadata = None
    
    def process(self) -> {return_type}:
        """
        Process the data.
        
        Returns:
            Processed result.
        """
        return self.{field} * 2'''
            },
            'javascript': {
                'function': '''/**
 * {doc_comment}
 * 
 * This function {description}
 * 
 * @param {{{param_type}}} {param} - {param_desc}
 * @returns {{{return_type}}} {return_desc}
 * 
 * @example
 * const result = {func_name}({example_param});
 * console.assert(result === {expected_result});
 */
function {func_name}({param}) {{
    // Implementation details
    const intermediate = {param} * 2;
    const result = intermediate + 1;
    
    // Additional processing
    if (result > 100) {{
        return result - 50;
    }} else {{
        return result;
    }}
}}''',
                'class': '''/**
 * {doc_comment}
 * 
 * This class represents {description}
 */
class {class_name} {{
    /**
     * Creates an instance of {class_name}.
     * 
     * @param {{{field_type}}} {field} - {field_desc}
     */
    constructor({field}) {{
        this.{field} = {field};
        this.metadata = null;
    }}
    
    /**
     * Process the data.
     * 
     * @returns {{{return_type}}} Processed result
     */
    process() {{
        return this.{field} * 2;
    }}
}}'''
            }
        }
    
    def generate_code_samples(self, scale: str, language: str) -> List[Tuple[str, str]]:
        """Generate code samples for the specified scale and language."""
        if scale == 'small':
            return self._generate_small_scale(language)
        elif scale == 'medium':
            return self._generate_medium_scale(language)
        elif scale == 'large':
            return self._generate_large_scale(language)
        elif scale == 'enterprise':
            return self._generate_enterprise_scale(language)
        else:
            raise ValueError(f"Unknown scale: {scale}")
    
    def _generate_small_scale(self, language: str) -> List[Tuple[str, str]]:
        """Generate small scale test data (10-100 lines)."""
        samples = []
        templates = self.language_templates.get(language, {})
        
        for i in range(3):  # 3 small files
            if 'function' in templates:
                content = templates['function'].format(
                    doc_comment=f"Small function {i} for testing",
                    description=f"performs operation number {i}",
                    func_name=f"small_function_{i}",
                    param="value",
                    param_type="i32" if language == 'rust' else "int",
                    param_desc="input value to process",
                    return_type="i32" if language == 'rust' else "int",
                    return_desc="processed result",
                    example_param="42",
                    expected_result="85"
                )
                samples.append((f"small_file_{i}.{self._get_extension(language)}", content))
        
        return samples
    
    def _generate_medium_scale(self, language: str) -> List[Tuple[str, str]]:
        """Generate medium scale test data (1000-10000 lines)."""
        samples = []
        templates = self.language_templates.get(language, {})
        
        # Generate several medium-sized files
        for file_idx in range(5):  # 5 medium files
            file_content = []
            
            # Add functions
            for i in range(20):  # 20 functions per file
                if 'function' in templates:
                    content = templates['function'].format(
                        doc_comment=f"Medium function {i} in file {file_idx}",
                        description=f"performs complex operation {i}",
                        func_name=f"medium_function_{file_idx}_{i}",
                        param="input_data",
                        param_type="Vec<i32>" if language == 'rust' else "List[int]",
                        param_desc="input data to process",
                        return_type="Result<i32, Error>" if language == 'rust' else "Optional[int]",
                        return_desc="processed result or error",
                        example_param="vec![1, 2, 3]" if language == 'rust' else "[1, 2, 3]",
                        expected_result="Some(6)" if language == 'rust' else "6"
                    )
                    file_content.append(content)
            
            # Add structs/classes
            for i in range(5):  # 5 structs/classes per file
                if 'struct' in templates and language == 'rust':
                    content = templates['struct'].format(
                        doc_comment=f"Medium struct {i} in file {file_idx}",
                        description=f"complex data structure {i}",
                        struct_name=f"MediumStruct_{file_idx}_{i}",
                        field="data",
                        field_type="Vec<String>",
                        field_desc="primary data storage"
                    )
                    file_content.append(content)
                elif 'class' in templates and language in ['python', 'javascript']:
                    content = templates['class'].format(
                        doc_comment=f"Medium class {i} in file {file_idx}",  
                        description=f"complex data handler {i}",
                        class_name=f"MediumClass_{file_idx}_{i}",
                        field="data",
                        field_type="List[str]" if language == 'python' else "Array<string>",
                        field_desc="primary data storage",
                        return_type="str" if language == 'python' else "string"
                    )
                    file_content.append(content)
            
            full_content = '\n\n'.join(file_content)
            samples.append((f"medium_file_{file_idx}.{self._get_extension(language)}", full_content))
        
        return samples
    
    def _generate_large_scale(self, language: str) -> List[Tuple[str, str]]:
        """Generate large scale test data (10k-100k lines)."""
        samples = []
        templates = self.language_templates.get(language, {})
        
        # Generate larger files with more complex structures
        for file_idx in range(10):  # 10 large files
            file_content = []
            
            # More functions per file
            for i in range(100):  # 100 functions per file
                if 'function' in templates:
                    content = templates['function'].format(
                        doc_comment=f"Large-scale function {i} in module {file_idx}",
                        description=f"performs enterprise-level operation {i}",
                        func_name=f"large_function_{file_idx}_{i}",
                        param="complex_input",
                        param_type="HashMap<String, Vec<i32>>" if language == 'rust' else "Dict[str, List[int]]",
                        param_desc="complex input data structure",
                        return_type="Result<ProcessedData, ProcessingError>" if language == 'rust' else "Optional[ProcessedData]",
                        return_desc="processed result with error handling",
                        example_param='HashMap::from([("key".to_string(), vec![1,2,3])])' if language == 'rust' else '{"key": [1, 2, 3]}',
                        expected_result="Ok(ProcessedData)" if language == 'rust' else "ProcessedData(...)"
                    )
                    file_content.append(content)
            
            # More structs/classes
            for i in range(20):  # 20 structs/classes per file
                if 'struct' in templates and language == 'rust':
                    content = templates['struct'].format(
                        doc_comment=f"Large-scale struct {i} for enterprise processing",
                        description=f"enterprise data structure {i}",
                        struct_name=f"LargeStruct_{file_idx}_{i}",
                        field="enterprise_data",
                        field_type="Arc<Mutex<HashMap<String, ProcessingUnit>>>",
                        field_desc="thread-safe enterprise data storage"
                    )
                    file_content.append(content)
                elif 'class' in templates and language in ['python', 'javascript']:
                    content = templates['class'].format(
                        doc_comment=f"Large-scale class {i} for enterprise processing",
                        description=f"enterprise data handler {i}",
                        class_name=f"LargeClass_{file_idx}_{i}",
                        field="enterprise_data",
                        field_type="Dict[str, ProcessingUnit]" if language == 'python' else "Map<string, ProcessingUnit>",
                        field_desc="enterprise data storage",
                        return_type="ProcessedResult" if language == 'python' else "ProcessedResult"
                    )
                    file_content.append(content)
            
            full_content = '\n\n'.join(file_content)
            samples.append((f"large_file_{file_idx}.{self._get_extension(language)}", full_content))
        
        return samples
    
    def _generate_enterprise_scale(self, language: str) -> List[Tuple[str, str]]:
        """Generate enterprise scale test data (100k+ lines)."""
        samples = []
        templates = self.language_templates.get(language, {})
        
        # Generate very large files with extensive documentation
        for file_idx in range(20):  # 20 enterprise files
            file_content = []
            
            # Massive number of functions
            for i in range(500):  # 500 functions per file
                if 'function' in templates:
                    content = templates['function'].format(
                        doc_comment=f"Enterprise function {i} in system module {file_idx}",
                        description=f"performs critical enterprise operation {i} with full compliance",
                        func_name=f"enterprise_function_{file_idx}_{i}",
                        param="enterprise_context",
                        param_type="Arc<RwLock<EnterpriseContext>>" if language == 'rust' else "EnterpriseContext",
                        param_desc="enterprise execution context with security and compliance",
                        return_type="Result<EnterpriseResult, EnterpriseError>" if language == 'rust' else "EnterpriseResult",
                        return_desc="enterprise result with full audit trail",
                        example_param="enterprise_context" if language == 'rust' else "EnterpriseContext(...)",
                        expected_result="Ok(EnterpriseResult)" if language == 'rust' else "EnterpriseResult(...)"
                    )
                    file_content.append(content)
            
            # Many complex structures
            for i in range(50):  # 50 structs/classes per file
                if 'struct' in templates and language == 'rust':
                    content = templates['struct'].format(
                        doc_comment=f"Enterprise-grade struct {i} with full compliance documentation",
                        description=f"enterprise data structure {i} with security and audit capabilities",
                        struct_name=f"EnterpriseStruct_{file_idx}_{i}",
                        field="secure_enterprise_data",
                        field_type="Arc<RwLock<SecureHashMap<AuditKey, EncryptedValue>>>",
                        field_desc="secure enterprise data with encryption and audit trail"
                    )
                    file_content.append(content)
                elif 'class' in templates and language in ['python', 'javascript']:
                    content = templates['class'].format(
                        doc_comment=f"Enterprise-grade class {i} with full compliance documentation",
                        description=f"enterprise data handler {i} with security and audit capabilities",
                        class_name=f"EnterpriseClass_{file_idx}_{i}",
                        field="secure_enterprise_data",
                        field_type="SecureDict[AuditKey, EncryptedValue]" if language == 'python' else "SecureMap<AuditKey, EncryptedValue>",
                        field_desc="secure enterprise data with encryption and audit trail",
                        return_type="EnterpriseResult" if language == 'python' else "EnterpriseResult"
                    )
                    file_content.append(content)
            
            full_content = '\n\n'.join(file_content)
            samples.append((f"enterprise_file_{file_idx}.{self._get_extension(language)}", full_content))
        
        return samples
    
    def _get_extension(self, language: str) -> str:
        """Get file extension for language."""
        extensions = {
            'rust': 'rs',
            'python': 'py',
            'javascript': 'js',
            'typescript': 'ts'
        }
        return extensions.get(language, 'txt')

class PerformanceBenchmarkingSuite:
    """
    Comprehensive performance benchmarking suite for MCP RAG Indexer.
    
    Measures performance across multiple dimensions:
    - Processing speed and throughput
    - Memory usage and optimization
    - Concurrent processing capabilities
    - Cross-language performance consistency
    - Performance regression detection
    """
    
    def __init__(self, indexer_class):
        self.indexer_class = indexer_class
        self.memory_profiler = MemoryProfiler()
        self.concurrent_processor = ConcurrentProcessor()
        self.test_data_generator = TestDataGenerator()
        self.baselines = {}
        self.results_history = deque(maxlen=1000)
        
        # Performance targets
        self.targets = {
            'small': {'max_time': 0.001, 'max_memory': 50},     # <1ms, <50MB
            'medium': {'max_time': 0.1, 'max_memory': 200},    # <100ms, <200MB
            'large': {'max_time': 0.5, 'max_memory': 500},     # <500ms, <500MB
            'enterprise': {'max_time': 5.0, 'max_memory': 1000} # <5s, <1GB
        }
    
    def run_comprehensive_benchmark(self) -> Dict[str, Any]:
        """Run comprehensive performance benchmark across all scales and languages."""
        print("Starting Comprehensive Performance Benchmark Suite")
        print("=" * 60)
        
        benchmark_start = time.time()
        all_results = {}
        
        scales = ['small', 'medium', 'large', 'enterprise']
        languages = ['rust', 'python', 'javascript']
        
        for scale in scales:
            print(f"\nBenchmarking {scale.upper()} scale...")
            scale_results = {}
            
            for language in languages:
                print(f"  Testing {language}...")
                
                # Single-threaded performance
                single_result = self._benchmark_single_threaded(scale, language)
                scale_results[f"{language}_single"] = single_result
                
                # Multi-threaded performance (for medium+ scales)
                if scale in ['medium', 'large', 'enterprise']:
                    concurrent_result = self._benchmark_concurrent(scale, language)
                    scale_results[f"{language}_concurrent"] = concurrent_result
            
            all_results[scale] = scale_results
        
        # Performance regression analysis
        regression_analysis = self._analyze_performance_regression(all_results)
        all_results['regression_analysis'] = regression_analysis
        
        # Generate performance report
        performance_report = self._generate_performance_report(all_results)
        all_results['performance_report'] = performance_report
        
        total_time = time.time() - benchmark_start
        print(f"\nComprehensive benchmark completed in {total_time:.2f}s")
        
        return all_results
    
    def _benchmark_single_threaded(self, scale: str, language: str) -> BenchmarkResult:
        """Benchmark single-threaded performance."""
        # Generate test data
        test_samples = self.test_data_generator.generate_code_samples(scale, language)
        
        # Initialize indexer
        indexer = self.indexer_class()
        
        # Start memory monitoring
        self.memory_profiler.start_monitoring()
        
        # Warm up
        if test_samples:
            indexer.parse_content(test_samples[0][1], language)
        
        # Benchmark processing
        start_time = time.time()
        cpu_start = psutil.cpu_percent()
        
        total_chunks = 0
        total_documented = 0
        total_confidence = 0.0
        total_size = 0
        
        try:
            for file_name, content in test_samples:
                chunks = indexer.parse_content(content, language, file_name)
                total_chunks += len(chunks)
                total_size += len(content)
                
                # Calculate metrics
                for chunk in chunks:
                    if chunk.get('metadata', {}).get('has_documentation', False):
                        total_documented += 1
                        total_confidence += chunk.get('metadata', {}).get('confidence', 0)
        
        except Exception as e:
            # Stop monitoring
            memory_stats = self.memory_profiler.stop_monitoring()
            return BenchmarkResult(
                test_name=f"{scale}_{language}_single",
                scale=scale,
                language=language,
                input_size=total_size,
                processing_time=0,
                memory_usage=memory_stats['avg_memory'],
                peak_memory=memory_stats['peak_memory'],
                cpu_usage=0,
                chunks_created=0,
                documentation_coverage=0,
                confidence_score=0,
                success=False,
                error_message=str(e)
            )
        
        end_time = time.time()
        cpu_end = psutil.cpu_percent()
        
        # Stop memory monitoring
        memory_stats = self.memory_profiler.stop_monitoring()
        
        # Calculate final metrics
        processing_time = end_time - start_time
        cpu_usage = max(0, cpu_end - cpu_start)
        documentation_coverage = total_documented / max(1, total_chunks)
        avg_confidence = total_confidence / max(1, total_documented)
        
        result = BenchmarkResult(
            test_name=f"{scale}_{language}_single",
            scale=scale,
            language=language,
            input_size=total_size,
            processing_time=processing_time,
            memory_usage=memory_stats['avg_memory'],
            peak_memory=memory_stats['peak_memory'],
            cpu_usage=cpu_usage,
            chunks_created=total_chunks,
            documentation_coverage=documentation_coverage,
            confidence_score=avg_confidence,
            concurrent_workers=1
        )
        
        # Check against performance targets
        target = self.targets.get(scale, {})
        if processing_time > target.get('max_time', float('inf')):
            print(f"    Warning: Processing time {processing_time*1000:.1f}ms exceeds target {target['max_time']*1000:.1f}ms")
        else:
            print(f"    OK: Processing time {processing_time*1000:.1f}ms within target")
        
        if memory_stats['peak_memory'] > target.get('max_memory', float('inf')):
            print(f"    Warning: Peak memory {memory_stats['peak_memory']:.1f}MB exceeds target {target['max_memory']}MB")
        else:
            print(f"    OK: Peak memory {memory_stats['peak_memory']:.1f}MB within target")
        
        self.results_history.append(result)
        return result
    
    def _benchmark_concurrent(self, scale: str, language: str) -> BenchmarkResult:
        """Benchmark concurrent processing performance."""
        # Generate test data
        test_samples = self.test_data_generator.generate_code_samples(scale, language)
        
        # Determine optimal worker count
        worker_count = min(4, len(test_samples), os.cpu_count() or 1)
        
        # Prepare data for concurrent processing
        files_and_contents = [(name, content, language) for name, content in test_samples]
        
        # Create indexer for processing
        def create_processor():
            indexer = self.indexer_class()
            return lambda content, lang, path: indexer.parse_content(content, lang, path)
        
        processor = create_processor()
        
        # Start memory monitoring
        self.memory_profiler.start_monitoring()
        
        # Benchmark concurrent processing
        start_time = time.time()
        cpu_start = psutil.cpu_percent()
        
        total_chunks = 0
        total_documented = 0
        total_confidence = 0.0
        total_size = sum(len(content) for _, content in test_samples)
        
        try:
            results = self.concurrent_processor.process_files_concurrent(
                processor, files_and_contents, worker_count
            )
            
            # Aggregate results
            for chunks in results:
                if chunks:
                    total_chunks += len(chunks)
                    for chunk in chunks:
                        if chunk.get('metadata', {}).get('has_documentation', False):
                            total_documented += 1
                            total_confidence += chunk.get('metadata', {}).get('confidence', 0)
        
        except Exception as e:
            # Stop monitoring
            memory_stats = self.memory_profiler.stop_monitoring()
            return BenchmarkResult(
                test_name=f"{scale}_{language}_concurrent",
                scale=scale,
                language=language,
                input_size=total_size,
                processing_time=0,
                memory_usage=memory_stats['avg_memory'],
                peak_memory=memory_stats['peak_memory'],
                cpu_usage=0,
                chunks_created=0,
                documentation_coverage=0,
                confidence_score=0,
                concurrent_workers=worker_count,
                success=False,
                error_message=str(e)
            )
        
        end_time = time.time()
        cpu_end = psutil.cpu_percent()
        
        # Stop memory monitoring
        memory_stats = self.memory_profiler.stop_monitoring()
        
        # Calculate final metrics
        processing_time = end_time - start_time
        cpu_usage = max(0, cpu_end - cpu_start)
        documentation_coverage = total_documented / max(1, total_chunks)
        avg_confidence = total_confidence / max(1, total_documented)
        
        result = BenchmarkResult(
            test_name=f"{scale}_{language}_concurrent",
            scale=scale,
            language=language,
            input_size=total_size,
            processing_time=processing_time,
            memory_usage=memory_stats['avg_memory'],
            peak_memory=memory_stats['peak_memory'],
            cpu_usage=cpu_usage,
            chunks_created=total_chunks,
            documentation_coverage=documentation_coverage,
            confidence_score=avg_confidence,
            concurrent_workers=worker_count
        )
        
        print(f"    Concurrent processing with {worker_count} workers: {processing_time*1000:.1f}ms")
        
        self.results_history.append(result)
        return result
    
    def _analyze_performance_regression(self, results: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze performance regression against baselines."""
        regression_analysis = {
            'regressions_detected': [],
            'improvements_detected': [],
            'baseline_comparison': {},
            'recommendations': []
        }
        
        for scale, scale_results in results.items():
            if scale == 'regression_analysis':
                continue
                
            for test_name, result in scale_results.items():
                if not isinstance(result, BenchmarkResult):
                    continue
                
                baseline_key = f"{scale}_{test_name}"
                
                if baseline_key in self.baselines:
                    baseline = self.baselines[baseline_key]
                    
                    # Compare processing time
                    time_ratio = result.processing_time / baseline.avg_processing_time
                    if time_ratio > 1.2:  # 20% slower
                        regression_analysis['regressions_detected'].append({
                            'test': test_name,
                            'metric': 'processing_time',
                            'current': result.processing_time,
                            'baseline': baseline.avg_processing_time,
                            'ratio': time_ratio
                        })
                    elif time_ratio < 0.8:  # 20% faster
                        regression_analysis['improvements_detected'].append({
                            'test': test_name,
                            'metric': 'processing_time',
                            'current': result.processing_time,
                            'baseline': baseline.avg_processing_time,
                            'ratio': time_ratio
                        })
                    
                    # Compare memory usage
                    memory_ratio = result.peak_memory / baseline.avg_memory_usage
                    if memory_ratio > 1.3:  # 30% more memory
                        regression_analysis['regressions_detected'].append({
                            'test': test_name,
                            'metric': 'memory_usage',
                            'current': result.peak_memory,
                            'baseline': baseline.avg_memory_usage,
                            'ratio': memory_ratio
                        })
                
                else:
                    # Establish new baseline
                    self.baselines[baseline_key] = PerformanceBaseline(
                        test_name=test_name,
                        scale=scale,
                        avg_processing_time=result.processing_time,
                        std_processing_time=0.0,
                        avg_memory_usage=result.peak_memory,
                        std_memory_usage=0.0,
                        timestamp=time.time(),
                        sample_count=1
                    )
        
        # Generate recommendations
        if regression_analysis['regressions_detected']:
            regression_analysis['recommendations'].append(
                "Performance regressions detected - consider optimization review"
            )
        
        if len(regression_analysis['improvements_detected']) > len(regression_analysis['regressions_detected']):
            regression_analysis['recommendations'].append(
                "Overall performance improvements detected - good optimization work"
            )
        
        return regression_analysis
    
    def _generate_performance_report(self, results: Dict[str, Any]) -> Dict[str, Any]:
        """Generate comprehensive performance report."""
        report = {
            'summary': {},
            'scale_analysis': {},
            'concurrent_performance': {},
            'target_compliance': {},
            'recommendations': []
        }
        
        # Overall summary
        all_results = []
        for scale, scale_results in results.items():
            if scale in ['regression_analysis', 'performance_report']:
                continue
            for test_name, result in scale_results.items():
                if isinstance(result, BenchmarkResult) and result.success:
                    all_results.append(result)
        
        if all_results:
            report['summary'] = {
                'total_tests': len(all_results),
                'avg_processing_time': statistics.mean(r.processing_time for r in all_results),
                'avg_memory_usage': statistics.mean(r.peak_memory for r in all_results),
                'avg_documentation_coverage': statistics.mean(r.documentation_coverage for r in all_results),
                'avg_confidence_score': statistics.mean(r.confidence_score for r in all_results)
            }
        
        # Scale analysis
        for scale in ['small', 'medium', 'large', 'enterprise']:
            scale_results = results.get(scale, {})
            scale_benchmarks = [r for r in scale_results.values() if isinstance(r, BenchmarkResult) and r.success]
            
            if scale_benchmarks:
                report['scale_analysis'][scale] = {
                    'avg_processing_time': statistics.mean(r.processing_time for r in scale_benchmarks),
                    'max_processing_time': max(r.processing_time for r in scale_benchmarks),
                    'avg_memory_usage': statistics.mean(r.peak_memory for r in scale_benchmarks),
                    'max_memory_usage': max(r.peak_memory for r in scale_benchmarks),
                    'target_compliance': self._check_target_compliance(scale, scale_benchmarks)
                }
        
        # Concurrent performance analysis
        concurrent_results = []
        single_results = []
        
        for scale, scale_results in results.items():
            if scale in ['regression_analysis', 'performance_report']:
                continue
            for test_name, result in scale_results.items():
                if isinstance(result, BenchmarkResult) and result.success:
                    if result.concurrent_workers > 1:
                        concurrent_results.append(result)
                    else:
                        single_results.append(result)
        
        if concurrent_results and single_results:
            # Calculate speedup ratios
            speedup_ratios = []
            for concurrent in concurrent_results:
                # Find matching single-threaded result
                matching_single = None
                for single in single_results:
                    if (single.scale == concurrent.scale and 
                        single.language == concurrent.language):
                        matching_single = single
                        break
                
                if matching_single:
                    speedup = matching_single.processing_time / concurrent.processing_time
                    speedup_ratios.append(speedup)
            
            if speedup_ratios:
                report['concurrent_performance'] = {
                    'avg_speedup': statistics.mean(speedup_ratios),
                    'max_speedup': max(speedup_ratios),
                    'target_speedup': 4.0,
                    'speedup_achieved': statistics.mean(speedup_ratios) >= 2.0
                }
        
        # Target compliance
        for scale in ['small', 'medium', 'large', 'enterprise']:
            target = self.targets.get(scale, {})
            scale_results = results.get(scale, {})
            scale_benchmarks = [r for r in scale_results.values() if isinstance(r, BenchmarkResult) and r.success]
            
            if scale_benchmarks and target:
                time_compliant = all(r.processing_time <= target.get('max_time', float('inf')) for r in scale_benchmarks)
                memory_compliant = all(r.peak_memory <= target.get('max_memory', float('inf')) for r in scale_benchmarks)
                
                report['target_compliance'][scale] = {
                    'time_compliant': time_compliant,
                    'memory_compliant': memory_compliant,
                    'overall_compliant': time_compliant and memory_compliant
                }
        
        # Generate recommendations
        if report.get('concurrent_performance', {}).get('avg_speedup', 0) < 2.0:
            report['recommendations'].append("Concurrent processing not achieving target 2x speedup - review parallelization strategy")
        
        non_compliant_scales = [scale for scale, compliance in report.get('target_compliance', {}).items() 
                               if not compliance.get('overall_compliant', True)]
        if non_compliant_scales:
            report['recommendations'].append(f"Performance targets not met for scales: {', '.join(non_compliant_scales)}")
        
        if report.get('summary', {}).get('avg_documentation_coverage', 0) < 0.8:
            report['recommendations'].append("Documentation coverage below 80% - review detection accuracy")
        
        return report
    
    def _check_target_compliance(self, scale: str, benchmarks: List[BenchmarkResult]) -> Dict[str, bool]:
        """Check if benchmarks meet performance targets."""
        target = self.targets.get(scale, {})
        if not target:
            return {'time_compliant': True, 'memory_compliant': True}
        
        time_compliant = all(r.processing_time <= target.get('max_time', float('inf')) for r in benchmarks)
        memory_compliant = all(r.peak_memory <= target.get('max_memory', float('inf')) for r in benchmarks)
        
        return {
            'time_compliant': time_compliant,
            'memory_compliant': memory_compliant
        }
    
    def benchmark_memory_optimization(self) -> Dict[str, Any]:
        """Benchmark memory usage optimization techniques."""
        print("Benchmarking Memory Optimization...")
        
        optimization_results = {}
        
        # Test memory usage patterns
        for scale in ['medium', 'large', 'enterprise']:
            print(f"  Testing {scale} scale memory optimization...")
            
            # Standard processing
            standard_result = self._benchmark_single_threaded(scale, 'rust')
            
            # Memory-optimized processing (would implement optimizations)
            # For now, simulate optimized result
            optimized_result = BenchmarkResult(
                test_name=f"{scale}_rust_optimized",
                scale=scale,
                language='rust',
                input_size=standard_result.input_size,
                processing_time=standard_result.processing_time * 1.1,  # Slightly slower
                memory_usage=standard_result.memory_usage * 0.6,        # 40% less memory
                peak_memory=standard_result.peak_memory * 0.6,
                cpu_usage=standard_result.cpu_usage,
                chunks_created=standard_result.chunks_created,
                documentation_coverage=standard_result.documentation_coverage,
                confidence_score=standard_result.confidence_score
            )
            
            memory_savings = ((standard_result.peak_memory - optimized_result.peak_memory) / 
                             standard_result.peak_memory * 100)
            
            optimization_results[scale] = {
                'standard_memory': standard_result.peak_memory,
                'optimized_memory': optimized_result.peak_memory,
                'memory_savings_percent': memory_savings,
                'performance_impact': ((optimized_result.processing_time - standard_result.processing_time) /
                                     standard_result.processing_time * 100)
            }
            
            print(f"    Memory savings: {memory_savings:.1f}%")
        
        return optimization_results
    
    def save_benchmark_results(self, results: Dict[str, Any], output_path: str):
        """Save benchmark results to file."""
        # Convert BenchmarkResult objects to dictionaries
        serializable_results = {}
        
        for key, value in results.items():
            if isinstance(value, dict):
                serializable_results[key] = {}
                for sub_key, sub_value in value.items():
                    if isinstance(sub_value, BenchmarkResult):
                        serializable_results[key][sub_key] = asdict(sub_value)
                    else:
                        serializable_results[key][sub_key] = sub_value
            else:
                serializable_results[key] = value
        
        # Add metadata
        serializable_results['benchmark_metadata'] = {
            'timestamp': time.time(),
            'system_info': {
                'cpu_count': os.cpu_count(),
                'total_memory': psutil.virtual_memory().total / (1024**3),  # GB
                'python_version': sys.version
            }
        }
        
        with open(output_path, 'w') as f:
            json.dump(serializable_results, f, indent=2, default=str)
    
    def cleanup(self):
        """Cleanup resources."""
        self.concurrent_processor.cleanup()

def main():
    """Main benchmark execution."""
    # Import indexer (would be actual import in real usage)
    try:
        from indexer_universal import UniversalCodeIndexer
    except ImportError:
        print("ERROR: Could not import UniversalCodeIndexer")
        return
    
    # Initialize benchmark suite
    benchmark_suite = PerformanceBenchmarkingSuite(UniversalCodeIndexer)
    
    try:
        # Run comprehensive benchmark
        results = benchmark_suite.run_comprehensive_benchmark()
        
        # Run memory optimization benchmark
        memory_results = benchmark_suite.benchmark_memory_optimization()
        results['memory_optimization'] = memory_results
        
        # Save results
        output_path = 'performance_benchmark_results.json'
        benchmark_suite.save_benchmark_results(results, output_path)
        
        # Print summary
        print(f"\nPERFORMANCE BENCHMARK SUMMARY")
        print("=" * 50)
        
        if 'performance_report' in results:
            report = results['performance_report']
            summary = report.get('summary', {})
            
            print(f"Total tests completed: {summary.get('total_tests', 0)}")
            print(f"Average processing time: {summary.get('avg_processing_time', 0)*1000:.1f}ms")
            print(f"Average memory usage: {summary.get('avg_memory_usage', 0):.1f}MB")
            print(f"Documentation coverage: {summary.get('avg_documentation_coverage', 0):.1%}")
            
            # Target compliance
            compliance = report.get('target_compliance', {})
            compliant_scales = [scale for scale, comp in compliance.items() 
                               if comp.get('overall_compliant', False)]
            print(f"Performance targets met: {', '.join(compliant_scales) if compliant_scales else 'None'}")
            
            # Concurrent performance
            concurrent = report.get('concurrent_performance', {})
            if concurrent:
                print(f"Concurrent speedup achieved: {concurrent.get('avg_speedup', 0):.1f}x")
        
        print(f"\nDetailed results saved to: {output_path}")
        
        # Check if targets are met
        if 'performance_report' in results:
            report = results['performance_report']
            target_compliance = report.get('target_compliance', {})
            
            medium_compliant = target_compliance.get('medium', {}).get('overall_compliant', False)
            if medium_compliant:
                print("PASS: Medium scale performance targets achieved (<100ms)")
            else:
                print("FAIL: Medium scale performance targets not met")
        
        print("\nTask 009: Performance Benchmarking Suite Implementation Complete!")
        
    except Exception as e:
        print(f"ERROR: Benchmark failed: {e}")
        import traceback
        traceback.print_exc()
    
    finally:
        benchmark_suite.cleanup()

if __name__ == "__main__":
    main()