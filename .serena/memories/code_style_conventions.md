# Code Style and Conventions

## Rust Code Style

### General Principles
- Follow Rust standard naming conventions (snake_case for functions/variables, CamelCase for types)
- Use `rustfmt` for automatic formatting
- Adhere to Clippy linting recommendations
- Keep modules focused and under 500 lines
- Functions should be under 50 lines with single responsibility

### Error Handling
- Use `anyhow::Result` for application errors
- Use `thiserror` for custom error types
- Always propagate errors with `?` operator where appropriate
- Provide context with `.context()` for better error messages

### Documentation
- Document all public APIs with `///` doc comments
- Include examples in documentation where helpful
- Use `//!` for module-level documentation
- Keep inline comments minimal and meaningful

### Testing
- Write unit tests in the same file using `#[cfg(test)]` module
- Integration tests go in `/tests` directory
- Use descriptive test names that explain what is being tested
- Follow the Arrange-Act-Assert pattern

### Async/Await
- Use `tokio` for async runtime
- Mark async functions with `async` keyword
- Use `async-trait` for async traits
- Avoid blocking operations in async contexts

### Feature Flags
- Use feature flags for optional dependencies
- Core functionality in `core` feature
- Advanced features behind appropriate flags (ml, vectordb, tantivy, tree-sitter)
- Document feature requirements in function docs

## TypeScript/JavaScript Style (for Node.js components)

### General
- Use ES6+ syntax
- Prefer const over let, avoid var
- Use async/await over promises
- Follow SPARC methodology for structured development

## File Organization
- Keep related functionality together in modules
- Use clear, descriptive file names
- Separate concerns into distinct modules
- Export only necessary public interfaces