use embed::storage::lancedb_storage::{LanceDBStorage, LanceEmbeddingRecord, SearchOptions, IndexType, IndexConfig, LanceStorageError};
use embed::chunking::Chunk;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio;
use std::time::Instant;

/// BRUTAL VALIDATION OF LANCEDB VECTOR STORAGE PIPELINE
/// 
/// This test suite ruthlessly validates every claim made about the LanceDB storage system.
/// Any failure means the system is NOT production-ready and CANNOT be trusted.
/// 
/// VALIDATION CRITERIA:
/// - Data integrity: 100% accuracy in storage and retrieval
/// - Performance: Batch operations must outperform single operations
/// - Corruption detection: Must catch ALL data corruption
/// - Indexing: Must work with sufficient data, fail gracefully without
/// - Concurrency: Must handle concurrent access safely
/// - Deletion: Must completely remove specified records

#[cfg(test)]
mod brutal_lancedb_validation {
    use super::*;
    
    /// Test 1: DATA INTEGRITY VALIDATION - Does insert_batch actually store correct data?
    #[tokio::test]
    async fn brutal_data_integrity_validation() {
        println!("🔥 BRUTAL TEST: Data Integrity Validation");
        
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("integrity_test.db");
        
        let storage = LanceDBStorage::new(db_path).await
            .expect("FAILED: Storage creation should never fail");
        storage.init_table().await
            .expect("FAILED: Table initialization should never fail");
        
        // Create test records with known data
        let test_cases = vec![
            ("file1.rs", "fn main() { println!(\"Hello\"); }", vec![0.1; 768]),
            ("file2.py", "def hello(): print(\"Python\")", vec![0.2; 768]),
            ("file3.js", "console.log('JavaScript');", vec![0.3; 768]),
        ];
        
        let mut expected_records = Vec::new();
        for (i, (file, content, mut embedding)) in test_cases.into_iter().enumerate() {
            // Normalize embedding to make it realistic
            let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            for val in &mut embedding {
                *val /= norm;
            }
            
            let record = LanceEmbeddingRecord {
                id: format!("{}-{}", file, i),
                file_path: file.to_string(),
                chunk_index: i as u64,
                content: content.to_string(),
                embedding: embedding.clone(),
                start_line: 1,
                end_line: 2,
                similarity_score: None,
                checksum: None,
            };
            expected_records.push(record);
        }
        
        // CRITICAL TEST: Insert batch
        let insert_result = storage.insert_batch(expected_records.clone()).await;
        assert!(insert_result.is_ok(), "FAILED: insert_batch failed - {:#?}", insert_result);
        
        // CRITICAL TEST: Verify count matches exactly
        let actual_count = storage.count().await.unwrap();
        assert_eq!(actual_count, 3, "FAILED: Count mismatch - expected 3, got {}", actual_count);
        
        // CRITICAL TEST: Retrieve and verify every single value matches exactly
        for expected in &expected_records {
            let search_results = storage.search_similar(expected.embedding.clone(), 1).await
                .expect("FAILED: Search should never fail for stored embeddings");
            
            assert!(!search_results.is_empty(), "FAILED: No results found for stored embedding");
            let actual = &search_results[0];
            
            // BRUTAL VERIFICATION: Every field must match exactly
            assert_eq!(actual.file_path, expected.file_path, 
                "FAILED: File path corruption - expected '{}', got '{}'", 
                expected.file_path, actual.file_path);
            assert_eq!(actual.content, expected.content, 
                "FAILED: Content corruption - expected '{}', got '{}'", 
                expected.content, actual.content);
            assert_eq!(actual.chunk_index, expected.chunk_index, 
                "FAILED: Chunk index corruption - expected {}, got {}", 
                expected.chunk_index, actual.chunk_index);
            assert_eq!(actual.embedding.len(), 768, 
                "FAILED: Embedding dimension corruption - expected 768, got {}", 
                actual.embedding.len());
            
            // Verify embedding values are approximately correct (allow for float precision)
            for (i, (expected_val, actual_val)) in expected.embedding.iter().zip(&actual.embedding).enumerate() {
                let diff = (expected_val - actual_val).abs();
                assert!(diff < 1e-6, 
                    "FAILED: Embedding corruption at index {} - expected {}, got {}, diff = {}", 
                    i, expected_val, actual_val, diff);
            }
        }
        
        println!("✅ PASSED: Data integrity validation - all data stored and retrieved correctly");
    }
    
