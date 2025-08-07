# Task 020: Integration Different Query Types Test
**Priority:** HIGH  
**Estimated Time:** 10 minutes  
**Phase:** 6 - Testing Reality Check  

## Objective
Test integrated search with different query types and patterns that real developers use when searching code.

## Success Criteria
- [ ] Test keyword queries: "async", "error handling"
- [ ] Test natural language: "how to handle file errors"
- [ ] Test code pattern queries: "impl Default for"
- [ ] Test mixed queries: combining keywords + concepts
- [ ] Verify appropriate method selection for each type

## Implementation Steps
1. Execute keyword queries through pipeline
2. Test natural language queries
3. Test code-specific pattern queries
4. Test mixed query types
5. Verify routing to appropriate search methods

## Validation
- Keyword queries use BM25 effectively
- Natural language queries leverage ML semantic search
- Code patterns found via exact/fuzzy search
- Mixed queries combine multiple search methods
- Query routing works automatically

## Notes
- Test query classification accuracy
- Verify each query type gets best search method
- Document query handling strategies