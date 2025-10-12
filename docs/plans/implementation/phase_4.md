# Phase 4: File Operations & Safety

**Status**: Not Started
**Duration**: 2-3 days
**Dependencies**: Phase 3 (format transformers)

---

## Overview

Implement safe file operations including atomic writes, automatic backups, state tracking with SHA-256 hashing, and diff generation. This phase ensures all file modifications are safe, reversible, and trackable.

## Goals

- Implement atomic file write operations
- Create automatic backup system
- Implement state tracker with SHA-256 hashing
- Generate unified diffs showing changes
- Handle file permissions correctly (0600)
- Support cross-platform paths
- Prevent partial writes and data corruption

## Success Criteria

- [ ] File writes are atomic (all-or-nothing)
- [ ] Backups created before modifications
- [ ] State tracker records all generated files
- [ ] SHA-256 hashes detect modifications
- [ ] Diffs show accurate changes
- [ ] File permissions set correctly
- [ ] Works on Unix and Windows
- [ ] All safety tests pass

## Subtasks

### 4.1: Create File Operations Module with Atomic Writes

**Objective**: Implement atomic file write operations

**Steps**:

1. Create `src/file_ops/mod.rs` as file operations module
2. Create `src/file_ops/writer.rs` for atomic writes
3. Implement atomic write function:
   ```rust
   pub fn write_file_atomic(
       path: &Path,
       content: &str,
       permissions: Option<u32>
   ) -> Result<(), FileOpError>
   ```
4. Algorithm (from spec Appendix D):
   - Create parent directories if needed (mkdir -p)
   - Create temporary file in same directory as target
   - Use `tempfile::NamedTempFile` for temp file
   - Write content to temp file
   - Validate written content (read back and compare)
   - Set file permissions (default 0600 for configs)
   - Atomically rename temp file to target path
   - On error: clean up temp file
5. Implement platform-specific permission handling:
   - Unix: use `std::os::unix::fs::PermissionsExt`
   - Windows: use default permissions
6. Handle edge cases:
   - Path contains non-existent directories
   - Insufficient permissions
   - Disk full errors
   - Long file paths
7. Write unit tests:
   - Successful atomic write
   - Write to non-existent directory (creates parents)
   - Permission denied error
   - Disk full error (mock)
   - Atomicity verification (concurrent reads)
8. Test on temporary directory
9. Verify permissions set correctly
10. Run tests: `cargo test atomic_write`

**Files Modified**:
- `src/file_ops/mod.rs`
- `src/file_ops/writer.rs`
- `src/error.rs` (add FileOpError)
- `Cargo.toml` (add tempfile dependency)

**Acceptance Criteria**:
- Writes are atomic (verified by concurrent tests)
- Temp files cleaned up on error
- Parent directories created automatically
- Permissions set correctly
- Tests pass on Unix and Windows

---

### 4.2: Implement Backup System

**Objective**: Create automatic backups before modifying files

**Steps**:

1. Create `src/file_ops/backup.rs`
2. Implement backup function:
   ```rust
   pub fn create_backup(
       original_path: &Path
   ) -> Result<Option<PathBuf>, FileOpError>
   ```
3. Algorithm:
   - Check if original file exists
   - If not: return Ok(None) (no backup needed)
   - If yes:
     - Generate backup path: `<original>.backup`
     - Use `std::fs::copy` to create backup
     - Preserve original file metadata (timestamps, permissions)
     - Return Ok(Some(backup_path))
4. Handle backup overwriting:
   - Each compile overwrites previous backup (single level)
   - Log when existing backup is replaced
5. Implement restore function (for future rollback):
   ```rust
   pub fn restore_from_backup(
       backup_path: &Path,
       target_path: &Path
   ) -> Result<(), FileOpError>
   ```
6. Add backup verification:
   - Verify backup file readable
   - Verify backup size matches original
7. Write unit tests:
   - Create backup of existing file
   - No backup when file doesn't exist
   - Backup preserves permissions
   - Backup preserves timestamps
   - Overwrite existing backup
   - Restore from backup
8. Test error conditions:
   - Original file unreadable
   - Backup location unwritable
9. Test on temporary files
10. Run tests: `cargo test backup`

**Files Modified**:
- `src/file_ops/backup.rs`
- `src/file_ops/mod.rs` (export)

