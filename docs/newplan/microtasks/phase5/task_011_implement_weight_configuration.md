# Task 011: Implement Weight Configuration System

## Objective
Create configurable weight system for different search methods in fusion.

## Context
Different search methods should have different importance weights based on query type.

## Actions Required
1. Create WeightConfig struct with method weights
2. Implement query-type-based weight selection
3. Add runtime weight adjustment capability
4. Create default weight profiles

## Expected Outcome
- Configurable weights for each search method
- Query-type-aware weight selection
- Runtime weight adjustment possible

## Files to Modify
- `src/search/config.rs`
- `src/search/fusion.rs`
- `src/search/unified_searcher.rs`

## Success Criteria
- [ ] Weight configuration system works
- [ ] Query-type-based weights functional
- [ ] Runtime adjustment possible

## Time Estimate: 10 minutes

## Priority: MEDIUM
Enhances fusion flexibility.