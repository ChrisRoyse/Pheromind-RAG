use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Read};
use anyhow::{Result, bail};
use std::time::{Duration, SystemTime};

/// Maximum file size we'll attempt to index (100MB)
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// Maximum symlink depth to prevent infinite loops
const MAX_SYMLINK_DEPTH: u32 = 5;

/// Retry configuration for locked files
const FILE_LOCK_RETRIES: u32 = 3;
const FILE_LOCK_RETRY_DELAY: Duration = Duration::from_millis(100);

#[derive(Debug, Clone)]
pub enum EdgeCaseError {
    FileTooLarge { path: PathBuf, size: u64 },
    SymlinkLoop { path: PathBuf, target: PathBuf },
    AccessDenied { path: PathBuf, details: String },
    BinaryFile { path: PathBuf, reason: String },
    NetworkPath { path: PathBuf, issue: String },
    FileLocked { path: PathBuf, process: Option<String> },
    InvalidUnicode { path: PathBuf, bytes: Vec<u8> },
    FilesystemFull { path: PathBuf, available: u64 },
}

impl std::fmt::Display for EdgeCaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeCaseError::FileTooLarge { path, size } => {
                write!(f, "ERROR[E1001]: File too large for indexing\n  \
                    File: {}\n  \
                    Size: {} MB (limit: {} MB)\n  \
                    Action: Add to .gitignore or increase max_file_size config\n  \
                    Reason: Large files can cause memory exhaustion",
                    path.display(), 
                    size / (1024 * 1024),
                    MAX_FILE_SIZE / (1024 * 1024)
                )
            }
            EdgeCaseError::SymlinkLoop { path, target } => {
                write!(f, "ERROR[E1002]: Symlink loop detected\n  \
                    File: {}\n  \
                    Target: {}\n  \
                    Action: Remove symlink or add to .gitignore\n  \
                    Reason: Following symlinks would cause infinite recursion",
                    path.display(),
                    target.display()
                )
            }
            EdgeCaseError::AccessDenied { path, details } => {
                write!(f, "ERROR[E1003]: File access denied\n  \
                    File: {}\n  \
                    Details: {}\n  \
                    Action: Check file permissions or close programs using the file\n  \
                    Reason: Operating system denied read access",
                    path.display(),
                    details
                )
            }
            EdgeCaseError::BinaryFile { path, reason } => {
                write!(f, "ERROR[E1004]: Binary file detected\n  \
                    File: {}\n  \
                    Detection: {}\n  \
                    Action: Add to .gitignore if not source code\n  \
                    Reason: Binary files cannot be indexed as text",
                    path.display(),
                    reason
                )
            }
            EdgeCaseError::NetworkPath { path, issue } => {
                write!(f, "ERROR[E1005]: Network path issue\n  \
                    Path: {}\n  \
                    Issue: {}\n  \
                    Action: Ensure network drive is connected and accessible\n  \
                    Reason: Network operations may be slow or unreliable",
                    path.display(),
                    issue
                )
            }
            EdgeCaseError::FileLocked { path, process } => {
                let process_info = process.as_ref()
                    .map(|p| format!(" by {}", p))
                    .unwrap_or_default();
                write!(f, "ERROR[E1006]: File locked{}\n  \
                    File: {}\n  \
                    Action: Close the program editing this file and retry\n  \
                    Reason: File is open for exclusive access",
                    process_info,
                    path.display()
                )
            }
            EdgeCaseError::InvalidUnicode { path, bytes } => {
                write!(f, "ERROR[E1007]: Invalid unicode in path\n  \
                    Path: {}\n  \
                    Invalid bytes: {:?}\n  \
                    Action: Rename file to use valid UTF-8 characters\n  \
                    Reason: Path contains non-UTF-8 byte sequences",
                    path.display(),
                    bytes
                )
            }
            EdgeCaseError::FilesystemFull { path, available } => {
                write!(f, "ERROR[E1008]: Filesystem full\n  \
                    Path: {}\n  \
                    Available space: {} bytes\n  \
                    Action: Free up disk space before continuing\n  \
                    Reason: Cannot write index updates without disk space",
                    path.display(),
                    available
                )
            }
        }
    }
}