**Acceptance Criteria**:
- Backups created before writes
- Metadata preserved
- Single backup level (overwrites previous)
- Restore function works
- Tests pass

---

### 4.3: Implement State Tracker with SHA-256 Hashing

**Objective**: Track generated files with content hashing

**Steps**:

1. Create `src/file_ops/state.rs`
2. Define state file structure:
   ```rust
   #[derive(Serialize, Deserialize)]
   pub struct StateFile {
       version: String,
       last_compile: DateTime<Utc>,
       generated_files: Vec<GeneratedFile>,
   }

   #[derive(Serialize, Deserialize)]
   pub struct GeneratedFile {
       tool: String,
       path: PathBuf,
       timestamp: DateTime<Utc>,
       hash: String, // "sha256:<hex>"
   }
   ```
3. Implement state tracker:
   ```rust
   pub struct StateTracker {
       state_file_path: PathBuf,
       state: StateFile,
   }
   ```
4. Implement methods:
   - `load(path: &Path) -> Result<Self, StateError>` - Load existing state
   - `add_generated_file(tool, path, hash)` - Add file to tracker
   - `get_file_hash(path) -> Option<String>` - Get recorded hash
   - `save() -> Result<(), StateError>` - Save state atomically
5. Implement SHA-256 hashing:
   ```rust
   fn hash_file(path: &Path) -> Result<String, FileOpError>
   ```
   - Use `sha2` crate
   - Read file in chunks (for large files)
   - Return hex-encoded hash with "sha256:" prefix
6. State file location: `~/.config/multi-agent-config/state/generated.json`
7. Handle corrupted state file:
   - Log warning
   - Delete corrupted file
   - Create new empty state
8. Write unit tests:
   - Create new state file
   - Load existing state file
   - Add files to state
   - Save state atomically
   - SHA-256 hashing correctness
   - Corrupted state file handling
9. Test state file schema matches spec
10. Run tests: `cargo test state`

**Files Modified**:
- `src/file_ops/state.rs`
- `src/file_ops/mod.rs` (export)
- `Cargo.toml` (add sha2, chrono dependencies)

**Acceptance Criteria**:
- State file format matches specification
- SHA-256 hashing works correctly
- State persists across runs
- Corrupted files handled gracefully
- Tests pass

---

### 4.4: Implement Diff Generation

**Objective**: Generate unified diffs showing configuration changes

**Steps**:

1. Create `src/file_ops/diff.rs`
2. Implement diff generation:
   ```rust
   pub fn generate_diff(
       old_content: Option<&str>,
       new_content: &str,
       path: &Path,
       tool_name: &str
   ) -> String
   ```
3. Algorithm:
   - If old_content is None: show as "[NEW FILE]"
   - If old_content exists: generate unified diff
   - Use `similar` or `diff` crate for diff algorithm
   - Format output according to spec (Section 4.3.3.4)
4. Output format:
   ```
   ================================================================================
   Tool: <tool_name>
   Path: <path>
   ================================================================================
   --- current
   +++ new
   @@ -line,count +line,count @@
    context line
   -removed line
   +added line
    context line
   ```
5. Handle large files:
   - Truncate if > 10000 lines
   - Add message: "[... truncated ...]"
6. Implement diff for all formats:
   - JSON (pretty-printed comparison)
   - TOML (line-by-line comparison)
7. Add color support (when TTY detected):
   - Red for removed lines
   - Green for added lines
   - Reset for context
8. Write unit tests:
   - New file (no old content)
   - Modified file (diff generation)
   - Identical files (no changes)
   - Large file truncation
   - JSON formatting
   - TOML formatting
9. Test color output (TTY vs non-TTY)
10. Run tests: `cargo test diff`

**Files Modified**:
- `src/file_ops/diff.rs`
- `src/file_ops/mod.rs` (export)
- `Cargo.toml` (add similar or diff dependency)

**Acceptance Criteria**:
- Diff format matches specification
- New files shown correctly
- Unified diff format correct
- Large files truncated
- Color support works
- Tests pass

---

### 4.5: Implement Safe Write Coordinator

**Objective**: Coordinate backup, write, and state tracking

**Steps**:

