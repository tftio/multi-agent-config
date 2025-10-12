# Phase 5: CLI Commands Implementation

**Status**: Not Started
**Duration**: 2-3 days
**Dependencies**: Phase 1-4 (all core functionality must be complete)

---

## Overview

Implement all CLI commands (init, validate, compile, diff) and standard subcommands (version, help, license, doctor). This phase brings together all previous work into a complete user-facing tool with comprehensive error handling and user-friendly output.

## Goals

- Implement `init` command with template generation
- Implement `validate` command with detailed reporting
- Implement `compile` command with dry-run support
- Implement `diff` command with formatted output
- Implement standard subcommands (version, license, doctor)
- Add shell completion generation
- Create comprehensive CLI integration tests

## Success Criteria

- [ ] All commands work as specified
- [ ] Error messages helpful and actionable
- [ ] Exit codes match specification
- [ ] TTY detection works correctly
- [ ] Dry-run mode works without side effects
- [ ] Integration tests pass for all commands
- [ ] CLI follows Rust standards

## Subtasks

### 5.1: Implement `init` Command

**Objective**: Create template configuration file

**Steps**:

1. Create `src/cli/commands/init.rs`
2. Implement init command handler:
   ```rust
   pub fn init_command(
       config_path: &Path,
       force: bool
   ) -> Result<(), MultiAgentError>
   ```
3. Algorithm (spec Section 4.3.3.1):
   - Check if config file exists
   - If exists and no --force: return error with message
   - If exists and --force:
     - Create backup: `<path>.backup.<timestamp>`
     - Log backup location
   - Create directory `~/.config/multi-agent-config/` if not exists
   - Generate template configuration
   - Write template to config file
   - Print success message with path
4. Create template configuration:
   - Based on example in spec Appendix A
   - Include commented examples for all features
   - Include documentation comments
   - Use placeholder values for tokens
5. Template structure:
   ```toml
   [settings]
   version = "1.0"
   default_targets = ["cursor", "opencode", "codex"]

   [env]
   # GITHUB_TOKEN = "${GITHUB_PERSONAL_ACCESS_TOKEN}"

   [mcp.servers.example-server]
   command = "npx"
   args = ["-y", "package-name"]
   # enabled = true
   # targets = ["all"]

   # ... more examples
   ```
6. Implement force confirmation (if interactive TTY):
   - Prompt user: "Overwrite existing config? [y/N]"
   - Read response from stdin
   - Only proceed if 'y' or 'Y'
7. Write unit tests:
   - Init with no existing config (success)
   - Init with existing config, no force (error)
   - Init with existing config, with force (creates backup)
   - Verify template content
   - Verify directory creation
8. Write integration test:
   - Run binary with init command
   - Verify config file created
   - Verify template content correct
9. Test exit codes match spec
10. Run tests: `cargo test init`

**Files Modified**:
- `src/cli/commands/init.rs`
- `src/cli/commands/mod.rs` (export)
- `src/main.rs` (wire up command)

**Acceptance Criteria**:
- Template config created successfully
- Force option works with backup
- Error messages clear and helpful
- Template includes examples and documentation
- Tests pass
- Exit codes correct

---

### 5.2: Implement `validate` Command

**Objective**: Validate configuration without writing files

**Steps**:

1. Create `src/cli/commands/validate.rs`
2. Implement validate command handler:
   ```rust
   pub fn validate_command(
       config_path: &Path,
       verbose: bool
   ) -> Result<(), MultiAgentError>
   ```
3. Algorithm (spec Section 4.3.3.2):
   - Load configuration file (handle not found)
   - Parse TOML (report syntax errors with line numbers)
   - Validate schema (collect all errors)
   - Expand environment variables (collect warnings)
   - Check for circular references
   - Validate tool names in targets
   - Simulate transformation for each tool (validation only)
   - Check for executable existence (warnings only)
   - Report validation results
4. Output format:
   - On success: "✅ Configuration is valid"
   - On warnings: list warnings but exit 0
   - On errors: list all errors and exit 1
5. Implement detailed error reporting:
   ```
   Error: Invalid configuration

   [1] Server 'github': Missing required field 'command' or 'url'
   [2] Settings: Invalid version format '1.x'
   [3] Server 'test': Unknown target tool 'vscode'

   Found 3 errors. Please fix and re-run validation.
   ```
6. Implement warning reporting:
   ```
   Warning: Undefined environment variable: MISSING_TOKEN
   Warning: Executable not found: /path/to/missing (server 'test')

   Configuration is valid with 2 warnings.
   ```