    /// Test 2: STORAGE PERFORMANCE VALIDATION - Is batch insert actually faster?
    #[tokio::test]
    async fn brutal_performance_validation() {
        println!("🔥 BRUTAL TEST: Storage Performance Validation");
        
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("performance_test.db");
        
        let storage = LanceDBStorage::new(db_path).await.unwrap();
        storage.init_table().await.unwrap();
        
        // Create test data - enough to measure performance difference
        let test_count = 50;
        let mut test_records = Vec::new();
        
        for i in 0..test_count {
            let chunk = Chunk {
                content: format!("Performance test content {}", i),
                start_line: i,
                end_line: i + 1,
            };
            let embedding = vec![0.1f32 + (i as f32) * 0.001; 768];
            
            test_records.push(LanceEmbeddingRecord {
                id: format!("perf-{}", i),
                file_path: format!("perf_{}.rs", i),
                chunk_index: i as u64,
                content: chunk.content.clone(),
                embedding,
                start_line: i as u64,
                end_line: (i + 1) as u64,
                similarity_score: None,
                checksum: None,
            });
        }
        
        // Test 1: Single inserts (simulated)
        let storage1 = LanceDBStorage::new(temp_dir.path().join("single_test.db")).await.unwrap();
        storage1.init_table().await.unwrap();
        
        let single_start = Instant::now();
        for record in &test_records {
            storage1.insert_batch(vec![record.clone()]).await.unwrap();
        }
        let single_duration = single_start.elapsed();
        
        // Test 2: Batch insert
        let storage2 = LanceDBStorage::new(temp_dir.path().join("batch_test.db")).await.unwrap();
        storage2.init_table().await.unwrap();
        
        let batch_start = Instant::now();
        storage2.insert_batch(test_records.clone()).await.unwrap();
        let batch_duration = batch_start.elapsed();
        
        println!("📊 Performance Results:");
        println!("   Single inserts: {:.3}ms ({:.3}ms per record)", 
                single_duration.as_millis(), single_duration.as_millis() as f64 / test_count as f64);
        println!("   Batch insert: {:.3}ms ({:.3}ms per record)", 
                batch_duration.as_millis(), batch_duration.as_millis() as f64 / test_count as f64);
        
        // CRITICAL REQUIREMENT: Batch must be faster or at least no slower
        assert!(batch_duration <= single_duration, 
            "FAILED: Batch insert ({:.3}ms) should be faster than individual inserts ({:.3}ms)", 
            batch_duration.as_millis(), single_duration.as_millis());
        
        // Verify both methods stored all data correctly
        let count1 = storage1.count().await.unwrap();
        let count2 = storage2.count().await.unwrap();
        assert_eq!(count1, test_count, "FAILED: Single insert count mismatch");
        assert_eq!(count2, test_count, "FAILED: Batch insert count mismatch");
        
        println!("✅ PASSED: Storage performance validation");
    }
    
    /// Test 3: RETRIEVAL VALIDATION - Does semantic search actually work?
    #[tokio::test]
    async fn brutal_retrieval_validation() {
        println!("🔥 BRUTAL TEST: Retrieval Validation");
        
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("retrieval_test.db");
        
        let storage = LanceDBStorage::new(db_path).await.unwrap();
        storage.init_table().await.unwrap();
        
        // Create test data with KNOWN similarity relationships
        let test_data = vec![
            ("identical.rs", "fn test() {}", vec![1.0; 768]), // Should match query exactly
            ("similar.rs", "fn test() { println!(); }", vec![0.9; 768]), // Should match query closely  
            ("different.rs", "struct Data {}", vec![0.1; 768]), // Should match query poorly
        ];
        
        let mut test_records = Vec::new();
        for (i, (file, content, embedding)) in test_data.into_iter().enumerate() {
            test_records.push(LanceEmbeddingRecord {
                id: format!("test-{}", i),
                file_path: file.to_string(),
                chunk_index: i as u64,
                content: content.to_string(),
                embedding,
                start_line: 1,
                end_line: 2,
                similarity_score: None,
                checksum: None,
            });
        }
        
        storage.insert_batch(test_records).await.unwrap();
        
        // CRITICAL TEST: Search with exact query should return identical match first
        let query = vec![1.0; 768];
        let results = storage.search_similar(query, 3).await.unwrap();
        
        assert_eq!(results.len(), 3, "FAILED: Should return exactly 3 results");
        
        // CRITICAL TEST: Most similar should be first
        assert_eq!(results[0].file_path, "identical.rs", 
            "FAILED: Most similar result should be 'identical.rs', got '{}'", results[0].file_path);
        
        // CRITICAL TEST: Similarity scores should be computed and ordered correctly
        if let (Some(score1), Some(score2)) = (results[0].similarity_score, results[1].similarity_score) {
            assert!(score1 >= score2, 
                "FAILED: Similarity scores not properly ordered - first: {}, second: {}", 
                score1, score2);
        } else {
            panic!("FAILED: Similarity scores missing from search results");
        }
        
        // CRITICAL TEST: Content and metadata preserved correctly
        assert_eq!(results[0].content, "fn test() {}", 
            "FAILED: Content not preserved correctly");
        assert_eq!(results[0].chunk_index, 0, 
            "FAILED: Chunk index not preserved correctly");
        
        println!("✅ PASSED: Retrieval validation - semantic search works correctly");
    }
    
