# 8. Failure Modes

This section enumerates all possible failure scenarios and required handling.

## 8.1 Configuration File Errors

### FM-1: Config File Not Found

**Scenario**: User runs compile/validate/diff without init, no config file exists

**Detection**: File existence check before open

**Handling**:
- Print: `Error: Configuration file not found: <path>`
- Print: `Run '<tool> init' to create a template configuration`
- Exit code: 2

### FM-2: Config File Unreadable

**Scenario**: Config file exists but lacks read permissions

**Detection**: Permission error on file open

**Handling**:
- Print: `Error: Cannot read configuration file: <path>`
- Print: `Permission denied. Check file permissions.`
- Exit code: 2

### FM-3: Invalid TOML Syntax

**Scenario**: Config file contains TOML syntax errors

**Detection**: TOML parser exception

**Handling**:
- Print: `Error: Invalid TOML syntax in <path>`
- Print: `<parser error message with line number>`
- Exit code: 3

### FM-4: Missing Required Section

**Scenario**: Config file lacks `[mcp.servers]` section

**Detection**: Schema validation after parse

**Handling**:
- Print: `Error: Configuration missing required section: [mcp.servers]`
- Print: `At least one MCP server must be defined`
- Exit code: 1

### FM-5: Missing Required Field

**Scenario**: Server definition lacks `command` (for STDIO) or `url` (for HTTP)

**Detection**: Schema validation

**Handling**:
- Print: `Error: Server '<name>' missing required field: command or url`
- Print: `STDIO servers require 'command', HTTP servers require 'url'`
- Exit code: 1

### FM-6: Invalid Field Type

**Scenario**: Field has wrong type (e.g., `args` is string instead of array)

**Detection**: Type checking during parse

**Handling**:
- Print: `Error: Invalid type for field 'args' in server '<name>'`
- Print: `Expected: array of strings, Got: <type>`
- Exit code: 1

### FM-7: Both Command and URL Present

**Scenario**: Server has both `command` and `url` fields

**Detection**: Schema validation

**Handling**:
- Print: `Error: Server '<name>' has both 'command' and 'url'`
- Print: `A server must be STDIO (command) OR HTTP (url), not both`
- Exit code: 1

## 8.2 Environment Variable Errors

### FM-8: Undefined Environment Variable

**Scenario**: Config references `${VAR}` not in shell environment

**Detection**: During expansion, variable not in os.environ

**Handling**:
- Print: `Warning: Undefined environment variable: VAR`
- Print: `Expanding to empty string`
- Continue execution (exit code: 0, but warn)

### FM-9: Circular Variable Reference

**Scenario**: `{VAR1} = "{VAR2}"`, `{VAR2} = "{VAR1}"`

**Detection**: Expansion depth exceeds limit (10)

**Handling**:
- Print: `Error: Circular variable reference detected in [env] section`
- Print: `Check definitions of: <involved variables>`
- Exit code: 1

### FM-10: Undefined Variable in [env] Section

**Scenario**: Config references `{VAR}` not defined in `[env]`

**Detection**: During expansion, variable not in env_section

**Handling**:
- Print: `Warning: Undefined variable in [env] section: VAR`
- Print: `Expanding to empty string`
- Continue execution (exit code: 0, but warn)

## 8.3 File Write Errors

### FM-11: Output Directory Not Exists

**Scenario**: Parent directory for output file doesn't exist

**Detection**: Directory check before write

**Handling**:
- Attempt to create parent directories (mkdir -p equivalent)
- If creation fails:
  - Print: `Error: Cannot create directory: <path>`
  - Print: `<OS error message>`
  - Exit code: 2

### FM-12: Output File Unwritable

**Scenario**: Lack write permissions for output location

**Detection**: Permission error on file write

**Handling**:
- Print: `Error: Cannot write to <path>`
- Print: `Permission denied. Check file permissions.`
- Print: `No changes were made (backup preserved if exists)`
- Exit code: 2

### FM-13: Disk Full

**Scenario**: Insufficient disk space for write

**Detection**: OS error during write (ENOSPC)

