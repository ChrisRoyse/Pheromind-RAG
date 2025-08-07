# Core Command Fallback Elimination Strategy

## MODERATE VIOLATION: Command Fallback Patterns  
**Impact**: Commands fail silently instead of proper error reporting

**Principle 0 Violation**: Core system commands provide fallback behavior that masks missing features or silent failures instead of failing explicitly

## Files Affected
- `src/main.rs:211-214` - Watch command fallback (prints error, exits with code 1)
- `src/main.rs:232-235` - Update command fallback (prints error, exits with code 1)  
- `src/main.rs:268-270` - Test command silent fallback (returns Ok when directory missing)

## Root Cause Analysis
1. **Watch command stub** exists when vectordb feature disabled, providing fake implementation that exits
2. **Update command stub** exists when vectordb feature disabled, providing fake implementation that exits
3. **Test command logic** silently returns success when vectortest directory is missing instead of failing explicitly
4. **Feature-disabled implementations** create the illusion that commands exist when they fundamentally cannot work

## Atomic Elimination Tasks

### Task 1: Remove Watch Command Stub Implementation (4 minutes)
**File**: `src/main.rs`
**Lines**: 210-214

**Test Condition**: Code fails to compile when watch command used without vectordb feature
**Implementation**:
1. Delete entire stub implementation block (lines 210-214)
2. Remove `#[cfg(not(feature = "vectordb"))]` conditional compilation
3. Ensure watch_command is only available when vectordb feature is enabled
4. Let compilation fail naturally when vectordb feature is disabled

**Verification**: `cargo check` without vectordb feature fails with "function not found" error

### Task 2: Remove Update Command Stub Implementation (4 minutes)
**File**: `src/main.rs`
**Lines**: 231-235

**Test Condition**: Code fails to compile when update command used without vectordb feature
**Implementation**:
1. Delete entire stub implementation block (lines 231-235)
2. Remove `#[cfg(not(feature = "vectordb"))]` conditional compilation
3. Ensure update_command is only available when vectordb feature is enabled
4. Let compilation fail naturally when vectordb feature is disabled

**Verification**: `cargo check` without vectordb feature fails with "function not found" error

### Task 3: Replace Test Command Silent Fallback with Explicit Error (3 minutes)
**File**: `src/main.rs`
**Lines**: 268-270

**Test Condition**: Function fails explicitly when vectortest directory missing
**Implementation**:
1. Replace silent `return Ok(())` with explicit error return
2. Return `Err(anyhow!("vectortest directory not found: {}", vectortest_path.display()))`
3. Remove the misleading success return for missing requirements
4. Ensure error message guides user toward resolution

**Verification**: Function fails with clear error when vectortest directory missing

### Task 4: Update Command Line Argument Processing (3 minutes)  
**File**: `src/main.rs`
**Lines**: Around command matching logic

**Test Condition**: CLI fails clearly when commands require unavailable features
**Implementation**:
1. Add compile-time guards around command definitions requiring vectordb
2. Ensure command line parser only includes available commands
3. Remove any runtime feature detection for commands
4. Let CLI fail to compile when required features unavailable

**Verification**: CLI only exposes commands that can actually function

### Task 5: Update Help Text and Documentation (2 minutes)
**File**: `src/main.rs` and related help text

**Test Condition**: Help text only shows available commands
**Implementation**:
1. Make help text generation conditional on available features
2. Remove help text for commands that cannot function without features
3. Add clear feature requirements to remaining help text
4. Ensure users understand feature requirements

**Verification**: Help output only shows functional commands

### Task 6: Update Error Handling Throughout Main (3 minutes)
**File**: `src/main.rs`
**Lines**: Throughout error handling

**Test Condition**: All command failures are explicit with actionable messages
**Implementation**:
1. Review all command implementations for silent failures
2. Replace any remaining silent returns with explicit errors
3. Ensure all error messages guide users toward resolution
4. Remove any catch-all fallback behavior

**Verification**: All command failures are explicit and actionable

### Task 7: Update Build and Feature Configuration (3 minutes)
**Files**: `Cargo.toml`, build scripts

**Test Condition**: Build configuration clearly defines command availability
**Implementation**:
1. Document which commands require which features in Cargo.toml
2. Update build configuration to make feature requirements clear
3. Add examples of proper feature flag usage
4. Remove any build-time fallback logic

**Verification**: Build configuration clearly expresses dependencies

### Task 8: Add Integration Tests for Feature Requirements (4 minutes)
**Files**: Integration test files

**Test Condition**: Tests verify compile-time command availability
**Implementation**:
1. Add tests that verify commands fail to compile without required features
2. Test that available commands work properly with required features
3. Test error handling for missing prerequisites 
4. Add CI checks for various feature combinations

**Verification**: Tests validate compile-time and runtime command behavior

## Success Criteria
- [ ] Watch command stub implementation does not exist
- [ ] Update command stub implementation does not exist
- [ ] Code fails to compile when using vectordb commands without vectordb feature
- [ ] Test command fails explicitly when vectortest directory missing
- [ ] All command failures are explicit with actionable error messages
- [ ] Help text only shows available commands based on compiled features
- [ ] Build configuration clearly defines command-feature dependencies
- [ ] Tests validate proper compile-time and runtime behavior

## Risk Mitigation
- **Breaking Changes**: Code using vectordb commands without feature will no longer compile
- **CLI Changes**: Available commands will vary based on compiled features
- **User Experience**: Must provide clear guidance about feature requirements
- **Build Complexity**: May need multiple build targets for different feature sets

## Implementation Order
Execute tasks 1-8 in sequence. Each task must complete successfully before proceeding to the next.

## Expected Outcome
Core commands will either compile and work correctly with required features enabled, or fail to compile with clear error messages when required features are not available. No stub implementations or silent fallbacks will exist under any circumstances. Commands will be honest about their availability and requirements at compile time.