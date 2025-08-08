use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy)]
pub enum EventType {
    Created,
    Modified,
    Removed,
}

#[derive(Debug, Clone)]
pub struct FileEvent {
    pub path: PathBuf,
    pub event_type: EventType,
    pub timestamp: SystemTime,
}

impl FileEvent {
    pub fn new(path: PathBuf, event_type: EventType) -> Self {
        Self {
            path,
            event_type,
            timestamp: SystemTime::now(),
        }
    }
    
    pub fn is_code_file(&self) -> bool {
        let extensions = [
            "rs", "ts", "js", "py", "go", "java", "cpp", "c", "h", "hpp",
            "jsx", "tsx", "rb", "php", "swift", "kt", "scala", "cs", "sql", "md"
        ];
        
        self.path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| extensions.contains(&ext))
            .unwrap_or(false)
    }
}