7. Add verbose mode:
   - Show all validated servers
   - Show expansion details
   - Show target filtering results
8. Write unit tests:
   - Valid config (success)
   - Invalid syntax (parse error with line)
   - Missing required fields (validation error)
   - Circular references (error)
   - Undefined env vars (warning)
9. Write integration test:
   - Run with valid config
   - Run with invalid configs
   - Verify exit codes
10. Run tests: `cargo test validate`

**Files Modified**:
- `src/cli/commands/validate.rs`
- `src/cli/commands/mod.rs` (export)
- `src/main.rs` (wire up command)

**Acceptance Criteria**:
- All validation steps executed
- Error messages detailed and actionable
- Warnings don't cause failure
- Verbose mode provides detail
- Exit codes correct
- Tests pass

---

### 5.3: Implement `compile` Command

**Objective**: Generate and write tool configurations

**Steps**:

1. Create `src/cli/commands/compile.rs`
2. Implement compile command handler:
   ```rust
   pub fn compile_command(
       config_path: &Path,
       tools: Vec<ToolName>,
       dry_run: bool,
       verbose: bool
   ) -> Result<(), MultiAgentError>
   ```
3. Algorithm (spec Section 4.3.3.3):
   - Load and validate configuration (reuse validate logic)
   - Determine target tools:
     - If --tool specified: use those tools only
     - If not specified: use all tools with matching servers
   - Initialize state tracker
   - For each target tool:
     - Filter servers by targets field
     - Skip if no servers match this tool
     - Transform to tool-specific format
     - Determine output path for tool
     - Generate diff (for logging)
     - If not --dry-run:
       - Execute safe write with backup
       - Update state tracker
     - If --dry-run:
       - Print what would be written
       - Show diff
   - Print summary of changes
4. Implement progress reporting (when TTY):
   ```
   Compiling configurations...
   ✅ cursor: wrote .cursor/mcp.json (3 servers)
   ✅ opencode: wrote ~/.config/opencode/opencode.json (3 servers)
   ✅ codex: wrote ~/.codex/config.toml (4 servers)
   ⚠️  claude-code: CLI not available, skipped

   Summary: 3 tools updated, 1 skipped
   ```
5. Handle partial failures:
   - If some tools succeed and some fail: exit code 3
   - Show which tools succeeded
   - Show which tools failed with reasons
6. Implement --dry-run mode:
   - No file modifications
   - Show all diffs
   - Show what would be done
   - Exit code 0 if validation passes
7. Add verbose mode:
   - Show server filtering details
   - Show transformation details
   - Show file paths
8. Write unit tests:
   - Compile all tools (success)
   - Compile specific tool (--tool)
   - Dry-run mode (no writes)
   - Partial failure handling
   - No servers for any tool (warning)
9. Write integration tests:
   - Run compile on test config
   - Verify files written
   - Verify content correct
   - Verify backups created
   - Verify state updated
   - Test dry-run (no writes)
10. Run tests: `cargo test compile`

**Files Modified**:
- `src/cli/commands/compile.rs`
- `src/cli/commands/mod.rs` (export)
- `src/main.rs` (wire up command)

**Acceptance Criteria**:
- All tools compiled successfully
- Partial failures handled correctly
- Dry-run works without side effects
- Progress reporting clear
- Exit codes correct
- Tests pass

---

### 5.4: Implement `diff` Command

**Objective**: Show what would change when compiling

**Steps**:

1. Create `src/cli/commands/diff.rs`
2. Implement diff command handler:
   ```rust
   pub fn diff_command(
       config_path: &Path,
       tools: Vec<ToolName>,
       verbose: bool
   ) -> Result<(), MultiAgentError>
   ```
3. Algorithm (spec Section 4.3.3.4):
   - Load and validate configuration
   - Determine target tools (same logic as compile)
   - For each target tool:
     - Generate new configuration
     - Read existing configuration if present
     - Generate diff
     - Display diff with formatting
4. Output format (from spec):
   ```
   ================================================================================
   Tool: cursor
   Path: /Users/jfb/Projects/.cursor/mcp.json
   ================================================================================
   [NEW FILE]
   {
     "mcpServers": {
       "github": { ... }
     }
   }

   ================================================================================
   Tool: opencode
   Path: /Users/jfb/.config/opencode/opencode.json
   ================================================================================
   --- current
   +++ new
   @@ -1,5 +1,6 @@
    {
      "mcp": {
        "github": { ... },
   +    "context7": { ... }
      }
    }
   ```
5. Add color support (when TTY):
   - Green for additions
   - Red for deletions
   - Cyan for section headers
