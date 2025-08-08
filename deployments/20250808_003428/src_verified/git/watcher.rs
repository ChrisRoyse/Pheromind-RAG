use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use anyhow::{Result, anyhow};

#[cfg(feature = "vectordb")]
use std::time::Instant;
#[cfg(feature = "vectordb")]
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
#[cfg(feature = "vectordb")]
use tokio::sync::RwLock;

#[cfg(feature = "vectordb")]
use crate::storage::lancedb_storage::LanceDBStorage;
#[cfg(feature = "vectordb")]
use crate::search::unified::UnifiedSearcher;
#[cfg(feature = "vectordb")]
use crate::config::Config;

#[derive(Debug, Clone)]
pub enum FileChange {
    Modified(PathBuf),
    Added(PathBuf),
    Deleted(PathBuf),
}

pub struct GitWatcher {
    repo_path: PathBuf,
}

impl GitWatcher {
    pub fn new(repo_path: PathBuf) -> Self {
        Self { repo_path }
    }
    
    pub fn get_changes(&self) -> Result<Vec<FileChange>> {
        // Validate repo_path for security - prevent directory traversal attacks
        if !self.repo_path.is_dir() {
            return Err(anyhow!("Invalid repository path: {:?} is not a directory", self.repo_path));
        }
        
        // Canonicalize path to prevent path traversal attacks
        let canonical_path = self.repo_path.canonicalize()
            .map_err(|e| anyhow!("Failed to canonicalize repository path {:?}: {}", self.repo_path, e))?;
            
        // Run git status --porcelain with validated path
        let output = Command::new("git")
            .args(&["status", "--porcelain"])
            .current_dir(&canonical_path)
            .output();
        
        let output = match output {
            Ok(o) => o,
            Err(e) => {
                return Err(anyhow!("Failed to run git status: {}. Make sure git is installed and this is a git repository.", e));
            }
        };
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Git status failed: {}", stderr));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut changes = Vec::new();
        
        for line in stdout.lines() {
            if line.len() < 3 {
                continue;
            }
            
            let status = &line[0..2];
            let file_path = canonical_path.join(line[3..].trim());
            
            // Only process code files
            if !self.is_code_file(&file_path) {
                continue;
            }
            
            match status {
                " M" | "M " | "MM" | "AM" => changes.push(FileChange::Modified(file_path)),
                "A " | "??" => changes.push(FileChange::Added(file_path)),
                " D" | "D " | "DD" => changes.push(FileChange::Deleted(file_path)),
                _ => {
                    log::debug!("Git watcher: Unrecognized status '{}' for file '{}' - status not tracked", status, file_path.display());
                }
            }
        }
        
        Ok(changes)
    }
    
    fn is_code_file(&self, path: &Path) -> bool {
        match path.extension().and_then(|s| s.to_str()) {
            Some(ext) => matches!(
                ext,
                "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | 
                "go" | "java" | "cpp" | "c" | "h" | "hpp" |
                "rb" | "php" | "swift" | "kt" | "scala" | "cs" |
                "sql" | "md"
            ),
            None => false,
        }
    }
}

#[cfg(feature = "vectordb")]
pub struct VectorUpdater {
    searcher: Arc<UnifiedSearcher>,
    storage: Arc<RwLock<LanceDBStorage>>,
}

#[cfg(feature = "vectordb")]
impl VectorUpdater {
    pub fn new(searcher: Arc<UnifiedSearcher>, storage: Arc<RwLock<LanceDBStorage>>) -> Self {
        Self { searcher, storage }
    }
    
    pub async fn update_file(&self, file_path: &Path, change: &FileChange) -> Result<()> {
        match change {
            FileChange::Deleted(_) => {
                self.delete_file_embeddings(file_path).await
            },
            FileChange::Modified(_) | FileChange::Added(_) => {
                // Delete old embeddings first - must succeed
                self.delete_file_embeddings(file_path).await?;
                
                // Re-index the file
                self.searcher.index_file(file_path).await
            }
        }
    }
    
    async fn delete_file_embeddings(&self, file_path: &Path) -> Result<()> {
        let storage = self.storage.write().await;
        storage.delete_by_file(&file_path.to_string_lossy()).await?;
        Ok(())
    }
    
    pub async fn batch_update(&self, changes: Vec<FileChange>) -> Result<UpdateStats> {
        let mut stats = UpdateStats::new();
        let start_time = Instant::now();
        
        // Group changes by type
        let mut modifications = Vec::new();
        let mut deletions = Vec::new();
        
        for change in changes {
            match &change {
                FileChange::Deleted(path) => deletions.push((path.clone(), change)),
                FileChange::Modified(path) | FileChange::Added(path) => {
                    modifications.push((path.clone(), change))
                }
            }
        }
        
        // Process deletions first (fast) - all deletions must succeed
        for (path, change) in deletions {
            self.update_file(&path, &change).await
                .map_err(|e| anyhow!("Failed to delete embeddings for {}: {}. Batch update cannot proceed with failed deletions.", path.display(), e))?;
            stats.deleted_files += 1;
        }
        
        // Process modifications/additions - all indexing must succeed
        for (path, change) in modifications {
            self.update_file(&path, &change).await
                .map_err(|e| anyhow!("Failed to index {}: {}. Batch update cannot proceed with failed indexing operations.", path.display(), e))?;
            stats.updated_files += 1;
        }
        
        stats.total_time = start_time.elapsed();
        Ok(stats)
    }
    