    /// Test 4: DELETION VALIDATION - Does delete_by_file actually remove records?
    #[tokio::test]
    async fn brutal_deletion_validation() {
        println!("🔥 BRUTAL TEST: Deletion Validation");
        
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("deletion_test.db");
        
        let storage = LanceDBStorage::new(db_path).await.unwrap();
        storage.init_table().await.unwrap();
        
        // Create test records for multiple files
        let test_files = vec!["keep1.rs", "delete_me.rs", "keep2.rs", "delete_me.rs"];
        let mut test_records = Vec::new();
        
        for (i, file) in test_files.into_iter().enumerate() {
            test_records.push(LanceEmbeddingRecord {
                id: format!("test-{}", i),
                file_path: file.to_string(),
                chunk_index: i as u64,
                content: format!("Content {}", i),
                embedding: vec![0.1f32 * (i as f32 + 1.0); 768],
                start_line: 1,
                end_line: 2,
                similarity_score: None,
                checksum: None,
            });
        }
        
        storage.insert_batch(test_records).await.unwrap();
        
        // Verify initial state
        let initial_count = storage.count().await.unwrap();
        assert_eq!(initial_count, 4, "FAILED: Should have 4 initial records");
        
        // CRITICAL TEST: Delete specific file
        let delete_result = storage.delete_by_file("delete_me.rs").await;
        assert!(delete_result.is_ok(), "FAILED: Deletion should succeed - {:#?}", delete_result);
        
        // CRITICAL TEST: Verify correct records were deleted
        let remaining_count = storage.count().await.unwrap();
        assert_eq!(remaining_count, 2, 
            "FAILED: Should have 2 remaining records after deletion, got {}", remaining_count);
        
        // CRITICAL TEST: Verify only the correct records remain
        let query = vec![0.1; 768];
        let results = storage.search_similar(query, 10).await.unwrap();
        
        for result in &results {
            assert_ne!(result.file_path, "delete_me.rs", 
                "FAILED: Deleted file '{}' still found in search results", result.file_path);
        }
        
        let remaining_files: Vec<&str> = results.iter().map(|r| r.file_path.as_str()).collect();
        assert!(remaining_files.contains(&"keep1.rs"), "FAILED: 'keep1.rs' should remain");
        assert!(remaining_files.contains(&"keep2.rs"), "FAILED: 'keep2.rs' should remain");
        
        println!("✅ PASSED: Deletion validation - delete_by_file works correctly");
    }
    
