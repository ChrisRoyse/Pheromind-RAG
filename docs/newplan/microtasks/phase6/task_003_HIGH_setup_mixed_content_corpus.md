# Task 003: Setup Mixed Content Test Corpus
**Priority:** HIGH  
**Estimated Time:** 10 minutes  
**Phase:** 6 - Testing Reality Check  

## Objective
Create a diverse test corpus mixing code, documentation, configuration, and data files to test search across all supported content types.

## Success Criteria
- [ ] Create `tests/fixtures/mixed_corpus/` directory
- [ ] Include real config files: Cargo.toml, .github/workflows, Dockerfile
- [ ] Add real data files: JSON, YAML, XML, CSV
- [ ] Include real log files and error outputs
- [ ] Mix binary and text files for edge case testing

## Implementation Steps
1. Create mixed content directory structure
2. Copy real configuration files from projects
3. Include actual CI/CD pipeline files
4. Add real data samples (anonymized if needed)
5. Include edge cases: empty files, very large files

## Validation
- Files represent real-world project structure
- Multiple file extensions and formats
- Size variety: 0 bytes to 1MB+
- Authentic content patterns
- No artificial test data

## Notes
- Include files that might cause parsing issues
- Test corpus should mirror real project complexity
- Document any content modifications for privacy