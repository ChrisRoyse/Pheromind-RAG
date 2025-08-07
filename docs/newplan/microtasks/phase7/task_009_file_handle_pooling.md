# Task 009: File Handle Pooling System

## Objective
Implement file handle pooling system to prevent file descriptor exhaustion and improve I/O performance through reuse.

## Time Estimate
10 minutes

## Priority
HIGH - Critical for preventing file descriptor limits in production

## Dependencies
- File system operations must exist

## Implementation Steps

### 1. Create File Handle Pool (4 min)
```rust
// src/resources/file_pool.rs
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::path::{Path, PathBuf};
use tokio::fs::File as AsyncFile;

#[derive(Debug, Clone)]
pub struct FilePoolConfig {
    pub max_open_files: usize,
    pub max_idle_time: Duration,
    pub cleanup_interval: Duration,
    pub max_files_per_path: usize,
}

impl Default for FilePoolConfig {
    fn default() -> Self {
        Self {
            max_open_files: 1000,
            max_idle_time: Duration::from_secs(300), // 5 minutes
            cleanup_interval: Duration::from_secs(60), // 1 minute
            max_files_per_path: 10,
        }
    }
}

#[derive(Debug)]
struct PooledFile {
    file: AsyncFile,
    last_used: Instant,
    usage_count: u64,
    path: PathBuf,
}

pub struct FilePool {
    config: FilePoolConfig,
    pools: Arc<Mutex<HashMap<PathBuf, VecDeque<PooledFile>>>>,
    total_files: Arc<Mutex<usize>>,
    last_cleanup: Arc<Mutex<Instant>>,
}

impl FilePool {
    pub fn new(config: FilePoolConfig) -> Self {
        Self {
            config,
            pools: Arc::new(Mutex::new(HashMap::new())),
            total_files: Arc::new(Mutex::new(0)),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub async fn get_file(&self, path: &Path) -> Result<FileHandle, FilePoolError> {
        // Trigger cleanup if needed
        self.maybe_cleanup().await;

        let mut pools = self.pools.lock().unwrap();
        let path_buf = path.to_path_buf();
        
        // Try to get existing file from pool
        if let Some(file_queue) = pools.get_mut(&path_buf) {
            if let Some(mut pooled_file) = file_queue.pop_front() {
                pooled_file.last_used = Instant::now();
                pooled_file.usage_count += 1;
                
                return Ok(FileHandle {
                    file: Some(pooled_file.file),
                    path: path_buf,
                    pool: Arc::downgrade(&Arc::new(self.clone())),
                    returned: false,
                });
            }
        }

        // Check total file limit
        let total = *self.total_files.lock().unwrap();
        if total >= self.config.max_open_files {
            return Err(FilePoolError::PoolExhausted {
                current: total,
                limit: self.config.max_open_files,
            });
        }

        // Create new file
        let file = AsyncFile::open(&path_buf).await
            .map_err(|e| FilePoolError::IoError(e))?;
        
        *self.total_files.lock().unwrap() += 1;
        
        Ok(FileHandle {
            file: Some(file),
            path: path_buf,
            pool: Arc::downgrade(&Arc::new(self.clone())),
            returned: false,
        })
    }

    pub async fn return_file(&self, file: AsyncFile, path: PathBuf) {
        let mut pools = self.pools.lock().unwrap();
        let queue = pools.entry(path.clone()).or_insert_with(VecDeque::new);
        
        // Check per-path limit
        if queue.len() < self.config.max_files_per_path {
            queue.push_back(PooledFile {
                file,
                last_used: Instant::now(),
                usage_count: 1,
                path,
            });
        } else {
            // Drop the file if pool is full
            *self.total_files.lock().unwrap() -= 1;
        }
    }
}
```

