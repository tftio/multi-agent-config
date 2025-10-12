# Phase 3: Format Transformers

**Status**: Not Started
**Duration**: 3-4 days
**Dependencies**: Phase 1 (parsing), Phase 2 (expansion)

---

## Overview

Implement the format transformation layer that converts the unified configuration into tool-specific formats. This phase creates transformers for Cursor (JSON), opencode.ai (JSON), Codex (TOML), and Claude Code (CLI or JSON). Each transformer applies tool-specific filtering and field mappings according to the specification.

## Goals

- Implement target filtering (servers by tool name)
- Create Cursor JSON transformer
- Create opencode.ai JSON transformer
- Create Codex TOML transformer
- Create Claude Code transformer
- Handle tool-specific fields correctly
- Filter HTTP servers for tools that don't support them

## Success Criteria

- [ ] Target filtering works correctly
- [ ] Cursor JSON output matches specification
- [ ] opencode.ai JSON output matches specification
- [ ] Codex TOML output matches specification
- [ ] Claude Code output generated correctly
- [ ] Tool-specific fields included only for correct tools
- [ ] HTTP servers excluded from tools that don't support them
- [ ] All transformation tests pass

## Subtasks

### 3.1: Create Transformer Module and Target Filtering

**Objective**: Set up transformation infrastructure and implement target filtering logic

**Steps**:

1. Create `src/transform/mod.rs` as transformation module
2. Create `src/transform/filter.rs` for target filtering
3. Implement `filter_servers_for_tool` function:
   ```rust
   pub fn filter_servers_for_tool(
       servers: &HashMap<String, ServerConfig>,
       tool_name: ToolName,
       default_targets: &[ToolName]
   ) -> HashMap<String, ServerConfig>
   ```
4. Algorithm from spec (Section 7.2):
   - Skip servers with `enabled = false`
   - Get targets from server or use default_targets
   - Expand "all" to all four tool names
   - Include server if tool_name is in targets
5. Implement `ToolName` enum methods:
   - `from_str(&str) -> Result<ToolName, ParseError>`
   - `to_string() -> String`
   - `all_tools() -> Vec<ToolName>` (returns all 4 tools)
6. Handle special case: HTTP servers for Cursor (skip with warning)
7. Write unit tests:
   - Filter with "all" target
   - Filter with specific targets
   - Filter with disabled servers
   - Filter with no matching targets
   - Default targets applied correctly
8. Test edge cases:
   - Empty server list
   - All servers disabled
   - HTTP server for Cursor (should be filtered)
9. Add logging for filtered servers
10. Run tests: `cargo test filter`

**Files Modified**:
- `src/transform/mod.rs`
- `src/transform/filter.rs`
- `src/config/types.rs` (add ToolName methods)

**Acceptance Criteria**:
- Filtering logic matches specification
- All test cases pass
- Warnings logged for filtered servers
- Tool name parsing works correctly

---

### 3.2: Implement Cursor JSON Transformer

**Objective**: Transform unified config to Cursor's mcp.json format

**Steps**:

1. Create `src/transform/cursor.rs`
2. Define Cursor output types:
   ```rust
   #[derive(Serialize)]
   struct CursorConfig {
       #[serde(rename = "mcpServers")]
       mcp_servers: HashMap<String, CursorServer>,
   }

   #[derive(Serialize)]
   struct CursorServer {
       command: String,
       args: Vec<String>,
       #[serde(skip_serializing_if = "Option::is_none")]
       env: Option<HashMap<String, String>>,
       #[serde(skip_serializing_if = "Option::is_none")]
       disabled: Option<bool>,
       #[serde(skip_serializing_if = "Option::is_none")]
       #[serde(rename = "autoApprove")]
       auto_approve: Option<Vec<String>>,
   }
   ```
3. Implement transformer:
   ```rust
   pub fn transform_for_cursor(
       servers: &HashMap<String, ServerConfig>,
       default_targets: &[ToolName]
   ) -> Result<String, TransformError>
   ```
