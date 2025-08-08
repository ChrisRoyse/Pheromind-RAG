pub mod watcher;

pub use watcher::{GitWatcher, FileChange, UpdateStats};
#[cfg(feature = "vectordb")]
pub use watcher::WatchCommand;