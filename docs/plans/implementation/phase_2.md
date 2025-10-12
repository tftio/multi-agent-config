# Phase 2: Environment Variable Expansion

**Status**: Not Started
**Duration**: 2-3 days
**Dependencies**: Phase 1 (Core data structures and parsing must be complete)

---

## Overview

Implement the environment variable expansion system that resolves `${VAR}` references from the shell environment and `{VAR}` references from the `[env]` section of the configuration. This phase includes circular reference detection and proper error handling for undefined variables.

## Goals

- Implement variable expansion algorithm matching specification
- Support `${VAR}` expansion from shell environment (os.environ)
- Support `{VAR}` expansion from [env] section
- Detect circular references (max depth 10)
- Handle undefined variables gracefully (warn, expand to empty string)
- Recursively expand nested variable references

## Success Criteria

- [ ] Environment variables from shell expand correctly
- [ ] Variables from [env] section expand correctly
- [ ] Nested variable references resolve properly
- [ ] Circular references detected and rejected
- [ ] Undefined variables produce warnings but don't fail
- [ ] Maximum depth limit (10) prevents infinite loops
- [ ] All expansion tests pass

## Subtasks

### 2.1: Create Variable Expansion Module

**Objective**: Set up the expansion module structure

**Steps**:

1. Create `src/expand/mod.rs` as expansion module
2. Create `src/expand/expander.rs` for expansion logic
3. Define `Expander` struct:
   ```rust
   pub struct Expander {
       env_section: HashMap<String, String>,
       shell_env: HashMap<String, String>,
       warnings: Vec<String>,
   }
   ```
4. Implement constructor:
   ```rust
   pub fn new(env_section: HashMap<String, String>, shell_env: HashMap<String, String>) -> Self
   ```
5. Define expansion result type:
   ```rust
   pub type ExpansionResult = Result<String, ExpansionError>;
   ```
6. Define `ExpansionError` enum in `src/error.rs`:
   - `CircularReference { var_name: String, depth: usize }`
   - `MaxDepthExceeded { current_depth: usize, max_depth: usize }`
7. Add module exports in `src/lib.rs`
8. Write module-level documentation
9. Add unit test skeleton
10. Verify module compiles

**Files Modified**:
- `src/expand/mod.rs`
- `src/expand/expander.rs`
- `src/error.rs`
- `src/lib.rs`

**Acceptance Criteria**:
- Module structure compiles
- Expander struct defined with required fields
- Error types defined

---

### 2.2: Implement Shell Environment Variable Expansion

**Objective**: Expand `${VAR}` references from shell environment

**Steps**:

1. In `src/expand/expander.rs`, implement:
   ```rust
   fn expand_shell_vars(&mut self, value: &str) -> ExpansionResult
   ```
2. Use regex to find all `${VAR}` patterns:
   - Pattern: `r"\$\{([^}]+)\}"`
3. For each match:
   - Extract variable name
   - Look up in `self.shell_env`
   - If found: replace with value
   - If not found: add warning to `self.warnings`, replace with empty string
4. Handle special characters in variable names
5. Process all matches in a single pass
6. Return expanded string
7. Write unit tests:
   - Single variable expansion
   - Multiple variables in one string
   - Undefined variable (warning check)
   - Empty variable value
   - Variable name with special characters
8. Test edge cases:
   - Empty input string
   - No variables in string
   - Escaped braces (if supported)
9. Ensure warnings are collected
10. Run tests: `cargo test expand_shell_vars`

**Files Modified**:
- `src/expand/expander.rs`
- `Cargo.toml` (add regex dependency)

**Acceptance Criteria**:
- Shell variables expand correctly
- Undefined variables produce warnings
- All unit tests pass
- No panics on edge cases

---

### 2.3: Implement Env Section Variable Expansion

**Objective**: Expand `{VAR}` references from [env] section

**Steps**:

1. In `src/expand/expander.rs`, implement:
   ```rust
   fn expand_env_vars(&mut self, value: &str, depth: usize) -> ExpansionResult
   ```
2. Use regex to find all `{VAR}` patterns (without $):
   - Pattern: `r"\{([^}]+)\}"`
   - Note: Must not match `${}` patterns (already handled)
3. For each match:
   - Extract variable name
   - Check depth limit (max 10)
   - If depth >= 10: return `MaxDepthExceeded` error
   - Look up in `self.env_section`
   - If found: recursively expand the value with depth+1
   - If not found: add warning, replace with empty string
