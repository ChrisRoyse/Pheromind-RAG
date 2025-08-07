// Level 3: Integration Tests
// Compile with: cargo test --features full-system --release
// Memory usage: ~2GB, Runtime: 5-15 minutes

#[cfg(feature = "test-integration")]
mod integration_tests {
    use embed_search::search::UnifiedSearcher;
    use std::path::PathBuf;
    use tempfile::TempDir;

    async fn create_test_environment() -> (UnifiedSearcher, TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_path_buf();
        let db_path = project_path.join("test.db");
        
        // Create test files with diverse content
        std::fs::write(project_path.join("main.rs"), r#"
            fn main() {
                println!("Hello, world!");
                let user = User::new("test");
                authenticate_user(&user);
            }
            
            fn authenticate_user(user: &User) {
                // Authentication logic here
                if user.is_valid() {
                    println!("User authenticated");
                }
            }
        "#).expect("Failed to write test file");
        
        std::fs::write(project_path.join("user.js"), r#"
            class User {
                constructor(name) {
                    this.name = name;
                    this.id = Math.random();
                }
                
                async authenticate() {
                    // Call authentication service
                    return await AuthService.verify(this);
                }
            }
            
            const AuthService = {
                verify: async (user) => {
                    return user.name !== 'invalid';
                }
            };
        "#).expect("Failed to write JS file");
        
        std::fs::write(project_path.join("database.py"), r#"
            import sqlite3
            
            class DatabaseConnection:
                def __init__(self, db_path):
                    self.connection = sqlite3.connect(db_path)
                
                def execute_query(self, query):
                    cursor = self.connection.cursor()
                    return cursor.execute(query)
                
                def get_user(self, user_id):
                    query = "SELECT * FROM users WHERE id = ?"
                    return self.execute_query(query, (user_id,))
        "#).expect("Failed to write Python file");
        
        let searcher = UnifiedSearcher::new_with_config(
            project_path.clone(),
            db_path,
            true // include test files
        ).await.expect("Failed to create searcher");
        
        (searcher, temp_dir, project_path)
    }

    #[tokio::test]
    async fn four_way_search_fusion() {
        let (searcher, _temp_dir, project_path) = create_test_environment().await;
        
        // Index the test files
        let stats = searcher.index_directory(&project_path).await
            .expect("Failed to index directory");
        
        assert!(stats.files_indexed >= 3);
        assert!(stats.chunks_created > 0);
        
        // Test multi-modal search
        let queries = vec![
            ("authenticate user", vec!["main.rs", "user.js"]),
            ("database connection", vec!["database.py"]),
            ("User class", vec!["user.js"]),
            ("execute query", vec!["database.py"]),
        ];
        
        for (query, expected_files) in queries {
            let results = searcher.search(query).await
                .expect("Search failed");
            
            assert!(!results.is_empty(), "No results for query: {}", query);
            
            // Check that at least one expected file is found
            let found_expected = expected_files.iter().any(|expected| {
                results.iter().any(|result| result.file.contains(expected))
            });
            
            assert!(found_expected, 
                "Query '{}' didn't find expected files {:?}. Found: {:?}", 
                query, expected_files, 
                results.iter().map(|r| &r.file).collect::<Vec<_>>()
            );
        }
    }

    #[tokio::test]
    async fn search_accuracy_measurement() {
        let (searcher, _temp_dir, project_path) = create_test_environment().await;
        
        // Index files
        searcher.index_directory(&project_path).await
            .expect("Failed to index");
        
        // Define ground truth with precision expectations
        let test_cases = vec![
            ("authentication logic", vec!["main.rs"], 0.8),
            ("User constructor", vec!["user.js"], 0.9),
            ("database query execution", vec!["database.py"], 0.8),
            ("async authenticate method", vec!["user.js"], 0.7),
        ];
        
        let mut total_precision = 0.0;
        
        for (query, expected_files, min_precision) in &test_cases {
            let results = searcher.search(query).await
                .expect("Search failed");
            
            let top_3: Vec<&str> = results.iter()
                .take(3)
                .map(|r| r.file.as_str())
                .collect();
            
            let found_count = expected_files.iter()
                .filter(|expected| top_3.iter().any(|result| result.contains(*expected)))
                .count();
            
            let precision = found_count as f64 / expected_files.len() as f64;
            total_precision += precision;
            
            assert!(precision >= *min_precision,
                "Query '{}' precision {:.2} below threshold {:.2}",
                query, precision, min_precision);
        }
        
        let average_precision = total_precision / test_cases.len() as f64;
        assert!(average_precision >= 0.75, "Overall precision too low: {:.2}", average_precision);
    }

    #[tokio::test]
    async fn performance_benchmarking() {
        let (searcher, _temp_dir, project_path) = create_test_environment().await;
        
        // Index files and measure time
        let index_start = std::time::Instant::now();
        let _stats = searcher.index_directory(&project_path).await
            .expect("Failed to index");
        let index_time = index_start.elapsed();
        
        // Should index quickly for small test set
        assert!(index_time.as_secs() < 5, "Indexing took too long: {:?}", index_time);
        
        // Test search latency
        let queries = vec!["user", "authenticate", "database", "query", "connection"];
        
        for query in queries {
            let search_start = std::time::Instant::now();
            let results = searcher.search(query).await.expect("Search failed");
            let search_time = search_start.elapsed();
            
            // Search should be fast
            assert!(search_time.as_millis() < 100, 
                "Query '{}' took {}ms (>100ms threshold)", query, search_time.as_millis());
            
            // Should return reasonable number of results
            assert!(results.len() <= 20, "Too many results returned: {}", results.len());
        }
    }

    #[tokio::test]
    async fn edge_cases_handling() {
        let (searcher, _temp_dir, _project_path) = create_test_environment().await;
        
        // Test empty queries
        let empty_results = searcher.search("").await.expect("Empty search failed");
        assert_eq!(empty_results.len(), 0, "Empty query should return no results");
        
        // Test whitespace queries
        let whitespace_results = searcher.search("   ").await.expect("Whitespace search failed");
        assert_eq!(whitespace_results.len(), 0, "Whitespace query should return no results");
        
        // Test very long query
        let long_query = "test ".repeat(100);
        let _long_results = searcher.search(&long_query).await.expect("Long search failed");
        // Should handle gracefully (not crash)
        
        // Test special characters
        let _special_results = searcher.search("@#$%^&*()").await.expect("Special char search failed");
        // Should handle gracefully
        
        // Test unicode
        let _unicode_results = searcher.search("🔍 search").await.expect("Unicode search failed");
        // Should handle gracefully
    }
}

#[cfg(not(feature = "test-integration"))]
mod integration_disabled {
    #[test]
    fn integration_tests_disabled() {
        println!("Integration tests skipped (test-integration feature not enabled)");
    }
}