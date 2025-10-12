# Phase 6: Testing & Documentation

**Status**: Not Started
**Duration**: 3-4 days
**Dependencies**: Phase 1-5 (all implementation must be complete)

---

## Overview

Complete the project with comprehensive testing to achieve 80% coverage, thorough documentation, security audits, cross-platform verification, and final release preparation. This phase ensures the tool is production-ready, well-documented, and maintainable.

## Goals

- Achieve 80% minimum test coverage across all modules
- Complete edge case and failure mode testing
- Write comprehensive user documentation
- Create example configurations
- Perform security audit
- Verify cross-platform compatibility
- Prepare for initial release

## Success Criteria

- [ ] Test coverage >= 80% (measured by cargo-tarpaulin)
- [ ] All edge cases from spec tested
- [ ] All failure modes from spec tested
- [ ] README complete with examples
- [ ] Example configurations provided
- [ ] Security requirements verified
- [ ] Tests pass on macOS, Linux, Windows
- [ ] No critical clippy warnings
- [ ] No security vulnerabilities (cargo-audit)
- [ ] Ready for v0.1.0 release

## Subtasks

### 6.1: Achieve 80% Test Coverage

**Objective**: Ensure all modules meet minimum coverage requirements

**Steps**:

1. Install cargo-tarpaulin (Linux only) or use GitHub Actions
2. Run coverage report:
   ```bash
   cargo tarpaulin --all --out Html --engine llvm --timeout 300
   ```
3. Review coverage report for each module:
   - config: >= 80%
   - expand: >= 80%
   - transform: >= 80%
   - file_ops: >= 80%
   - cli: >= 80%
4. Identify uncovered code paths:
   - Error handling branches
   - Edge cases
   - Platform-specific code
5. Add tests for uncovered code:
   - Unit tests for missing branches
   - Integration tests for workflows
   - Error scenario tests
6. Document intentionally untested code:
   - Platform-specific code (test separately)
   - Panic handlers
   - Unreachable code (with justification)
7. Run coverage again and verify >= 80%
8. Add coverage badge to README
9. Set up codecov.io or similar for CI
10. Document coverage target in CLAUDE.md

**Files Modified**:
- Tests in all modules
- `README.md` (coverage badge)
- `.github/workflows/ci.yml` (coverage reporting)

**Acceptance Criteria**:
- Overall coverage >= 80%
- Each module >= 80% coverage
- Uncovered code documented
- Coverage report in CI

---

### 6.2: Test All Edge Cases from Specification

**Objective**: Verify handling of all edge cases (Section 9 of spec)

**Steps**:

1. Create `tests/edge_cases_tests.rs`
2. Test each edge case from spec:
   - **EC-1**: Empty configuration → generates minimal output
   - **EC-2**: Server with empty args → args: []
   - **EC-3**: Server with empty env → env omitted
   - **EC-4**: Targets contains "all" and specific tools → "all" takes precedence
   - **EC-5**: Duplicate server names → TOML parser handles
   - **EC-6**: Environment variable with empty value → expand to empty
   - **EC-7**: Variable name with special characters → attempt expansion
   - **EC-8**: Nested variable references → resolve up to depth 10
   - **EC-9**: Variable reference in server name → treat as literal
   - **EC-10**: Output path is symlink → follow symlink
   - **EC-11**: Output path is directory → error
   - **EC-12**: Output directory is read-only → error
   - **EC-13**: Very long file path → handle or error
   - **EC-14**: Special characters in path → sanitize
   - **EC-15**: Cursor project has no .cursor directory → create it
   - **EC-16**: opencode.ai existing config has unrelated settings → preserve
   - **EC-17**: Codex existing config has comments → warn about loss
   - **EC-18**: HTTP server for Cursor → skip with warning
   - **EC-19**: Multiple --tool flags → process all
   - **EC-20**: --tool repeated with same value → deduplicate
   - **EC-21**: Dry-run with no output → report no changes
   - **EC-22**: Diff on non-existent file → show [NEW FILE]
   - **EC-23**: Config contains non-ASCII characters → preserve
   - **EC-24**: Config contains Windows line endings → accept
   - **EC-25**: Config contains NUL bytes → error
   - **EC-26**: State file corrupted → recreate
   - **EC-27**: State file references non-existent file → warn
   - **EC-28**: Generated file modified by user → overwrite with warning
3. For each edge case:
   - Create test fixture if needed
   - Implement test
   - Verify behavior matches spec
   - Document test in comments
4. Group tests by category (config, env, file system, CLI, etc.)
5. Run all edge case tests: `cargo test edge_cases`
6. Verify all 28 edge cases covered
7. Document any edge cases not testable (with reason)
8. Create test report summarizing edge case coverage
9. Review with spec document
10. Update spec if implementation differs (document reason)

