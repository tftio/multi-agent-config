# Changelog

All notable changes to multi-agent-config will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2025-10-12

### Added
- Unified TOML configuration format for AI coding tools
- Support for 4 tools: Cursor, opencode.ai, Codex, Claude Code
- Environment variable expansion (${SHELL_VAR} and {CONFIG_VAR})
- Circular reference detection (max depth 10)
- Format transformers for all supported tools
- Atomic file writes with automatic backups
- State tracking with SHA-256 hashing
- Diff preview before applying changes
- CLI commands: init, validate, compile, diff
- Comprehensive error handling with exit codes
- Cross-platform support (macOS, Linux, Windows)
- Template configuration generation
- Target filtering (per-server tool selection)
- Tool-specific field handling (disabled, autoApprove, timeouts)
- HTTP server support (opencode, codex, claude-code)
- Complete test suite (149 tests)
- Comprehensive documentation and examples
- GitHub Actions CI/CD pipeline
- Security auditing with cargo-audit
- Dependency compliance with cargo-deny
- Git hooks with peter-hook
- Version management with versioneer

### Implementation Details
- **Phase 1**: Core data structures, TOML parsing, schema validation
- **Phase 2**: Environment variable expansion with circular detection
- **Phase 3**: Format transformers (Cursor JSON, opencode JSON, Codex TOML, Claude Code JSON)
- **Phase 4**: Atomic file operations, backups, state tracking, diff generation
- **Phase 5**: CLI command implementations

[Unreleased]: https://github.com/jfb/multi-agent-config/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/jfb/multi-agent-config/releases/tag/multi-agent-config-v0.1.1
