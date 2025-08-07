# Task 006: Fix Tantivy Search Integration

## Objective
Ensure Tantivy search integrates properly with UnifiedSearcher.

## Context
Tantivy search may have specific integration challenges due to its index-based nature.

## Actions Required
1. Verify Tantivy search implements SearchTrait correctly
2. Handle Tantivy index lifecycle in unified context
3. Fix Tantivy result conversion to unified format
4. Ensure proper async handling for Tantivy operations

## Expected Outcome
- Tantivy search works correctly in UnifiedSearcher
- Index handling is properly managed
- Results are correctly converted and scored

## Files to Modify
- `src/search/tantivy_search.rs`
- `src/search/unified_searcher.rs`

## Success Criteria
- [ ] Tantivy integration compiles
- [ ] Index operations work correctly
- [ ] Result conversion is accurate

## Time Estimate: 10 minutes

## Priority: HIGH
Important search method for performance.