**Files Modified**:
- `tests/edge_cases_tests.rs`
- Test fixtures as needed

**Acceptance Criteria**:
- All 28 edge cases tested
- Behavior matches specification
- Tests documented
- Edge case report created

---

### 6.3: Test All Failure Modes from Specification

**Objective**: Verify handling of all failure modes (Section 8 of spec)

**Steps**:

1. Create `tests/failure_modes_tests.rs`
2. Test each failure mode from spec:
   - **FM-1**: Config file not found → error + suggestion
   - **FM-2**: Config file unreadable → permission error
   - **FM-3**: Invalid TOML syntax → parse error with line
   - **FM-4**: Missing required section → validation error
   - **FM-5**: Missing required field → validation error
   - **FM-6**: Invalid field type → type error
   - **FM-7**: Both command and URL present → validation error
   - **FM-8**: Undefined environment variable → warning
   - **FM-9**: Circular variable reference → error
   - **FM-10**: Undefined variable in [env] → warning
   - **FM-11**: Output directory not exists → create it
   - **FM-12**: Output file unwritable → permission error
   - **FM-13**: Disk full → ENOSPC error (mock)
   - **FM-14**: Backup creation failed → abort
   - **FM-15**: Claude CLI not available → warning, skip
   - **FM-16**: Claude CLI command failed → error
   - **FM-17**: Invalid JSON generated → error (bug)
   - **FM-18**: Invalid TOML generated → error (bug)
   - **FM-19**: Config modified during execution → warning
   - **FM-20**: Output file modified during execution → error
   - **FM-21**: Lock file exists → error (if implemented)
   - **FM-22**: Invalid tool name → error
   - **FM-23**: Invalid targets combination → warning
   - **FM-24**: No servers for any tool → warning
   - **FM-25**: Executable not found → warning
3. For each failure mode:
   - Create test scenario
   - Trigger failure condition
   - Verify error message format
   - Verify exit code
   - Verify suggestions provided
4. Mock file system errors where necessary:
   - Permission denied
   - Disk full
   - Read-only filesystem
5. Use temporary directories for file tests
6. Verify cleanup on failures
7. Run all failure mode tests: `cargo test failure_modes`
8. Create failure mode coverage report
9. Document any failure modes not testable
10. Verify error messages match spec format

**Files Modified**:
- `tests/failure_modes_tests.rs`
- Test fixtures and mocks as needed

**Acceptance Criteria**:
- All 25 failure modes tested
- Error messages match specification
- Exit codes correct
- Cleanup verified
- Failure mode report created

---

### 6.4: Write Comprehensive README

**Objective**: Create user-friendly documentation

**Steps**:

1. Update `README.md` with complete documentation
2. Structure:
   ```markdown
   # multi-agent-config

   [Brief description]

   ## Features
   [Key features]

   ## Installation
   [Installation instructions]

   ## Quick Start
   [Minimal example]

   ## Configuration
   [Configuration guide]

   ## Commands
   [Command reference]

   ## Examples
   [Real-world examples]

   ## Troubleshooting
   [Common issues]

   ## Development
   [Contribution guide]

   ## License
   [License info]
   ```
3. Write clear feature list:
   - Single TOML config for all tools
   - Environment variable expansion
   - Atomic file writes with backups
   - Diff preview before changes
   - State tracking
   - Cross-platform support
4. Installation section:
   - From crates.io: `cargo install multi-agent-config`
   - From source: `cargo install --path .`
   - Binary releases (GitHub)
5. Quick start with minimal example:
   ```bash
   # Initialize config
   multi-agent-config init

   # Edit ~/.config/multi-agent-config/config.toml

   # Validate config
   multi-agent-config validate

   # Preview changes
   multi-agent-config diff

   # Apply changes
   multi-agent-config compile
   ```
6. Configuration guide:
   - TOML structure explanation
   - Field descriptions
   - Environment variable usage
   - Target tool selection
7. Command reference:
   - Each command with flags
   - Examples for each
   - Exit codes
8. Real-world examples:
   - GitHub MCP server
   - Multiple servers
   - Tool-specific configuration
9. Troubleshooting section:
   - Common errors
   - Solutions
   - How to get help
10. Add badges: build status, coverage, crates.io

**Files Modified**:
- `README.md`

**Acceptance Criteria**:
- README complete and clear
- Examples work correctly
- Installation instructions accurate
- Troubleshooting helpful
- Badges added

---

### 6.5: Create Example Configurations

**Objective**: Provide real-world configuration examples

**Steps**:

1. Create `examples/` directory
2. Create example configurations:
   - `examples/minimal.toml` - Simplest valid config
   - `examples/complete.toml` - Full-featured config (from spec Appendix A)
   - `examples/github-only.toml` - Single server example
   - `examples/multi-tool.toml` - Different servers for different tools
   - `examples/http-servers.toml` - HTTP MCP servers
   - `examples/with-secrets.toml` - Environment variable usage
