pub mod git_watcher;
pub mod events;
pub mod updater;
pub mod debouncer;
pub mod edge_cases;

pub use git_watcher::GitWatcher;
pub use events::{FileEvent, EventType};
pub use updater::IndexUpdater;
pub use debouncer::Debouncer;
pub use edge_cases::{EdgeCaseHandler, EdgeCaseError};