impl std::error::Error for EdgeCaseError {}

pub struct EdgeCaseHandler;

impl EdgeCaseHandler {
    /// Check if a file should be processed, handling all edge cases
    pub fn validate_file(path: &Path) -> Result<()> {
        // Check if path exists
        if !path.exists() {
            bail!("File does not exist: {}", path.display());
        }

        // Check for symlinks
        if let Err(e) = Self::check_symlink(path) {
            return Err(anyhow::Error::new(e));
        }

        // Check file size
        if let Err(e) = Self::check_file_size(path) {
            return Err(anyhow::Error::new(e));
        }

        // Check if binary
        if let Err(e) = Self::check_binary_file(path) {
            return Err(anyhow::Error::new(e));
        }

        // Check for network paths on Windows
        #[cfg(target_os = "windows")]
        if let Err(e) = Self::check_network_path(path) {
            return Err(anyhow::Error::new(e));
        }

        Ok(())
    }

    /// Check if file is a symlink and handle appropriately
    fn check_symlink(path: &Path) -> Result<(), EdgeCaseError> {
        let metadata = fs::symlink_metadata(path)
            .map_err(|e| EdgeCaseError::AccessDenied {
                path: path.to_path_buf(),
                details: format!("Cannot read metadata: {}", e),
            })?;

        if metadata.is_symlink() {
            let target = fs::read_link(path)
                .map_err(|_| EdgeCaseError::SymlinkLoop {
                    path: path.to_path_buf(),
                    target: PathBuf::from("<unknown>"),
                })?;

            // Check for symlink loops by resolving the chain
            let mut current = target.clone();
            let mut depth = 0;
            
            while depth < MAX_SYMLINK_DEPTH {
                if !current.exists() {
                    break;
                }
                
                let meta = fs::symlink_metadata(&current)
                    .map_err(|_| EdgeCaseError::SymlinkLoop {
                        path: path.to_path_buf(),
                        target: current.clone(),
                    })?;
                    
                if !meta.is_symlink() {
                    break;
                }
                
                current = fs::read_link(&current)
                    .map_err(|_| EdgeCaseError::SymlinkLoop {
                        path: path.to_path_buf(),
                        target: current.clone(),
                    })?;
                    
                depth += 1;
            }
            
            if depth >= MAX_SYMLINK_DEPTH {
                return Err(EdgeCaseError::SymlinkLoop {
                    path: path.to_path_buf(),
                    target,
                });
            }
        }

        Ok(())
    }

    /// Check file size to prevent OOM
    fn check_file_size(path: &Path) -> Result<(), EdgeCaseError> {
        let metadata = fs::metadata(path)
            .map_err(|e| EdgeCaseError::AccessDenied {
                path: path.to_path_buf(),
                details: format!("Cannot read file size: {}", e),
            })?;

        let size = metadata.len();
        if size > MAX_FILE_SIZE {
            return Err(EdgeCaseError::FileTooLarge {
                path: path.to_path_buf(),
                size,
            });
        }

        Ok(())
    }

    /// Check if file is binary by examining first 8KB
    fn check_binary_file(path: &Path) -> Result<(), EdgeCaseError> {
        let mut file = fs::File::open(path)
            .map_err(|e| EdgeCaseError::AccessDenied {
                path: path.to_path_buf(),
                details: format!("Cannot open file: {}", e),
            })?;

        let mut buffer = vec![0u8; 8192];
        let bytes_read = file.read(&mut buffer)
            .map_err(|e| EdgeCaseError::AccessDenied {
                path: path.to_path_buf(),
                details: format!("Cannot read file: {}", e),
            })?;

        buffer.truncate(bytes_read);

        // Check for null bytes (strong indicator of binary)
        if buffer.contains(&0) {
            return Err(EdgeCaseError::BinaryFile {
                path: path.to_path_buf(),
                reason: "Contains null bytes".to_string(),
            });
        }

        // Check for high percentage of non-printable characters
        let non_printable = buffer.iter()
            .filter(|&&b| b < 32 && b != 9 && b != 10 && b != 13)
            .count();
        
        if non_printable > bytes_read / 10 {
            return Err(EdgeCaseError::BinaryFile {
                path: path.to_path_buf(),
                reason: format!("{}% non-printable characters", 
                    (non_printable * 100) / bytes_read),
            });
        }

        Ok(())
    }

