use crate::search::bm25_fixed::{BM25Searcher, Document};
use std::collections::HashMap;

/// Reference BM25 IDF implementation using pure mathematical formula
/// IDF(t) = log((N - df + 0.5) / (df + 0.5))
/// Where N = total documents, df = document frequency of term t
fn reference_idf(total_docs: usize, doc_frequency: usize) -> f64 {
    let n = total_docs as f64;
    let df = doc_frequency as f64;
    
    // Standard BM25 IDF formula
    ((n - df + 0.5) / (df + 0.5)).ln()
}

/// Calculate reference IDF scores for all terms in a corpus
fn calculate_reference_idf_scores(documents: &[Document]) -> HashMap<String, f64> {
    let mut term_doc_counts: HashMap<String, usize> = HashMap::new();
    let total_docs = documents.len();
    
    // Count document frequency for each term
    for doc in documents {
        let mut doc_terms: std::collections::HashSet<String> = std::collections::HashSet::new();
        
        // Tokenize document content (simple whitespace splitting)
        for word in doc.content.split_whitespace() {
            let term = word.to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>();
            
            if !term.is_empty() {
                doc_terms.insert(term);
            }
        }
        
        // Count document frequency for each unique term in this document
        for term in doc_terms {
            *term_doc_counts.entry(term).or_insert(0) += 1;
        }
    }
    
    // Calculate IDF for each term
    let mut idf_scores = HashMap::new();
    for (term, doc_freq) in term_doc_counts {
        let idf = reference_idf(total_docs, doc_freq);
        idf_scores.insert(term, idf);
    }
    
    idf_scores
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparative_bm25_idf_simple_cases() {
        // Test case 1: Simple programming concepts
        let documents = vec![
            Document {
                id: 1,
                content: "function hello world".to_string(),
            },
            Document {
                id: 2,
                content: "function goodbye world".to_string(),
            },
            Document {
                id: 3,
                content: "class hello test".to_string(),
            },
        ];
        
        let searcher = BM25Searcher::new(documents.clone());
        let reference_scores = calculate_reference_idf_scores(&documents);
        
        // Compare IDF scores for each term
        for (term, expected_idf) in reference_scores {
            if let Some(actual_idf) = searcher.get_idf_score(&term) {
                let diff = (actual_idf - expected_idf).abs();
                assert!(
                    diff < 1e-6,
                    "IDF mismatch for term '{}': expected {:.10}, got {:.10}, diff: {:.2e}",
                    term, expected_idf, actual_idf, diff
                );
            } else {
                panic!("Term '{}' not found in searcher IDF scores", term);
            }
        }
    }

    #[test]
    fn test_comparative_bm25_idf_rust_code() {
        let documents = vec![
            Document {
                id: 1,
                content: "fn main() { println!(\"Hello, world!\"); }".to_string(),
            },
            Document {
                id: 2,
                content: "struct Point { x: i32, y: i32 }".to_string(),
            },
            Document {
                id: 3,
                content: "impl Point { fn new(x: i32, y: i32) -> Point { Point { x, y } } }".to_string(),
            },
            Document {
                id: 4,
                content: "fn calculate_distance(p1: Point, p2: Point) -> f64 { /* math */ }".to_string(),
            },
        ];
        
        let searcher = BM25Searcher::new(documents.clone());
        let reference_scores = calculate_reference_idf_scores(&documents);
        
        // Verify critical Rust terms
        let critical_terms = ["fn", "struct", "impl", "point"];
        
        for term in critical_terms {
            if let Some(expected_idf) = reference_scores.get(term) {
                if let Some(actual_idf) = searcher.get_idf_score(term) {
                    let diff = (actual_idf - expected_idf).abs();
                    assert!(
                        diff < 1e-6,
                        "Critical Rust term '{}' IDF mismatch: expected {:.10}, got {:.10}",
                        term, expected_idf, actual_idf
                    );
                }
            }
        }
    }

    #[test]
    fn test_comparative_bm25_idf_javascript_code() {
        let documents = vec![
            Document {
                id: 1,
                content: "function fetchData() { return fetch('/api/data'); }".to_string(),
            },
            Document {
                id: 2,
                content: "const users = []; function addUser(user) { users.push(user); }".to_string(),
            },
            Document {
                id: 3,
                content: "class UserService { constructor() { this.users = []; } }".to_string(),
            },
            Document {
                id: 4,
                content: "async function processUsers() { for (const user of users) { await processUser(user); } }".to_string(),
            },
        ];
        
        let searcher = BM25Searcher::new(documents.clone());
        let reference_scores = calculate_reference_idf_scores(&documents);
        
        // Compare all calculated IDF scores
        for (term, expected_idf) in reference_scores {
            if let Some(actual_idf) = searcher.get_idf_score(&term) {
                let diff = (actual_idf - expected_idf).abs();
                assert!(
                    diff < 1e-6,
                    "JavaScript term '{}' IDF mismatch: expected {:.10}, got {:.10}, diff: {:.2e}",
                    term, expected_idf, actual_idf, diff
                );
            }
        }
    }

    #[test]
    fn test_comparative_bm25_idf_python_code() {
        let documents = vec![
            Document {
                id: 1,
                content: "def calculate_sum(numbers): return sum(numbers)".to_string(),
            },
            Document {
                id: 2,
                content: "class DataProcessor: def __init__(self): self.data = []".to_string(),
            },
            Document {
                id: 3,
                content: "import pandas as pd; df = pd.DataFrame(data)".to_string(),
            },
            Document {
                id: 4,
                content: "for item in data: process_item(item)".to_string(),
            },
            Document {
                id: 5,
                content: "def process_item(item): print(f'Processing {item}')".to_string(),
            },
        ];
        
        let searcher = BM25Searcher::new(documents.clone());
        let reference_scores = calculate_reference_idf_scores(&documents);
        
        // Verify Python-specific terms
        let python_terms = ["def", "class", "import", "self"];
        
        for term in python_terms {
            if let Some(expected_idf) = reference_scores.get(term) {
                if let Some(actual_idf) = searcher.get_idf_score(term) {
                    let diff = (actual_idf - expected_idf).abs();
                    assert!(
                        diff < 1e-6,
                        "Python term '{}' IDF mismatch: expected {:.10}, got {:.10}",
                        term, expected_idf, actual_idf
                    );
                }
            }
        }
    }

    #[test]
    fn test_comparative_bm25_idf_mixed_content() {
        let documents = vec![
            Document {
                id: 1,
                content: "The quick brown fox jumps over the lazy dog".to_string(),
            },
            Document {
                id: 2,
                content: "Rust is a systems programming language that runs blazingly fast".to_string(),
            },
            Document {
                id: 3,
                content: "Machine learning algorithms can process large datasets efficiently".to_string(),
            },
            Document {
                id: 4,
                content: "Web development frameworks like React and Vue are popular".to_string(),
            },
            Document {
                id: 5,
                content: "Database queries using SQL can retrieve specific data records".to_string(),
            },
        ];
        
        let searcher = BM25Searcher::new(documents.clone());
        let reference_scores = calculate_reference_idf_scores(&documents);
        
        // Test comprehensive comparison
        let mut compared_terms = 0;
        for (term, expected_idf) in reference_scores {
            if let Some(actual_idf) = searcher.get_idf_score(&term) {
                let diff = (actual_idf - expected_idf).abs();
                assert!(
                    diff < 1e-6,
                    "Mixed content term '{}' IDF mismatch: expected {:.10}, got {:.10}, diff: {:.2e}",
                    term, expected_idf, actual_idf, diff
                );
                compared_terms += 1;
            }
        }
        
        // Ensure we compared a reasonable number of terms
        assert!(compared_terms > 15, "Should have compared more terms, only got {}", compared_terms);
    }

    #[test]
    fn test_comparative_bm25_idf_edge_cases() {
        // Test case 1: All documents contain the same term
        let documents_all_same = vec![
            Document { id: 1, content: "common word".to_string() },
            Document { id: 2, content: "common word".to_string() },
            Document { id: 3, content: "common word".to_string() },
        ];
        
        let searcher = BM25Searcher::new(documents_all_same.clone());
        let reference_scores = calculate_reference_idf_scores(&documents_all_same);
        
        for (term, expected_idf) in reference_scores {
            if let Some(actual_idf) = searcher.get_idf_score(&term) {
                let diff = (actual_idf - expected_idf).abs();
                assert!(
                    diff < 1e-6,
                    "All-same term '{}' IDF mismatch: expected {:.10}, got {:.10}",
                    term, expected_idf, actual_idf
                );
            }
        }
        
        // Test case 2: Single document
        let single_doc = vec![
            Document { id: 1, content: "unique single document content".to_string() },
        ];
        
        let searcher_single = BM25Searcher::new(single_doc.clone());
        let reference_single = calculate_reference_idf_scores(&single_doc);
        
        for (term, expected_idf) in reference_single {
            if let Some(actual_idf) = searcher_single.get_idf_score(&term) {
                let diff = (actual_idf - expected_idf).abs();
                assert!(
                    diff < 1e-6,
                    "Single doc term '{}' IDF mismatch: expected {:.10}, got {:.10}",
                    term, expected_idf, actual_idf
                );
            }
        }
    }

    #[test]
    fn test_comparative_bm25_idf_large_corpus() {
        let mut documents = Vec::new();
        
        // Generate a larger corpus with varied content
        let content_templates = [
            "function process data with algorithm",
            "class implements interface for service",
            "method returns result from computation",
            "variable stores value for calculation",
            "object contains properties and methods",
            "array holds multiple elements efficiently",
            "string represents text data structure",
            "number performs mathematical operations",
            "boolean indicates true or false state",
            "loop iterates through collection items",
        ];
        
        for i in 0..20 {
            documents.push(Document {
                id: i + 1,
                content: format!("{} iteration {}", content_templates[i % content_templates.len()], i),
            });
        }
        
        let searcher = BM25Searcher::new(documents.clone());
        let reference_scores = calculate_reference_idf_scores(&documents);
        
        let mut verified_terms = 0;
        for (term, expected_idf) in reference_scores {
            if let Some(actual_idf) = searcher.get_idf_score(&term) {
                let diff = (actual_idf - expected_idf).abs();
                assert!(
                    diff < 1e-6,
                    "Large corpus term '{}' IDF mismatch: expected {:.10}, got {:.10}, diff: {:.2e}",
                    term, expected_idf, actual_idf, diff
                );
                verified_terms += 1;
            }
        }
        
        // Should verify a substantial number of unique terms
        assert!(verified_terms > 25, "Large corpus should have more unique terms, got {}", verified_terms);
    }

    #[test]
    fn test_idf_mathematical_properties() {
        let documents = vec![
            Document { id: 1, content: "rare unique special".to_string() },
            Document { id: 2, content: "common frequent common".to_string() },
            Document { id: 3, content: "common frequent common".to_string() },
            Document { id: 4, content: "common frequent common".to_string() },
        ];
        
        let searcher = BM25Searcher::new(documents.clone());
        
        // Rare terms should have higher IDF than common terms
        let rare_idf = searcher.get_idf_score("rare").expect("Should find 'rare'");
        let common_idf = searcher.get_idf_score("common").expect("Should find 'common'");
        
        assert!(
            rare_idf > common_idf,
            "Rare terms should have higher IDF than common terms: rare={:.6} vs common={:.6}",
            rare_idf, common_idf
        );
        
        // Verify mathematical bounds
        assert!(rare_idf > 0.0, "IDF for rare terms should be positive");
        // Note: common_idf might be negative for very frequent terms, which is expected in BM25
    }

    #[test]
    fn test_idf_consistency_across_queries() {
        let documents = vec![
            Document { id: 1, content: "consistent term appears here".to_string() },
            Document { id: 2, content: "another document with different words".to_string() },
            Document { id: 3, content: "consistent term appears again".to_string() },
        ];
        
        let searcher = BM25Searcher::new(documents);
        
        // IDF should be consistent regardless of how we access it
        let idf1 = searcher.get_idf_score("consistent").expect("Should find term");
        let idf2 = searcher.get_idf_score("consistent").expect("Should find term again");
        
        assert_eq!(idf1, idf2, "IDF should be consistent across multiple accesses");
        
        // Verify the IDF value makes mathematical sense
        // df=2 (appears in 2 docs), N=3 total docs
        // IDF = ln((3-2+0.5)/(2+0.5)) = ln(1.5/2.5) = ln(0.6) ≈ -0.51
        let expected_idf = ((3.0_f64 - 2.0 + 0.5) / (2.0 + 0.5)).ln();
        let diff = (idf1 - expected_idf).abs();
        
        assert!(
            diff < 1e-6,
            "Manual calculation verification failed: expected {:.10}, got {:.10}",
            expected_idf, idf1
        );
    }
}