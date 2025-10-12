# 9. Edge Cases

This section documents unusual scenarios and required behavior.

## 9.1 Configuration Edge Cases

### EC-1: Empty Configuration

**Scenario**: Config file exists but has no MCP servers

**Behavior**:
- Validation: Pass with warning "No MCP servers defined"
- Compile: Generate empty/minimal config for each tool
- Exit code: 0

### EC-2: Server with Empty Args

**Scenario**: `args = []` (empty array)

**Behavior**:
- Valid configuration
- Generate output with empty args array
- JSON: `"args": []`
- Command line: No args passed

### EC-3: Server with Empty Env

**Scenario**: `env = {}` (empty table)

**Behavior**:
- Valid configuration
- Omit `env` field from output (treat as not present)

### EC-4: Targets Contains "all" and Specific Tools

**Scenario**: `targets = ["all", "cursor"]`

**Behavior**:
- "all" takes precedence
- Equivalent to `targets = ["claude-code", "cursor", "opencode", "codex"]`
- No duplicates in processing

### EC-5: Duplicate Server Names

**Scenario**: Two `[mcp.servers.github]` sections

**Behavior**:
- TOML parser behavior: Later definition overwrites earlier
- Tool accepts whichever definition TOML parser returns
- Warn: "Duplicate server name detected (TOML parser handled)"

## 9.2 Environment Variable Edge Cases

### EC-6: Environment Variable with Empty Value

**Scenario**: `export MYVAR=""`, config uses `${MYVAR}`

**Behavior**:
- Expand to empty string
- No warning (empty is a valid value)

### EC-7: Variable Name Contains Special Characters

**Scenario**: `{MY-VAR}` or `{MY.VAR}` (non-alphanumeric)

**Behavior**:
- If valid TOML key: attempt expansion
- If not in env section: warning and empty string
- Recommendation: Use alphanumeric + underscore only

### EC-8: Nested Variable References

**Scenario**:
```toml
[env]
A = "value"
B = "{A}_suffix"
C = "{B}_more"
```

**Behavior**:
- Resolve in order: A → B (becomes "value_suffix") → C (becomes "value_suffix_more")
- Maximum 10 levels deep

### EC-9: Variable Reference in Server Name

**Scenario**: `[mcp.servers.{SERVERNAME}]`

**Behavior**:
- NOT supported (TOML keys evaluated at parse time)
- Treat as literal string (server named "{SERVERNAME}")
- Document: Variable expansion only in values, not keys

## 9.3 File System Edge Cases

### EC-10: Output Path is Symlink

**Scenario**: `.cursor/mcp.json` is symlink to another file

**Behavior**:
- Follow symlink for read (to create backup)
- Write to symlink target (normal symlink behavior)
- Backup created at symlink location, not target

### EC-11: Output Path is Directory

**Scenario**: User has directory named `.cursor/mcp.json`

**Behavior**:
- Detect: path exists and is directory
- Print: `Error: Expected file, found directory: <path>`
- Exit code: 2

### EC-12: Output Directory is Read-Only

**Scenario**: `.cursor/` directory has no write permission

**Behavior**:
- Cannot create/modify `mcp.json`
- Print: `Error: Cannot write to <path>`
- Print: `Parent directory is read-only`
- Exit code: 2

### EC-13: Very Long File Path

**Scenario**: Config generates path > 260 chars (Windows) or > 4096 chars (Linux)

**Behavior**:
- Attempt write normally
- If OS error (path too long):
  - Print: `Error: File path too long: <path>`
  - Exit code: 2

### EC-14: Special Characters in Path

**Scenario**: Server name contains spaces, quotes, or non-ASCII

**Behavior**:
- Path construction: Use server name as-is
- Sanitization: Replace `/` with `-` (directory separator conflict)
- Document: Recommend alphanumeric + hyphen for server names

## 9.4 Tool-Specific Edge Cases

### EC-15: Cursor: Project Has No .cursor Directory

**Scenario**: Running compile in directory without `.cursor/`

**Behavior**:
- Create `.cursor/` directory
- Create `mcp.json` inside
- Set appropriate permissions (755 for directory, 644 for file)

### EC-16: opencode.ai: Existing Config Has Unrelated Settings

**Scenario**: `opencode.json` has `model`, `provider`, etc.

**Behavior**:
- Preserve all fields except `mcp`
- Only replace `mcp` section
- Validate JSON structure after merge

### EC-17: Codex: Existing Config Has Comments

**Scenario**: `config.toml` has TOML comments

**Behavior**:
- Comments outside `[mcp_servers.*]` sections: preserved
- Comments inside `[mcp_servers.*]` sections: lost (section replaced)
- Document: "Comments in MCP sections will be removed"

### EC-18: Server Enabled for Tool That Doesn't Support It

**Scenario**: HTTP server targeted at Cursor (Cursor doesn't support HTTP MCP)

**Behavior**:
- Detect: Server has `url` field, tool is Cursor
- Print: `Warning: Server '<name>' is HTTP, skipping for Cursor (unsupported)`
- Exclude from Cursor output

## 9.5 Command-Line Edge Cases

### EC-19: Multiple --tool Flags

**Scenario**: `compile --tool cursor --tool opencode`

**Behavior**:
- Compile for both specified tools
- Process in order specified
- Exit code: 0 if all succeed, 3 if any fail

### EC-20: --tool Repeated with Same Value

**Scenario**: `compile --tool cursor --tool cursor`

**Behavior**:
- Deduplicate: treat as single `--tool cursor`
- Warn: "Duplicate tool specified: cursor"

### EC-21: Dry-Run with No Output

**Scenario**: `compile --dry-run` when no changes needed

**Behavior**:
- Print: "No changes needed (configurations up-to-date)"
- Exit code: 0

### EC-22: Diff on Non-Existent File

**Scenario**: `diff` when target tool has no existing config

**Behavior**:
- Show: "[NEW FILE]" with full content
- Not an error

## 9.6 Encoding Edge Cases

### EC-23: Config Contains Non-ASCII Characters

**Scenario**: Server name or value has emoji, Chinese, etc.

**Behavior**:
- Accept if valid UTF-8
- Preserve in output files
- JSON: Escape unicode sequences (`\uXXXX`) or UTF-8
- TOML: UTF-8 encoding

### EC-24: Config Contains Windows Line Endings (CRLF)

**Scenario**: Config file created on Windows

**Behavior**:
- TOML parser MUST handle both LF and CRLF
- Output files use platform-native line endings (LF on Unix, CRLF on Windows)

### EC-25: Config Contains NUL Bytes

**Scenario**: Binary data in config file

**Behavior**:
- TOML parser error (invalid character)
- Print: `Error: Invalid character in configuration file`
- Exit code: 3

## 9.7 State Tracking Edge Cases

### EC-26: State File Corrupted

**Scenario**: `generated.json` is invalid JSON

**Behavior**:
- Print: `Warning: State file corrupted, recreating`
- Delete state file
- Proceed as if no state exists
- Continue normally

### EC-27: State File References Non-Existent File

**Scenario**: State says file was generated, but it doesn't exist

**Behavior**:
- Print: `Warning: Generated file missing: <path>`
- Remove from state
- Regenerate on next compile

### EC-28: Generated File Modified by User

**Scenario**: User manually edits output file after generation

**Behavior**:
- Detect: Hash in state doesn't match current file
- Print: `Warning: <path> was manually modified`
- Print: `Overwriting with generated configuration (backup created)`
- Continue with write
