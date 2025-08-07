# Task 1.008: Fix Unified Search Unused Field Warnings

**Time Estimate**: 6 minutes
**Dependencies**: None
**File(s) to Modify**: `src/search/unified.rs`

## Objective
Fix warnings about unused fields and methods in UnifiedSearcher.

## Success Criteria
- [ ] No warnings about unused fields `fusion` and `project_path`
- [ ] No warnings about unused methods
- [ ] Fields are either used or marked as intentionally unused
- [ ] Clean compilation

## Instructions

### Step 1: Mark unused fields
```rust
// In UnifiedSearcher struct around line 50:
#[allow(dead_code)]
fusion: SimpleFusion,
#[allow(dead_code)]
project_path: PathBuf,
```

### Step 2: Mark unused methods
```rust
// Mark these methods with #[allow(dead_code)]:
#[allow(dead_code)]
async fn search_bm25(&self, query: &str) -> Result<Vec<BM25Match>> {

#[allow(dead_code)]
async fn expand_to_three_chunk(&self, fused_match: FusedResult) -> Result<SearchResult> {

#[allow(dead_code)]
fn find_chunk_for_line(&self, chunks: &[Chunk], line_number: usize) -> Result<usize> {
```

### Step 3: Verify warning cleanup
```bash
cargo check --all-features
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --all-features
```

## Troubleshooting
- If methods should be used, implement their usage instead
- Consider if fields are actually needed for future functionality

## Next Task
test_009 - Fix test file dead code warnings