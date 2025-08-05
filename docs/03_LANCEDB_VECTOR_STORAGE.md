# Phase 3: Git File Watching

## **PHASE OVERVIEW - SIMPLE GIT UPDATES**

**GOAL**: Git-based file change detection and vector database updates  
**APPROACH**: Use git status to detect changes, update only modified files  
**MEASUREMENT**: Verify changes are detected and indexed correctly  
**TIMELINE**: Week 3 (Tasks 021-030)

## **KEY INSIGHT: GIT KNOWS BEST**

**SIMPLICITY**: Git already tracks file changes perfectly - just use it!

**Core Components**:
1. **Git Status**: Detect modified/added/deleted files
2. **Incremental Updates**: Only re-embed changed files
3. **Vector Cleanup**: Remove embeddings for deleted/modified chunks
4. **Automatic Mode**: Optional background watching

## **GIT WATCHING TASK BREAKDOWN (021-030)**

### **Core Git Integration Tasks (021-025): Change Detection**

#### **Task 021: Git Status Parser**
**Goal**: Parse git status to find changed files  
**Duration**: 3 hours  
**Dependencies**: Phase 2 completion

**Implementation**:
```rust
use std::process::Command;

pub struct GitWatcher {
    repo_path: PathBuf,
}

#[derive(Debug, Clone)]
pub enum FileChange {
    Modified(PathBuf),
    Added(PathBuf),
    Deleted(PathBuf),
}

impl GitWatcher {
    pub fn new(repo_path: PathBuf) -> Self {
        Self { repo_path }
    }
    
    pub fn get_changes(&self) -> Result<Vec<FileChange>> {
        // Run git status --porcelain
        let output = Command::new("git")
            .args(&["status", "--porcelain"])
            .current_dir(&self.repo_path)
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("Git status failed"));
        }
        
        let stdout = String::from_utf8(output.stdout)?;
        let mut changes = Vec::new();
        
        for line in stdout.lines() {
            if line.len() < 3 {
                continue;
            }
            
            let status = &line[0..2];
            let file_path = PathBuf::from(line[3..].trim());
            
            // Only process code files
            if !self.is_code_file(&file_path) {
                continue;
            }
            
            match status {
                " M" | "M " | "MM" => changes.push(FileChange::Modified(file_path)),
                "A " | "AM" => changes.push(FileChange::Added(file_path)),
                " D" | "D " => changes.push(FileChange::Deleted(file_path)),
                "??" => changes.push(FileChange::Added(file_path)), // Untracked
                _ => {} // Ignore other statuses
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
                "rb" | "php" | "swift" | "kt" | "scala"
            ),
            None => false,
        }
    }
}
```

#### **Task 022: Vector Database Updater**
**Goal**: Update embeddings for changed files  
**Duration**: 4 hours  
**Dependencies**: Task 021

**Implementation**:
```rust
pub struct VectorUpdater {
    storage: VectorStorage,
    chunker: SimpleRegexChunker,
    embedder: MiniLMEmbedder,
}

impl VectorUpdater {
    pub async fn update_file(&mut self, file_path: &Path, change: &FileChange) -> Result<()> {
        match change {
            FileChange::Deleted(_) => {
                self.delete_file_embeddings(file_path).await
            },
            FileChange::Modified(_) | FileChange::Added(_) => {
                // Delete old embeddings first
                self.delete_file_embeddings(file_path).await?;
                
                // Re-index the file
                self.index_file(file_path).await
            }
        }
    }
    
    async fn delete_file_embeddings(&mut self, file_path: &Path) -> Result<()> {
        // Remove all embeddings for this file
        self.storage.delete_by_file(file_path.to_str().unwrap())?;
        Ok(())
    }
    
    async fn index_file(&mut self, file_path: &Path) -> Result<()> {
        // Read file content
        let content = std::fs::read_to_string(file_path)?;
        
        // Chunk the file
        let chunks = self.chunker.chunk_file(&content);
        
        // Embed and store each chunk
        for (idx, chunk) in chunks.iter().enumerate() {
            let embedding = self.embedder.embed(&chunk.content)?;
            
            self.storage.insert_embedding(
                file_path.to_str().unwrap(),
                idx,
                chunk,
                embedding
            )?;
        }
        
        Ok(())
    }
}
```

#### **Task 023: Batch Update Processing**
**Goal**: Process multiple file changes efficiently  
**Duration**: 3 hours  
**Dependencies**: Task 022

**Implementation**:
```rust
impl VectorUpdater {
    pub async fn batch_update(&mut self, changes: Vec<FileChange>) -> Result<UpdateStats> {
        let mut stats = UpdateStats::default();
        let start_time = Instant::now();
        
        // Group changes by type for efficient processing
        let mut modifications = Vec::new();
        let mut deletions = Vec::new();
        
        for change in changes {
            match &change {
                FileChange::Deleted(path) => deletions.push(path.clone()),
                FileChange::Modified(path) | FileChange::Added(path) => {
                    modifications.push(path.clone())
                }
            }
        }
        
        // Process deletions first (fast)
        for path in deletions {
            self.delete_file_embeddings(&path).await?;
            stats.deleted_files += 1;
        }
        
        // Process modifications/additions
        for path in modifications {
            match self.index_file(&path).await {
                Ok(_) => stats.updated_files += 1,
                Err(e) => {
                    eprintln!("Failed to index {}: {}", path.display(), e);
                    stats.failed_files += 1;
                }
            }
        }
        
        stats.total_time = start_time.elapsed();
        Ok(stats)
    }
}

#[derive(Default)]
pub struct UpdateStats {
    pub updated_files: usize,
    pub deleted_files: usize,
    pub failed_files: usize,
    pub total_time: Duration,
}
```

