# Git Watcher Edge Cases Analysis

## Critical Edge Cases That WILL Occur in Production

### 1. **Symlinks and Junction Points**
- **Issue**: Following symlinks can cause infinite loops or duplicate indexing
- **Windows Specific**: Junction points and directory symlinks behave differently
- **Required Handling**: Detect and skip symlinks, log with full path for debugging

### 2. **File Access Denied / Permission Errors**
- **Issue**: Files open in exclusive mode by IDEs, antivirus, or other processes
- **Common**: .git/index.lock, temp files, files being saved
- **Required Handling**: Retry with exponential backoff, clear error message with process holding lock

### 3. **Large Files (>100MB)**
- **Issue**: Reading entire file into memory causes OOM
- **Common**: Log files, database dumps, generated artifacts
- **Required Handling**: Stream processing, size check before reading, skip with warning

### 4. **Rapid File Moves/Renames**
- **Issue**: Move appears as delete + create, can lose tracking
- **Common**: IDE refactoring, git operations
- **Required Handling**: Track by inode/file ID, correlate delete+create events

### 5. **Binary Files Misidentified as Code**
- **Issue**: .exe, .dll with code-like extensions, or no extension
- **Common**: Compiled outputs, vendor binaries
- **Required Handling**: Magic byte detection, content sampling

### 6. **Filesystem Full / Quota Exceeded**
- **Issue**: Can't write index updates, can't create temp files
- **Required Handling**: Check available space, graceful degradation, clear error

### 7. **Network Drives and UNC Paths**
- **Issue**: High latency, connection drops, path format issues
- **Windows**: \\\\server\\share paths need special handling
- **Required Handling**: Timeout configuration, connection retry, path normalization

### 8. **File Locks During Save**
- **Issue**: Editors create temp files, rename to target (atomic save)
- **Common**: VS Code, Vim, Emacs all do this
- **Required Handling**: Recognize temp file patterns, wait for rename completion

### 9. **Case Sensitivity Mismatches**
- **Issue**: Windows (case-insensitive) vs Linux (case-sensitive)
- **Example**: File.rs and file.rs treated as same on Windows
- **Required Handling**: Canonical path comparison, platform-aware matching

### 10. **Unicode and Special Characters**
- **Issue**: Path encoding issues, especially Windows vs UTF-8
- **Common**: Emoji in filenames, non-ASCII characters
- **Required Handling**: Proper UTF-8 handling, percent encoding for special chars

### 11. **Watcher Process Termination**
- **Issue**: Ctrl+C, kill signal, system shutdown
- **Required Handling**: Signal handlers, graceful shutdown, state persistence

### 12. **Git Operations During Watch**
- **Issue**: git checkout/merge creates many changes at once
- **Common**: Branch switches, rebases
- **Required Handling**: Batch processing, operation detection, pause during git ops

## Error Message Requirements

Each error MUST include:
1. **What happened** - Specific operation that failed
2. **Why it failed** - Root cause with system error
3. **Where it failed** - Full file path and line number
4. **How to fix** - Actionable steps for user
5. **Error code** - For automated handling

Example:
```
ERROR[E1001]: Failed to read file for indexing
  File: C:\project\src\large_file.rs
  Reason: File size (156MB) exceeds maximum limit (100MB)
  Action: Add file to .gitignore or increase max_file_size in config
  Context: During incremental update triggered by file modification
```