4. Track visited variables for circular reference detection
5. Process all matches
6. Return expanded string
7. Write unit tests:
   - Single env var expansion
   - Multiple env vars
   - Nested expansion (A references B)
   - Deep nesting (up to depth 9)
   - Depth limit exceeded (depth 10)
   - Circular reference detection
8. Test undefined env section variables
9. Test combinations of shell and env vars
10. Run tests: `cargo test expand_env_vars`

**Files Modified**:
- `src/expand/expander.rs`

**Acceptance Criteria**:
- Env section variables expand correctly
- Nested expansion works (up to depth 10)
- Depth limit enforced
- Undefined env vars produce warnings
- All tests pass

---

### 2.4: Implement Circular Reference Detection

**Objective**: Detect and prevent circular variable references

**Steps**:

1. Add tracking structure:
   ```rust
   struct ExpansionContext {
       visited: HashSet<String>,
       depth: usize,
   }
   ```
2. Modify expansion methods to accept context
3. Before expanding a variable:
   - Check if variable name is in `visited` set
   - If yes: return `CircularReference` error with variable name
   - If no: add to `visited` set
4. After expansion completes, remove from `visited` set
5. Implement detection algorithm:
   ```rust
   fn detect_circular_ref(&self, var_name: &str, context: &ExpansionContext) -> Result<(), ExpansionError>
   ```
6. Write unit tests for circular references:
   - Simple cycle: A → B → A
   - Complex cycle: A → B → C → A
   - Self-reference: A → A
   - Multiple independent cycles
7. Test error message clarity
8. Ensure expansion stops on cycle detection
9. Verify warnings collected before error
10. Run tests: `cargo test circular`

**Files Modified**:
- `src/expand/expander.rs`
- `src/error.rs` (improve error messages)

**Acceptance Criteria**:
- Circular references detected reliably
- Error messages include variable name and cycle info
- Tests cover all circular reference patterns
- No false positives

---

### 2.5: Implement Combined Expansion Entry Point

**Objective**: Create main expansion function for all value types

**Steps**:

1. Implement public method:
   ```rust
   pub fn expand_value(&mut self, value: &str) -> ExpansionResult
   ```
2. Algorithm:
   - Initialize depth counter to 0
   - Loop while depth < max_depth and string contains `{` or `${`:
     - First expand shell vars (`${}`)
     - Then expand env vars (`{}`)
     - Check if anything changed
     - If no changes: break (done)
     - Increment depth
   - If depth reached limit: return error
   - Return expanded string
3. Collect all warnings during expansion
4. Implement expansion for complex types:
   ```rust
   pub fn expand_string(&mut self, s: String) -> ExpansionResult
   pub fn expand_vec(&mut self, vec: Vec<String>) -> Result<Vec<String>, ExpansionError>
   pub fn expand_hashmap(&mut self, map: HashMap<String, String>) -> Result<HashMap<String, String>, ExpansionError>
   ```
5. Add method to retrieve warnings:
   ```rust
   pub fn take_warnings(&mut self) -> Vec<String>
   ```
6. Write integration tests:
   - Mixed shell and env vars
   - Complex nesting
   - All value types (string, vec, hashmap)
7. Test complete configuration expansion
8. Benchmark expansion performance
9. Optimize if necessary
10. Run all tests: `cargo test expand`

**Files Modified**:
- `src/expand/expander.rs`

**Acceptance Criteria**:
- Single entry point for all expansions
- All value types supported
- Warnings accessible after expansion
- Integration tests pass
- Performance acceptable (<1ms for typical config)

---

### 2.6: Integrate Expansion with Configuration Parser

**Objective**: Apply expansion to parsed configurations

**Steps**:

1. Update `src/config/parser.rs`
2. Add expansion step after TOML parsing:
   ```rust
   pub fn parse_and_expand_config(path: &Path) -> Result<MultiAgentConfig, MultiAgentError>
   ```
3. Algorithm:
   - Parse TOML file into config struct
   - Extract `[env]` section
   - Get shell environment variables
   - Create `Expander` instance
   - Expand all values in MCP server configurations:
     - `command` field (string)
     - `args` field (vec)
     - `env` field (hashmap)
     - `url` field (string)
     - `bearer_token` field (string)
   - Collect and log warnings
   - Return expanded configuration
