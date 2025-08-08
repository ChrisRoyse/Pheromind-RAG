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

pub struct GitWatcher {
    searcher: Arc<RwLock<UnifiedSearcher>>,
    gitignore: Gitignore,
    update_queue: mpsc::UnboundedSender<FileEvent>,
    updater: Arc<IndexUpdater>,
    watched_path: PathBuf,
    _watcher_guard: Option<notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>>,
}

impl GitWatcher {
    pub fn new(
        repo_path: &Path,
        searcher: Arc<RwLock<UnifiedSearcher>>,
    ) -> Result<Self> {
        // Build gitignore matcher
        let mut builder = GitignoreBuilder::new(repo_path);
        let gitignore_path = repo_path.join(".gitignore");
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
            watched_path: repo_path.to_path_buf(),
            _watcher_guard: None,
        })
    }

    pub fn start_watching(&mut self) -> Result<()> {
        let tx_clone = self.update_queue.clone();
        let gitignore_clone = self.gitignore.clone();
        let _watched_path = self.watched_path.clone();
        
        // Setup debounced watcher
        let mut debouncer = new_debouncer(
            Duration::from_millis(500),
            move |res: DebounceEventResult| {
                match res {
                    Ok(events) => {
                        for event in events {
                            Self::process_event(event, &tx_clone, &gitignore_clone);
                        }
                    }
                    Err(e) => {
                        eprintln!("Watch error: {:?}", e);
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
        Ok(())
    }
    
    pub fn stop_watching(&mut self) {
        self._watcher_guard = None;
        println!("🛑 Stopped file watch");
    }

    fn process_event(
        event: DebouncedEvent,
        tx: &mpsc::UnboundedSender<FileEvent>,
        gitignore: &Gitignore,
    ) {
        let paths = match event.kind {
            DebouncedEventKind::Any => vec![event.path],
            _ => vec![event.path],
        };
        
        for path in paths {
            // Skip if gitignored
            if gitignore.matched(&path, path.is_dir()).is_ignore() {
                continue;
            }

            // Skip non-code files
            if !Self::is_code_file(&path) {
                continue;
            }

            // Map DebouncedEventKind to our EventType
            let event_type = match event.kind {
                DebouncedEventKind::Any => EventType::Modified,
                // Note: notify-debouncer-mini doesn't provide detailed event types
                // We'll need to check file existence to determine if it's created/removed
                _ => {
                    if path.exists() {
                        EventType::Modified
                    } else {
                        EventType::Removed
                    }
                }
            };

            let file_event = FileEvent::new(path, event_type);
            let _ = tx.send(file_event);
        }
    }

    fn is_code_file(path: &Path) -> bool {
        let extensions = [
            "rs", "ts", "js", "py", "go", "java", "cpp", "c", "h", "hpp",
            "jsx", "tsx", "rb", "php", "swift", "kt", "scala", "cs", "sql", "md"
        ];
        
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| extensions.contains(&ext))
            .unwrap_or(false)
    }
}