3. For each example:
   - Add comments explaining configuration
   - Include documentation of fields
   - Show common patterns
   - Test that example is valid
4. Create `examples/README.md` explaining examples
5. Add validation tests for examples:
   ```rust
   #[test]
   fn validate_example_configs() {
       for example in glob("examples/*.toml") {
           assert!(validate_config(example).is_ok());
       }
   }
   ```
6. Document expected output for each example
7. Create screenshot or sample output
8. Link examples from main README
9. Test examples work with actual tools
10. Keep examples updated with spec

**Files Modified**:
- `examples/*.toml` (new files)
- `examples/README.md` (new file)
- `README.md` (link to examples)
- `tests/example_validation.rs` (new file)

**Acceptance Criteria**:
- Multiple example configs provided
- Examples well-documented
- Examples validated in tests
- Examples linked from README

---

### 6.6: Perform Security Audit

**Objective**: Verify all security requirements met

**Steps**:

1. Review security requirements from spec Section 10
2. Verify SEC-1: Environment variables expanded only in memory
   - Audit code for logging/display of env vars
   - Verify no plaintext storage
3. Verify SEC-2: Error messages don't include env var values
   - Review all error messages
   - Test error scenarios
4. Verify SEC-3: Dry-run and diff redact credentials
   - Test with token values
   - Verify redaction patterns work
5. Verify SEC-4: Config files created with mode 0600
   - Test file permissions after creation
   - Unix only (Windows uses default ACLs)
6. Verify SEC-5: Backup files preserve permissions
   - Test backup permission preservation
7. Verify SEC-6: State files created with mode 0600
   - Test state file permissions
8. Verify SEC-7: Path traversal rejection
   - Test `..` in paths
   - Verify rejection
9. Verify SEC-8: Output path validation
   - Test system directory writes (should fail)
   - Test path validation logic
10. Run cargo-audit for vulnerabilities:
    ```bash
    cargo audit
    ```
11. Run cargo-deny for license compliance:
    ```bash
    cargo deny check
    ```
12. Document security assumptions
13. Create security audit report
14. Fix any security issues found
15. Update SECURITY.md if needed

**Files Modified**:
- Security tests as needed
- `SECURITY.md` (if needed)
- Audit documentation

**Acceptance Criteria**:
- All security requirements verified
- No cargo-audit vulnerabilities
- No license compliance issues
- Security audit report created
- Path traversal prevented
- Credentials properly protected

---

### 6.7: Cross-Platform Verification

**Objective**: Verify tool works on macOS, Linux, Windows

**Steps**:

1. Set up test environments:
   - macOS (local or CI)
   - Linux (Ubuntu in CI)
   - Windows (CI)
2. Run full test suite on each platform:
   ```bash
   cargo test --all --verbose
   ```
3. Test platform-specific behavior:
   - File permissions (Unix vs Windows)
   - Path separators (/ vs \\)
   - Line endings (LF vs CRLF)
   - Home directory resolution
4. Test CLI on each platform:
   - Run all commands
   - Verify output correct
   - Test with real config files
5. Test file operations:
   - Atomic writes
   - Backups
   - Permission setting
6. Document platform differences:
   - Known limitations
   - Platform-specific features
   - Workarounds if needed
7. Update GitHub Actions to test all platforms:
   ```yaml
   strategy:
     matrix:
       os: [ubuntu-latest, macos-latest, windows-latest]
       rust: [stable, "1.85.0"]
   ```
8. Run builds for all target architectures:
   - x86_64-unknown-linux-gnu
   - x86_64-apple-darwin
   - aarch64-apple-darwin
   - x86_64-pc-windows-msvc
9. Test release binaries on each platform
10. Document tested platforms in README

**Files Modified**:
- `.github/workflows/ci.yml` (platform matrix)
- `README.md` (platform support)
- Platform-specific test documentation

**Acceptance Criteria**:
- Tests pass on all platforms
- Platform-specific behavior documented
- CI tests all platforms
- Release binaries work on all targets

---

### 6.8: Final Quality Checks and Release Preparation

**Objective**: Prepare for v0.1.0 release

**Steps**:

1. Run complete quality check pipeline:
   ```bash
   just ci
   ```
2. Verify all quality checks pass:
   - `cargo fmt --check` - Formatting
   - `cargo clippy` - Lints
   - `cargo test --all` - Tests
   - `cargo audit` - Security
   - `cargo deny check` - Dependencies
3. Update version to 0.1.0:
   ```bash
   versioneer minor  # 0.0.1 -> 0.1.0
   ```
4. Update CHANGELOG.md:
   - Document all features
   - Document breaking changes (none for 0.1.0)
   - Document bug fixes
   - Link to GitHub issues
