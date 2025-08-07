# Task 007: BM25 Ranking Quality Real-World Test
**Priority:** HIGH  
**Estimated Time:** 10 minutes  
**Phase:** 6 - Testing Reality Check  

## Objective
Test BM25 ranking quality with real queries against real codebase, ensuring results are ordered logically.

## Success Criteria
- [ ] Test 50+ real queries against Rust corpus
- [ ] Verify most relevant results appear in top 10
- [ ] Test ranking stability across similar queries
- [ ] Validate no obvious mis-rankings
- [ ] Compare rankings with developer expectations

## Implementation Steps
1. Execute real query dataset against BM25
2. Manually review top 10 results for each query
3. Identify any obviously wrong rankings
4. Test query variations and ranking consistency
5. Document ranking quality metrics

## Validation
- "error handling" ranks error-related code highest
- Function name searches return actual function definitions
- Concept searches return relevant code sections
- No completely irrelevant results in top 10
- Ranking order makes intuitive sense

## Notes
- Focus on subjective relevance quality
- Document any ranking surprises or issues
- Compare similar queries for consistency