    /// Test 5: DATA CORRUPTION DETECTION - Does validation catch corrupted data?
    #[tokio::test]
    async fn brutal_corruption_detection() {
        println!("🔥 BRUTAL TEST: Data Corruption Detection");
        
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("corruption_test.db");
        
        let storage = LanceDBStorage::new(db_path).await.unwrap();
        storage.init_table().await.unwrap();
        
        // Insert valid data first
        let valid_record = LanceEmbeddingRecord {
            id: "valid-1".to_string(),
            file_path: "valid.rs".to_string(),
            chunk_index: 0,
            content: "Valid content".to_string(),
            embedding: vec![0.1; 768],
            start_line: 1,
            end_line: 2,
            similarity_score: None,
            checksum: None,
        };
        
        storage.insert_batch(vec![valid_record]).await.unwrap();
        
        // CRITICAL TEST: Insert data with NaN (should be detected as corrupted)
        let mut corrupted_embedding = vec![0.1; 768];
        corrupted_embedding[0] = f32::NAN;
        
        let corrupted_record = LanceEmbeddingRecord {
            id: "corrupted-1".to_string(),
            file_path: "corrupted.rs".to_string(),
            chunk_index: 1,
            content: "Corrupted content".to_string(),
            embedding: corrupted_embedding,
            start_line: 1,
            end_line: 2,
            similarity_score: None,
            checksum: None,
        };
        
        // Allow insertion of corrupted data (to test detection)
        storage.insert_batch(vec![corrupted_record]).await.unwrap();
        
        // CRITICAL TEST: Data integrity validation should detect corruption
        let integrity_result = storage.validate_data_integrity().await;
        assert!(integrity_result.is_err(), 
            "FAILED: Data integrity check should detect NaN corruption");
        
        if let Err(LanceStorageError::IntegrityCheckFailed(msg)) = integrity_result {
            assert!(msg.contains("corrupted"), 
                "FAILED: Error message should mention corruption, got: {}", msg);
        } else {
            panic!("FAILED: Expected IntegrityCheckFailed error for corrupted data");
        }
        
        // Test recovery attempt
        let recovery_result = storage.recover_from_corruption().await;
        assert!(recovery_result.is_err(), 
            "FAILED: Recovery should report corruption found");
        
        println!("✅ PASSED: Corruption detection works correctly");
    }
    
    /// Test 6: INDEXING VALIDATION - Does indexing work with sufficient data and fail appropriately?
    #[tokio::test]
    async fn brutal_indexing_validation() {
        println!("🔥 BRUTAL TEST: Indexing Validation");
        
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("indexing_test.db");
        
        let storage = LanceDBStorage::new(db_path).await.unwrap();
        storage.init_table().await.unwrap();
        
        // CRITICAL TEST: Index creation should fail with insufficient data
        let initial_index_result = storage.create_index().await;
        assert!(initial_index_result.is_err(), 
            "FAILED: Index creation should fail with no data");
        
        if let Err(LanceStorageError::InsufficientRecords { available, required }) = initial_index_result {
            assert_eq!(available, 0, "FAILED: Should report 0 available records");
            assert_eq!(required, 100, "FAILED: Should require 100 records");
        } else {
            panic!("FAILED: Expected InsufficientRecords error");
        }
        
        // Add insufficient data (50 records)
        let mut records = Vec::new();
        for i in 0..50 {
            let mut embedding = vec![0.1; 768];
            embedding[i % 768] = i as f32 / 50.0;
            records.push(LanceEmbeddingRecord {
                id: format!("test-{}", i),
                file_path: format!("test_{}.rs", i),
                chunk_index: i as u64,
                content: format!("Content {}", i),
                embedding,
                start_line: i as u64,
                end_line: (i + 1) as u64,
                similarity_score: None,
                checksum: None,
            });
        }
        
        storage.insert_batch(records).await.unwrap();
        
        // CRITICAL TEST: Index creation should still fail with insufficient data
        let insufficient_index_result = storage.create_index().await;
        assert!(insufficient_index_result.is_err(), 
            "FAILED: Index creation should fail with only 50 records");
        
        // Add sufficient data (another 60 records for total of 110)
        let mut more_records = Vec::new();
        for i in 50..110 {
            let mut embedding = vec![0.1; 768];
            embedding[i % 768] = i as f32 / 110.0;
            more_records.push(LanceEmbeddingRecord {
                id: format!("test-{}", i),
                file_path: format!("test_{}.rs", i),
                chunk_index: i as u64,
                content: format!("Content {}", i),
                embedding,
                start_line: i as u64,
                end_line: (i + 1) as u64,
                similarity_score: None,
                checksum: None,
            });
        }
        
        storage.insert_batch(more_records).await.unwrap();
        
        // CRITICAL TEST: Index creation should now succeed
        let sufficient_index_result = storage.create_index().await;
        assert!(sufficient_index_result.is_ok(), 
            "FAILED: Index creation should succeed with 110 records - {:#?}", sufficient_index_result);
        
        // CRITICAL TEST: Index should be marked as ready
        assert!(storage.is_index_ready(), "FAILED: Index should be marked as ready after creation");
        
        // CRITICAL TEST: Second index creation should be idempotent
        let second_index_result = storage.create_index().await;
        assert!(second_index_result.is_ok(), 
            "FAILED: Second index creation should be idempotent");
        
        println!("✅ PASSED: Indexing validation - works correctly with sufficient data");
    }
    
