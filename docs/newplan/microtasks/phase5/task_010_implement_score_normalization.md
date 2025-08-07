# Task 010: Implement Score Normalization

## Objective
Implement score normalization to make scores from different search methods comparable.

## Context
Different search methods return scores in different ranges, need normalization for fair fusion.

## Actions Required
1. Analyze score ranges from each search method
2. Implement min-max normalization
3. Add z-score normalization as alternative
4. Make normalization method configurable

## Expected Outcome
- Scores from different methods are comparable
- Fusion results are more accurate
- Multiple normalization strategies available

## Files to Modify
- `src/search/fusion.rs`
- `src/search/unified_searcher.rs`

## Success Criteria
- [ ] Score normalization implemented
- [ ] Multiple normalization methods available
- [ ] Fusion accuracy improved

## Time Estimate: 10 minutes

## Priority: MEDIUM
Important for fusion quality.