4. Update validator to work with expanded config
5. Ensure expanded values replace original placeholders
6. Write integration test:
   - Create test config with variables
   - Set test environment variables
   - Parse and expand
   - Verify values expanded correctly
7. Test warning collection
8. Test error propagation
9. Update Phase 1 tests if needed
10. Run all tests: `cargo test`

**Files Modified**:
- `src/config/parser.rs`
- `src/config/mod.rs` (re-export new function)

**Acceptance Criteria**:
- Expansion integrated with parsing
- All server fields expanded
- Warnings logged appropriately
- Integration tests pass
- No regression in Phase 1 tests

---

### 2.7: Add Comprehensive Expansion Tests

**Objective**: Achieve 80% test coverage for expansion module

**Steps**:

1. Create `tests/expansion_tests.rs` for integration tests
2. Test scenarios from specification:
   - EC-6: Empty environment variable value
   - EC-7: Variable name with special characters
   - EC-8: Nested variable references (up to 10 levels)
   - EC-9: Variable reference in server name (not supported)
   - FM-8: Undefined environment variable
   - FM-9: Circular variable reference
   - FM-10: Undefined variable in [env] section
3. Create test fixtures:
   - `tests/fixtures/expansion/` directory
   - Various TOML configs with variable references
4. Test complete config expansion:
   - Real-world example from Appendix A
   - Multiple servers with different variable types
   - Mixed shell and env variables
5. Test error messages:
   - Verify format matches specification
   - Check suggestions are helpful
6. Test warning messages:
   - Verify warnings don't cause failure
   - Check warning format
7. Test performance:
   - Large number of variables (100+)
   - Deep nesting (approaching limit)
   - Should complete in <10ms
8. Run coverage: `cargo tarpaulin --lib --bins`
9. Aim for >= 80% coverage of expansion module
10. Document any uncovered code with justification

**Files Modified**:
- `tests/expansion_tests.rs`
- `tests/fixtures/expansion/*.toml`

**Acceptance Criteria**:
- All spec test cases covered
- 80% coverage of expansion module
- Integration tests pass
- Performance tests pass
- Coverage report generated

---

## Testing Strategy

**Unit Tests**:
- Shell variable expansion
- Env section variable expansion
- Circular reference detection
- Depth limit enforcement
- Warning collection
- Error handling

**Integration Tests**:
- Complete configuration expansion
- Mixed variable types
- Real-world configuration examples
- Error propagation

**Coverage Target**: 80% minimum for expansion module

---

## Files Created/Modified

### New Files
- `src/expand/mod.rs` - Expansion module
- `src/expand/expander.rs` - Expansion logic
- `tests/expansion_tests.rs` - Integration tests
- `tests/fixtures/expansion/*.toml` - Test fixtures

### Modified Files
- `src/config/parser.rs` - Integration with parser
- `src/error.rs` - Expansion errors
- `src/lib.rs` - Module exports
- `Cargo.toml` - Dependencies

---

## Dependencies Added

```toml
[dependencies]
regex = "1.10"
```

---

## Commit Strategy

Each subtask should result in one conventional commit:

1. `feat(expand): create variable expansion module`
2. `feat(expand): implement shell environment variable expansion`
3. `feat(expand): implement env section variable expansion`
4. `feat(expand): implement circular reference detection`
5. `feat(expand): implement combined expansion entry point`
6. `feat(config): integrate expansion with configuration parser`
7. `test(expand): add comprehensive expansion tests`

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Regex performance on large configs | Low | Benchmark and optimize if needed |
| Circular reference false positives | Medium | Thorough testing of detection algorithm |
| Undefined variable handling | Low | Follow spec exactly (warn, expand to empty) |
| Complex nesting edge cases | Medium | Comprehensive test matrix |

---

## Reference Implementation

Refer to Appendix C in architecture document for algorithm pseudo-code:
- `./architecture/16-appendices.md` Section C

---

## Phase Completion Checklist

- [ ] All subtasks completed and committed
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] Code coverage >= 80%
- [ ] No clippy warnings
- [ ] Code formatted with rustfmt
- [ ] Performance benchmarks pass
- [ ] Phase review completed
- [ ] Ready for Phase 3

---

## Next Phase

After Phase 2 completion, proceed to [Phase 3: Format Transformers](phase_3.md)
