// Simple file utility functions
use std::path::Path;

pub fn file_exists(path: &Path) -> bool {
    path.exists()
}

pub fn is_file(path: &Path) -> bool {
    path.is_file()
}

pub fn get_file_size(path: &Path) -> Result<u64, std::io::Error> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.len())
}