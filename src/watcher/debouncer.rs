use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use super::events::FileEvent;

pub struct Debouncer {
    pending_events: HashMap<PathBuf, (FileEvent, Instant)>,
    debounce_duration: Duration,
}

impl Debouncer {
    pub fn new(debounce_duration: Duration) -> Self {
        Self {
            pending_events: HashMap::new(),
            debounce_duration,
        }
    }
    
    pub fn add_event(&mut self, event: FileEvent) {
        self.pending_events.insert(
            event.path.clone(),
            (event, Instant::now())
        );
    }
    
    pub fn get_ready_events(&mut self) -> Vec<FileEvent> {
        let now = Instant::now();
        let mut ready = Vec::new();
        
        self.pending_events.retain(|_path, (event, timestamp)| {
            if now.duration_since(*timestamp) >= self.debounce_duration {
                ready.push(event.clone());
                false // Remove from pending
            } else {
                true // Keep in pending
            }
        });
        
        ready
    }
    
    pub fn has_pending(&self) -> bool {
        !self.pending_events.is_empty()
    }
    
    pub fn oldest_pending_age(&self) -> Option<Duration> {
        let now = Instant::now();
        self.pending_events.values()
            .map(|(_, timestamp)| now.duration_since(*timestamp))
            .max()
    }
}