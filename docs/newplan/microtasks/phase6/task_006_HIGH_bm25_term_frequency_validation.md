# Task 006: BM25 Term Frequency Validation Test
**Priority:** HIGH  
**Estimated Time:** 10 minutes  
**Phase:** 6 - Testing Reality Check  

## Objective
Validate that BM25 term frequency and document frequency calculations are mathematically correct with real content.

## Success Criteria
- [ ] Manually calculate expected TF-IDF for sample documents
- [ ] Verify BM25 implementation matches theoretical scores
- [ ] Test with documents of varying lengths
- [ ] Validate IDF calculation across corpus
- [ ] Confirm k1 and b parameters affect ranking correctly

## Implementation Steps
1. Select 3-5 documents with known term frequencies
2. Manually calculate expected BM25 scores
3. Compare with implementation output
4. Test parameter sensitivity (k1=1.2, b=0.75)
5. Verify document length normalization

## Validation
- Manual calculations match implementation within 0.001
- Longer documents appropriately penalized
- Rare terms score higher than common terms
- Parameter changes affect ranking as expected
- No mathematical errors in score calculation

## Notes
- Use simple documents for manual verification
- Document the BM25 formula being tested
- Verify edge cases: single term, empty documents