    pub async fn batch_update_with_progress(&self, changes: Vec<FileChange>) -> Result<UpdateStats> {
        let total = changes.len();
        let mut stats = UpdateStats::new();
        let start_time = Instant::now();
        
        for (i, change) in changes.into_iter().enumerate() {
            let path = match &change {
                FileChange::Modified(p) | FileChange::Added(p) | FileChange::Deleted(p) => p,
            };
            
            #[cfg(debug_assertions)]
            println!("[{}/{}] Processing: {}", i + 1, total, path.display());
            
            self.update_file(path, &change).await
                .map_err(|e| anyhow!("Failed to process {}: {}. Progressive batch update cannot continue with failed operations.", path.display(), e))?;
            
            match change {
                FileChange::Deleted(_) => stats.deleted_files += 1,
                _ => stats.updated_files += 1,
            }
        }
        
        stats.total_time = start_time.elapsed();
        #[cfg(debug_assertions)]
        println!("✅ Update complete: {}", stats);
        Ok(stats)
    }
}

#[cfg(feature = "vectordb")]
pub struct WatchCommand {
    watcher: GitWatcher,
    updater: VectorUpdater,
    interval: Duration,
    enabled: Arc<AtomicBool>,
}

#[cfg(feature = "vectordb")]
impl WatchCommand {
    pub fn new(repo_path: PathBuf, searcher: Arc<UnifiedSearcher>, storage: Arc<RwLock<LanceDBStorage>>) -> Result<Self> {
        Ok(Self {
            watcher: GitWatcher::new(repo_path),
            updater: VectorUpdater::new(searcher, storage),
            interval: Duration::from_secs(Config::git_poll_interval_secs()
                .map_err(|e| anyhow!("Git poll interval not configured: {}. Configuration is required for git watching.", e))?),
            enabled: Arc::new(AtomicBool::new(false)),
        })
    }
    
    pub async fn start(&self) {
        self.enabled.store(true, Ordering::Relaxed);
        println!("👁️  Starting file watch (checking every {} seconds)", self.interval.as_secs());
        
        let enabled = self.enabled.clone();
        let watcher = self.watcher.clone();
        let updater = self.updater.clone();
        let interval = self.interval;
        
        tokio::spawn(async move {
            while enabled.load(Ordering::Relaxed) {
                if let Err(e) = Self::check_and_update(&watcher, &updater).await {
                    eprintln!("Watch error: {}", e);
                }
                
                tokio::time::sleep(interval).await;
            }
        });
    }
    
    pub fn stop(&self) {
        self.enabled.store(false, Ordering::Relaxed);
        println!("🛑 Stopped file watch");
    }
    
    pub fn is_running(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
    
    async fn check_and_update(watcher: &GitWatcher, updater: &VectorUpdater) -> Result<()> {
        let changes = watcher.get_changes()?;
        
        if !changes.is_empty() {
            #[cfg(debug_assertions)]
            println!("🔄 Detected {} file changes", changes.len());
            let stats = updater.batch_update(changes).await?;
            #[cfg(debug_assertions)]
            println!("📊 {}", stats);
        }
        
        Ok(())
    }
    
    pub async fn run_once(&self) -> Result<UpdateStats> {
        let changes = self.watcher.get_changes()?;
        
        if changes.is_empty() {
            #[cfg(debug_assertions)]
            println!("No changes detected");
            return Ok(UpdateStats::new());
        }
        
        #[cfg(debug_assertions)]
        println!("🔄 Detected {} file changes", changes.len());
        self.updater.batch_update_with_progress(changes).await
    }
}

#[derive(Debug)]
pub struct UpdateStats {
    pub updated_files: usize,
    pub deleted_files: usize,
    pub failed_files: usize,
    pub total_time: Duration,
}

impl UpdateStats {
    pub fn new() -> Self {
        Self {
            updated_files: 0,
            deleted_files: 0,
            failed_files: 0,
            total_time: Duration::from_secs(0),
        }
    }
}

impl std::fmt::Display for UpdateStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Updated {} files, deleted {} files, {} failures in {:.2}s",
            self.updated_files,
            self.deleted_files,
            self.failed_files,
            self.total_time.as_secs_f32()
        )
    }
}

// Implement Clone for GitWatcher to use in async context
impl Clone for GitWatcher {
    fn clone(&self) -> Self {
        Self {
            repo_path: self.repo_path.clone(),
        }
    }
}

// Implement Clone for VectorUpdater
#[cfg(feature = "vectordb")]
impl Clone for VectorUpdater {
    fn clone(&self) -> Self {
        Self {
            searcher: self.searcher.clone(),
            storage: self.storage.clone(),
        }
    }
}