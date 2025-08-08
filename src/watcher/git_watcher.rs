use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, DebounceEventResult, DebouncedEventKind};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::mpsc;
use anyhow::Result;

use crate::search::unified::UnifiedSearcher;
use super::events::{FileEvent, EventType};
use super::updater::IndexUpdater;
use super::edge_cases::EdgeCaseHandler;

pub struct GitWatcher {
    searcher: Arc<RwLock<UnifiedSearcher>>,
    gitignore: Gitignore,
    update_queue: mpsc::UnboundedSender<FileEvent>,
    updater: Arc<IndexUpdater>,
    watched_path: PathBuf,
    _watcher_guard: Option<notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>>,
    error_count: Arc<RwLock<u32>>,
    max_errors: u32,
}

impl GitWatcher {
    pub fn new(
        repo_path: &Path,
        searcher: Arc<RwLock<UnifiedSearcher>>,
    ) -> Result<Self> {
        // Validate and normalize the path
        let normalized_path = EdgeCaseHandler::normalize_path(repo_path);
        
        // Check disk space before starting
        EdgeCaseHandler::check_disk_space(&normalized_path)?;
        
        // Build gitignore matcher
        let mut builder = GitignoreBuilder::new(&normalized_path);
        let gitignore_path = normalized_path.join(".gitignore");
        if gitignore_path.exists() {
            builder.add(gitignore_path);
        }
        let gitignore = builder.build()?;

        // Create update channel
        let (tx, rx) = mpsc::unbounded_channel();

        // Create updater
        let updater = Arc::new(IndexUpdater::new(
            Arc::clone(&searcher),
            rx,
        ));

        Ok(Self {
            searcher,
            gitignore,
            update_queue: tx,
            updater,
            watched_path: normalized_path,
            _watcher_guard: None,
            error_count: Arc::new(RwLock::new(0)),
            max_errors: 100, // Stop after 100 consecutive errors
        })
    }

    pub fn start_watching(&mut self) -> Result<()> {
        let tx_clone = self.update_queue.clone();
        let gitignore_clone = self.gitignore.clone();
        let error_count = Arc::clone(&self.error_count);
        let max_errors = self.max_errors;
        let _watched_path = self.watched_path.clone();
        
        // Setup debounced watcher
        let mut debouncer = new_debouncer(
            Duration::from_millis(500),
            move |res: DebounceEventResult| {
                match res {
                    Ok(events) => {
                        // Reset error count on successful events
                        if let Ok(mut count) = error_count.write() {
                            *count = 0;
                        }
                        
                        for event in events {
                            Self::process_event_with_edge_cases(
                                event, 
                                &tx_clone, 
                                &gitignore_clone,
                                &error_count,
                                max_errors
                            );
                        }
                    }
                    Err(error) => {
                        // Handle watcher errors
                            eprintln!("ERROR[E2001]: File watcher error\n  \
                                Details: {:?}\n  \
                                Action: Check if the watched directory still exists\n  \
                                Reason: Filesystem notification failed", error);
                            
                        
                        if let Ok(mut count) = error_count.write() {
                            *count += 1;
                            if *count >= max_errors {
                                eprintln!("ERROR[E2002]: Too many consecutive errors ({})\n  \
                                    Action: Restarting watcher may be required\n  \
                                    Reason: System may be under heavy load or filesystem issues",
                                    *count);
                            }
                        }
                    }
                }
            },
        )?;

        // Start watching
        debouncer.watcher().watch(&self.watched_path, RecursiveMode::Recursive)?;
        
        // Store the debouncer to keep it alive
        self._watcher_guard = Some(debouncer);

        // Start the updater
        self.updater.start();

        println!("📁 Watching for changes in: {:?}", self.watched_path);
        println!("ℹ️  Edge case handling enabled with detailed error reporting");
        Ok(())
    }
    
    pub fn stop_watching(&mut self) {
        self._watcher_guard = None;
        println!("🛑 Stopped file watch");
        
        // Log final statistics
        if let Ok(count) = self.error_count.read() {
            if *count > 0 {
                println!("⚠️  Total errors during watch session: {}", count);
            }
        }
    }

    fn process_event_with_edge_cases(
        event: DebouncedEvent,
        tx: &mpsc::UnboundedSender<FileEvent>,
        gitignore: &Gitignore,
        error_count: &Arc<RwLock<u32>>,
        max_errors: u32,
    ) {
        // Handle path extraction with proper error reporting
        let path = EdgeCaseHandler::normalize_path(&event.path);
        
        // Skip if gitignored
        if gitignore.matched(&path, path.is_dir()).is_ignore() {
            log::debug!("Skipping gitignored path: {:?}", path);
            return;
        }

        // Skip directories
        if path.is_dir() {
            return;
        }

        // Skip non-code files
        if !Self::is_code_file(&path) {
            log::debug!("Skipping non-code file: {:?}", path);
            return;
        }

        // Determine event type based on file existence and kind
        let event_type = match event.kind {
            DebouncedEventKind::Any => {
                // For 'Any' events, check file existence
                if path.exists() {
                    // Validate the file before processing
                    match EdgeCaseHandler::validate_file(&path) {
                        Ok(_) => EventType::Modified,
                        Err(e) => {
                            // Log the specific edge case error
                            eprintln!("{}", e);
                            
                            // Track error count
                            if let Ok(mut count) = error_count.write() {
                                *count += 1;
                                if *count >= max_errors {
                                    eprintln!("⚠️  Reached maximum error threshold ({})", max_errors);
                                }
                            }
                            return;
                        }
                    }
                } else {
                    EventType::Removed
                }
            }
            _ => {
                // For other event kinds, also check existence
                if path.exists() {
                    match EdgeCaseHandler::validate_file(&path) {
                        Ok(_) => EventType::Modified,
                        Err(e) => {
                            eprintln!("{}", e);
                            if let Ok(mut count) = error_count.write() {
                                *count += 1;
                            }
                            return;
                        }
                    }
                } else {
                    EventType::Removed
                }
            }
        };

        // Send the validated event
        let file_event = FileEvent::new(path.clone(), event_type);
        
        match tx.send(file_event) {
            Ok(_) => {
                log::debug!("Queued event for: {:?}", path);
            }
            Err(e) => {
                eprintln!("ERROR[E2003]: Failed to queue file event\n  \
                    File: {:?}\n  \
                    Error: {}\n  \
                    Action: Check if the indexer is still running\n  \
                    Reason: Update channel may be closed",
                    path, e);
            }
        }
    }

    pub fn is_code_file(path: &Path) -> bool {
        // First check extension
        let extensions = [
            "rs", "ts", "js", "py", "go", "java", "cpp", "c", "h", "hpp",
            "jsx", "tsx", "rb", "php", "swift", "kt", "scala", "cs", "sql", 
            "md", "toml", "yaml", "yml", "json", "xml", "html", "css", "scss"
        ];
        
        let has_valid_extension = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| extensions.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false);
            
        if !has_valid_extension {
            return false;
        }
        
        // Additional check for files that might have code extensions but are binary
        // This is a quick check - full validation happens in EdgeCaseHandler
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            // Skip common binary patterns
            if filename.ends_with(".min.js") || 
               filename.ends_with(".min.css") ||
               filename.contains(".bundle.") ||
               filename.starts_with(".") && !filename.starts_with(".git") {
                return false;
            }
        }
        
        true
    }

    pub fn get_error_count(&self) -> u32 {
        self.error_count.read().map(|c| *c).unwrap_or(0)
    }
    
    pub fn reset_error_count(&self) {
        if let Ok(mut count) = self.error_count.write() {
            *count = 0;
        }
    }
}