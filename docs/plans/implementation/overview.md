# Multi-Agent-Config Implementation Plan

**Project**: multi-agent-config - Unified Configuration Manager for AI Coding Tools
**Version**: 1.0
**Created**: 2025-10-12
**Status**: Ready for Implementation

---

## Mission Statement

Implement a Rust-based CLI tool that compiles a single unified TOML configuration into tool-specific MCP (Model Context Protocol) server configurations for multiple AI coding assistants. The tool eliminates configuration duplication, ensures consistency across tools, centralizes credential management, and provides safe configuration updates with validation and backup capabilities.

## Target Tools

1. **Claude Code** - Anthropic's CLI coding assistant
2. **Cursor** - AI-powered code editor
3. **opencode.ai** - Terminal-based AI coding agent
4. **OpenAI Codex** - OpenAI's CLI coding assistant

## Scope

### In Scope (V1.0)

- **Core Functionality**:
  - Parse unified TOML configuration file
  - Validate configuration schema and required fields
  - Expand environment variable references (`${VAR}` and `{VAR}`)
  - Detect circular variable references
  - Filter servers by target tool
  - Transform to tool-specific formats (JSON/TOML)
  - Generate configurations for all 4 target tools

- **Safety Features**:
  - Atomic file writes (write to temp, then rename)
  - Automatic backup of existing configurations
  - State tracking for generated files
  - SHA-256 hashing for change detection
  - Dry-run mode (show changes without writing)
  - Diff generation (unified diff format)

- **Security**:
  - Environment variable expansion without shell execution
  - Credential redaction in logs and error messages
  - File permissions (0600 for configs)
  - Path traversal prevention

- **CLI Commands**:
  - `init` - Create template configuration
  - `validate` - Check configuration correctness
  - `compile` - Generate and write tool configurations
  - `diff` - Show what would change without writing

- **Error Handling**:
  - Comprehensive error messages with suggestions
  - Exit codes matching specification
  - 25+ failure modes properly handled
  - 28+ edge cases properly handled

- **Testing**:
  - 80% minimum code coverage
  - Unit tests for all core modules
  - Integration tests for CLI commands
  - Edge case test matrix
  - Failure mode test matrix

- **Documentation**:
  - README with usage examples
  - CLI help text for all commands
  - CLAUDE.md with implementation guidance
  - Example configurations

### Out of Scope (Future Versions)

- Prompt file management
- Custom agent/mode configuration
- Watch mode for auto-compilation
- Plugin architecture for custom transformers
- Rollback command (manual rollback via backups in V1.0)
- Lock files for concurrent execution prevention

## Success Criteria

- [ ] User can define all MCP servers in one TOML file
- [ ] Tool generates valid configurations for all 4 target tools
- [ ] Configurations remain synchronized across tools
- [ ] No manual editing of tool-specific config files required
- [ ] Existing configurations safely backed up before modification
- [ ] Invalid configurations detected before application
- [ ] Environment variables expanded correctly with circular reference detection
- [ ] Atomic file operations prevent partial writes
- [ ] All tests pass with 80% coverage minimum
- [ ] Cross-platform support (macOS, Linux, Windows)
- [ ] CLI follows Rust standards from CLAUDE.md
- [ ] Version management integrated with versioneer
- [ ] Git hooks configured via peter-hook

## Phase Breakdown

### Phase 1: Project Foundation & Core Parsing
**Duration**: 2-3 days
**Dependencies**: None

- Set up core data structures for configuration schema
- Implement TOML parsing and deserialization
- Implement schema validation (required fields, types)
- Basic CLI scaffolding with clap
- Initial error handling framework

### Phase 2: Environment Variable Expansion
**Duration**: 2-3 days
**Dependencies**: Phase 1

- Implement variable expansion algorithm
- Support `${VAR}` from shell environment
- Support `{VAR}` from [env] section
- Circular reference detection (max depth 10)
- Undefined variable handling (warnings)

### Phase 3: Format Transformers
**Duration**: 3-4 days
**Dependencies**: Phase 1, Phase 2

- Implement Cursor JSON transformer
- Implement opencode.ai JSON transformer
- Implement Codex TOML transformer
- Implement Claude Code transformer (CLI or JSON)
- Target filtering logic
- Tool-specific field handling