5. Review all documentation:
   - README complete
   - CLAUDE.md updated
   - Example configs accurate
   - Comments in code clear
6. Test installation from source:
   ```bash
   cargo install --path .
   multi-agent-config --version
   ```
7. Create release checklist:
   - [ ] All tests passing
   - [ ] Documentation complete
   - [ ] Examples working
   - [ ] Security audit done
   - [ ] Cross-platform tested
   - [ ] Version updated
   - [ ] CHANGELOG updated
   - [ ] Git hooks passing
8. Review GitHub Actions workflows:
   - CI pipeline complete
   - Release automation ready
   - Dependency scanning enabled
9. Tag release:
   ```bash
   versioneer tag --tag-format "multi-agent-config-v{version}"
   ```
10. Push release:
    ```bash
    git push origin main --tags
    ```

**Files Modified**:
- `VERSION`
- `Cargo.toml`
- `CHANGELOG.md`
- Git tags

**Acceptance Criteria**:
- Version updated to 0.1.0
- All quality checks pass
- Documentation complete
- Release tagged
- Ready for GitHub release

---

## Testing Strategy

**Coverage Testing**:
- Run cargo-tarpaulin
- Verify >= 80% coverage
- Document uncovered code

**Edge Case Testing**:
- All 28 edge cases from spec
- Documented and verified

**Failure Mode Testing**:
- All 25 failure modes from spec
- Error messages verified

**Integration Testing**:
- Full workflows tested
- Cross-platform verification

**Security Testing**:
- All security requirements verified
- Vulnerability scanning
- Dependency compliance

---

## Files Created/Modified

### New Files
- `tests/edge_cases_tests.rs` - Edge case tests
- `tests/failure_modes_tests.rs` - Failure mode tests
- `tests/example_validation.rs` - Example config tests
- `examples/*.toml` - Example configurations
- `examples/README.md` - Examples documentation
- `CHANGELOG.md` - Version history

### Modified Files
- `README.md` - Complete documentation
- `CLAUDE.md` - Updated implementation notes
- `Cargo.toml` - Version and metadata
- `VERSION` - Version file
- `.github/workflows/ci.yml` - Platform matrix
- All test files (achieving coverage)

---

## Deliverables Checklist

- [ ] 80% test coverage achieved
- [ ] All edge cases tested (28/28)
- [ ] All failure modes tested (25/25)
- [ ] README complete with examples
- [ ] Example configurations provided (5+)
- [ ] Security audit passed
- [ ] Cross-platform tests passing
- [ ] Version 0.1.0 released
- [ ] CHANGELOG updated
- [ ] Documentation complete
- [ ] CI/CD pipeline functional
- [ ] Binary releases built
- [ ] Ready for public use

---

## Commit Strategy

Each subtask should result in one conventional commit:

1. `test: achieve 80% test coverage across all modules`
2. `test: add all edge case tests from specification`
3. `test: add all failure mode tests from specification`
4. `docs: write comprehensive README`
5. `docs: create example configurations`
6. `security: perform security audit and verification`
7. `ci: add cross-platform verification`
8. `release: prepare for v0.1.0 release`

---

## Success Metrics

| Metric | Target | Status |
|--------|--------|--------|
| Test Coverage | >= 80% | ⏸️ Not Started |
| Edge Cases Tested | 28/28 | ⏸️ Not Started |
| Failure Modes Tested | 25/25 | ⏸️ Not Started |
| Security Requirements | 13/13 | ⏸️ Not Started |
| Platform Support | 3/3 (Mac, Linux, Win) | ⏸️ Not Started |
| Documentation Pages | >= 5 | ⏸️ Not Started |
| Example Configs | >= 5 | ⏸️ Not Started |

---

## Post-Phase 6 Activities

After Phase 6 completion:

1. Create GitHub release for v0.1.0
2. Publish to crates.io (if ready)
3. Announce release
4. Monitor for issues
5. Gather user feedback
6. Plan v0.2.0 features

---

## Phase Completion Checklist

- [ ] All subtasks completed and committed
- [ ] Test coverage >= 80% verified
- [ ] All edge cases tested
- [ ] All failure modes tested
- [ ] Security audit passed
- [ ] Cross-platform tests passing
- [ ] Documentation complete
- [ ] Examples provided
- [ ] Version 0.1.0 tagged
- [ ] GitHub release created
- [ ] Phase review completed
- [ ] **Project Complete!**

---

## Celebration

**Congratulations!** If all phases are complete:
- You've built a production-ready Rust CLI tool
- You've followed all Rust best practices
- You've achieved comprehensive test coverage
- You've created excellent documentation
- You're ready for users

🎉 **Project multi-agent-config v0.1.0 is complete!** 🎉
