# Phase 1: Baseline & Evaluation Foundation

## **PHASE OVERVIEW - EVIDENCE-BASED DEVELOPMENT**

**GOAL**: Establish measurable baseline and comprehensive evaluation framework  
**APPROACH**: Build exact text search baseline with robust accuracy measurement  
**MEASUREMENT**: Create ground truth dataset and success metrics before optimization  
**TIMELINE**: 4 weeks (Tasks 001-015)

## **KEY INSIGHT: MEASUREMENT BEFORE OPTIMIZATION**

**PROBLEM**: Cannot improve what cannot be measured accurately  
**SOLUTION**: Build comprehensive evaluation framework before any optimization

**Foundation Requirements**:
- **Baseline System**: High-quality exact text search using proven tools
- **Ground Truth Dataset**: 500+ real developer queries with known correct results
- **Success Metrics**: Clear definition of search success (relevant result in top 5)
- **A/B Testing Framework**: Compare improvements against baseline objectively

## **EVIDENCE-BASED TASK BREAKDOWN (001-015)**

### **Baseline System Tasks (001-005): Exact Text Search Foundation**

#### **Task 001: High-Quality Exact Text Search**
**Goal**: Implement production-quality exact text search using ripgrep  
**Duration**: 4 hours (realistic)  
**Dependencies**: None

**TDD Cycle**:
1. **RED Phase**: Test exact text search fails to find known code patterns
2. **GREEN Phase**: Implement ripgrep-based exact search with file filtering
3. **REFACTOR Phase**: Add result ranking by relevance and file importance

```rust
pub struct ExactTextSearch {
    ripgrep_engine: RipgrepSearch,
    file_filter: FileTypeFilter,
    result_ranker: SimpleRanking,
}

impl ExactTextSearch {
    pub fn search(&self, query: &str, path: &Path) -> Result<Vec<SearchResult>> {
        // Use ripgrep for exact text matching
        let raw_results = self.ripgrep_engine.search(query, path)?;
        
        // Filter by relevant file types
        let filtered = self.file_filter.filter_code_files(raw_results);
        
        // Simple ranking by match quality
        let ranked = self.result_ranker.rank_by_relevance(filtered);
        
        Ok(ranked)
    }
}

```

#### **Task 002: Simple Query Processing**  
**Goal**: Basic query cleaning and keyword extraction for better exact search  
**Duration**: 3 hours  
**Dependencies**: Task 001

**TDD Cycle**:
1. **RED Phase**: Test raw user queries don't work well with exact search
2. **GREEN Phase**: Basic query cleaning (remove stop words, extract keywords)
3. **REFACTOR Phase**: Smart query expansion with programming synonyms

```rust
pub struct SimpleQueryProcessor {
    stop_words: HashSet<String>,
    programming_synonyms: HashMap<String, Vec<String>>,
}

impl SimpleQueryProcessor {
    pub fn process(&self, raw_query: &str) -> ProcessedQuery {
        // Clean and normalize query
        let cleaned = self.remove_stop_words(raw_query);
        let keywords = self.extract_keywords(&cleaned);
        
        // Add programming-specific synonyms
        let expanded = self.add_synonyms(&keywords);
        
        ProcessedQuery {
            original: raw_query.to_string(),
            keywords,
            expanded_terms: expanded,
            search_terms: self.build_search_terms(&keywords, &expanded),
        }
    }
}
```

#### **Task 003: File Type and Language Detection**
**Goal**: Detect programming languages and prioritize relevant files  
**Duration**: 2 hours  
**Dependencies**: Task 002

**TDD Cycle**:
1. **RED Phase**: Test search doesn't prioritize relevant file types
2. **GREEN Phase**: Simple file extension detection and filtering
3. **REFACTOR Phase**: Project language detection for better file prioritization

```rust
pub struct FileTypeDetector {
    language_extensions: HashMap<String, Language>,
    priority_files: HashSet<String>,
}

impl FileTypeDetector {
    pub fn detect_project_languages(&self, root_path: &Path) -> Vec<Language> {
        // Count files by language
        let file_counts = self.count_files_by_language(root_path);
        
        // Return languages sorted by prevalence
        file_counts.into_iter()
            .sorted_by(|a, b| b.1.cmp(&a.1))
            .map(|(lang, _)| lang)
            .collect()
    }
    
    pub fn should_search_file(&self, file_path: &Path, languages: &[Language]) -> bool {
        if let Some(ext) = file_path.extension() {
            if let Some(lang) = self.language_extensions.get(ext.to_str().unwrap()) {
                return languages.contains(lang);
            }
        }
        false
    }
}
```

#### **Task 004: Result Ranking and Scoring**
**Goal**: Rank exact search results by relevance and file importance  
**Duration**: 3 hours  
**Dependencies**: Task 003

**TDD Cycle**:
1. **RED Phase**: Test search results aren't ranked by relevance
2. **GREEN Phase**: Simple scoring based on exact matches and file importance
3. **REFACTOR Phase**: Multi-factor ranking (match quality + file type + context)

```rust
pub struct SearchResultRanker {
    file_importance: FileImportanceCalculator,
    match_quality: MatchQualityScorer,
}

impl SearchResultRanker {
    pub fn rank(&self, results: Vec<RawSearchResult>) -> Vec<RankedSearchResult> {
        results.into_iter()
            .map(|result| self.score_result(result))
            .sorted_by(|a, b| b.total_score.partial_cmp(&a.total_score).unwrap())
            .collect()
    }
    
    fn score_result(&self, result: RawSearchResult) -> RankedSearchResult {
        let match_score = self.match_quality.score(&result);
        let file_score = self.file_importance.score(&result.file_path);
        let context_score = self.calculate_context_relevance(&result);
        
        RankedSearchResult {
            result,
            total_score: match_score * 0.5 + file_score * 0.3 + context_score * 0.2,
            match_score,
            file_score,
            context_score,
        }
    }
}
```