**Handling**:
- Print: `Error: Cannot write to <path>`
- Print: `Disk full or quota exceeded`
- Print: `No changes were made (backup preserved if exists)`
- Exit code: 2

### FM-14: Backup Creation Failed

**Scenario**: Cannot create backup of existing file

**Detection**: Error during backup copy operation

**Handling**:
- Print: `Error: Cannot create backup of <path>`
- Print: `<OS error message>`
- Print: `Aborting to prevent data loss`
- Exit code: 2
- MUST NOT continue with write operation

## 8.4 Tool-Specific Errors

### FM-15: Claude CLI Not Available

**Scenario**: Claude Code selected but `claude` command not found

**Detection**: Command existence check (which/where)

**Handling**:
- Print: `Warning: 'claude' command not found`
- Print: `Skipping Claude Code configuration`
- Print: `Install Claude Code CLI or remove 'claude-code' from targets`
- Continue with other tools (exit code: 0, but warn)

### FM-16: Claude CLI Command Failed

**Scenario**: `claude mcp add` command returns non-zero exit code

**Detection**: Process exit code check

**Handling**:
- Print: `Error: Failed to add MCP server to Claude Code`
- Print: `Server: <name>`
- Print: `Command output: <stderr>`
- Exit code: 3 (partial failure)

### FM-17: Invalid JSON Generated

**Scenario**: Generated JSON fails to parse (implementation bug)

**Detection**: Validate generated JSON before write

**Handling**:
- Print: `Error: Generated invalid JSON for <tool>`
- Print: `This is a bug. Please report with your configuration.`
- Print: `Generated content: <sanitized JSON>`
- Exit code: 1

### FM-18: Invalid TOML Generated

**Scenario**: Generated TOML fails to parse (implementation bug)

**Detection**: Validate generated TOML before write

**Handling**:
- Print: `Error: Generated invalid TOML for Codex`
- Print: `This is a bug. Please report with your configuration.`
- Print: `Generated content: <sanitized TOML>`
- Exit code: 1

## 8.5 Concurrent Access Errors

### FM-19: Config File Modified During Execution

**Scenario**: Another process modifies config while tool is running

**Detection**: Compare file modification time before and after read

**Handling**:
- Print: `Warning: Configuration file modified during execution`
- Print: `Using version read at: <timestamp>`
- Continue (changes take effect on next run)

### FM-20: Output File Modified During Execution

**Scenario**: Tool modifies output file between backup and write

**Detection**: Compare backup hash with current file before write

**Handling**:
- Print: `Error: Output file modified since backup: <path>`
- Print: `Another process may have changed the file`
- Print: `Aborting to prevent data loss (backup preserved)`
- Exit code: 2

### FM-21: Lock File Exists

**Scenario**: Another instance of tool is running (if lock files used)

**Detection**: Lock file existence check

**Handling**:
- Print: `Error: Another instance is already running`
- Print: `If this is incorrect, remove: <lock-file-path>`
- Exit code: 4

## 8.6 Validation Errors

### FM-22: Invalid Tool Name

**Scenario**: `targets` or `--tool` specifies unknown tool

**Detection**: Tool name validation

**Handling**:
- Print: `Error: Unknown tool name: '<name>'`
- Print: `Valid tools: claude-code, cursor, opencode, codex`
- Exit code: 1

### FM-23: Invalid Targets Combination

**Scenario**: Server has empty `targets` array

**Detection**: After target resolution

**Handling**:
- Print: `Warning: Server '<name>' has no targets`
- Print: `Server will not be compiled for any tool`
- Continue (not an error, might be intentional)

### FM-24: No Servers for Any Tool

**Scenario**: All servers disabled or no targets match

**Detection**: After filtering servers for all tools

**Handling**:
- Print: `Warning: No servers configured for any tool`
- Print: `Check 'enabled' and 'targets' fields`
- Exit code: 0 (not an error, but unusual)

### FM-25: Executable Not Found

**Scenario**: `command` field references non-existent executable

**Detection**: Executable existence check (which/where)

**Handling**:
- Print: `Warning: Command not found: '<command>' (server '<name>')`
- Print: `Server may fail to start on target tool`
- Continue (validation only, tool might be installed elsewhere)