1. Create `src/file_ops/coordinator.rs`
2. Implement safe write workflow:
   ```rust
   pub fn safe_write_with_backup(
       path: &Path,
       content: &str,
       tool_name: &str,
       state_tracker: &mut StateTracker
   ) -> Result<(), FileOpError>
   ```
3. Algorithm (spec Section 6, Appendix D):
   - Step 1: Create backup if file exists
   - Step 2: Write content atomically to temp file
   - Step 3: Validate written content (parse, verify format)
   - Step 4: Set file permissions (0600)
   - Step 5: Atomically rename to target path
   - Step 6: Calculate SHA-256 hash of written file
   - Step 7: Update state tracker
   - Step 8: Save state tracker
   - On error at any step: preserve backup, clean up temp files
4. Implement rollback on failure:
   - If write succeeds but state save fails: warn but continue
   - If write fails: backup remains, no changes
5. Add comprehensive logging:
   - Log each step
   - Log backup creation
   - Log successful writes
   - Log state updates
6. Write unit tests:
   - Complete successful write workflow
   - Failure at backup step
   - Failure at write step
   - Failure at state save step
   - Verify backup preserved on failure
   - Verify temp files cleaned up
7. Test idempotency:
   - Writing same content twice produces same result
   - No backup corruption
8. Test concurrent safety (if lock files added)
9. Integration test: full compile workflow
10. Run tests: `cargo test safe_write`

**Files Modified**:
- `src/file_ops/coordinator.rs`
- `src/file_ops/mod.rs` (export)

**Acceptance Criteria**:
- All steps execute in correct order
- Failures handled gracefully
- Backups preserved on errors
- State consistent after failures
- Integration tests pass

---

### 4.6: Implement Path and Permission Utilities

**Objective**: Handle cross-platform paths and permissions

**Steps**:

1. Create `src/file_ops/utils.rs`
2. Implement path utilities:
   ```rust
   pub fn resolve_tool_config_path(tool: ToolName) -> Result<PathBuf, FileOpError>
   pub fn ensure_parent_dir(path: &Path) -> Result<(), FileOpError>
   pub fn validate_path(path: &Path) -> Result<(), FileOpError>
   ```
3. Define config paths for each tool:
   - Cursor: `./.cursor/mcp.json` (project-relative)
   - opencode: `~/.config/opencode/opencode.json`
   - Codex: `~/.codex/config.toml`
   - Claude Code: TBD (CLI-based, no file)
4. Implement home directory resolution:
   - Use `dirs` crate for cross-platform `~` expansion
   - Handle `$HOME` environment variable
5. Implement path validation (SEC-7, SEC-8):
   - Reject paths with `..` (parent directory traversal)
   - Reject paths to system directories (/etc, /usr, etc.)
   - Ensure paths within expected directories
6. Implement permission utilities:
   ```rust
   pub fn set_secure_permissions(path: &Path) -> Result<(), FileOpError>
   pub fn check_writeable(path: &Path) -> Result<(), FileOpError>
   ```
7. Handle platform differences:
   - Unix: use chmod for 0600
   - Windows: use default ACLs
8. Write unit tests:
   - Path resolution for each tool
   - Home directory expansion
   - Parent directory creation
   - Path validation (reject ..)
   - Permission setting (Unix)
   - Writeable check
9. Test edge cases:
   - Missing home directory
   - Read-only file system
   - Symlinks
10. Run tests: `cargo test path_utils`

**Files Modified**:
- `src/file_ops/utils.rs`
- `src/file_ops/mod.rs` (export)
- `Cargo.toml` (add dirs dependency)

**Acceptance Criteria**:
- Path resolution works cross-platform
- Home directory expanded correctly
- Path traversal prevented
- Permissions set securely
- Tests pass on Unix and Windows

---

### 4.7: Add Comprehensive Safety Tests

**Objective**: Verify all safety properties from specification

**Steps**:

1. Create `tests/file_ops_tests.rs` for integration tests
2. Test invariants from spec Section 6:
   - INV-2: Atomicity (file either fully written or not at all)
   - INV-3: Backup preservation (exact copy of original)
   - INV-5: Generated files are valid format
   - INV-6: State tracker accuracy
   - INV-7: Idempotency (same input → same output)
   - INV-8: Rollback safety (backups intact on failure)