4. Algorithm (from spec Section 5.3 and 7.3):
   - Filter servers for Cursor
   - Skip HTTP servers (Cursor doesn't support)
   - For each STDIO server:
     - Map command field directly
     - Map args field directly (empty array if not present)
     - Map env field (only if present)
     - Map disabled field (only if present)
     - Map autoApprove field (only if present)
   - Serialize to JSON with 2-space indentation
5. Handle tool-specific fields:
   - Include Cursor-specific: disabled, autoApprove
   - Exclude Codex-specific: startup_timeout_sec, tool_timeout_sec
6. Validate generated JSON before returning
7. Write unit tests:
   - Single STDIO server
   - Multiple servers
   - Server with env vars
   - Server with Cursor-specific fields
   - HTTP server (should be excluded)
   - Empty server list
8. Test against example from spec Appendix B.1
9. Verify JSON indentation (2 spaces)
10. Run tests: `cargo test cursor`

**Files Modified**:
- `src/transform/cursor.rs`
- `src/transform/mod.rs` (export)
- `src/error.rs` (add TransformError)

**Acceptance Criteria**:
- Output matches spec format exactly
- JSON is valid and well-formatted
- HTTP servers excluded with warning
- Tool-specific fields handled correctly
- Tests pass including spec example

---

### 3.3: Implement opencode.ai JSON Transformer

**Objective**: Transform unified config to opencode.ai's opencode.json format

**Steps**:

1. Create `src/transform/opencode.rs`
2. Define opencode.ai output types:
   ```rust
   #[derive(Serialize, Deserialize)]
   struct OpencodeConfig {
       mcp: HashMap<String, OpencodeServer>,
       #[serde(flatten)]
       other: HashMap<String, Value>, // Preserve other sections
   }

   #[derive(Serialize)]
   #[serde(tag = "type", rename_all = "lowercase")]
   enum OpencodeServer {
       Local {
           command: Vec<String>,
           #[serde(skip_serializing_if = "Option::is_none")]
           env: Option<HashMap<String, String>>,
           enabled: bool,
       },
       Remote {
           url: String,
           #[serde(skip_serializing_if = "Option::is_none")]
           headers: Option<HashMap<String, String>>,
           enabled: bool,
       },
   }
   ```
3. Implement transformer:
   ```rust
   pub fn transform_for_opencode(
       servers: &HashMap<String, ServerConfig>,
       default_targets: &[ToolName],
       existing_config: Option<&Path>
   ) -> Result<String, TransformError>
   ```
4. Algorithm (from spec Section 5.4 and 7.4):
   - Filter servers for opencode
   - For STDIO servers:
     - Set type: "local"
     - Combine command and args into single array: [command, ...args]
     - Include env if present
     - Set enabled: true
   - For HTTP servers:
     - Set type: "remote"
     - Include url field
     - If bearer_token present: create headers with Authorization: Bearer <token>
     - Set enabled: true
   - If existing config file exists:
     - Read and parse
     - Preserve all top-level keys except "mcp"
     - Replace only "mcp" section
   - Serialize to JSON with 2-space indentation
5. Implement merge logic for existing configs
6. Write unit tests:
   - STDIO server transformation
   - HTTP server transformation
   - Server with bearer_token (becomes Authorization header)
   - Merge with existing config (preserve other sections)
   - Empty config
7. Test against example from spec Appendix B.2
8. Test edge case: existing config with other top-level sections
9. Verify command array format: [exe, arg1, arg2, ...]
10. Run tests: `cargo test opencode`

**Files Modified**:
- `src/transform/opencode.rs`
- `src/transform/mod.rs` (export)

**Acceptance Criteria**:
- STDIO servers formatted correctly (command as array)
- HTTP servers formatted correctly (with headers)
- Bearer tokens become Authorization headers
- Existing config sections preserved
- Tests pass including spec example

---

### 3.4: Implement Codex TOML Transformer

**Objective**: Transform unified config to Codex's config.toml format

**Steps**:

1. Create `src/transform/codex.rs`
2. Define Codex output types:
   ```rust
   #[derive(Serialize)]
   struct CodexConfig {
       #[serde(flatten)]
       mcp_servers: HashMap<String, CodexServer>,
       #[serde(flatten)]
       other: HashMap<String, Value>, // Preserve other sections
   }

   #[derive(Serialize)]
   #[serde(untagged)]
   enum CodexServer {
       Stdio {
           command: String,
           #[serde(skip_serializing_if = "Option::is_none")]
           args: Option<Vec<String>>,
           #[serde(skip_serializing_if = "Option::is_none")]
           env: Option<HashMap<String, String>>,
           #[serde(skip_serializing_if = "Option::is_none")]
           startup_timeout_sec: Option<u32>,
           #[serde(skip_serializing_if = "Option::is_none")]
           tool_timeout_sec: Option<u32>,
       },
       Http {
           url: String,
           #[serde(skip_serializing_if = "Option::is_none")]
           bearer_token: Option<String>,
       },
   }
   ```
3. Implement transformer:
   ```rust
   pub fn transform_for_codex(
       servers: &HashMap<String, ServerConfig>,
       default_targets: &[ToolName],
       existing_config: Option<&Path>
   ) -> Result<String, TransformError>
   ```
4. Algorithm (from spec Section 5.5 and 7.5):
   - Filter servers for codex
   - For STDIO servers:
     - Map command field
     - Map args field (omit if empty)
     - Create `[mcp_servers.<name>.env]` subsection if env present
     - Include startup_timeout_sec if present (default 30)
     - Include tool_timeout_sec if present (default 60)
   - For HTTP servers:
     - Map url field
     - Map bearer_token field if present
   - Section prefix: `mcp_servers.<name>` (not `mcp.servers.<name>`)
   - If existing config exists:
     - Read and parse
     - Preserve all sections except `[mcp_servers.*]`
     - Replace all `[mcp_servers.*]` sections
   - Serialize to TOML
5. Handle env as separate TOML table section
6. Write unit tests:
   - STDIO server with defaults
   - STDIO server with custom timeouts
   - STDIO server with env vars
   - HTTP server
   - HTTP server with bearer token
   - Merge with existing config (preserve other sections)
7. Test against example from spec Appendix B.3
8. Verify TOML formatting
9. Test section prefix: `mcp_servers` not `mcp.servers`
10. Run tests: `cargo test codex`

**Files Modified**:
- `src/transform/codex.rs`
- `src/transform/mod.rs` (export)

**Acceptance Criteria**:
- TOML output matches specification
- Section prefix is `mcp_servers`
- Env vars as separate subsection
- Timeout fields included when present
- Existing config sections preserved
- Tests pass including spec example

---

### 3.5: Implement Claude Code Transformer

**Objective**: Transform unified config to Claude Code format (CLI or JSON)

**Steps**:

1. Create `src/transform/claude.rs`
2. Determine implementation approach:
   - **Option A (preferred)**: Generate CLI commands
   - **Option B (fallback)**: Generate JSON file (if path known)
3. For CLI approach:
   ```rust
   pub fn transform_for_claude_cli(
       servers: &HashMap<String, ServerConfig>,
       default_targets: &[ToolName]
   ) -> Result<Vec<String>, TransformError>
   ```
4. Algorithm (from spec Section 5.2):
   - Filter servers for claude-code
   - For each server, generate command:
     ```
     claude mcp add <name> [--env KEY=VALUE]... -- <command> [args...]
     ```
   - Format env vars as `--env KEY=VALUE` flags
   - Append `-- <command> <arg1> <arg2>...`
5. Return vector of commands to execute
6. Add CLI execution helper (for Phase 5):
   ```rust
   pub fn execute_claude_commands(commands: Vec<String>) -> Result<(), TransformError>
   ```
7. Check if `claude` CLI is available
8. Execute commands sequentially
9. Capture stderr for errors
10. Write unit tests:
    - Single server command generation
    - Server with env vars (multiple --env flags)
    - Server with no env vars
    - Multiple servers (multiple commands)
    - CLI not available (error handling)
11. Test command format matches spec
12. Consider Option B (JSON) if needed
13. Document current status of Claude Code config
14. Run tests: `cargo test claude`

**Files Modified**:
- `src/transform/claude.rs`
- `src/transform/mod.rs` (export)

**Acceptance Criteria**:
- CLI commands generated correctly
- Command format matches specification
- Env vars formatted as flags
- CLI availability checked
- Tests pass
- Fallback approach documented

---

### 3.6: Create Integration Tests for All Transformers

**Objective**: Verify all transformers work with real configurations

**Steps**:

1. Create `tests/transform_tests.rs`
2. Create test fixture: `tests/fixtures/complete_config.toml`
   - Use example from spec Appendix A
   - Include all server types and tool targets
3. Test complete transformation pipeline:
   - Parse config
   - Expand variables
   - Transform for each tool
   - Validate output
4. For each tool:
   - Verify output is valid format (JSON/TOML parseable)
   - Verify correct servers included
   - Verify correct servers excluded
   - Verify tool-specific fields present/absent
5. Test transformation consistency:
   - Server with "all" target appears in all tools
   - Server with specific target only in that tool
   - HTTP servers excluded from Cursor
6. Test merge behavior:
   - Create existing opencode config with other sections
   - Transform and verify other sections preserved
   - Create existing codex config with other sections
   - Transform and verify other sections preserved
7. Test error cases:
   - Invalid server configuration
   - Unsupported server type for tool
8. Benchmark transformation performance:
   - 100 servers should transform in <100ms
9. Verify against spec examples in Appendix B
10. Run integration tests: `cargo test transform_tests`

**Files Modified**:
- `tests/transform_tests.rs`
- `tests/fixtures/complete_config.toml`
- `tests/fixtures/existing_opencode.json`
- `tests/fixtures/existing_codex.toml`

**Acceptance Criteria**:
- All spec examples pass
- Integration tests comprehensive
- Transformation pipeline works end-to-end
- Performance acceptable
- Coverage >= 80% for transform module

---

### 3.7: Add Transformer Validation and Error Handling

**Objective**: Ensure generated outputs are always valid

**Steps**:

1. Create `src/transform/validator.rs`
2. Implement output validation functions:
   ```rust
   pub fn validate_json(json: &str) -> Result<(), ValidationError>
   pub fn validate_toml(toml: &str) -> Result<(), ValidationError>
   ```
3. For JSON validation:
   - Parse with serde_json
   - Verify structure matches expected schema
   - Check all required fields present
4. For TOML validation:
   - Parse with toml crate
   - Verify structure matches expected schema
5. Add validation calls before returning from transformers
6. If validation fails:
   - Return error with details
   - Include generated content in error (sanitized)
   - Log as bug (should never happen if tests pass)
7. Implement output sanitization for error messages:
   - Redact bearer tokens
   - Redact env var values matching patterns
   - Pattern: API_KEY, TOKEN, SECRET, PASSWORD, CREDENTIAL
8. Write unit tests:
   - Valid outputs pass validation
   - Invalid outputs fail validation
   - Sanitization removes credentials
9. Test failure modes:
   - Invalid JSON structure
   - Invalid TOML structure
   - Missing required fields
10. Run tests: `cargo test validator`

**Files Modified**:
- `src/transform/validator.rs`
- `src/transform/cursor.rs` (add validation)
- `src/transform/opencode.rs` (add validation)
- `src/transform/codex.rs` (add validation)
- `src/error.rs` (add validation errors)

**Acceptance Criteria**:
- All transformer outputs validated
- Invalid outputs detected
- Credentials redacted in errors
- Validation tests pass
- No false positives

---

## Testing Strategy

**Unit Tests**:
- Target filtering logic
- Each transformer individually
- Tool-specific field handling
- Credential redaction
- Output validation

**Integration Tests**:
- Complete transformation pipeline
- Spec example transformations
- Merge behavior with existing configs
- Performance benchmarks

**Coverage Target**: 80% minimum for transform module

---

## Files Created/Modified

### New Files
- `src/transform/mod.rs` - Transformation module
- `src/transform/filter.rs` - Target filtering
- `src/transform/cursor.rs` - Cursor transformer
- `src/transform/opencode.rs` - opencode.ai transformer
- `src/transform/codex.rs` - Codex transformer
- `src/transform/claude.rs` - Claude Code transformer
- `src/transform/validator.rs` - Output validation
- `tests/transform_tests.rs` - Integration tests
- `tests/fixtures/complete_config.toml` - Test fixture
- `tests/fixtures/existing_opencode.json` - Test fixture
- `tests/fixtures/existing_codex.toml` - Test fixture

### Modified Files
- `src/error.rs` - Transform errors
- `src/lib.rs` - Module exports
- `Cargo.toml` - Dependencies

---

## Dependencies Added

```toml
[dependencies]
serde_json = "1.0"
# toml already added in Phase 1
```

---

## Commit Strategy

Each subtask should result in one conventional commit:

1. `feat(transform): create transformer module and target filtering`
2. `feat(transform): implement Cursor JSON transformer`
3. `feat(transform): implement opencode.ai JSON transformer`
4. `feat(transform): implement Codex TOML transformer`
5. `feat(transform): implement Claude Code transformer`
6. `test(transform): create integration tests for all transformers`
7. `feat(transform): add transformer validation and error handling`

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Spec examples don't match actual tool formats | High | Verify with actual tool installations |
| Claude Code config method unknown | Medium | Implement CLI approach, document limitations |
| Complex merge logic for existing configs | Medium | Comprehensive tests, careful TOML/JSON handling |
| Tool-specific field handling errors | Medium | Thorough test coverage of all field combinations |

---

## Reference Specifications

- Output specifications: `./architecture/05-output-specifications.md`
- Transformation rules: `./architecture/07-transformation-rules.md`
- Example outputs: `./architecture/16-appendices.md` Appendix B

---

## Phase Completion Checklist

- [ ] All subtasks completed and committed
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] Code coverage >= 80%
- [ ] No clippy warnings
- [ ] Code formatted with rustfmt
- [ ] All spec examples validated
- [ ] Phase review completed
- [ ] Ready for Phase 4

---

## Next Phase

After Phase 3 completion, proceed to [Phase 4: File Operations & Safety](phase_4.md)