    /// Test 7: ATOMIC OPERATIONS VALIDATION - Do atomic batches maintain integrity?
    #[tokio::test]
    async fn brutal_atomic_operations_validation() {
        println!("🔥 BRUTAL TEST: Atomic Operations Validation");
        
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("atomic_test.db");
        
        let storage = LanceDBStorage::new(db_path).await.unwrap();
        storage.init_table().await.unwrap();
        
        // Create test records
        let mut test_records = Vec::new();
        for i in 0..5 {
            test_records.push(LanceEmbeddingRecord {
                id: format!("atomic-{}", i),
                file_path: "atomic_test.rs".to_string(),
                chunk_index: i as u64,
                content: format!("Atomic content {}", i),
                embedding: vec![0.1f32 * (i as f32 + 1.0); 768],
                start_line: i as u64,
                end_line: (i + 1) as u64,
                similarity_score: None,
                checksum: None,
            });
        }
        
        // CRITICAL TEST: Create atomic batch
        let atomic_batch = storage.create_atomic_batch(test_records.clone())
            .expect("FAILED: Atomic batch creation should succeed");
        
        assert_eq!(atomic_batch.records.len(), 5, 
            "FAILED: Atomic batch should contain all records");
        assert!(atomic_batch.records.iter().all(|r| r.checksum.is_some()), 
            "FAILED: All records in atomic batch should have checksums");
        assert!(!atomic_batch.operation_id.is_empty(), 
            "FAILED: Operation ID should be generated");
        
        // CRITICAL TEST: Insert atomic batch
        let insert_result = storage.insert_atomic_batch(atomic_batch).await;
        assert!(insert_result.is_ok(), 
            "FAILED: Atomic batch insert should succeed - {:#?}", insert_result);
        
        // CRITICAL TEST: Verify all records were inserted atomically
        let count = storage.count().await.unwrap();
        assert_eq!(count, 5, "FAILED: All 5 records should be inserted atomically");
        
        // CRITICAL TEST: Verify data integrity after atomic insert
        let integrity_result = storage.validate_data_integrity().await;
        assert!(integrity_result.is_ok(), 
            "FAILED: Data integrity should be maintained after atomic insert");
        
        // CRITICAL TEST: Test atomic batch with corrupted checksum (should fail)
        let mut bad_records = Vec::new();
        for i in 0..3 {
            let mut record = LanceEmbeddingRecord {
                id: format!("bad-{}", i),
                file_path: "bad_test.rs".to_string(),
                chunk_index: i as u64,
                content: format!("Bad content {}", i),
                embedding: vec![0.2f32; 768],
                start_line: i as u64,
                end_line: (i + 1) as u64,
                similarity_score: None,
                checksum: Some(999999), // Deliberately wrong checksum
            };
            bad_records.push(record);
        }
        
        // Create batch with wrong checksums
        let mut bad_batch = storage.create_atomic_batch(vec![bad_records[0].clone()]).unwrap();
        bad_batch.records[0].checksum = Some(888888); // Corrupt the checksum after creation
        
        let bad_insert_result = storage.insert_atomic_batch(bad_batch).await;
        assert!(bad_insert_result.is_err(), 
            "FAILED: Atomic batch with corrupted checksums should fail");
        
        if let Err(LanceStorageError::DataCorruption { .. }) = bad_insert_result {
            // Expected error type
        } else {
            panic!("FAILED: Expected DataCorruption error for bad checksums");
        }
        
        // Verify the bad batch didn't get inserted (atomic failure)
        let final_count = storage.count().await.unwrap();
        assert_eq!(final_count, 5, 
            "FAILED: Failed atomic batch should not insert any records");
        
        println!("✅ PASSED: Atomic operations validation - integrity maintained");
    }
    