#### **Task 005: Baseline System Integration**
**Goal**: Integrate all baseline components into working search system  
**Duration**: 2 hours  
**Dependencies**: Task 004

### **Evaluation Framework Tasks (006-010): Measurement Infrastructure**

#### **Task 006: Ground Truth Dataset Creation**
**Goal**: Create 500+ real developer queries with known correct results  
**Duration**: 8 hours (manual work required)  
**Dependencies**: Task 005

**TDD Cycle**:
1. **RED Phase**: Test evaluation system has no ground truth data
2. **GREEN Phase**: Manual creation of 100 query-result pairs for common patterns
3. **REFACTOR Phase**: Expand to 500+ queries covering diverse search scenarios

#### **Task 007: Accuracy Measurement Framework**
**Goal**: Implement objective accuracy measurement against ground truth  
**Duration**: 4 hours  
**Dependencies**: Task 006

#### **Task 008: A/B Testing Infrastructure**
**Goal**: Framework to compare different search approaches objectively  
**Duration**: 3 hours  
**Dependencies**: Task 007

#### **Task 009: Performance Benchmarking**
**Goal**: Measure baseline system latency and resource usage  
**Duration**: 2 hours  
**Dependencies**: Task 008

#### **Task 010: Success Metrics Definition**
**Goal**: Define clear success criteria (e.g., relevant result in top 5)  
**Duration**: 1 hour  
**Dependencies**: Task 009

#### **Task 011: Baseline Accuracy Measurement**
**Goal**: Measure baseline exact search accuracy against ground truth  
**Duration**: 2 hours  
**Dependencies**: Task 010

#### **Task 012: Performance Validation**
**Goal**: Validate baseline meets performance requirements  
**Duration**: 2 hours  
**Dependencies**: Task 011

#### **Task 013: Error Analysis**
**Goal**: Analyze where baseline fails and why  
**Duration**: 3 hours  
**Dependencies**: Task 012

#### **Task 014: Documentation and Reporting**
**Goal**: Document baseline performance and create improvement roadmap  
**Duration**: 2 hours  
**Dependencies**: Task 013

#### **Task 015: Phase 1 Validation**
**Goal**: Final validation that baseline foundation is solid for Phase 2  
**Duration**: 1 hour  
**Dependencies**: Task 014


## **SUCCESS CRITERIA**

### **Phase 1 Targets**
- **Baseline System**: Production-quality exact text search working
- **Ground Truth Dataset**: 500+ validated query-result pairs
- **Evaluation Framework**: Objective accuracy measurement system
- **Performance**: <200ms search latency, <1GB memory usage
- **Documentation**: Clear baseline accuracy and failure modes documented

### **Integration Points**
- **Phase 2 Input**: Solid baseline system with evaluation framework
- **Comparison**: All Phase 2 improvements measured against Phase 1 baseline
- **Foundation**: Proven infrastructure for adding semantic search

## **ARCHITECTURE**

```rust
pub struct BaselineSearchSystem {
    // Core search components
    text_search: ExactTextSearch,
    query_processor: SimpleQueryProcessor,
    file_detector: FileTypeDetector,
    result_ranker: SearchResultRanker,
    
    // Evaluation infrastructure
    evaluation_framework: AccuracyMeasurement,
    performance_monitor: PerformanceMonitor,
    ab_testing: ABTestingFramework,
}

pub struct BaselineSearchResult {
    pub query: String,
    pub results: Vec<RankedSearchResult>,
    pub total_found: usize,
    pub search_time: Duration,
    pub accuracy_score: Option<f32>, // Only available if ground truth exists
}

impl BaselineSearchSystem {
    pub fn search(&self, query: &str, project_path: &Path) -> BaselineSearchResult {
        let start_time = Instant::now();
        
        // 1. Process query (simple cleaning and expansion)
        let processed = self.query_processor.process(query);
        
        // 2. Detect relevant file types for project
        let languages = self.file_detector.detect_project_languages(project_path);
        
        // 3. Execute exact text search
        let raw_results = self.text_search.search(&processed, project_path, &languages)?;
        
        // 4. Rank results by relevance
        let ranked_results = self.result_ranker.rank(raw_results);
        
        // 5. Measure accuracy if ground truth available
        let accuracy = self.evaluation_framework.measure_accuracy(query, &ranked_results);
        
        BaselineSearchResult {
            query: query.to_string(),
            results: ranked_results,
            total_found: ranked_results.len(),
            search_time: start_time.elapsed(),
            accuracy_score: accuracy,
        }
    }
}
```

## **OPTIMIZATION RESULTS**

**BEFORE (Complex Query Intelligence)**:
- 25 tasks for advanced ML-based query understanding
- Unproven accuracy improvements with high complexity
- Heavy dependencies on training data and ML expertise
- High maintenance overhead

**AFTER (Baseline & Evaluation Foundation)**:
- 15 focused tasks for measurement and proven search
- Solid foundation with objective evaluation framework
- Simple, proven technologies with known performance
- Low maintenance overhead, high reliability

**Result**: Evidence-based development foundation that enables accurate measurement of all future improvements.