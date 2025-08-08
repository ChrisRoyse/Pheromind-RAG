# Code Style and Conventions

## Code Structure Guidelines
- **File Limits**: Never create files longer than 500 lines of code
- **Function Limits**: Functions should be under 50 lines with single responsibility
- **Class Limits**: Classes should be under 100 lines and represent a single concept
- **Modularity**: Code organized into clearly separated modules by feature/responsibility

## Error Handling
- **No Panics**: Use `Result<T, E>` instead of `unwrap()` or `expect()`
- **Fail Fast**: Check for errors early and raise exceptions immediately
- **Comprehensive Errors**: Use `thiserror` for structured error types
- **Error Context**: Use `anyhow` for error context and chaining

## Feature Flag Patterns
- Use `#[cfg(feature = "name")]` for conditional compilation
- Always provide feature-gated fallbacks that return meaningful errors
- Document feature requirements in function/module comments
- Group related features logically

## Concurrency Patterns
- Use `Arc<RwLock<T>>` for shared mutable state
- Prefer `tokio::join!` for parallel execution
- No blocking operations in async contexts
- Always handle async errors properly

## Naming Conventions
- **Modules**: snake_case (e.g., `search_fusion`, `vector_storage`)
- **Structs**: PascalCase (e.g., `UnifiedSearcher`, `FusedResult`)  
- **Functions**: snake_case (e.g., `search_semantic`, `expand_to_three_chunk`)
- **Constants**: SCREAMING_SNAKE_CASE
- **Features**: kebab-case (e.g., `search-basic`, `full-system`)

## Documentation
- Every public function/struct/enum must have doc comments
- Use `///` for public API documentation
- Use `//` for implementation comments
- Document feature requirements and error conditions
- Include examples for complex APIs

## Testing
- Unit tests in same file with `#[cfg(test)] mod tests`
- Integration tests in `tests/` directory
- Property-based testing for complex algorithms
- Mock external dependencies
- Test all error conditions

## Safety and Reliability
- **No Unsafe Code**: Avoid `unsafe` blocks unless absolutely necessary
- **No Unwraps**: Replace `.unwrap()` with proper error handling
- **Validate Inputs**: Check all inputs for validity
- **Graceful Degradation**: System should work with reduced features
- **Memory Bounds**: Use bounded caches and collections