#### **Task 024: Watch Command Implementation**
**Goal**: Create a watch command that runs periodically  
**Duration**: 2 hours  
**Dependencies**: Task 023

**Implementation**:
```rust
pub struct WatchCommand {
    watcher: GitWatcher,
    updater: VectorUpdater,
    interval: Duration,
    enabled: Arc<AtomicBool>,
}

impl WatchCommand {
    pub fn new(repo_path: PathBuf, updater: VectorUpdater) -> Self {
        Self {
            watcher: GitWatcher::new(repo_path),
            updater,
            interval: Duration::from_secs(5), // Check every 5 seconds
            enabled: Arc::new(AtomicBool::new(false)),
        }
    }
    
    pub fn start(&mut self) {
        self.enabled.store(true, Ordering::Relaxed);
        let enabled = self.enabled.clone();
        
        std::thread::spawn(move || {
            while enabled.load(Ordering::Relaxed) {
                if let Err(e) = self.check_and_update() {
                    eprintln!("Watch error: {}", e);
                }
                
                std::thread::sleep(self.interval);
            }
        });
    }
    
    pub fn stop(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }
    
    fn check_and_update(&mut self) -> Result<()> {
        let changes = self.watcher.get_changes()?;
        
        if !changes.is_empty() {
            println!("Detected {} file changes", changes.len());
            let stats = self.updater.batch_update(changes).await?;
            println!(
                "Updated {} files, deleted {} files in {:?}",
                stats.updated_files, stats.deleted_files, stats.total_time
            );
        }
        
        Ok(())
    }
}
```

#### **Task 025: State Persistence**
**Goal**: Remember last update state across restarts  
**Duration**: 2 hours  
**Dependencies**: Task 024

### **Integration Tasks (026-030): Production Features**

#### **Task 026: Progress Reporting**
**Goal**: Show progress during large updates  
**Duration**: 2 hours  
**Dependencies**: Task 025

**Implementation**:
```rust
use indicatif::{ProgressBar, ProgressStyle};

impl VectorUpdater {
    pub async fn batch_update_with_progress(&mut self, changes: Vec<FileChange>) -> Result<UpdateStats> {
        let pb = ProgressBar::new(changes.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .progress_chars("##-")
        );
        
        let mut stats = UpdateStats::default();
        
        for (i, change) in changes.iter().enumerate() {
            let path = match change {
                FileChange::Modified(p) | FileChange::Added(p) | FileChange::Deleted(p) => p,
            };
            
            pb.set_message(format!("Processing: {}", path.display()));
            
            match self.update_file(path, change).await {
                Ok(_) => match change {
                    FileChange::Deleted(_) => stats.deleted_files += 1,
                    _ => stats.updated_files += 1,
                },
                Err(e) => {
                    eprintln!("Failed to process {}: {}", path.display(), e);
                    stats.failed_files += 1;
                }
            }
            
            pb.set_position((i + 1) as u64);
        }
        
        pb.finish_with_message("Update complete");
        Ok(stats)
    }
}
```

#### **Task 027: Ignore Patterns**
**Goal**: Support .gitignore-style patterns  
**Duration**: 2 hours  
**Dependencies**: Task 026

#### **Task 028: Error Recovery**
**Goal**: Handle partial updates gracefully  
**Duration**: 2 hours  
**Dependencies**: Task 027

#### **Task 029: Watch Status API**
**Goal**: API to check watch status and stats  
**Duration**: 1 hour  
**Dependencies**: Task 028

#### **Task 030: Phase 3 Completion**
**Goal**: Integration testing with real repositories  
**Duration**: 1 hour  
**Dependencies**: Task 029

## **SUCCESS CRITERIA**

### **Phase 3 Targets**
- **Git Integration**: Reliable change detection
- **Incremental Updates**: Only changed files processed
- **Performance**: <1s to detect and start updating
- **Reliability**: Handle errors gracefully
- **Watch Mode**: Optional automatic updates

### **Deliverables**
- Git status parser
- Vector database updater
- Batch processing
- Watch command
- Progress reporting

## **ARCHITECTURE**

```rust
// Phase 3 additions
pub struct Phase3GitWatch {
    pub watcher: GitWatcher,
    pub updater: VectorUpdater,
    pub watch_command: WatchCommand,
}

// Simple API
impl Phase3GitWatch {
    pub fn check_changes(&self) -> Result<Vec<FileChange>> {
        self.watcher.get_changes()
    }
    
    pub async fn update_changes(&mut self) -> Result<UpdateStats> {
        let changes = self.check_changes()?;
        self.updater.batch_update(changes).await
    }
    
    pub fn start_watching(&mut self) {
        self.watch_command.start();
    }
    
    pub fn stop_watching(&self) {
        self.watch_command.stop();
    }
}
```

## **WEEK 3 DELIVERABLES**

1. **Git Detection**: Parse git status for changes
2. **Update Logic**: Re-embed only changed files
3. **Batch Processing**: Efficient multi-file updates
4. **Watch Mode**: Automatic periodic checking
5. **Ready for MCP**: Foundation for toggle command

**Next Phase**: MCP server with full tool suite