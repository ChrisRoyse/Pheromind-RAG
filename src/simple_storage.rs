use anyhow::Result;
use arrow_array::{RecordBatch, Float32Array, StringArray, FixedSizeListArray, Int32Array, RecordBatchIterator};
use arrow_schema::{DataType, Field, Schema};
use lancedb::{Connection, Table};
use lancedb::query::{QueryBase, ExecutableQuery};  // Import traits
use futures_util::stream::TryStreamExt;  // For try_next() and stream iteration
use std::sync::Arc;

/// Simple LanceDB storage using correct API
pub struct VectorStorage {
    connection: Connection,
    table: Option<Table>,
}

impl VectorStorage {
    pub async fn new(db_path: &str) -> Result<Self> {
        // Correct LanceDB connection API
        let connection = lancedb::connect(db_path).execute().await?;
        
        Ok(Self {
            connection,
            table: None,
        })
    }

    /// Store embeddings with metadata
    pub async fn store(&mut self, 
                      contents: Vec<String>, 
                      embeddings: Vec<Vec<f32>>, 
                      file_paths: Vec<String>) -> Result<()> {
        
        // Create Arrow schema for LanceDB - correct format
        let embedding_dim = embeddings.first().map(|e| e.len()).unwrap_or(768) as i32;
        
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("content", DataType::Utf8, false),
            Field::new("file_path", DataType::Utf8, false),
            Field::new("vector", 
                      DataType::FixedSizeList(
                          Arc::new(Field::new("item", DataType::Float32, true)),
                          embedding_dim,
                      ), 
                      false),
        ]));

        // Convert data to Arrow format
        let ids: Vec<i32> = (0..contents.len() as i32).collect();
        let id_array = Int32Array::from(ids);
        let content_array = StringArray::from(contents);
        let file_path_array = StringArray::from(file_paths);
        
        // Convert embeddings to FixedSizeListArray
        let flat_embeddings: Vec<f32> = embeddings.into_iter().flatten().collect();
        let values = Float32Array::from(flat_embeddings);
        let embedding_array = FixedSizeListArray::try_new(
            Arc::new(Field::new("item", DataType::Float32, true)),
            embedding_dim,
            Arc::new(values),
            None,
        )?;

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(id_array),
                Arc::new(content_array), 
                Arc::new(file_path_array),
                Arc::new(embedding_array),
            ],
        )?;

        // Create or append to table using correct LanceDB API
        if self.table.is_none() {
            // Convert RecordBatch to RecordBatchIterator for LanceDB compatibility
            let data = RecordBatchIterator::new(vec![Ok(batch.clone())].into_iter(), batch.schema());
            let table = self.connection
                .create_table("documents", data)
                .execute()
                .await?;
            self.table = Some(table);
        } else {
            // Append to existing table
            if let Some(table) = &mut self.table {
                let data = RecordBatchIterator::new(vec![Ok(batch)].into_iter(), schema.clone());
                table.add(data).execute().await?;
            }
        }

        Ok(())
    }

    /// Search using correct LanceDB vector search API
    pub async fn search(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>> {
        if let Some(table) = &self.table {
            // Correct LanceDB vector search API
            let results = table
                .query()
                .nearest_to(query_embedding)?
                .limit(limit)
                .execute()
                .await?;

            let mut search_results = Vec::new();
            
            // Convert results to our format - fix stream iteration
            let mut result_stream = results;
            while let Some(batch) = result_stream.try_next().await? {
                for row_idx in 0..batch.num_rows() {
                    let content = batch.column_by_name("content")
                        .and_then(|col| col.as_any().downcast_ref::<StringArray>())
                        .and_then(|arr| arr.value(row_idx).parse().ok())
                        .unwrap_or_default();
                        
                    let file_path = batch.column_by_name("file_path")
                        .and_then(|col| col.as_any().downcast_ref::<StringArray>())
                        .and_then(|arr| arr.value(row_idx).parse().ok())
                        .unwrap_or_default();

                    // Extract distance/score from LanceDB result
                    let distance = batch.column_by_name("_distance")
                        .and_then(|col| col.as_any().downcast_ref::<Float32Array>())
                        .and_then(|arr| arr.value(row_idx).into())
                        .unwrap_or(1.0);
                    
                    // Convert distance to similarity score (lower distance = higher similarity)
                    let score = 1.0 / (1.0 + distance);
                    
                    search_results.push(SearchResult {
                        content,
                        file_path,
                        score,
                    });
                }
            }

            Ok(search_results)
        } else {
            Ok(vec![])
        }
    }

    /// Clear all data
    pub async fn clear(&mut self) -> Result<()> {
        if let Some(table) = &mut self.table {
            table.delete("true").await?; // Delete all rows
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct SearchResult {
    pub content: String,
    pub file_path: String,
    pub score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_vector_storage() -> Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        
        let mut storage = VectorStorage::new(db_path.to_str().unwrap()).await?;
        
        // Test data
        let contents = vec!["Hello world".to_string()];
        let embeddings = vec![vec![0.1; 768]]; // 768-dim embedding
        let file_paths = vec!["test.rs".to_string()];
        
        storage.store(contents, embeddings, file_paths).await?;
        
        let results = storage.search(vec![0.1; 768], 5).await?;
        assert!(!results.is_empty());
        
        Ok(())
    }
}