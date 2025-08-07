# Task Completion Checklist

## After Making Code Changes

### 1. Format Code
```bash
cargo fmt
```

### 2. Run Linter
```bash
cargo clippy --all-features
```

### 3. Run Tests
```bash
# Run all tests to ensure nothing is broken
cargo test

# If working with specific features, test those
cargo test --features "full-system"
```

### 4. Check Type Safety
```bash
# For Rust
cargo check --all-features

# For TypeScript components
npm run typecheck
```

### 5. Update Documentation
- Update inline documentation if APIs changed
- Update README.md if features were added/modified
- Update CLAUDE.md if development workflow changed

### 6. Verify Build
```bash
cargo build --all-features
```

### 7. Run Integration Tests (if applicable)
```bash
cargo test --test "*integration*"
```

### 8. Check for Security Issues
```bash
cargo audit
```

## Before Committing

1. Review all changes with `git diff`
2. Ensure no sensitive data or keys are included
3. Check that all new files are in appropriate directories (not root)
4. Verify feature flags are correctly configured
5. Ensure error handling is proper (using Result types)

## For SPARC Development

When using SPARC methodology:
1. Complete all phases: Specification → Pseudocode → Architecture → Refinement → Completion
2. Ensure tests are written before implementation (TDD)
3. Validate against original requirements
4. Document design decisions

## Performance Considerations

If changes might affect performance:
1. Run benchmarks: `cargo bench`
2. Check memory usage stays under 2GB
3. Verify search latency remains <500ms
4. Test with large codebases for scalability