3. Test failure modes:
   - FM-11: Output directory doesn't exist (creates it)
   - FM-12: Output file unwritable (permission denied)
   - FM-13: Disk full (ENOSPC)
   - FM-14: Backup creation failed (aborts)
   - FM-20: Output file modified during execution
4. Test edge cases:
   - EC-10: Output path is symlink
   - EC-11: Output path is directory (error)
   - EC-12: Output directory is read-only
   - EC-13: Very long file path
   - EC-14: Special characters in path
5. Test security requirements (Section 10):
   - SEC-4: Files created with mode 0600
   - SEC-5: Backup files preserve permissions
   - SEC-6: State files created with mode 0600
   - SEC-7: Path traversal prevention
   - SEC-8: Output path validation
6. Test concurrent operations:
   - Multiple processes writing (if lock files)
   - File modified during backup/write
7. Test cross-platform behavior:
   - Unix permissions vs Windows ACLs
   - Path separators (/ vs \\)
8. Simulate disk full condition (mock)
9. Verify all temporary files cleaned up
10. Run comprehensive test suite: `cargo test file_ops_tests`

**Files Modified**:
- `tests/file_ops_tests.rs`
- `tests/fixtures/` (test files)

**Acceptance Criteria**:
- All invariants verified
- All failure modes tested
- All edge cases covered
- Security requirements met
- Tests pass on Unix and Windows
- Coverage >= 80%

---

## Testing Strategy

**Unit Tests**:
- Atomic write operations
- Backup creation and restore
- State tracking and hashing
- Diff generation
- Path utilities
- Permission handling

**Integration Tests**:
- Complete safe write workflow
- Failure recovery
- Idempotency
- Concurrent operations (if applicable)

**Coverage Target**: 80% minimum for file_ops module

---

## Files Created/Modified

### New Files
- `src/file_ops/mod.rs` - File operations module
- `src/file_ops/writer.rs` - Atomic writes
- `src/file_ops/backup.rs` - Backup system
- `src/file_ops/state.rs` - State tracker
- `src/file_ops/diff.rs` - Diff generation
- `src/file_ops/coordinator.rs` - Safe write coordinator
- `src/file_ops/utils.rs` - Path and permission utilities
- `tests/file_ops_tests.rs` - Integration tests

### Modified Files
- `src/error.rs` - File operation errors
- `src/lib.rs` - Module exports
- `Cargo.toml` - Dependencies

---

## Dependencies Added

```toml
[dependencies]
tempfile = "3.8"
sha2 = "0.10"
chrono = { version = "0.4", features = ["serde"] }
similar = "2.3"  # or diff = "0.1" for diff generation
dirs = "5.0"
```

---

## Commit Strategy

Each subtask should result in one conventional commit:

1. `feat(file-ops): create file operations module with atomic writes`
2. `feat(file-ops): implement backup system`
3. `feat(file-ops): implement state tracker with SHA-256 hashing`
4. `feat(file-ops): implement diff generation`
5. `feat(file-ops): implement safe write coordinator`
6. `feat(file-ops): implement path and permission utilities`
7. `test(file-ops): add comprehensive safety tests`

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Atomicity failure on Windows | High | Use tempfile crate (handles platform differences) |
| Race conditions | Medium | Test concurrent operations, consider lock files |
| Disk full during write | Medium | Test with mock, ensure cleanup |
| Symlink handling inconsistencies | Low | Test symlink behavior, document |

---

## Reference Specifications

- Invariants: `./architecture/06-invariants.md`
- Failure modes: `./architecture/08-failure-modes.md`
- Security requirements: `./architecture/10-security-requirements.md`
- Operational requirements: `./architecture/11-operational-requirements.md`
- Safe write protocol: `./architecture/16-appendices.md` Appendix D

---

## Phase Completion Checklist

- [ ] All subtasks completed and committed
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] Code coverage >= 80%
- [ ] No clippy warnings
- [ ] Code formatted with rustfmt
- [ ] All invariants verified
- [ ] All security requirements met
- [ ] Cross-platform testing complete
- [ ] Phase review completed
- [ ] Ready for Phase 5

---

## Next Phase

After Phase 4 completion, proceed to [Phase 5: CLI Commands Implementation](phase_5.md)