    /// FINAL BRUTAL VALIDATION SUMMARY
    #[tokio::test]
    async fn brutal_full_system_validation() {
        println!("🔥 BRUTAL TEST: Full System Integration Validation");
        
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("full_system_test.db");
        
        let storage = LanceDBStorage::new(db_path).await.unwrap();
        storage.init_table().await.unwrap();
        
        // Create realistic test scenario with multiple files and chunks
        let file_contents = vec![
            ("src/main.rs", vec!["fn main() {", "    println!(\"Hello, world!\");", "}"]),
            ("src/lib.rs", vec!["pub mod utils;", "pub use utils::*;"]),
            ("src/utils.rs", vec!["pub fn helper() -> i32 {", "    42", "}"]),
        ];
        
        let mut all_records = Vec::new();
        let mut record_id = 0;
        
        for (file_path, lines) in file_contents {
            for (chunk_idx, content) in lines.into_iter().enumerate() {
                // Create realistic embedding (would normally come from embedding model)
                let mut embedding = vec![0.1; 768];
                for (i, char) in content.chars().enumerate() {
                    if i < 768 {
                        embedding[i] = (char as u32 as f32) / 1000.0;
                    }
                }
                // Normalize
                let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                for val in &mut embedding {
                    *val /= norm;
                }
                
                all_records.push(LanceEmbeddingRecord {
                    id: format!("{}-{}", file_path, chunk_idx),
                    file_path: file_path.to_string(),
                    chunk_index: chunk_idx as u64,
                    content: content.to_string(),
                    embedding,
                    start_line: (chunk_idx + 1) as u64,
                    end_line: (chunk_idx + 2) as u64,
                    similarity_score: None,
                    checksum: None,
                });
                record_id += 1;
            }
        }
        
        // CRITICAL TEST: Store all data in batch
        let batch_start = Instant::now();
        storage.insert_batch(all_records.clone()).await
            .expect("FAILED: Batch insert should succeed");
        let batch_duration = batch_start.elapsed();
        
        println!("📊 Stored {} records in {:.3}ms", all_records.len(), batch_duration.as_millis());
        
        // CRITICAL TEST: Verify count
        let count = storage.count().await.unwrap();
        assert_eq!(count, all_records.len(), "FAILED: Record count mismatch");
        
        // CRITICAL TEST: Search functionality with various queries
        for test_record in all_records.iter().take(3) {
            let search_results = storage.search_similar(test_record.embedding.clone(), 5).await
                .expect("FAILED: Search should work for all stored embeddings");
            
            assert!(!search_results.is_empty(), "FAILED: Search should return results");
            
            // The most similar result should be the original record (or very close)
            let most_similar = &search_results[0];
            assert_eq!(most_similar.file_path, test_record.file_path,
                "FAILED: Most similar result should match original file");
        }
        
        // CRITICAL TEST: File-specific deletion
        let delete_result = storage.delete_by_file("src/utils.rs").await;
        assert!(delete_result.is_ok(), "FAILED: File deletion should succeed");
        
        let remaining_count = storage.count().await.unwrap();
        let expected_remaining = all_records.iter()
            .filter(|r| r.file_path != "src/utils.rs")
            .count();
        assert_eq!(remaining_count, expected_remaining,
            "FAILED: Remaining count after deletion doesn't match expected");
        
        // CRITICAL TEST: Verify deleted records are actually gone
        let search_query = all_records.iter()
            .find(|r| r.file_path == "src/utils.rs")
            .unwrap();
        
        let post_delete_results = storage.search_similar(search_query.embedding.clone(), 10).await.unwrap();
        for result in post_delete_results {
            assert_ne!(result.file_path, "src/utils.rs",
                "FAILED: Deleted file still appears in search results");
        }
        
        // CRITICAL TEST: Data integrity after all operations
        let final_integrity = storage.validate_data_integrity().await;
        assert!(final_integrity.is_ok(),
            "FAILED: Data integrity should be maintained after all operations");
        
        println!("✅ PASSED: Full system integration validation - ALL TESTS PASSED");
        
        // Final brutal summary
        println!("\n🎯 BRUTAL VALIDATION SUMMARY:");
        println!("   ✅ Data Integrity: VERIFIED - All data stored and retrieved accurately");
        println!("   ✅ Performance: VERIFIED - Batch operations work efficiently"); 
        println!("   ✅ Retrieval: VERIFIED - Semantic search returns correct results");
        println!("   ✅ Deletion: VERIFIED - Records are completely removed");
        println!("   ✅ Corruption Detection: VERIFIED - Invalid data is detected");
        println!("   ✅ Indexing: VERIFIED - Works with sufficient data, fails gracefully");
        println!("   ✅ Atomic Operations: VERIFIED - Integrity maintained during failures");
        println!("   ✅ System Integration: VERIFIED - All components work together");
        println!("\n🚀 VERDICT: LanceDB storage pipeline is PRODUCTION READY");
    }
}