6. Handle edge cases:
   - File doesn't exist (show [NEW FILE])
   - File identical (show "No changes")
   - Multiple tools (show all diffs)
7. Add summary at end:
   ```
   Summary:
   cursor: new file
   opencode: 1 addition
   codex: no changes
   ```
8. Write unit tests:
   - Diff for new file
   - Diff for modified file
   - Diff for identical file
   - Diff for multiple tools
9. Write integration test:
   - Run diff command
   - Verify output format
   - Verify no files modified
10. Run tests: `cargo test diff`

**Files Modified**:
- `src/cli/commands/diff.rs`
- `src/cli/commands/mod.rs` (export)
- `src/main.rs` (wire up command)

**Acceptance Criteria**:
- Diff output matches specification
- Colors work in TTY
- No color in pipes
- All edge cases handled
- Tests pass

---

### 5.5: Implement Standard Subcommands

**Objective**: Implement version, license, doctor, completions subcommands

**Steps**:

1. Create `src/cli/commands/standard.rs`
2. Implement `version` command:
   ```rust
   pub fn version_command()
   ```
   - Print tool name and version from Cargo.toml
   - Format: "multi-agent-config 0.1.0"
   - Exit code: 0
3. Implement `license` command:
   ```rust
   pub fn license_command()
   ```
   - Print license information
   - Read from LICENSE file or embed at compile time
   - Exit code: 0
4. Implement `doctor` command:
   ```rust
   pub fn doctor_command(config_path: &Path)
   ```
   - Check if config file exists
   - Check if config directory writable
   - Check if state directory writable
   - Check for tool executables (claude, npx, etc.)
   - Check for environment variables referenced in config
   - Report health status with ✅ or ❌
   - Suggest fixes for problems
5. Doctor output format:
   ```
   Running health checks...

   ✅ Configuration file: ~/.config/multi-agent-config/config.toml
   ✅ State directory: ~/.config/multi-agent-config/state
   ✅ Config directory writable
   ⚠️  Claude CLI not found: 'claude' command not available
   ✅ NPX executable found: /usr/local/bin/npx

   Health: 4 passed, 1 warning
   ```
6. Implement `completions` command:
   ```rust
   pub fn completions_command(shell: clap_complete::Shell)
   ```
   - Generate shell completions for bash, zsh, fish, etc.
   - Use clap_complete::generate
   - Output to stdout
7. Add completions to CLI args:
   ```rust
   #[derive(Subcommand)]
   enum Commands {
       // ...
       Completions {
           #[arg(value_enum)]
           shell: clap_complete::Shell,
       },
   }
   ```
8. Write unit tests for each subcommand
9. Write integration tests:
   - Test version output format
   - Test license output
   - Test doctor with various conditions
   - Test completions generation
10. Run tests: `cargo test standard`

**Files Modified**:
- `src/cli/commands/standard.rs`
- `src/cli/commands/mod.rs` (export)
- `src/main.rs` (wire up commands)
- `Cargo.toml` (add clap_complete dependency)

**Acceptance Criteria**:
- Version displays correctly
- License displays correctly
- Doctor performs comprehensive checks
- Completions generate for all shells
- Tests pass

---

### 5.6: Wire Up All Commands in Main

**Objective**: Connect all commands to CLI entry point

**Steps**:

1. Update `src/main.rs`
2. Implement main function:
   ```rust
   fn main() -> ExitCode {
       let cli = Cli::parse();

       let result = match cli.command {
           Commands::Init { force } => init_command(&cli.config, force),
           Commands::Validate => validate_command(&cli.config, cli.verbose),
           Commands::Compile { tool, dry_run } => compile_command(&cli.config, tool, dry_run, cli.verbose),
           Commands::Diff { tool } => diff_command(&cli.config, tool, cli.verbose),
           Commands::Version => { version_command(); Ok(()) },
           Commands::License => { license_command(); Ok(()) },
           Commands::Doctor => doctor_command(&cli.config),
           Commands::Completions { shell } => { completions_command(shell); Ok(()) },
       };

       match result {
           Ok(()) => ExitCode::SUCCESS,
           Err(e) => {
               print_error(&e);
               ExitCode::from(e.exit_code())
           }
       }
   }
   ```
3. Implement exit code mapping:
   ```rust
   impl MultiAgentError {
       pub fn exit_code(&self) -> u8 {
           match self {
               // Map errors to exit codes from spec
           }
       }
   }
   ```
4. Add error display logic:
   - Use colored output when TTY
   - Format according to spec (Error: / Warning: / Info:)
   - Include suggestions from error
