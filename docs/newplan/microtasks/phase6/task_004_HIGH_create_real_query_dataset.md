# Task 004: Create Real Query Dataset
**Priority:** HIGH  
**Estimated Time:** 10 minutes  
**Phase:** 6 - Testing Reality Check  

## Objective
Create a comprehensive dataset of real search queries that developers actually use when searching codebases.

## Success Criteria
- [ ] Create `tests/fixtures/real_queries.json` file
- [ ] 100+ real developer search queries
- [ ] Categories: function names, error messages, concepts, patterns
- [ ] Include expected result types for each query
- [ ] Mix of simple and complex multi-term queries

## Implementation Steps
1. Collect real queries from developer surveys/logs
2. Categorize by search intent and complexity
3. Include Rust-specific queries (trait, impl, macro, etc.)
4. Add natural language queries
5. Document expected result patterns for each

## Validation
- Queries represent authentic developer needs
- Range of complexity and specificity
- Include both successful and edge-case queries
- No synthetic/academic test queries
- Queries work against our test corpus

## Notes
- Sources: Stack Overflow searches, GitHub search patterns
- Include common typos and variations
- Test both keyword and semantic search patterns