### Phase 4: File Operations & Safety
**Duration**: 2-3 days
**Dependencies**: Phase 3

- Implement atomic file writes
- Implement backup creation
- Implement state tracker (SHA-256 hashing)
- Implement diff generation (unified format)
- File permission handling (0600)
- Directory creation with error handling

### Phase 5: CLI Commands Implementation
**Duration**: 2-3 days
**Dependencies**: Phase 4

- Implement `init` command with template
- Implement `validate` command
- Implement `compile` command (with --tool, --dry-run)
- Implement `diff` command
- Standard subcommands (version, help, license, doctor)
- Shell completion generation

### Phase 6: Testing & Documentation
**Duration**: 3-4 days
**Dependencies**: Phase 5

- Unit tests for all modules (80% coverage)
- Integration tests for CLI commands
- Edge case tests (28 cases)
- Failure mode tests (25 cases)
- Update README with examples
- Create example configurations
- Update CLAUDE.md
- Security audit
- Cross-platform testing

## Technical Stack

- **Language**: Rust (edition 2024, MSRV 1.85.0)
- **CLI Framework**: clap with derive macros
- **Serialization**: serde, toml, serde_json
- **File Operations**: tempfile, sha2
- **Error Handling**: anyhow, thiserror
- **Testing**: cargo test, cargo tarpaulin
- **Quality Tools**: clippy, rustfmt, cargo-audit, cargo-deny

## Key Architectural Decisions

1. **Rust Implementation**: Type safety, performance, zero-cost abstractions
2. **Atomic Writes**: Write to temp file, then rename (ensures consistency)
3. **Single Backup Level**: Each compile overwrites previous backup (simplicity)
4. **State Tracking**: JSON file with SHA-256 hashes for change detection
5. **Error First**: Comprehensive error handling from the start
6. **Test Driven**: Write tests before or alongside implementation
7. **Standards Compliant**: Follow all Rust standards from CLAUDE.md

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Claude Code config format TBD | High | Implement CLI-based approach first, JSON fallback ready |
| Complex environment variable expansion | Medium | Reference implementation in spec Appendix C |
| Cross-platform path handling | Medium | Use std::path consistently, test on all platforms |
| Atomic writes on Windows | Low | Use tempfile crate which handles platform differences |
| Circular reference detection | Low | Well-defined algorithm with max depth limit |

## Integration Points

- **versioneer**: Version management (VERSION file ↔ Cargo.toml sync)
- **peter-hook**: Git hooks for pre-commit and pre-push validation
- **GitHub Actions**: CI/CD with multi-platform builds and releases
- **cargo-deny**: Dependency compliance and license checking
- **cargo-audit**: Security vulnerability scanning

## Related Documentation

- Architecture specification: `./architecture/*.md`
- Rust standards: `~/.claude/CLAUDE.md`, `/CLAUDE.md`
- Configuration schema: `./architecture/12-configuration-schema-reference.md`
- Error handling spec: `./architecture/13-error-handling-specifications.md`
- Testing requirements: `./architecture/14-testing-requirements.md`

## Plan Execution Workflow

1. Start from clean git status
2. Read this overview and the active phase file
3. Create branch: `multi-agent-config_phase_{n}`
4. Work subtasks sequentially
5. Commit after each subtask (Conventional Commits)
6. Run version bump after each subtask: `versioneer patch`
7. Update plan files with completion status
8. When phase complete, squash commits
9. Move to next phase

## GitHub Issues & Tracking

GitHub issues and Asana tracking are **not required** for this project. Phase tracking will be maintained in the phase markdown files only.

---

## Phase Status

- [Phase 1: Project Foundation & Core Parsing](phase_1.md) - **Not Started**
- [Phase 2: Environment Variable Expansion](phase_2.md) - **Not Started**
- [Phase 3: Format Transformers](phase_3.md) - **Not Started**
- [Phase 4: File Operations & Safety](phase_4.md) - **Not Started**
- [Phase 5: CLI Commands Implementation](phase_5.md) - **Not Started**
- [Phase 6: Testing & Documentation](phase_6.md) - **Not Started**