### 2. Implement File Handle with Automatic Return (3 min)
```rust
use std::sync::Weak;
use tokio::io::{AsyncRead, AsyncWrite, AsyncSeek};

pub struct FileHandle {
    file: Option<AsyncFile>,
    path: PathBuf,
    pool: Weak<FilePool>,
    returned: bool,
}

impl FileHandle {
    pub fn path(&self) -> &Path {
        &self.path
    }
    
    pub async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(ref mut file) = self.file {
            use tokio::io::AsyncReadExt;
            file.read(buf).await
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "File handle already returned to pool"
            ))
        }
    }
    
    pub async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(ref mut file) = self.file {
            use tokio::io::AsyncWriteExt;
            file.write(buf).await
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "File handle already returned to pool"
            ))
        }
    }
    
    pub async fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        if let Some(ref mut file) = self.file {
            use tokio::io::AsyncSeekExt;
            file.seek(pos).await
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "File handle already returned to pool"
            ))
        }
    }
    
    pub async fn return_to_pool(mut self) {
        if let Some(file) = self.file.take() {
            if let Some(pool) = self.pool.upgrade() {
                pool.return_file(file, self.path.clone()).await;
            }
            self.returned = true;
        }
    }
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        if !self.returned {
            if let Some(pool) = self.pool.upgrade() {
                if let Some(file) = self.file.take() {
                    // Use blocking return in destructor
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            pool.return_file(file, self.path.clone()).await;
                        });
                    });
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FilePoolError {
    #[error("File pool exhausted: {current} files open, limit is {limit}")]
    PoolExhausted { current: usize, limit: usize },
    #[error("I/O error: {0}")]
    IoError(std::io::Error),
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },
    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },
}
```

### 3. Add Pool Cleanup and Monitoring (3 min)
```rust
impl FilePool {
    async fn maybe_cleanup(&self) {
        let mut last_cleanup = self.last_cleanup.lock().unwrap();
        let now = Instant::now();
        
        if now.duration_since(*last_cleanup) < self.config.cleanup_interval {
            return;
        }
        
        *last_cleanup = now;
        drop(last_cleanup);
        
        self.cleanup_expired_files().await;
    }
    
    async fn cleanup_expired_files(&self) {
        let mut pools = self.pools.lock().unwrap();
        let cutoff_time = Instant::now() - self.config.max_idle_time;
        let mut files_cleaned = 0;
        
        for (_, queue) in pools.iter_mut() {
            let original_len = queue.len();
            queue.retain(|pooled_file| {
                if pooled_file.last_used < cutoff_time {
                    files_cleaned += 1;
                    false
                } else {
                    true
                }
            });
        }
        
        // Update total count
        *self.total_files.lock().unwrap() -= files_cleaned;
        
        // Remove empty queues
        pools.retain(|_, queue| !queue.is_empty());
    }
    
    pub fn get_pool_stats(&self) -> FilePoolStats {
        let pools = self.pools.lock().unwrap();
        let total = *self.total_files.lock().unwrap();
        
        let mut per_path_stats = HashMap::new();
        let mut total_pooled = 0;
        
        for (path, queue) in pools.iter() {
            let count = queue.len();
            total_pooled += count;
            per_path_stats.insert(path.clone(), count);
        }
        
        FilePoolStats {
            total_files: total,
            pooled_files: total_pooled,
            active_files: total - total_pooled,
            unique_paths: pools.len(),
            per_path_stats,
            utilization_ratio: total as f64 / self.config.max_open_files as f64,
        }
    }
    
    // Force cleanup for testing
    pub async fn force_cleanup(&self) {
        self.cleanup_expired_files().await;
    }
}

#[derive(Debug)]
pub struct FilePoolStats {
    pub total_files: usize,
    pub pooled_files: usize,
    pub active_files: usize,
    pub unique_paths: usize,
    pub per_path_stats: HashMap<PathBuf, usize>,
    pub utilization_ratio: f64,
}

// Integration with search operations
pub struct FilePooledSearch {
    pool: Arc<FilePool>,
    search_engine: SearchEngine,
}

impl FilePooledSearch {
    pub fn new(pool_config: FilePoolConfig) -> Self {
        Self {
            pool: Arc::new(FilePool::new(pool_config)),
            search_engine: SearchEngine::new(),
        }
    }
    
    pub async fn search_file(&self, path: &Path, query: &str) -> Result<Vec<SearchResult>, FilePoolError> {
        let mut file_handle = self.pool.get_file(path).await?;
        
        // Use the file for searching
        let mut contents = String::new();
        file_handle.read_to_string(&mut contents).await
            .map_err(FilePoolError::IoError)?;
        
        // File is automatically returned to pool when handle is dropped
        Ok(self.search_engine.search_text(&contents, query))
    }
}
```

## Validation
- [ ] File handles are properly pooled and reused
- [ ] Pool limits prevent descriptor exhaustion
- [ ] Automatic cleanup removes expired files
- [ ] File handles are returned automatically on drop
- [ ] Pool statistics provide visibility

## Success Criteria
- File handle pool prevents descriptor exhaustion
- Automatic cleanup maintains pool health
- File handles are automatically returned to pool
- Pool statistics show utilization and performance
- Integration with search operations works seamlessly

## Next Task
task_010 - Implement thread pool management system
