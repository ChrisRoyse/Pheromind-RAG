# Task 005: BM25 Real Rust Code Search Test
**Priority:** HIGH  
**Estimated Time:** 10 minutes  
**Phase:** 6 - Testing Reality Check  

## Objective
Test BM25 search functionality with real Rust source code files, verifying actual term matching and ranking accuracy.

## Success Criteria
- [ ] Test BM25 with real Rust corpus (from task_001)
- [ ] Verify exact keyword matching works correctly
- [ ] Test Rust-specific syntax: `fn`, `impl`, `struct`, `enum`
- [ ] Validate TF-IDF scoring produces logical rankings
- [ ] Test with 20+ real search queries

## Implementation Steps
1. Load real Rust corpus into BM25 index
2. Execute searches for common Rust patterns
3. Verify results contain actual matching terms
4. Test ranking order makes semantic sense
5. Benchmark search speed with real data

## Validation
- Search for "async fn" returns actual async functions
- Struct searches rank files with more struct definitions higher
- Complex queries work: "impl Default for"
- No false positives in results
- Search latency < 100ms for corpus

## Notes
- Test against multiple Rust versions/styles
- Verify tokenization handles Rust syntax correctly
- Check ranking quality, not just presence of results