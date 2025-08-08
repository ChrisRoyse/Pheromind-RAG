use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

use crate::search::unified::UnifiedSearcher;
use super::events::{FileEvent, EventType};

pub struct IndexUpdater {
    searcher: Arc<RwLock<UnifiedSearcher>>,
}

impl IndexUpdater {
    pub fn new(
        searcher: Arc<RwLock<UnifiedSearcher>>,
        _rx: mpsc::UnboundedReceiver<FileEvent>,
    ) -> Self {
        Self {
            searcher,
        }
    }

    pub fn start(&self) {
        // This is a placeholder for starting the updater
        // In production, this would spawn a background task
        println!("Index updater started");
    }
    
    pub async fn process_events(&self, events: Vec<FileEvent>) {
        for event in events {
            self.process_event(event).await;
        }
    }

    async fn process_event(&self, event: FileEvent) {
        // Clone the Arc to avoid holding the lock during async operations
        let searcher_arc = Arc::clone(&self.searcher);
        
        // We need to handle async operations properly
        // Since update_file and remove_file are async and need &self (not &mut self),
        // we can call them on a read guard
        let result = {
            let searcher = match searcher_arc.read() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to acquire searcher lock: {}", e);
                    return;
                }
            };
            
            match event.event_type {
                EventType::Created | EventType::Modified => {
                    // Call async method on the read guard
                    searcher.update_file(&event.path).await
                }
                EventType::Removed => {
                    searcher.remove_file(&event.path).await
                }
            }
        };
        
        match result {
            Ok(_) => {
                println!("✅ Processed {:?}: {:?}", event.event_type, event.path);
            }
            Err(e) => {
                eprintln!("Failed to process file {:?}: {}", event.path, e);
            }
        }
    }
}