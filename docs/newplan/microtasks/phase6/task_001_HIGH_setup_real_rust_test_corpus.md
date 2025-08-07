# Task 001: Setup Real Rust Test Corpus
**Priority:** HIGH  
**Estimated Time:** 10 minutes  
**Phase:** 6 - Testing Reality Check  

## Objective
Create a real test corpus with actual Rust source files from various projects to test search functionality with genuine code patterns.

## Success Criteria
- [ ] Create `tests/fixtures/rust_corpus/` directory
- [ ] Copy 50+ real Rust files from different projects (std lib, cargo, rustc)
- [ ] Include various file types: lib.rs, main.rs, mod.rs, tests
- [ ] Files contain real syntax: structs, enums, traits, implementations
- [ ] Total corpus size: 500KB-2MB of actual code

## Implementation Steps
1. Create test corpus directory structure
2. Download/copy real Rust files from open source projects
3. Organize by project/module for traceability
4. Create manifest file listing all files with metadata
5. Verify files contain diverse Rust constructs

## Validation
- All files compile with `rustc --parse-only`
- Corpus contains minimum 50 files
- Files span different complexity levels
- No synthetic/dummy content

## Notes
- Use actual projects like: rust-lang/rust, tokio-rs/tokio, serde-rs/serde
- Maintain original file structure context
- Document source attribution for each file