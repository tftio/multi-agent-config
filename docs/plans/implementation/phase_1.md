# Phase 1: Project Foundation & Core Parsing

**Status**: Not Started
**Duration**: 2-3 days
**Dependencies**: None

---

## Overview

Establish the foundational data structures, TOML parsing capabilities, and schema validation for the multi-agent-config configuration manager. This phase creates the core types and validation logic that all subsequent phases will build upon.

## Goals

- Define Rust data structures matching the configuration schema
- Implement TOML parsing and deserialization using serde
- Implement comprehensive schema validation
- Set up basic CLI structure with clap
- Establish error handling patterns

## Success Criteria

- [ ] All configuration schema types defined and compilable
- [ ] TOML files can be parsed into Rust structures
- [ ] Schema validation catches all required field violations
- [ ] Schema validation catches type mismatches
- [ ] Basic CLI accepts commands and global options
- [ ] Error messages follow specification format
- [ ] Unit tests for parsing and validation pass

## Subtasks

### 1.1: Define Core Configuration Data Structures

**Objective**: Create Rust types matching the unified configuration schema

**Steps**:

1. Create `src/config/mod.rs` as main configuration module
2. Create `src/config/types.rs` for core data structures
3. Define `MultiAgentConfig` struct with sections:
   - `settings: Option<Settings>`
   - `env: Option<HashMap<String, String>>`
   - `mcp: McpConfig`
4. Define `Settings` struct:
   - `version: String` (required, validated as semver)
   - `default_targets: Option<Vec<String>>` (default: ["cursor", "opencode", "codex"])
5. Define `McpConfig` struct:
   - `servers: HashMap<String, ServerConfig>`
6. Define `ServerConfig` enum or struct supporting:
   - STDIO variant: `command`, `args`, `env`, `enabled`, `targets`, tool-specific fields
   - HTTP variant: `url`, `bearer_token`, `enabled`, `targets`
7. Define `ToolName` enum: `ClaudeCode`, `Cursor`, `Opencode`, `Codex`, `All`
8. Add serde derive macros: `#[derive(Debug, Clone, Serialize, Deserialize)]`
9. Add field-level serde attributes for optional fields with defaults
10. Add documentation comments for all public types

**Files Modified**:
- `src/config/mod.rs`
- `src/config/types.rs`
- `Cargo.toml` (add serde, toml dependencies)

**Acceptance Criteria**:
- Code compiles without warnings
- All types derive Debug, Clone, Serialize, Deserialize
- Documentation comments on all public items

---

### 1.2: Implement TOML Parsing

**Objective**: Parse TOML configuration files into Rust structures

**Steps**:

1. Create `src/config/parser.rs`
2. Implement `parse_config_file(path: &Path) -> Result<MultiAgentConfig, ConfigError>`
3. Handle file reading errors (not found, permission denied)
4. Handle TOML parsing errors with line numbers
5. Use `toml::from_str()` for deserialization
6. Add `ConfigError` enum in `src/error.rs`:
   - `FileNotFound(PathBuf)`
   - `PermissionDenied(PathBuf)`
   - `ParseError(String, usize)` (message, line number)
   - `ValidationError(String)`
7. Implement `Display` for `ConfigError` following spec format
8. Add helper function `read_file_utf8(path: &Path) -> Result<String, ConfigError>`
9. Write unit tests for parsing valid TOML
10. Write unit tests for parsing errors (invalid syntax, missing fields)

**Files Modified**:
- `src/config/parser.rs`
- `src/error.rs`
- `Cargo.toml` (ensure toml dependency)

**Acceptance Criteria**:
- Valid TOML files parse successfully
- Invalid TOML returns parse error with line number
- Missing files return FileNotFound error
- All error messages follow spec format

---

### 1.3: Implement Schema Validation

**Objective**: Validate parsed configuration against schema requirements

**Steps**:

