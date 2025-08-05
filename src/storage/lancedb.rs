use std::path::PathBuf;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use arrow::array::{Array, StringArray, Int32Array, Float32Array, FixedSizeListArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use lance::dataset::{Dataset, WriteParams};
use lance::index::vector::VectorIndexParams;
use crate::chunking::Chunk;

#[derive(Debug)]
pub enum StorageError {
    DatabaseError(String),
    SchemaError(String),
    InsertError(String),
    SearchError(String),
    InvalidInput(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            StorageError::SchemaError(msg) => write!(f, "Schema error: {}", msg),
            StorageError::InsertError(msg) => write!(f, "Insert error: {}", msg),
            StorageError::SearchError(msg) => write!(f, "Search error: {}", msg),
            StorageError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<lance::Error> for StorageError {
    fn from(err: lance::Error) -> Self {
        StorageError::DatabaseError(err.to_string())
    }
}

impl From<arrow::error::ArrowError> for StorageError {
    fn from(err: arrow::error::ArrowError) -> Self {
        StorageError::SchemaError(err.to_string())
    }
}

pub struct VectorStorage {
    db_path: PathBuf,
    dataset: Option<Dataset>,
    schema: Option<Arc<Schema>>,
}

impl VectorStorage {
    pub async fn new(db_path: PathBuf) -> Result<Self, StorageError> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| StorageError::DatabaseError(format!("Failed to create directory: {}", e)))?;
        }
        
        Ok(Self {
            db_path,
            dataset: None,
            schema: None,
        })
    }
    
    pub async fn init_schema(&mut self) -> Result<(), StorageError> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("file_path", DataType::Utf8, false),
            Field::new("chunk_index", DataType::Int32, false),
            Field::new("content", DataType::Utf8, false),
            Field::new("embedding", DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                384, // MiniLM embedding dimension
            ), false),
            Field::new("start_line", DataType::Int32, false),
            Field::new("end_line", DataType::Int32, false),
        ]));
        
        self.schema = Some(schema.clone());
        
        // Create empty dataset if it doesn't exist
        if !self.db_path.exists() {
            let empty_batch = RecordBatch::new_empty(schema);
            let dataset = Dataset::write(
                empty_batch,
                &self.db_path.to_string_lossy(),
                Some(WriteParams::default())
            ).await?;
            self.dataset = Some(dataset);
        } else {
            // Open existing dataset
            let dataset = Dataset::open(&self.db_path.to_string_lossy()).await?;
            self.dataset = Some(dataset);
        }
        
        Ok(())
    }
    
    pub fn get_schema(&self) -> Option<Arc<Schema>> {
        self.schema.clone()
    }
    
    pub async fn insert_embedding(
        &mut self,
        file_path: &str,
        chunk_index: usize,
        chunk: &Chunk,
        embedding: Vec<f32>
    ) -> Result<(), StorageError> {
        if embedding.len() != 384 {
            return Err(StorageError::InvalidInput(
                format!("Embedding must be 384-dimensional, got {}", embedding.len())
            ));
        }
        
        let dataset = self.dataset.as_mut()
            .ok_or_else(|| StorageError::SchemaError("Schema not initialized".to_string()))?;
        
        let schema = self.schema.as_ref()
            .ok_or_else(|| StorageError::SchemaError("Schema not available".to_string()))?;
        
        // Create unique ID for this embedding
        let id = format!("{}-{}", file_path, chunk_index);
        
        // Create record batch with single row
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vec![id])),
                Arc::new(StringArray::from(vec![file_path])),
                Arc::new(Int32Array::from(vec![chunk_index as i32])),
                Arc::new(StringArray::from(vec![chunk.content.clone()])),
                Arc::new(FixedSizeListArray::from_iter_primitive::<arrow::datatypes::Float32Type, _, _>(
                    vec![Some(embedding)],
                    384
                )),
                Arc::new(Int32Array::from(vec![chunk.start_line as i32])),
                Arc::new(Int32Array::from(vec![chunk.end_line as i32])),
            ],
        )?;
        
        // Append to dataset
        *dataset = Dataset::write(
            batch,
            &self.db_path.to_string_lossy(),
            Some(WriteParams::default().mode(lance::dataset::WriteMode::Append))
        ).await?;
        
        Ok(())
    }
    
    pub async fn insert_batch(
        &mut self,
        embeddings_data: Vec<(&str, usize, Chunk, Vec<f32>)>
    ) -> Result<(), StorageError> {
        if embeddings_data.is_empty() {
            return Ok(());
        }
        
        // Validate all embeddings are 384-dimensional
        for (_, _, _, embedding) in &embeddings_data {
            if embedding.len() != 384 {
                return Err(StorageError::InvalidInput(
                    format!("All embeddings must be 384-dimensional, got {}", embedding.len())
                ));
            }
        }
        
        let dataset = self.dataset.as_mut()
            .ok_or_else(|| StorageError::SchemaError("Schema not initialized".to_string()))?;
        
        let schema = self.schema.as_ref()
            .ok_or_else(|| StorageError::SchemaError("Schema not available".to_string()))?;
        
        // Prepare arrays
        let mut ids = Vec::new();
        let mut file_paths = Vec::new();
        let mut chunk_indices = Vec::new();
        let mut contents = Vec::new();
        let mut embeddings = Vec::new();
        let mut start_lines = Vec::new();
        let mut end_lines = Vec::new();
        
        for (file_path, chunk_index, chunk, embedding) in embeddings_data {
            ids.push(format!("{}-{}", file_path, chunk_index));
            file_paths.push(file_path.to_string());
            chunk_indices.push(chunk_index as i32);
            contents.push(chunk.content);
            embeddings.push(Some(embedding));
            start_lines.push(chunk.start_line as i32);
            end_lines.push(chunk.end_line as i32);
        }
        
        // Create record batch
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(ids)),
                Arc::new(StringArray::from(file_paths)),
                Arc::new(Int32Array::from(chunk_indices)),
                Arc::new(StringArray::from(contents)),
                Arc::new(FixedSizeListArray::from_iter_primitive::<arrow::datatypes::Float32Type, _, _>(
                    embeddings,
                    384
                )),
                Arc::new(Int32Array::from(start_lines)),
                Arc::new(Int32Array::from(end_lines)),
            ],
        )?;
        
        // Append to dataset
        *dataset = Dataset::write(
            batch,
            &self.db_path.to_string_lossy(),
            Some(WriteParams::default().mode(lance::dataset::WriteMode::Append))
        ).await?;
        
        Ok(())
    }
    
    pub async fn delete_by_file(&mut self, file_path: &str) -> Result<(), StorageError> {
        let dataset = self.dataset.as_mut()
            .ok_or_else(|| StorageError::SchemaError("Schema not initialized".to_string()))?;
        
        // Create filter to delete rows where file_path matches
        let filter = format!("file_path != '{}'", file_path);
        
        // Execute delete by creating new dataset with filtered data
        let scanner = dataset.scan();
        let filtered = scanner.filter(&filter)?;
        let batches: Result<Vec<_>, _> = filtered.try_into_stream().await?
            .try_collect().await;
        
        let batches = batches.map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        if !batches.is_empty() {
            // Recreate dataset with filtered data
            let combined = arrow::compute::concat_batches(
                &batches[0].schema(),
                &batches
            )?;
            
            *dataset = Dataset::write(
                combined,
                &self.db_path.to_string_lossy(),
                Some(WriteParams::default().mode(lance::dataset::WriteMode::Overwrite))
            ).await?;
        } else {
            // No data left, create empty dataset
            let empty_batch = RecordBatch::new_empty(
                self.schema.as_ref().unwrap().clone()
            );
            *dataset = Dataset::write(
                empty_batch,
                &self.db_path.to_string_lossy(),
                Some(WriteParams::default().mode(lance::dataset::WriteMode::Overwrite))
            ).await?;
        }
        
        Ok(())
    }
    
    pub async fn clear_all(&mut self) -> Result<(), StorageError> {
        let schema = self.schema.as_ref()
            .ok_or_else(|| StorageError::SchemaError("Schema not available".to_string()))?;
        
        // Create empty dataset
        let empty_batch = RecordBatch::new_empty(schema.clone());
        let dataset = Dataset::write(
            empty_batch,
            &self.db_path.to_string_lossy(),
            Some(WriteParams::default().mode(lance::dataset::WriteMode::Overwrite))
        ).await?;
        
        self.dataset = Some(dataset);
        Ok(())
    }
    
    pub async fn count(&self) -> Result<usize, StorageError> {
        let dataset = self.dataset.as_ref()
            .ok_or_else(|| StorageError::SchemaError("Schema not initialized".to_string()))?;
        
        let count = dataset.count_rows(None).await?;
        Ok(count)
    }
    
    pub async fn prepare_for_search(&mut self) -> Result<(), StorageError> {
        let dataset = self.dataset.as_mut()
            .ok_or_else(|| StorageError::SchemaError("Schema not initialized".to_string()))?;
        
        // Create vector index for similarity search
        let index_params = VectorIndexParams::ivf_pq()
            .num_partitions(256)
            .num_sub_vectors(96); // 384 / 4 = 96 sub-vectors
        
        dataset.create_index(&["embedding"], lance::index::IndexType::Vector, Some(Box::new(index_params)), true).await
            .map_err(|e| StorageError::SearchError(format!("Failed to create vector index: {}", e)))?;
        
        Ok(())
    }
}

// Thread-safe implementation
unsafe impl Send for VectorStorage {}
unsafe impl Sync for VectorStorage {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_basic_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        
        let storage = VectorStorage::new(db_path).await;
        assert!(storage.is_ok());
    }
    
    #[tokio::test]
    async fn test_schema_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_schema.lance");
        
        let mut storage = VectorStorage::new(db_path).await.unwrap();
        let result = storage.init_schema().await;
        assert!(result.is_ok());
        
        let schema = storage.get_schema().unwrap();
        assert_eq!(schema.fields().len(), 7);
    }
    
    #[tokio::test]
    async fn test_embedding_insertion() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_insert.lance");
        
        let mut storage = VectorStorage::new(db_path).await.unwrap();
        storage.init_schema().await.unwrap();
        
        let chunk = Chunk {
            content: "fn test() {}".to_string(),
            start_line: 1,
            end_line: 1,
        };
        
        let embedding = vec![0.1f32; 384];
        let result = storage.insert_embedding("test.rs", 0, &chunk, embedding).await;
        assert!(result.is_ok());
        
        let count = storage.count().await.unwrap();
        assert_eq!(count, 1);
    }
}