    /// Check for network paths on Windows
    #[cfg(target_os = "windows")]
    fn check_network_path(path: &Path) -> Result<(), EdgeCaseError> {
        let path_str = path.to_string_lossy();
        
        // Check for UNC paths
        if path_str.starts_with("\\\\") {
            // Try to access the path with a timeout
            let start = SystemTime::now();
            let result = fs::metadata(path);
            let elapsed = start.elapsed().unwrap_or(Duration::from_secs(0));
            
            if elapsed > Duration::from_secs(2) {
                return Err(EdgeCaseError::NetworkPath {
                    path: path.to_path_buf(),
                    issue: format!("Slow network response ({}ms)", elapsed.as_millis()),
                });
            }
            
            if result.is_err() {
                return Err(EdgeCaseError::NetworkPath {
                    path: path.to_path_buf(),
                    issue: "Network path not accessible".to_string(),
                });
            }
        }
        
        Ok(())
    }

    /// Try to read a file with retries for locked files
    pub fn read_file_with_retry(path: &Path) -> Result<String> {
        let mut _last_error = None;
        
        for attempt in 0..FILE_LOCK_RETRIES {
            match fs::read_to_string(path) {
                Ok(content) => return Ok(content),
                Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                    _last_error = Some(e);
                    if attempt < FILE_LOCK_RETRIES - 1 {
                        std::thread::sleep(FILE_LOCK_RETRY_DELAY);
                        continue;
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
        
        Err(EdgeCaseError::FileLocked {
            path: path.to_path_buf(),
            process: Self::find_locking_process(path),
        }.into())
    }

    /// Try to find which process has a file locked (Windows only)
    #[cfg(target_os = "windows")]
    fn find_locking_process(_path: &Path) -> Option<String> {
        // This would require Windows API calls to enumerate handles
        // For now, return a generic message
        Some("another process".to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    fn find_locking_process(_path: &Path) -> Option<String> {
        // On Unix, use lsof if available
        None
    }

    /// Normalize path for cross-platform compatibility
    pub fn normalize_path(path: &Path) -> PathBuf {
        // Convert to canonical path if possible
        path.canonicalize().unwrap_or_else(|_| {
            // If canonicalize fails, at least clean up the path
            let mut normalized = PathBuf::new();
            for component in path.components() {
                match component {
                    std::path::Component::ParentDir => {
                        normalized.pop();
                    }
                    std::path::Component::Normal(c) => {
                        normalized.push(c);
                    }
                    _ => {
                        normalized.push(component.as_os_str());
                    }
                }
            }
            normalized
        })
    }

    /// Check if filesystem has enough space for operations
    pub fn check_disk_space(path: &Path) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            use winapi::um::fileapi::GetDiskFreeSpaceExW;
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            
            let path_str: Vec<u16> = OsStr::new(path.to_str().unwrap_or("."))
                .encode_wide()
                .chain(Some(0))
                .collect();
            
            let mut free_bytes: u64 = 0;
            let mut total_bytes: u64 = 0;
            let mut free_bytes_available: u64 = 0;
            
            unsafe {
                let result = GetDiskFreeSpaceExW(
                    path_str.as_ptr(),
                    &mut free_bytes_available as *mut u64 as *mut _,
                    &mut total_bytes as *mut u64 as *mut _,
                    &mut free_bytes as *mut u64 as *mut _,
                );
                
                if result == 0 {
                    return Err(EdgeCaseError::FilesystemFull {
                        path: path.to_path_buf(),
                        available: 0,
                    }.into());
                }
            }
            
            // Require at least 100MB free
            if free_bytes_available < 100 * 1024 * 1024 {
                return Err(EdgeCaseError::FilesystemFull {
                    path: path.to_path_buf(),
                    available: free_bytes_available,
                }.into());
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // Unix implementation would use statvfs
        }
        
        Ok(())
    }
}