1. Create `src/config/validator.rs`
2. Implement `validate_config(config: &MultiAgentConfig) -> Result<(), Vec<ValidationError>>`
3. Collect all validation errors (don't fail on first error)
4. Validate `settings.version`:
   - Check format matches regex: `^\d+\.\d+(\.\d+)?$`
   - Currently only "1.0" is supported
5. Validate `settings.default_targets`:
   - All tool names are valid (claude-code, cursor, opencode, codex, all)
   - Deduplicate and warn on duplicates
6. Validate MCP servers section:
   - At least one server must be defined
   - Each server has either `command` OR `url`, not both
   - STDIO servers (with `command`) have valid command field
   - HTTP servers (with `url`) have URL starting with http:// or https://
   - `bearer_token` only present with `url`
7. Validate server-level fields:
   - `targets` contains only valid tool names
   - Tool-specific fields logged when present for wrong tool (info level)
8. Validate field types are correct (handled by serde, but add custom validation)
9. Implement `validate_executable(command: &str) -> bool` (checks if executable exists)
   - Warn if command not found, but don't fail validation
10. Write unit tests for each validation rule

**Files Modified**:
- `src/config/validator.rs`
- `src/error.rs` (add `ValidationError` struct)

**Acceptance Criteria**:
- All validation rules from spec implemented
- Invalid configs return detailed error list
- Warnings don't cause validation failure
- Executable existence checked (warning only)

---

### 1.4: Set Up CLI Structure with Clap

**Objective**: Create basic CLI accepting commands and options

**Steps**:

1. Update `src/main.rs` to use clap
2. Define `Cli` struct with derive macros:
   ```rust
   #[derive(Parser)]
   #[command(name = "multi-agent-config")]
   #[command(version = env!("CARGO_PKG_VERSION"))]
   #[command(about = "Unified configuration manager for AI coding tools")]
   ```
3. Add global options:
   - `--config <path>`: Config file path (default: `~/.config/multi-agent-config/config.toml`)
   - `--verbose`: Enable verbose logging
4. Define `Commands` enum with variants:
   - `Init { force: bool }` - Create template config
   - `Validate` - Validate config without writing
   - `Compile { tool: Vec<String>, dry_run: bool }` - Generate configs
   - `Diff { tool: Vec<String> }` - Show what would change
   - `Version` - Show version information
   - `License` - Show license information
   - `Doctor` - Health check and diagnostics
5. Implement command dispatch in main:
   ```rust
   match cli.command {
       Commands::Validate => validate_command(&config_path),
       Commands::Version => version_command(),
       // ... other commands as stubs
   }
   ```
6. Implement stub functions returning "Not yet implemented" for Phase 5 commands
7. Add `is_terminal` crate for TTY detection
8. Add colored output support (but respect TTY detection)
9. Create `src/cli/mod.rs` for CLI-related code
10. Write integration test that CLI binary accepts all commands

**Files Modified**:
- `src/main.rs`
- `src/cli/mod.rs`
- `Cargo.toml` (add clap, is-terminal, colored dependencies)

**Acceptance Criteria**:
- CLI binary compiles and runs
- `--help` shows all commands and options
- `--version` displays version from Cargo.toml
- All commands parse correctly (even if not implemented)
- TTY detection works correctly

---

### 1.5: Establish Error Handling Patterns

**Objective**: Create consistent error handling framework

**Steps**:

1. Update `src/error.rs` as main error module
2. Define `MultiAgentError` enum covering all error categories:
   - Config errors (parse, validate, file I/O)
   - Environment variable errors
   - Transformation errors
   - File operation errors
   - CLI errors
3. Implement `Display` for all error types following spec format:
   ```
   Error: <summary>
   <details>
   <suggestion>
   ```
4. Implement `From` conversions for common error types:
   - `std::io::Error -> MultiAgentError`
   - `toml::de::Error -> MultiAgentError`
5. Create helper macros for common error patterns
6. Define exit codes as constants matching spec:
   ```rust
   pub const EXIT_SUCCESS: i32 = 0;
   pub const EXIT_VALIDATION_ERROR: i32 = 1;
   pub const EXIT_FILE_ERROR: i32 = 2;
   pub const EXIT_PARTIAL_FAILURE: i32 = 3;
   pub const EXIT_LOCK_ERROR: i32 = 4;
   ```
7. Create `src/cli/output.rs` for formatted output functions:
   - `print_error(error: &MultiAgentError)`
   - `print_warning(message: &str)`
   - `print_info(message: &str)`
   - `print_success(message: &str)`
8. Respect TTY detection (no colors in non-TTY)
9. Write unit tests for error formatting
10. Write unit tests for exit code mapping

**Files Modified**:
- `src/error.rs`
- `src/cli/output.rs`
- `src/lib.rs` (export error types)

**Acceptance Criteria**:
- All error types implement Display with correct format
- Error messages include suggestions where appropriate
- Exit codes match specification
- Output respects TTY detection
- Error tests pass

---

### 1.6: Create Initial Unit Tests

**Objective**: Establish testing patterns and test core functionality

**Steps**:

1. Create `src/config/types_test.rs` (or use `#[cfg(test)]` modules)
2. Test TOML deserialization with valid configs:
   - Minimal config (required fields only)
   - Complete config (all fields)
   - Mixed STDIO and HTTP servers
3. Test TOML deserialization with invalid configs:
   - Missing required fields (expect error)
   - Wrong field types (expect error)
   - Both command and url present (expect error via validation)
4. Test schema validation:
   - Valid configs pass
   - Invalid version format fails
   - Unknown tool names in targets fail
   - Servers with no command or url fail
5. Test error formatting:
   - Each error type formats correctly
   - Multi-line error messages work
6. Test CLI parsing:
   - Global options parse correctly
   - Commands parse with correct arguments
7. Use `tempfile` crate for temporary test files
8. Use `assert_matches!` or similar for enum matching
9. Aim for 80% coverage of phase 1 code
10. Run tests: `cargo test`

**Files Modified**:
- `src/config/types.rs` (add `#[cfg(test)]` module)
- `src/config/parser.rs` (add `#[cfg(test)]` module)
- `src/config/validator.rs` (add `#[cfg(test)]` module)
- `Cargo.toml` (add tempfile as dev-dependency)

**Acceptance Criteria**:
- All tests pass: `cargo test`
- Coverage >= 80% for phase 1 modules
- Tests cover both success and failure cases
- Tests use temporary files (no hardcoded paths)

---

### 1.7: Update Dependencies and Run Quality Checks

**Objective**: Ensure code quality and dependency compliance

**Steps**:

1. Review `Cargo.toml` dependencies:
   - serde = { version = "1.0", features = ["derive"] }
   - toml = "0.8"
   - clap = { version = "4.5", features = ["derive"] }
   - is-terminal = "0.4"
   - colored = "2.1"
   - anyhow = "1.0"
   - thiserror = "1.0"
2. Add dev-dependencies:
   - tempfile = "3.8"
3. Run `cargo fmt` to format code
4. Run `cargo clippy -- -D warnings` to check for lints
5. Run `cargo test` to ensure all tests pass
6. Run `cargo build` to ensure project builds
7. Run `cargo audit` to check for security issues
8. Run `cargo deny check` for dependency compliance
9. Fix any warnings or errors
10. Commit changes with conventional commit message

**Files Modified**:
- `Cargo.toml`
- `Cargo.lock`
- All source files (formatting)

**Acceptance Criteria**:
- `cargo fmt` produces no changes
- `cargo clippy` produces no warnings
- `cargo test` all tests pass
- `cargo build` succeeds
- `cargo audit` shows no vulnerabilities
- No dependency compliance issues

---

## Testing Strategy

**Unit Tests**:
- Configuration type deserialization
- TOML parsing (valid and invalid)
- Schema validation rules
- Error formatting
- CLI argument parsing

**Integration Tests**: (Deferred to Phase 5)
- CLI commands execute correctly

**Coverage Target**: 80% minimum for all phase 1 modules

---

## Files Created/Modified

### New Files
- `src/config/mod.rs` - Configuration module
- `src/config/types.rs` - Core data structures
- `src/config/parser.rs` - TOML parsing
- `src/config/validator.rs` - Schema validation
- `src/error.rs` - Error types
- `src/cli/mod.rs` - CLI module
- `src/cli/output.rs` - Formatted output

### Modified Files
- `src/main.rs` - CLI entry point
- `src/lib.rs` - Library exports
- `Cargo.toml` - Dependencies

---

## Dependencies Added

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
clap = { version = "4.5", features = ["derive"] }
is-terminal = "0.4"
colored = "2.1"
anyhow = "1.0"
thiserror = "1.0"

[dev-dependencies]
tempfile = "3.8"
```

---

## Commit Strategy

Each subtask should result in one conventional commit:

1. `feat(config): add core configuration data structures`
2. `feat(config): implement TOML parsing`
3. `feat(config): implement schema validation`
4. `feat(cli): set up CLI structure with clap`
5. `feat(error): establish error handling patterns`
6. `test(config): add unit tests for core parsing`
7. `chore(deps): add dependencies and run quality checks`

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Complex serde deserialization | Medium | Start simple, add complexity incrementally |
| Validation logic bugs | High | Comprehensive unit tests for each validation rule |
| Error message clarity | Medium | Follow spec examples exactly, get early feedback |

---

## Phase Completion Checklist

- [ ] All subtasks completed and committed
- [ ] All unit tests passing
- [ ] Code coverage >= 80%
- [ ] No clippy warnings
- [ ] Code formatted with rustfmt
- [ ] Dependencies audited
- [ ] Phase review completed
- [ ] Ready for Phase 2

---

## Next Phase

After Phase 1 completion, proceed to [Phase 2: Environment Variable Expansion](phase_2.md)