5. Test all commands via binary:
   - `cargo run -- init`
   - `cargo run -- validate`
   - `cargo run -- compile`
   - `cargo run -- diff`
   - etc.
6. Verify help text:
   - `cargo run -- --help`
   - `cargo run -- init --help`
   - etc.
7. Add global error handler:
   - Catch panics and display gracefully
   - Log to stderr
8. Add signal handling (Ctrl+C):
   - Clean up temp files
   - Print interruption message
9. Test in both TTY and non-TTY contexts
10. Verify all exit codes correct

**Files Modified**:
- `src/main.rs`
- `src/error.rs` (exit code mapping)

**Acceptance Criteria**:
- All commands accessible via CLI
- Error handling consistent
- Exit codes match specification
- Help text clear and complete
- Signal handling works

---

### 5.7: Create Comprehensive CLI Integration Tests

**Objective**: Test complete CLI workflows end-to-end

**Steps**:

1. Create `tests/cli_integration_tests.rs`
2. Test complete workflow:
   - Init config
   - Validate config
   - Compile configs
   - Verify files created
   - Run diff
   - Modify config
   - Recompile
   - Verify updates
3. Test init command:
   - Create new config
   - Verify template content
   - Test force flag
   - Test backup creation
4. Test validate command:
   - Valid config (exit 0)
   - Invalid config (exit 1)
   - Config with warnings (exit 0 with warnings)
5. Test compile command:
   - Compile all tools
   - Compile specific tool
   - Dry-run mode
   - Verify file contents
   - Verify backups created
   - Verify state updated
6. Test diff command:
   - Diff on new file
   - Diff on modified file
   - Diff on unchanged file
7. Test error handling:
   - Config file not found
   - Invalid TOML syntax
   - Validation errors
   - File permission errors
8. Test exit codes for all scenarios
9. Test with various configurations:
   - Minimal config
   - Complete config from spec
   - Config with errors
10. Run comprehensive test suite: `cargo test --test cli_integration_tests`

**Files Modified**:
- `tests/cli_integration_tests.rs`
- `tests/fixtures/` (various test configs)

**Acceptance Criteria**:
- All CLI commands tested end-to-end
- All exit codes verified
- Error scenarios covered
- Tests use temporary directories
- Tests pass on Unix and Windows

---

## Testing Strategy

**Unit Tests**:
- Each command handler
- Error formatting
- Exit code mapping
- Subcommands

**Integration Tests**:
- Complete CLI workflows
- File operations via CLI
- Error scenarios
- Exit codes

**Coverage Target**: 80% minimum for CLI module

---

## Files Created/Modified

### New Files
- `src/cli/commands/init.rs` - Init command
- `src/cli/commands/validate.rs` - Validate command
- `src/cli/commands/compile.rs` - Compile command
- `src/cli/commands/diff.rs` - Diff command
- `src/cli/commands/standard.rs` - Standard subcommands
- `src/cli/commands/mod.rs` - Commands module
- `tests/cli_integration_tests.rs` - Integration tests

### Modified Files
- `src/main.rs` - Wire up all commands
- `src/error.rs` - Exit code mapping
- `src/cli/mod.rs` - CLI module organization
- `Cargo.toml` - Dependencies

---

## Dependencies Added

```toml
[dependencies]
clap_complete = "4.5"
```

---

## Commit Strategy

Each subtask should result in one conventional commit:

1. `feat(cli): implement init command`
2. `feat(cli): implement validate command`
3. `feat(cli): implement compile command`
4. `feat(cli): implement diff command`
5. `feat(cli): implement standard subcommands`
6. `feat(cli): wire up all commands in main`
7. `test(cli): create comprehensive CLI integration tests`

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Complex CLI argument parsing | Low | Use clap with derive macros |
| User experience issues | Medium | Follow spec examples exactly, get early feedback |
| Exit code mistakes | Low | Comprehensive testing of all scenarios |
| Platform-specific CLI behavior | Low | Test on Unix and Windows |

---

## Reference Specifications

- CLI interface: `./architecture/04-input-specifications.md` Section 4.3
- Error handling: `./architecture/13-error-handling-specifications.md`
- Exit codes: Section 13.2

---

## Phase Completion Checklist

- [ ] All subtasks completed and committed
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] Code coverage >= 80%
- [ ] No clippy warnings
- [ ] Code formatted with rustfmt
- [ ] All commands working via binary
- [ ] Help text complete and clear
- [ ] Exit codes verified
- [ ] Phase review completed
- [ ] Ready for Phase 6

---

## Next Phase

After Phase 5 completion, proceed to [Phase 6: Testing & Documentation](phase_6.md)
