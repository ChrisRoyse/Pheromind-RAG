# Config System Fallback Elimination Strategy

## CRITICAL VIOLATION: Complete Fallback Infrastructure
**Impact**: Systems run with fallback configurations instead of failing when not properly configured

**Principle 0 Violation**: The config system provides extensive default/fallback behavior that masks configuration failures

## Files Affected
- `src/config/mod.rs:10-15` - SearchBackend Default derive
- `src/config/safe_config.rs:86-123` - Full Default implementation for Config

## Root Cause Analysis
1. **SearchBackend enum** has `#[derive(Default)]` and `#[default]` attribute
2. **Config struct in safe_config.rs** has complete `Default` implementation
3. These allow systems to continue operating with fallback values instead of failing explicitly

## Atomic Elimination Tasks

### Task 1: Remove SearchBackend Default Derive (3 minutes)
**File**: `src/config/mod.rs`
**Lines**: 10-15

**Test Condition**: Compile fails when SearchBackend::default() is used
**Implementation**:
1. Remove `Default` from derive macro on line 10
2. Remove `#[default]` attribute from line 13
3. Verify compilation fails when default behavior is expected

**Verification**: `cargo check` should show errors where SearchBackend::default() was used

### Task 2: Remove Config Default Implementation (5 minutes)
**File**: `src/config/safe_config.rs`
**Lines**: 86-123

**Test Condition**: Compile fails when Config::default() is used
**Implementation**:
1. Delete entire `impl Default for Config` block (lines 86-123)
2. Ensure no references to Config::default() exist in same file
3. Update any test functions that relied on Config::default()

**Verification**: `cargo check` should show errors where Config::default() was used

### Task 3: Update Test Functions Using Fallbacks (4 minutes)
**File**: `src/config/safe_config.rs`
**Lines**: 562-565, 614, 624

**Test Condition**: Tests explicitly construct config without defaults
**Implementation**:
1. Replace `Config::default()` with explicit config construction
2. Use existing valid configuration values from other tests
3. Ensure all test configs have explicit values for every field

**Verification**: `cargo test` passes with explicit configurations

### Task 4: Audit and Replace SearchBackend Default Usage (3 minutes)
**Files**: Throughout codebase

**Test Condition**: No implicit SearchBackend defaults exist
**Implementation**:
1. Search codebase for `SearchBackend::default()` usage
2. Replace with explicit `SearchBackend::Tantivy` where appropriate
3. Or require explicit configuration where defaults were used

**Verification**: `grep -r "SearchBackend::default" src/` returns no results

### Task 5: Add Compile-Time Guards Against Defaults (2 minutes)
**Files**: Both config files

**Test Condition**: Compiler prevents future default implementations
**Implementation**:
1. Add comment block explicitly forbidding Default implementations
2. Consider using `#[allow(dead_code)]` to make intentional compilation failures

**Verification**: Clear documentation exists preventing future fallback additions

### Task 6: Update Documentation and Comments (2 minutes)
**Files**: Both config files

**Test Condition**: Documentation reflects no-fallback policy
**Implementation**:
1. Update comments to reflect that all configuration must be explicit
2. Remove any references to "default behavior"
3. Add warnings about principle 0 enforcement

**Verification**: Comments accurately reflect fallback-free implementation

## Success Criteria
- [ ] `SearchBackend::default()` compilation fails
- [ ] `Config::default()` compilation fails  
- [ ] All tests pass with explicit configuration
- [ ] No implicit default behavior anywhere in config system
- [ ] Clear documentation prevents future fallback additions
- [ ] All configuration errors fail fast with descriptive messages

## Risk Mitigation
- **Breaking Changes**: This will break code that relied on defaults
- **Test Updates**: All tests must be updated to use explicit configs
- **Integration Points**: Other systems depending on config defaults will fail
- **Recovery Path**: Create explicit configuration templates/examples

## Implementation Order
Execute tasks 1-6 in sequence. Each task must complete successfully before proceeding to the next.

## Expected Outcome
Configuration system will either work correctly with proper explicit configuration or fail clearly with informative error messages. No fallback behavior will be permitted under any circumstances.