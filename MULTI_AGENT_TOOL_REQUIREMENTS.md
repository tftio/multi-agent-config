# Multi-Agent AI Coding Tool Configuration Manager - Requirements Specification

**Version**: 1.0
**Last Updated**: 2025-10-12
**Status**: Draft

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Problem Statement](#2-problem-statement)
3. [System Overview](#3-system-overview)
4. [Input Specifications](#4-input-specifications)
5. [Output Specifications](#5-output-specifications)
6. [Invariants](#6-invariants)
7. [Transformation Rules](#7-transformation-rules)
8. [Failure Modes](#8-failure-modes)
9. [Edge Cases](#9-edge-cases)
10. [Security Requirements](#10-security-requirements)
11. [Operational Requirements](#11-operational-requirements)
12. [Configuration Schema Reference](#12-configuration-schema-reference)
13. [Error Handling Specifications](#13-error-handling-specifications)
14. [Testing Requirements](#14-testing-requirements)
15. [Future Extensibility](#15-future-extensibility)
16. [Appendices](#16-appendices)

---

## 1. Executive Summary

### 1.1 Purpose

This document specifies the requirements for a unified configuration management tool that compiles a single TOML configuration into tool-specific formats for multiple AI coding assistants.

### 1.2 Target Tools

The system MUST support the following AI coding tools:

1. **Claude Code** - Anthropic's CLI coding assistant
2. **Cursor** - AI-powered code editor
3. **opencode.ai** - Terminal-based AI coding agent
4. **OpenAI Codex** - OpenAI's CLI coding assistant

### 1.3 Scope

The tool MUST:
- Parse a unified TOML configuration file
- Generate tool-specific MCP (Model Context Protocol) server configurations
- Support environment variable expansion
- Validate configurations before application
- Provide dry-run and diff capabilities
- Handle existing configurations safely (backup, merge)
- Track generated files for safe updates

The tool MAY in future versions:
- Manage prompt files across tools
- Support custom agents/modes
- Provide watch mode for auto-compilation

### 1.4 High-Level Architecture

```
┌─────────────────────────────────────┐
│  Unified Configuration (TOML)       │
│  ~/.config/agent-sync/config.toml   │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  Configuration Compiler              │
│  - Parse TOML                        │
│  - Expand environment variables      │
│  - Validate schema                   │
│  - Filter by target                  │
│  - Transform to tool formats         │
└──────────────┬──────────────────────┘
               │
               ├──────┬──────┬──────┬──────┐
               ▼      ▼      ▼      ▼      │
            Claude  Cursor opencode Codex  │
            Config  JSON   JSON     TOML   │
```

---

## 2. Problem Statement

### 2.1 Current Pain Points

1. **Configuration Duplication**: Users must manually maintain identical MCP server configurations across 4+ different tools
2. **Inconsistency Risk**: Manual updates lead to configuration drift between tools
3. **Credential Management**: API keys and secrets duplicated across multiple files
4. **Update Overhead**: Adding/removing a single MCP server requires editing 4+ files
5. **Format Complexity**: Each tool uses different JSON/TOML schemas

### 2.2 Desired Outcomes

1. **Single Source of Truth**: Define MCP servers once in unified format
2. **Automatic Consistency**: All tools receive identical server configurations
3. **Centralized Secrets**: Environment variables defined once, expanded everywhere
4. **Simple Updates**: Add/remove/modify servers in one location
5. **Safety**: Validate before applying, backup existing configs, rollback on failure

### 2.3 Success Criteria

- User can define all MCP servers in one TOML file
- Tool generates valid configurations for all 4 target tools
- Configurations remain synchronized across tools
- No manual editing of tool-specific config files required
- Existing configurations safely backed up before modification
- Invalid configurations detected before application

---

## 3. System Overview

### 3.1 Components

1. **Configuration Parser**: Reads and validates unified TOML
2. **Environment Expander**: Resolves environment variable references
3. **Schema Validator**: Validates against required fields and types
4. **Target Filter**: Selects servers for specific tools
5. **Format Transformer**: Converts unified format to tool-specific schemas
6. **File Writer**: Safely writes configurations with backups
7. **State Tracker**: Records generated files for updates
8. **Diff Generator**: Compares current vs new configurations
9. **CLI Interface**: Provides user commands

### 3.2 Data Flow

```
Input TOML
    ↓
Parse & Validate
    ↓
Expand Environment Variables
    ↓
For Each Target Tool:
    ↓
    Filter Servers (by 'targets' field)
    ↓
    Transform to Tool Schema
    ↓
    Validate Output Schema
    ↓
    Backup Existing Config (if exists)
    ↓
    Write New Config
    ↓
    Update State Tracker
```

### 3.3 Execution Modes

1. **init**: Create template configuration
2. **validate**: Check configuration correctness without writing
3. **compile**: Generate and write all tool configurations
4. **diff**: Show what would change without writing
5. **watch** (future): Monitor config file and auto-compile on changes

---

## 4. Input Specifications

### 4.1 Primary Input: Unified Configuration File

**Location**: `~/.config/agent-sync/config.toml` (default) or user-specified path

**Format**: TOML (Tom's Obvious, Minimal Language)

**Encoding**: UTF-8

**Line Endings**: LF (Unix) or CRLF (Windows) - tool MUST handle both

### 4.2 Unified Configuration Schema

#### 4.2.1 Top-Level Structure

```toml
[settings]
# Settings section (optional)

[env]
# Environment variable definitions (optional)

[mcp.servers.<server-name>]
# MCP server definitions (required - at least one)

[tools.<tool-name>]
# Tool-specific overrides (optional, future)

[prompts.<prompt-name>]
# Prompt definitions (optional, future)
```

#### 4.2.2 Settings Section

**Purpose**: Global tool configuration

**Schema**:

```toml
[settings]
version = "1.0"                    # Required: string, semver format
default_targets = ["tool1", ...]  # Optional: array of strings, default ["cursor", "opencode", "codex"]
```

**Field Specifications**:

- `version` (required): String matching semantic versioning (MAJOR.MINOR.PATCH)
  - MUST be present
  - Used for schema evolution
  - Current version: "1.0"

- `default_targets` (optional): Array of strings
  - Valid values: `["claude-code", "cursor", "opencode", "codex"]`
  - Default if omitted: `["cursor", "opencode", "codex"]`
  - Used when servers don't specify `targets` field
  - Empty array is valid (compiles nothing)

**Validation Rules**:
- `version` MUST match regex: `^\d+\.\d+(\.\d+)?$`
- `default_targets` elements MUST be one of the valid tool names
- Duplicate tool names in array MUST be deduplicated

#### 4.2.3 Environment Variables Section

**Purpose**: Define reusable environment variable references

**Schema**:

```toml
[env]
VAR_NAME = "value or ${OTHER_VAR}"
ANOTHER_VAR = "${SHELL_ENV_VAR}"
```

**Field Specifications**:

- Key: Any valid TOML key (alphanumeric, underscore, hyphen)
- Value: String that may contain:
  - Literal values
  - `${VAR_NAME}` references to shell environment variables
  - `{VAR_NAME}` references to other vars in this section

**Resolution Order**:
1. Variables in `[env]` section are resolved in definition order
2. Within a value, `${VAR}` is resolved from shell environment
3. Within a value, `{VAR}` is resolved from `[env]` section (after all definitions loaded)
4. Circular references MUST be detected and rejected

**Validation Rules**:
- Undefined variable references MUST emit warning but not fail (expand to empty string)
- Circular references MUST cause validation failure
- Maximum expansion depth: 10 levels

#### 4.2.4 MCP Server Section

**Purpose**: Define MCP servers to be provisioned across tools

**Schema**:

```toml
[mcp.servers.<server-name>]
command = "executable"           # Required for STDIO servers
args = ["arg1", "arg2"]         # Optional: default []
env = { KEY = "value" }         # Optional: environment variables
enabled = true                   # Optional: default true
targets = ["cursor", "opencode"] # Optional: default from settings.default_targets
url = "https://example.com"     # Required for HTTP servers (mutually exclusive with command)
bearer_token = "token"          # Optional: for HTTP servers
disabled = false                # Optional: Cursor-specific, default false
autoApprove = []                # Optional: Cursor-specific, array of strings
startup_timeout_sec = 30        # Optional: Codex-specific, default 30
tool_timeout_sec = 60           # Optional: Codex-specific, default 60
```

**Field Specifications**:

- `<server-name>`: Valid TOML key, used as identifier across all tools
  - MUST be unique within `[mcp.servers]`
  - SHOULD be lowercase with hyphens (e.g., "github", "context-7")
  - MUST NOT contain dots (conflicts with TOML table syntax)

- `command` (required for STDIO, prohibited for HTTP): String
  - Path to executable or command name
  - MUST be executable on target system (validation warning if not found)
  - Examples: "npx", "docker", "/usr/local/bin/server", "uvx"

- `args` (optional): Array of strings
  - Arguments passed to command
  - Default: `[]` (empty array)
  - Order preserved
  - May contain environment variable references: `{VAR}` or `${VAR}`

- `env` (optional): Inline table or table section
  - Key-value pairs of environment variables for the server process
  - Keys: String (environment variable names)
  - Values: String (may contain `{VAR}` or `${VAR}` references)
  - Examples:
    ```toml
    env = { KEY = "value", TOKEN = "{API_TOKEN}" }
    # OR
    [mcp.servers.myserver.env]
    KEY = "value"
    TOKEN = "{API_TOKEN}"
    ```

- `enabled` (optional): Boolean
  - Default: `true`
  - If `false`, server is excluded from compilation
  - Useful for temporarily disabling without deletion

- `targets` (optional): Array of strings
  - Valid values: `["claude-code", "cursor", "opencode", "codex", "all"]`
  - Default: value from `settings.default_targets`
  - `"all"` expands to all four tools
  - Empty array means server is not compiled for any tool

- `url` (required for HTTP servers): String
  - HTTP/HTTPS URL for remote MCP server
  - MUST start with `http://` or `https://`
  - Mutually exclusive with `command` field
  - Examples: `"https://mcp.figma.com/mcp"`

- `bearer_token` (optional, HTTP only): String
  - Bearer token for Authorization header
  - May contain environment variable references
  - Only used with HTTP servers (url field present)

- `disabled` (optional, Cursor-specific): Boolean
  - Default: `false`
  - Maps to Cursor's "disabled" field
  - Distinct from `enabled` (enabled controls compilation, disabled is in output)

- `autoApprove` (optional, Cursor-specific): Array of strings
  - Tool names to auto-approve in Cursor
  - Only included in Cursor output

- `startup_timeout_sec` (optional, Codex-specific): Integer
  - Seconds to wait for server startup
  - Default: 30
  - Only included in Codex output

- `tool_timeout_sec` (optional, Codex-specific): Integer
  - Seconds to wait for tool execution
  - Default: 60
  - Only included in Codex output

**Validation Rules**:

1. Each server MUST have either `command` OR `url`, not both
2. If `command` present, it's a STDIO server
3. If `url` present, it's an HTTP server
4. `bearer_token` only valid with `url`
5. Tool-specific fields silently omitted for other tools
6. Server with no matching `targets` emits warning
7. `enabled = false` servers MUST be completely excluded from output

### 4.3 Command-Line Interface

#### 4.3.1 General Syntax

```
<tool-name> [global-options] <command> [command-options]
```

**Tool Name**: Implementation-defined (e.g., `agent-sync`, `ai-sync`, `mcp-sync`)

#### 4.3.2 Global Options

```
--config <path>     Path to config.toml (default: ~/.config/agent-sync/config.toml)
--help              Show help message and exit
--version           Show version and exit
```

#### 4.3.3 Commands

##### 4.3.3.1 `init`

**Purpose**: Create template configuration file

**Syntax**: `<tool> init [--force]`

**Options**:
- `--force`: Overwrite existing config (requires confirmation)

**Behavior**:
1. Check if config file exists
2. If exists and no `--force`: error "Config file already exists"
3. If exists and `--force`: backup to `config.toml.backup.<timestamp>`
4. Create directory `~/.config/agent-sync/` if not exists
5. Write template config with commented examples
6. Exit with code 0

**Exit Codes**:
- 0: Success
- 1: File exists and no --force
- 2: Permission denied writing file
- 3: Parent directory not creatable

##### 4.3.3.2 `validate`

**Purpose**: Validate configuration without writing files

**Syntax**: `<tool> validate [--config <path>]`

**Behavior**:
1. Load configuration file
2. Parse TOML (fail on syntax errors)
3. Validate schema (check required fields, types)
4. Expand environment variables (warn on undefined)
5. Check for circular references
6. Validate tool names in targets
7. For each server, simulate transformation to each tool format
8. Report validation results
9. Exit with code 0 if valid, 1 if invalid

**Output**:
- On success: "Configuration is valid"
- On warnings: List warnings but exit 0
- On errors: List all errors and exit 1

**Exit Codes**:
- 0: Valid configuration
- 1: Invalid configuration
- 2: File not found
- 3: Parse error

##### 4.3.3.3 `compile`

**Purpose**: Generate and write tool configurations

**Syntax**: `<tool> compile [--tool <name>]... [--dry-run] [--config <path>]`

**Options**:
- `--tool <name>`: Compile only for specified tool (repeatable)
  - Valid values: `claude-code`, `cursor`, `opencode`, `codex`
  - If omitted: compile for all tools with matching servers
- `--dry-run`: Show what would be done without writing files

**Behavior**:
1. Load and validate configuration
2. For each target tool (filtered by --tool if specified):
   a. Filter servers by targets field
   b. Skip if no servers match this tool
   c. Transform to tool-specific format
   d. Determine output path for tool
   e. If not --dry-run:
      - Backup existing config to `<filename>.backup`
      - Create parent directories if needed
      - Write new configuration atomically
      - Update state tracker
   f. If --dry-run:
      - Print what would be written
3. Print summary of changes
4. Exit with code 0 if successful

**Atomicity Requirement**:
- File writes MUST be atomic (write to temp file, then rename)
- If any write fails, previous backups MUST remain
- State tracker updated only after successful write

**Exit Codes**:
- 0: Success (all tools compiled)
- 1: Validation error
- 2: Write error (permission, disk full)
- 3: Partial failure (some tools failed, some succeeded)

##### 4.3.3.4 `diff`

**Purpose**: Show what would change when compiling

**Syntax**: `<tool> diff [--tool <name>]... [--config <path>]`

**Options**:
- `--tool <name>`: Show diff only for specified tool

**Behavior**:
1. Load and validate configuration
2. For each target tool:
   a. Generate new configuration
   b. Read existing configuration if present
   c. Display diff:
      - If file doesn't exist: "[NEW FILE]" + full content
      - If file exists: unified diff format
   d. Show summary of changes

**Output Format**:
```
================================================================================
Tool: cursor
Path: /path/to/.cursor/mcp.json
================================================================================
[NEW FILE]
{
  "mcpServers": { ... }
}

================================================================================
Tool: opencode
Path: /path/to/.config/opencode/opencode.json
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

**Exit Codes**:
- 0: Success (diff displayed)
- 1: Validation error
- 2: File read error

### 4.4 Secondary Inputs

#### 4.4.1 Environment Variables

**Source**: Shell environment at time of execution

**Access**: Via `${VAR_NAME}` syntax in configuration

**Security**:
- Sensitive values (tokens, passwords) SHOULD be in environment, not config file
- Tool MUST NOT log or display environment variable values
- Undefined environment variables SHOULD produce warning but expand to empty string

#### 4.4.2 Existing Tool Configurations

**Purpose**: For backup and merge operations

**Behavior**:
- Tool MUST read existing config files before writing
- Tool MUST create backups with `.backup` extension
- Backups MUST preserve original timestamps

---

## 5. Output Specifications

### 5.1 Output Files Overview

| Tool | Format | Default Path |
|------|--------|--------------|
| Claude Code | JSON or CLI | `~/.config/claude-code/mcp.json` (TBD) or CLI commands |
| Cursor | JSON | `.cursor/mcp.json` (project-level) |
| opencode.ai | JSON | `~/.config/opencode/opencode.json` |
| OpenAI Codex | TOML | `~/.codex/config.toml` |

### 5.2 Claude Code Output

**Status**: Configuration method TBD - may use CLI commands or JSON file

**Option A: CLI-based (current preference)**

Execute: `claude mcp add <name> [--env KEY=VALUE]... -- <command> [args...]`

For each server with `claude-code` in targets:
```bash
claude mcp add <server-name> \
  --env KEY1=VALUE1 \
  --env KEY2=VALUE2 \
  -- <command> <arg1> <arg2>
```

**Requirements**:
- Tool MUST check if `claude` CLI is available
- Tool MUST execute commands sequentially
- Tool MUST capture and report any errors
- Tool SHOULD first remove existing servers: `claude mcp remove <name>`

**Option B: File-based (if available)**

**Path**: `~/.config/claude-code/mcp.json` (exact path TBD)

**Schema**: TBD - similar to Cursor format

### 5.3 Cursor Output

**Path**: `.cursor/mcp.json` (relative to current working directory)

**Format**: JSON

**Encoding**: UTF-8

**Indentation**: 2 spaces

**Schema**:

```json
{
  "mcpServers": {
    "<server-name>": {
      "command": "string",
      "args": ["string", ...],
      "env": {
        "KEY": "value",
        ...
      },
      "disabled": false,
      "autoApprove": ["string", ...]
    },
    ...
  }
}
```

**Field Specifications**:

- `mcpServers` (required): Object containing all MCP servers

- `<server-name>` (required): Server identifier from input

- `command` (required): String
  - Direct copy from input `command` field
  - Environment variables MUST be expanded

- `args` (required): Array of strings
  - Direct copy from input `args` field
  - Empty array if no args in input
  - Environment variables in args MUST be expanded

- `env` (optional): Object
  - Only included if input has `env` field
  - Keys: Environment variable names (strings)
  - Values: Expanded environment variable values (strings)
  - Environment variables MUST be expanded

- `disabled` (optional): Boolean
  - Only included if input has `disabled` field
  - Maps from input `disabled` or inverted `enabled`

- `autoApprove` (optional): Array of strings
  - Only included if input has `autoApprove` field
  - Direct copy from input

**Transformation Rules**:

1. Only servers with `cursor` in `targets` field are included
2. Servers with `enabled = false` are excluded entirely
3. HTTP servers (with `url` field) are excluded (Cursor doesn't support)
4. Environment variable references MUST be expanded
5. Field order in JSON is not significant
6. Unknown fields from input are silently ignored
7. Output MUST be valid JSON (pass `json.loads()` or equivalent)

**Example**:

Input:
```toml
[mcp.servers.github]
command = "docker"
args = ["run", "-i", "--rm", "-e", "GITHUB_PERSONAL_ACCESS_TOKEN", "ghcr.io/github/github-mcp-server"]
env = { GITHUB_PERSONAL_ACCESS_TOKEN = "{GITHUB_TOKEN}" }
enabled = true
targets = ["all"]
```

Output (assuming `GITHUB_TOKEN` in env is `ghp_abc123`):
```json
{
  "mcpServers": {
    "github": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "-e", "GITHUB_PERSONAL_ACCESS_TOKEN", "ghcr.io/github/github-mcp-server"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_abc123"
      }
    }
  }
}
```

### 5.4 opencode.ai Output

**Path**: `~/.config/opencode/opencode.json`

**Format**: JSON

**Encoding**: UTF-8

**Indentation**: 2 spaces

**Schema**:

```json
{
  "mcp": {
    "<server-name>": {
      "type": "local",
      "command": ["string", ...],
      "env": {
        "KEY": "value",
        ...
      },
      "enabled": true
    },
    ...
  },
  ... (other sections preserved from existing file)
}
```

**Field Specifications**:

- `mcp` (required): Object containing all MCP servers
  - If existing file has other top-level keys, they MUST be preserved
  - Only the `mcp` section is replaced

- `<server-name>` (required): Server identifier from input

- `type` (required): String
  - For STDIO servers (command field): `"local"`
  - For HTTP servers (url field): `"remote"`

- `command` (required for local): Array of strings
  - First element: value from input `command` field
  - Remaining elements: values from input `args` array
  - Example: input `command="npx"`, `args=["-y", "pkg"]` → output `command=["npx", "-y", "pkg"]`
  - Environment variables MUST be expanded

- `url` (required for remote): String
  - Direct copy from input `url` field
  - Only present when `type = "remote"`

- `headers` (optional, remote only): Object
  - If input has `bearer_token`: include `{"Authorization": "Bearer <token>"}`
  - If input has `env` for remote servers: include as headers

- `env` (optional, local only): Object
  - Only included if input has `env` field
  - Keys: Environment variable names (strings)
  - Values: Expanded environment variable values (strings)

- `enabled` (required): Boolean
  - Always `true` (servers with `enabled=false` are excluded)

**Transformation Rules**:

1. Only servers with `opencode` in `targets` field are included
2. Servers with `enabled = false` are excluded entirely
3. Read existing `~/.config/opencode/opencode.json` if present
4. Preserve all top-level keys except `mcp`
5. Replace entire `mcp` section with generated configuration
6. If no existing file, create with only `mcp` section
7. Environment variables MUST be expanded
8. Output MUST be valid JSON

**Example**:

Input:
```toml
[mcp.servers.context7]
command = "npx"
args = ["-y", "@upstash/context7-mcp@latest"]
enabled = true
targets = ["all"]
```

Output:
```json
{
  "mcp": {
    "context7": {
      "type": "local",
      "command": ["npx", "-y", "@upstash/context7-mcp@latest"],
      "enabled": true
    }
  }
}
```

### 5.5 OpenAI Codex Output

**Path**: `~/.codex/config.toml`

**Format**: TOML

**Encoding**: UTF-8

**Schema**:

```toml
[mcp_servers.<server-name>]
command = "string"
args = ["string", ...]
startup_timeout_sec = 30
tool_timeout_sec = 60

[mcp_servers.<server-name>.env]
KEY = "value"

[mcp_servers.<http-server-name>]
url = "https://example.com"
bearer_token = "token"
```

**Field Specifications**:

- `[mcp_servers.<server-name>]` (required): TOML table for each server
  - Note: Different prefix from input (`mcp_servers` vs `mcp.servers`)

- STDIO servers:
  - `command` (required): String from input `command` field
  - `args` (optional): Array from input `args` field
  - `env` section (optional): As `[mcp_servers.<server-name>.env]` subtable
  - `startup_timeout_sec` (optional): From input, default 30
  - `tool_timeout_sec` (optional): From input, default 60

- HTTP servers:
  - `url` (required): String from input `url` field
  - `bearer_token` (optional): String from input `bearer_token` field

**Transformation Rules**:

1. Only servers with `codex` in `targets` field are included
2. Servers with `enabled = false` are excluded entirely
3. Read existing `~/.codex/config.toml` if present
4. Preserve all sections except `[mcp_servers.*]`
5. Replace all `[mcp_servers.*]` sections with generated configuration
6. Environment variables MUST be expanded
7. Cursor-specific fields (`disabled`, `autoApprove`) are omitted
8. If input has both STDIO and HTTP servers for Codex, both are included
9. Output MUST be valid TOML

**Example**:

Input:
```toml
[mcp.servers.context7]
command = "npx"
args = ["-y", "@upstash/context7-mcp@latest"]
targets = ["codex"]
startup_timeout_sec = 45

[mcp.servers.figma]
url = "https://mcp.figma.com/mcp"
bearer_token = "{FIGMA_TOKEN}"
targets = ["codex"]
```

Output (assuming `FIGMA_TOKEN=fig_abc123`):
```toml
[mcp_servers.context7]
command = "npx"
args = ["-y", "@upstash/context7-mcp@latest"]
startup_timeout_sec = 45

[mcp_servers.figma]
url = "https://mcp.figma.com/mcp"
bearer_token = "fig_abc123"
```

### 5.6 Backup Files

**Naming Convention**: `<original-filename>.backup`

**Behavior**:
- Created before any write operation
- Contains exact copy of existing file
- Preserves original file permissions
- Only created if target file exists
- Each compile operation overwrites previous backup (single backup level)

**Example**:
- Original: `.cursor/mcp.json`
- Backup: `.cursor/mcp.json.backup`

### 5.7 State Tracking File

**Path**: `~/.config/agent-sync/state/generated.json`

**Purpose**: Track which files were generated by this tool

**Format**: JSON

**Schema**:

```json
{
  "version": "1.0",
  "last_compile": "2025-10-12T15:30:00Z",
  "generated_files": [
    {
      "tool": "cursor",
      "path": "/path/to/.cursor/mcp.json",
      "timestamp": "2025-10-12T15:30:00Z",
      "hash": "sha256:abc123..."
    },
    ...
  ]
}
```

**Usage**:
- Updated after each successful compile
- Used to detect manual modifications
- Enables safe updates (only overwrite if hash matches)
- Supports future rollback functionality

---

## 6. Invariants

These properties MUST hold true at all times during and after tool execution:

### 6.1 Configuration Consistency

**INV-1**: For any MCP server with `targets = ["all"]`, the server configuration MUST be identical across all tool outputs (modulo format differences)

**Verification**: After compilation, deserialize all tool configs and verify server definitions match

### 6.2 Atomicity

**INV-2**: File write operations MUST be atomic - either the entire file is written or no changes occur

**Implementation**: Write to temporary file, then atomic rename

### 6.3 Backup Preservation

**INV-3**: If a backup file is created, it MUST contain an exact copy of the original file at the time of backup

**Verification**: Compare byte-for-byte before and after backup creation

### 6.4 Environment Variable Security

**INV-4**: Environment variable values MUST NEVER be stored in plaintext in any log, error message, or state file visible to other users

**Implementation**: Redact environment variable values in all output except generated config files

### 6.5 Generated File Markers

**INV-5**: All generated configuration files MUST be valid in their respective formats (parseable by target tool)

**Verification**: After generation, parse with format-specific parser (JSON.parse, TOML parser)

### 6.6 State Consistency

**INV-6**: The state tracker MUST accurately reflect all files generated in the most recent successful compile operation

**Verification**: State file updated only after all writes succeed

### 6.7 Idempotency

**INV-7**: Running compile twice with the same input and environment MUST produce identical outputs

**Verification**: Hash generated files before and after second compile

### 6.8 Rollback Safety

**INV-8**: If compilation fails partway through, all backup files MUST remain intact and the system MUST be in a recoverable state

**Implementation**: Only delete backups after successful completion of all writes

### 6.9 Path Handling

**INV-9**: All file paths MUST be correctly handled on both Unix (/) and Windows (\) systems

**Implementation**: Use platform-agnostic path manipulation libraries

### 6.10 Character Encoding

**INV-10**: All file operations MUST use UTF-8 encoding consistently

**Verification**: Files parseable with UTF-8 decoder

---

## 7. Transformation Rules

### 7.1 Environment Variable Expansion

**Algorithm**:

```
function expand_value(value: string, env_section: dict, shell_env: dict) -> string:
    result = value
    depth = 0
    max_depth = 10

    while depth < max_depth and ("{" in result or "${" in result):
        changed = false

        # Expand ${VAR} from shell environment
        for each match of pattern "\$\{([^}]+)\}":
            var_name = match.group(1)
            if var_name in shell_env:
                result = result.replace(match.group(0), shell_env[var_name])
                changed = true
            else:
                warn("Undefined environment variable: " + var_name)
                result = result.replace(match.group(0), "")
                changed = true

        # Expand {VAR} from env section
        for each match of pattern "\{([^}]+)\}":
            var_name = match.group(1)
            if var_name in env_section:
                result = result.replace(match.group(0), env_section[var_name])
                changed = true
            else:
                warn("Undefined variable in env section: " + var_name)
                result = result.replace(match.group(0), "")
                changed = true

        if not changed:
            break

        depth += 1

    if depth >= max_depth:
        error("Maximum expansion depth exceeded (circular reference?)")

    return result
```

**Rules**:
1. `${VAR}` references shell environment
2. `{VAR}` references `[env]` section
3. Undefined variables expand to empty string with warning
4. Circular references detected via depth limit
5. Expansion happens before any output generation
6. Expanded values are never logged or displayed

### 7.2 Target Filtering

**Algorithm**:

```
function filter_servers_for_tool(servers: dict, tool_name: string, default_targets: list) -> dict:
    result = {}

    for server_name, server_config in servers:
        # Skip disabled servers
        if not server_config.get("enabled", true):
            continue

        # Get targets for this server
        targets = server_config.get("targets", default_targets)

        # Expand "all" to all tool names
        if "all" in targets:
            targets = ["claude-code", "cursor", "opencode", "codex"]

        # Check if this tool is in targets
        if tool_name in targets:
            result[server_name] = server_config

    return result
```

### 7.3 Unified TOML → Cursor JSON

**Mapping**:

| Input Field | Output Field | Transformation |
|------------|--------------|----------------|
| `command` | `command` | Direct copy |
| `args` | `args` | Direct copy as array |
| `env` | `env` | Object with expanded values |
| `disabled` | `disabled` | Direct copy if present |
| `autoApprove` | `autoApprove` | Direct copy if present |

**Exclusions**:
- HTTP servers (url field present)
- Codex-specific fields (startup_timeout_sec, tool_timeout_sec)
- Tool-specific fields for other tools

### 7.4 Unified TOML → opencode.ai JSON

**Mapping**:

| Input Field | Output Field | Transformation |
|------------|--------------|----------------|
| `command` | `type` | → `"local"` |
| `command` + `args` | `command` | → `[command, ...args]` |
| `env` | `env` | Object with expanded values |
| `url` | `type` | → `"remote"` |
| `url` | `url` | Direct copy |
| `bearer_token` | `headers.Authorization` | → `"Bearer <token>"` |
| `enabled` | `enabled` | → `true` (false excluded) |

**Special Rules**:
- STDIO servers get `type: "local"`, command as array
- HTTP servers get `type: "remote"`, url field
- Bearer token becomes Authorization header

### 7.5 Unified TOML → Codex TOML

**Mapping**:

| Input Field | Output Field | Transformation |
|------------|--------------|----------------|
| `mcp.servers.<name>` | `mcp_servers.<name>` | Prefix change |
| `command` | `command` | Direct copy |
| `args` | `args` | Direct copy as array |
| `env` | `[mcp_servers.<name>.env]` | Separate table |
| `url` | `url` | Direct copy |
| `bearer_token` | `bearer_token` | Direct copy |
| `startup_timeout_sec` | `startup_timeout_sec` | Direct copy if present |
| `tool_timeout_sec` | `tool_timeout_sec` | Direct copy if present |

**Special Rules**:
- Section prefix changes from `mcp.servers` to `mcp_servers`
- Environment variables become subsection `[mcp_servers.<name>.env]`
- Both STDIO and HTTP servers supported
- Preserve other sections in existing config.toml

### 7.6 Merge Strategy for Existing Configs

**Current Behavior** (V1.0):
- Complete replacement of MCP server sections
- Preserve other sections (for Codex and opencode.ai)
- Create backup before any replacement

**Algorithm**:

```
function merge_config(existing_config: dict, new_mcp_section: dict, tool: string) -> dict:
    if tool == "cursor":
        # Cursor: complete replacement
        return {"mcpServers": new_mcp_section}

    elif tool == "opencode":
        # opencode: replace mcp section, preserve others
        result = existing_config.copy() if existing_config else {}
        result["mcp"] = new_mcp_section
        return result

    elif tool == "codex":
        # Codex: replace all mcp_servers sections, preserve others
        result = {}
        for section in existing_config:
            if not section.startswith("mcp_servers."):
                result[section] = existing_config[section]
        result.update(new_mcp_section)
        return result
```

---

## 8. Failure Modes

This section enumerates all possible failure scenarios and required handling.

### 8.1 Configuration File Errors

#### FM-1: Config File Not Found

**Scenario**: User runs compile/validate/diff without init, no config file exists

**Detection**: File existence check before open

**Handling**:
- Print: `Error: Configuration file not found: <path>`
- Print: `Run '<tool> init' to create a template configuration`
- Exit code: 2

#### FM-2: Config File Unreadable

**Scenario**: Config file exists but lacks read permissions

**Detection**: Permission error on file open

**Handling**:
- Print: `Error: Cannot read configuration file: <path>`
- Print: `Permission denied. Check file permissions.`
- Exit code: 2

#### FM-3: Invalid TOML Syntax

**Scenario**: Config file contains TOML syntax errors

**Detection**: TOML parser exception

**Handling**:
- Print: `Error: Invalid TOML syntax in <path>`
- Print: `<parser error message with line number>`
- Exit code: 3

#### FM-4: Missing Required Section

**Scenario**: Config file lacks `[mcp.servers]` section

**Detection**: Schema validation after parse

**Handling**:
- Print: `Error: Configuration missing required section: [mcp.servers]`
- Print: `At least one MCP server must be defined`
- Exit code: 1

#### FM-5: Missing Required Field

**Scenario**: Server definition lacks `command` (for STDIO) or `url` (for HTTP)

**Detection**: Schema validation

**Handling**:
- Print: `Error: Server '<name>' missing required field: command or url`
- Print: `STDIO servers require 'command', HTTP servers require 'url'`
- Exit code: 1

#### FM-6: Invalid Field Type

**Scenario**: Field has wrong type (e.g., `args` is string instead of array)

**Detection**: Type checking during parse

**Handling**:
- Print: `Error: Invalid type for field 'args' in server '<name>'`
- Print: `Expected: array of strings, Got: <type>`
- Exit code: 1

#### FM-7: Both Command and URL Present

**Scenario**: Server has both `command` and `url` fields

**Detection**: Schema validation

**Handling**:
- Print: `Error: Server '<name>' has both 'command' and 'url'`
- Print: `A server must be STDIO (command) OR HTTP (url), not both`
- Exit code: 1

### 8.2 Environment Variable Errors

#### FM-8: Undefined Environment Variable

**Scenario**: Config references `${VAR}` not in shell environment

**Detection**: During expansion, variable not in os.environ

**Handling**:
- Print: `Warning: Undefined environment variable: VAR`
- Print: `Expanding to empty string`
- Continue execution (exit code: 0, but warn)

#### FM-9: Circular Variable Reference

**Scenario**: `{VAR1} = "{VAR2}"`, `{VAR2} = "{VAR1}"`

**Detection**: Expansion depth exceeds limit (10)

**Handling**:
- Print: `Error: Circular variable reference detected in [env] section`
- Print: `Check definitions of: <involved variables>`
- Exit code: 1

#### FM-10: Undefined Variable in [env] Section

**Scenario**: Config references `{VAR}` not defined in `[env]`

**Detection**: During expansion, variable not in env_section

**Handling**:
- Print: `Warning: Undefined variable in [env] section: VAR`
- Print: `Expanding to empty string`
- Continue execution (exit code: 0, but warn)

### 8.3 File Write Errors

#### FM-11: Output Directory Not Exists

**Scenario**: Parent directory for output file doesn't exist

**Detection**: Directory check before write

**Handling**:
- Attempt to create parent directories (mkdir -p equivalent)
- If creation fails:
  - Print: `Error: Cannot create directory: <path>`
  - Print: `<OS error message>`
  - Exit code: 2

#### FM-12: Output File Unwritable

**Scenario**: Lack write permissions for output location

**Detection**: Permission error on file write

**Handling**:
- Print: `Error: Cannot write to <path>`
- Print: `Permission denied. Check file permissions.`
- Print: `No changes were made (backup preserved if exists)`
- Exit code: 2

#### FM-13: Disk Full

**Scenario**: Insufficient disk space for write

**Detection**: OS error during write (ENOSPC)

**Handling**:
- Print: `Error: Cannot write to <path>`
- Print: `Disk full or quota exceeded`
- Print: `No changes were made (backup preserved if exists)`
- Exit code: 2

#### FM-14: Backup Creation Failed

**Scenario**: Cannot create backup of existing file

**Detection**: Error during backup copy operation

**Handling**:
- Print: `Error: Cannot create backup of <path>`
- Print: `<OS error message>`
- Print: `Aborting to prevent data loss`
- Exit code: 2
- MUST NOT continue with write operation

### 8.4 Tool-Specific Errors

#### FM-15: Claude CLI Not Available

**Scenario**: Claude Code selected but `claude` command not found

**Detection**: Command existence check (which/where)

**Handling**:
- Print: `Warning: 'claude' command not found`
- Print: `Skipping Claude Code configuration`
- Print: `Install Claude Code CLI or remove 'claude-code' from targets`
- Continue with other tools (exit code: 0, but warn)

#### FM-16: Claude CLI Command Failed

**Scenario**: `claude mcp add` command returns non-zero exit code

**Detection**: Process exit code check

**Handling**:
- Print: `Error: Failed to add MCP server to Claude Code`
- Print: `Server: <name>`
- Print: `Command output: <stderr>`
- Exit code: 3 (partial failure)

#### FM-17: Invalid JSON Generated

**Scenario**: Generated JSON fails to parse (implementation bug)

**Detection**: Validate generated JSON before write

**Handling**:
- Print: `Error: Generated invalid JSON for <tool>`
- Print: `This is a bug. Please report with your configuration.`
- Print: `Generated content: <sanitized JSON>`
- Exit code: 1

#### FM-18: Invalid TOML Generated

**Scenario**: Generated TOML fails to parse (implementation bug)

**Detection**: Validate generated TOML before write

**Handling**:
- Print: `Error: Generated invalid TOML for Codex`
- Print: `This is a bug. Please report with your configuration.`
- Print: `Generated content: <sanitized TOML>`
- Exit code: 1

### 8.5 Concurrent Access Errors

#### FM-19: Config File Modified During Execution

**Scenario**: Another process modifies config while tool is running

**Detection**: Compare file modification time before and after read

**Handling**:
- Print: `Warning: Configuration file modified during execution`
- Print: `Using version read at: <timestamp>`
- Continue (changes take effect on next run)

#### FM-20: Output File Modified During Execution

**Scenario**: Tool modifies output file between backup and write

**Detection**: Compare backup hash with current file before write

**Handling**:
- Print: `Error: Output file modified since backup: <path>`
- Print: `Another process may have changed the file`
- Print: `Aborting to prevent data loss (backup preserved)`
- Exit code: 2

#### FM-21: Lock File Exists

**Scenario**: Another instance of tool is running (if lock files used)

**Detection**: Lock file existence check

**Handling**:
- Print: `Error: Another instance is already running`
- Print: `If this is incorrect, remove: <lock-file-path>`
- Exit code: 4

### 8.6 Validation Errors

#### FM-22: Invalid Tool Name

**Scenario**: `targets` or `--tool` specifies unknown tool

**Detection**: Tool name validation

**Handling**:
- Print: `Error: Unknown tool name: '<name>'`
- Print: `Valid tools: claude-code, cursor, opencode, codex`
- Exit code: 1

#### FM-23: Invalid Targets Combination

**Scenario**: Server has empty `targets` array

**Detection**: After target resolution

**Handling**:
- Print: `Warning: Server '<name>' has no targets`
- Print: `Server will not be compiled for any tool`
- Continue (not an error, might be intentional)

#### FM-24: No Servers for Any Tool

**Scenario**: All servers disabled or no targets match

**Detection**: After filtering servers for all tools

**Handling**:
- Print: `Warning: No servers configured for any tool`
- Print: `Check 'enabled' and 'targets' fields`
- Exit code: 0 (not an error, but unusual)

#### FM-25: Executable Not Found

**Scenario**: `command` field references non-existent executable

**Detection**: Executable existence check (which/where)

**Handling**:
- Print: `Warning: Command not found: '<command>' (server '<name>')`
- Print: `Server may fail to start on target tool`
- Continue (validation only, tool might be installed elsewhere)

---

## 9. Edge Cases

This section documents unusual scenarios and required behavior.

### 9.1 Configuration Edge Cases

#### EC-1: Empty Configuration

**Scenario**: Config file exists but has no MCP servers

**Behavior**:
- Validation: Pass with warning "No MCP servers defined"
- Compile: Generate empty/minimal config for each tool
- Exit code: 0

#### EC-2: Server with Empty Args

**Scenario**: `args = []` (empty array)

**Behavior**:
- Valid configuration
- Generate output with empty args array
- JSON: `"args": []`
- Command line: No args passed

#### EC-3: Server with Empty Env

**Scenario**: `env = {}` (empty table)

**Behavior**:
- Valid configuration
- Omit `env` field from output (treat as not present)

#### EC-4: Targets Contains "all" and Specific Tools

**Scenario**: `targets = ["all", "cursor"]`

**Behavior**:
- "all" takes precedence
- Equivalent to `targets = ["claude-code", "cursor", "opencode", "codex"]`
- No duplicates in processing

#### EC-5: Duplicate Server Names

**Scenario**: Two `[mcp.servers.github]` sections

**Behavior**:
- TOML parser behavior: Later definition overwrites earlier
- Tool accepts whichever definition TOML parser returns
- Warn: "Duplicate server name detected (TOML parser handled)"

### 9.2 Environment Variable Edge Cases

#### EC-6: Environment Variable with Empty Value

**Scenario**: `export MYVAR=""`, config uses `${MYVAR}`

**Behavior**:
- Expand to empty string
- No warning (empty is a valid value)

#### EC-7: Variable Name Contains Special Characters

**Scenario**: `{MY-VAR}` or `{MY.VAR}` (non-alphanumeric)

**Behavior**:
- If valid TOML key: attempt expansion
- If not in env section: warning and empty string
- Recommendation: Use alphanumeric + underscore only

#### EC-8: Nested Variable References

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

#### EC-9: Variable Reference in Server Name

**Scenario**: `[mcp.servers.{SERVERNAME}]`

**Behavior**:
- NOT supported (TOML keys evaluated at parse time)
- Treat as literal string (server named "{SERVERNAME}")
- Document: Variable expansion only in values, not keys

### 9.3 File System Edge Cases

#### EC-10: Output Path is Symlink

**Scenario**: `.cursor/mcp.json` is symlink to another file

**Behavior**:
- Follow symlink for read (to create backup)
- Write to symlink target (normal symlink behavior)
- Backup created at symlink location, not target

#### EC-11: Output Path is Directory

**Scenario**: User has directory named `.cursor/mcp.json`

**Behavior**:
- Detect: path exists and is directory
- Print: `Error: Expected file, found directory: <path>`
- Exit code: 2

#### EC-12: Output Directory is Read-Only

**Scenario**: `.cursor/` directory has no write permission

**Behavior**:
- Cannot create/modify `mcp.json`
- Print: `Error: Cannot write to <path>`
- Print: `Parent directory is read-only`
- Exit code: 2

#### EC-13: Very Long File Path

**Scenario**: Config generates path > 260 chars (Windows) or > 4096 chars (Linux)

**Behavior**:
- Attempt write normally
- If OS error (path too long):
  - Print: `Error: File path too long: <path>`
  - Exit code: 2

#### EC-14: Special Characters in Path

**Scenario**: Server name contains spaces, quotes, or non-ASCII

**Behavior**:
- Path construction: Use server name as-is
- Sanitization: Replace `/` with `-` (directory separator conflict)
- Document: Recommend alphanumeric + hyphen for server names

### 9.4 Tool-Specific Edge Cases

#### EC-15: Cursor: Project Has No .cursor Directory

**Scenario**: Running compile in directory without `.cursor/`

**Behavior**:
- Create `.cursor/` directory
- Create `mcp.json` inside
- Set appropriate permissions (755 for directory, 644 for file)

#### EC-16: opencode.ai: Existing Config Has Unrelated Settings

**Scenario**: `opencode.json` has `model`, `provider`, etc.

**Behavior**:
- Preserve all fields except `mcp`
- Only replace `mcp` section
- Validate JSON structure after merge

#### EC-17: Codex: Existing Config Has Comments

**Scenario**: `config.toml` has TOML comments

**Behavior**:
- Comments outside `[mcp_servers.*]` sections: preserved
- Comments inside `[mcp_servers.*]` sections: lost (section replaced)
- Document: "Comments in MCP sections will be removed"

#### EC-18: Server Enabled for Tool That Doesn't Support It

**Scenario**: HTTP server targeted at Cursor (Cursor doesn't support HTTP MCP)

**Behavior**:
- Detect: Server has `url` field, tool is Cursor
- Print: `Warning: Server '<name>' is HTTP, skipping for Cursor (unsupported)`
- Exclude from Cursor output

### 9.5 Command-Line Edge Cases

#### EC-19: Multiple --tool Flags

**Scenario**: `compile --tool cursor --tool opencode`

**Behavior**:
- Compile for both specified tools
- Process in order specified
- Exit code: 0 if all succeed, 3 if any fail

#### EC-20: --tool Repeated with Same Value

**Scenario**: `compile --tool cursor --tool cursor`

**Behavior**:
- Deduplicate: treat as single `--tool cursor`
- Warn: "Duplicate tool specified: cursor"

#### EC-21: Dry-Run with No Output

**Scenario**: `compile --dry-run` when no changes needed

**Behavior**:
- Print: "No changes needed (configurations up-to-date)"
- Exit code: 0

#### EC-22: Diff on Non-Existent File

**Scenario**: `diff` when target tool has no existing config

**Behavior**:
- Show: "[NEW FILE]" with full content
- Not an error

### 9.6 Encoding Edge Cases

#### EC-23: Config Contains Non-ASCII Characters

**Scenario**: Server name or value has emoji, Chinese, etc.

**Behavior**:
- Accept if valid UTF-8
- Preserve in output files
- JSON: Escape unicode sequences (`\uXXXX`) or UTF-8
- TOML: UTF-8 encoding

#### EC-24: Config Contains Windows Line Endings (CRLF)

**Scenario**: Config file created on Windows

**Behavior**:
- TOML parser MUST handle both LF and CRLF
- Output files use platform-native line endings (LF on Unix, CRLF on Windows)

#### EC-25: Config Contains NUL Bytes

**Scenario**: Binary data in config file

**Behavior**:
- TOML parser error (invalid character)
- Print: `Error: Invalid character in configuration file`
- Exit code: 3

### 9.7 State Tracking Edge Cases

#### EC-26: State File Corrupted

**Scenario**: `generated.json` is invalid JSON

**Behavior**:
- Print: `Warning: State file corrupted, recreating`
- Delete state file
- Proceed as if no state exists
- Continue normally

#### EC-27: State File References Non-Existent File

**Scenario**: State says file was generated, but it doesn't exist

**Behavior**:
- Print: `Warning: Generated file missing: <path>`
- Remove from state
- Regenerate on next compile

#### EC-28: Generated File Modified by User

**Scenario**: User manually edits output file after generation

**Behavior**:
- Detect: Hash in state doesn't match current file
- Print: `Warning: <path> was manually modified`
- Print: `Overwriting with generated configuration (backup created)`
- Continue with write

---

## 10. Security Requirements

### 10.1 Credential Handling

**SEC-1**: Environment variables containing credentials MUST be expanded only in memory, never logged

**SEC-2**: Error messages MUST NOT include environment variable values

**SEC-3**: Dry-run and diff output MUST redact credential values

**Implementation**:
```
If field name matches (API_KEY|TOKEN|SECRET|PASSWORD|CREDENTIAL):
    Display as: <redacted>
```

### 10.2 File Permissions

**SEC-4**: Configuration files MUST be created with mode 0600 (user read/write only)

**SEC-5**: Backup files MUST preserve original file permissions

**SEC-6**: State files MUST be created with mode 0600

### 10.3 Path Traversal Prevention

**SEC-7**: Tool MUST reject config paths containing `..` (parent directory)

**SEC-8**: Output paths MUST be validated before write:
- Must be within expected directories
- Cannot write to system directories (/etc, /usr, etc.)

### 10.4 Code Injection Prevention

**SEC-9**: Environment variable expansion MUST NOT execute shell commands

**SEC-10**: Server `command` fields are NOT validated for safety (user responsibility)

**SEC-11**: Tool MUST NOT evaluate user input as code (no eval/exec)

### 10.5 Audit Logging

**SEC-12**: Tool SHOULD log:
- Configuration file read
- Files written
- Backups created
- Errors encountered

**SEC-13**: Logs MUST NOT contain credential values

---

## 11. Operational Requirements

### 11.1 Dry-Run Mode

**OP-1**: Dry-run MUST perform all validation steps

**OP-2**: Dry-run MUST NOT modify any files (including backups)

**OP-3**: Dry-run output MUST show what would be generated

**OP-4**: Dry-run exit code MUST match what actual run would return

### 11.2 Validation

**OP-5**: Validation MUST check:
- TOML syntax
- Required fields
- Field types
- Circular references
- Valid tool names
- Executable existence (warning only)

**OP-6**: Validation SHOULD check:
- Recommended field values
- Common mistakes
- Performance implications (too many servers)

### 11.3 Diff Generation

**OP-7**: Diff MUST use unified diff format

**OP-8**: Diff MUST show:
- File path
- Tool name
- Whether file is new or modified

**OP-9**: Diff MUST handle:
- Non-existent files (show as new)
- Binary files (show size change)
- Large files (truncate if > 10000 lines)

### 11.4 State Tracking

**OP-10**: State file MUST include:
- Version
- Timestamp of last compile
- List of generated files with paths and hashes

**OP-11**: State file MUST be updated atomically (write to temp, rename)

**OP-12**: State file errors MUST NOT prevent compilation (warn and continue)

### 11.5 Rollback Capability

**OP-13**: Backup files enable manual rollback:
```bash
cp .cursor/mcp.json.backup .cursor/mcp.json
```

**OP-14**: Tool SHOULD provide `rollback` command in future versions

### 11.6 Lock Files

**OP-15**: Tool MAY use lock files to prevent concurrent execution

**OP-16**: Lock files MUST include:
- Process ID
- Start timestamp
- Host name

**OP-17**: Stale locks (process not running) MUST be cleaned up automatically

### 11.7 Idempotency

**OP-18**: Running compile twice with same input MUST produce identical output

**OP-19**: Repeated execution MUST NOT corrupt state

**OP-20**: Backup files MAY be overwritten on repeated execution

---

## 12. Configuration Schema Reference

### 12.1 Input Schema (TOML)

Complete formal specification of unified configuration format.

```toml
# ------------------------------------------------------------------------------
# Settings Section (Optional)
# ------------------------------------------------------------------------------
[settings]
version = "1.0"                           # Required: string (semver)
default_targets = ["cursor", "opencode"]  # Optional: array<string>

# ------------------------------------------------------------------------------
# Environment Variables Section (Optional)
# ------------------------------------------------------------------------------
[env]
VAR_NAME = "value"                        # Optional: string
ANOTHER = "${SHELL_VAR}"                  # Reference to shell environment
COMPOSITE = "{VAR_NAME}_suffix"           # Reference to env section

# ------------------------------------------------------------------------------
# MCP Servers Section (Required)
# ------------------------------------------------------------------------------

# STDIO Server Example
[mcp.servers.example-stdio]
command = "npx"                           # Required: string (executable name or path)
args = ["-y", "package"]                  # Optional: array<string>
enabled = true                            # Optional: boolean (default true)
targets = ["all"]                         # Optional: array<string> | ["all"]
disabled = false                          # Optional: boolean (Cursor-specific)
autoApprove = ["tool1"]                   # Optional: array<string> (Cursor-specific)
startup_timeout_sec = 30                  # Optional: integer (Codex-specific)
tool_timeout_sec = 60                     # Optional: integer (Codex-specific)

[mcp.servers.example-stdio.env]           # Optional: table
API_KEY = "{MYVAR}"                       # Environment variables for server

# HTTP Server Example
[mcp.servers.example-http]
url = "https://example.com/mcp"           # Required (for HTTP): string (URL)
bearer_token = "{TOKEN}"                  # Optional: string
enabled = true                            # Optional: boolean (default true)
targets = ["codex"]                       # Optional: array<string> | ["all"]

# ------------------------------------------------------------------------------
# Type Specifications
# ------------------------------------------------------------------------------

# version: string matching /^\d+\.\d+(\.\d+)?$/
# default_targets: ["claude-code" | "cursor" | "opencode" | "codex" | "all"]
# command: string (path or executable name)
# args: array<string> (may contain {VAR} or ${VAR})
# env: table<string, string> (values may contain {VAR} or ${VAR})
# enabled: boolean
# targets: array<string> where string in ["claude-code", "cursor", "opencode", "codex", "all"]
# url: string starting with "http://" or "https://"
# bearer_token: string
# disabled: boolean
# autoApprove: array<string>
# startup_timeout_sec: integer > 0
# tool_timeout_sec: integer > 0
```

### 12.2 Cursor Output Schema (JSON)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["mcpServers"],
  "properties": {
    "mcpServers": {
      "type": "object",
      "patternProperties": {
        "^[a-zA-Z0-9_-]+$": {
          "type": "object",
          "required": ["command", "args"],
          "properties": {
            "command": {
              "type": "string",
              "description": "Executable command"
            },
            "args": {
              "type": "array",
              "items": { "type": "string" },
              "description": "Command arguments"
            },
            "env": {
              "type": "object",
              "patternProperties": {
                "^[A-Z_][A-Z0-9_]*$": { "type": "string" }
              },
              "description": "Environment variables"
            },
            "disabled": {
              "type": "boolean",
              "description": "Whether server is disabled"
            },
            "autoApprove": {
              "type": "array",
              "items": { "type": "string" },
              "description": "Tools to auto-approve"
            }
          }
        }
      }
    }
  }
}
```

### 12.3 opencode.ai Output Schema (JSON)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["mcp"],
  "properties": {
    "mcp": {
      "type": "object",
      "patternProperties": {
        "^[a-zA-Z0-9_-]+$": {
          "oneOf": [
            {
              "type": "object",
              "required": ["type", "command", "enabled"],
              "properties": {
                "type": { "const": "local" },
                "command": {
                  "type": "array",
                  "items": { "type": "string" },
                  "minItems": 1,
                  "description": "[executable, ...args]"
                },
                "env": {
                  "type": "object",
                  "patternProperties": {
                    "^[A-Z_][A-Z0-9_]*$": { "type": "string" }
                  }
                },
                "enabled": { "type": "boolean" }
              }
            },
            {
              "type": "object",
              "required": ["type", "url", "enabled"],
              "properties": {
                "type": { "const": "remote" },
                "url": {
                  "type": "string",
                  "pattern": "^https?://"
                },
                "headers": {
                  "type": "object",
                  "patternProperties": {
                    "^.+$": { "type": "string" }
                  }
                },
                "enabled": { "type": "boolean" }
              }
            }
          ]
        }
      }
    }
  }
}
```

### 12.4 Codex Output Schema (TOML)

```toml
# Formal schema in prose (TOML has no standard schema language)

# Each server is a table: [mcp_servers.<server-name>]
# Server names: alphanumeric, underscore, hyphen

# STDIO servers have:
[mcp_servers.example]
command = "string"                      # Required
args = ["string", ...]                  # Optional (array of strings)
startup_timeout_sec = integer           # Optional (positive integer)
tool_timeout_sec = integer              # Optional (positive integer)

[mcp_servers.example.env]              # Optional (table)
ENV_VAR = "string"

# HTTP servers have:
[mcp_servers.remote-example]
url = "string"                          # Required (http:// or https://)
bearer_token = "string"                 # Optional
```

### 12.5 State File Schema (JSON)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["version", "last_compile", "generated_files"],
  "properties": {
    "version": {
      "type": "string",
      "pattern": "^\\d+\\.\\d+(\\.\\d+)?$"
    },
    "last_compile": {
      "type": "string",
      "format": "date-time",
      "description": "ISO 8601 timestamp"
    },
    "generated_files": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["tool", "path", "timestamp", "hash"],
        "properties": {
          "tool": {
            "type": "string",
            "enum": ["claude-code", "cursor", "opencode", "codex"]
          },
          "path": {
            "type": "string",
            "description": "Absolute path to generated file"
          },
          "timestamp": {
            "type": "string",
            "format": "date-time"
          },
          "hash": {
            "type": "string",
            "pattern": "^sha256:[a-f0-9]{64}$",
            "description": "SHA-256 hash of file contents"
          }
        }
      }
    }
  }
}
```

---

## 13. Error Handling Specifications

### 13.1 Error Message Format

All error messages MUST follow this format:

```
<severity>: <summary>
<details>
<suggestion>
```

**Severity Levels**:
- `Error` - Fatal, prevents execution
- `Warning` - Non-fatal, execution continues
- `Info` - Informational message

**Example**:
```
Error: Configuration file not found: ~/.config/agent-sync/config.toml
The configuration file does not exist or cannot be accessed.
Run 'agent-sync init' to create a template configuration.
```

### 13.2 Exit Codes

| Code | Meaning | Examples |
|------|---------|----------|
| 0 | Success | All operations completed successfully |
| 1 | Validation error | Invalid configuration, missing required fields |
| 2 | File system error | Cannot read/write files, permissions |
| 3 | Partial failure | Some tools succeeded, some failed |
| 4 | Lock error | Another instance running |
| 5-99 | Reserved | Future use |

### 13.3 Logging

**Requirements**:

**LOG-1**: Tool SHOULD support optional logging to file

**LOG-2**: Log levels: DEBUG, INFO, WARN, ERROR

**LOG-3**: Log format:
```
[2025-10-12T15:30:00Z] [INFO] Starting compilation
[2025-10-12T15:30:01Z] [DEBUG] Loaded 7 servers from config
[2025-10-12T15:30:02Z] [WARN] Undefined environment variable: MISSING_VAR
[2025-10-12T15:30:03Z] [INFO] Generated cursor config: .cursor/mcp.json
```

**LOG-4**: Credentials MUST be redacted in logs

### 13.4 User-Facing vs Debug Messages

**User Messages** (stdout):
- Brief, actionable
- Suitable for end users
- No implementation details

**Debug Messages** (stderr or log file):
- Detailed technical information
- Stack traces on exceptions
- Intermediate values
- Enabled with `--verbose` or `--debug` flag

---

## 14. Testing Requirements

### 14.1 Unit Test Coverage

Minimum 80% code coverage for:
- Configuration parsing
- Environment variable expansion
- Schema validation
- Format transformation
- File operations

### 14.2 Integration Test Scenarios

**INT-1**: End-to-end compile with all tools

**INT-2**: Dry-run produces correct output without writing

**INT-3**: Diff shows accurate changes

**INT-4**: Backup and rollback procedure

**INT-5**: Concurrent execution handling

### 14.3 Validation Test Cases

Test matrix of valid/invalid configurations:

| Test Case | Config | Expected Result |
|-----------|--------|-----------------|
| Valid-1 | Complete config with all fields | Pass |
| Valid-2 | Minimal config (required fields only) | Pass |
| Valid-3 | Mixed STDIO and HTTP servers | Pass |
| Invalid-1 | Missing [mcp.servers] | Fail: Missing required section |
| Invalid-2 | Server with no command or url | Fail: Missing required field |
| Invalid-3 | Both command and url | Fail: Mutually exclusive |
| Invalid-4 | Invalid TOML syntax | Fail: Parse error |
| Invalid-5 | Circular env references | Fail: Circular reference |

### 14.4 Edge Case Test Matrix

Each edge case from Section 9 MUST have corresponding test:

- EC-1: Empty configuration → generates minimal output
- EC-6: Empty env var value → expands to empty string
- EC-10: Symlink output path → follows symlink correctly
- EC-23: Non-ASCII characters → preserved correctly
- (etc.)

### 14.5 Failure Mode Test Matrix

Each failure mode from Section 8 MUST have corresponding test:

- FM-1: Config not found → correct error message and exit code
- FM-8: Undefined env var → warning but continues
- FM-11: Output directory missing → creates directory
- FM-14: Backup fails → aborts without writing
- (etc.)

---

## 15. Future Extensibility

### 15.1 Plugin Architecture (Future)

**Design Considerations**:
- Allow custom format transformers
- Plugin registration mechanism
- Plugin API version compatibility

**Example**:
```python
class FormatTransformer:
    def transform(self, servers: dict, env: dict) -> str:
        # Return tool-specific configuration
        pass
```

### 15.2 Schema Versioning

**Migration Strategy**:
- Version field in config enables breaking changes
- Tool detects old versions and migrates automatically
- Migration logged and user-confirmed

**Example**:
```
Detected configuration version 1.0
Migrating to version 2.0...
- Renamed field: 'disabled' → 'enabled' (inverted)
- Added required field: 'schema' (default: 'stdio')
Migration complete. Backup saved to: config.toml.v1.backup
```

### 15.3 Adding New Tools

**Process**:
1. Research tool's configuration format
2. Add tool name to valid targets
3. Implement format transformer
4. Add output path mapping
5. Update documentation
6. Add test cases

**Interface**:
```python
def transform_for_new_tool(servers: dict, env: dict) -> Output:
    # Implementation
    pass
```

### 15.4 Prompt Management (Future)

**Planned Features**:
- Unified prompt definitions in TOML
- Generate tool-specific prompt files (CLAUDE.md, .cursorrules, AGENTS.md)
- Merge strategies for existing prompts
- Template variables in prompts

**Example Config**:
```toml
[prompts.coding-standards]
files = ["~/.config/agent-sync/prompts/coding-standards.md"]
targets = ["all"]
merge_strategy = "append"
variables = { LANGUAGE = "Python" }
```

### 15.5 Watch Mode (Future)

**Behavior**:
- Monitor config file for changes
- Auto-compile on save
- Debounce rapid changes (wait 500ms)
- Notification on success/failure

**Command**:
```bash
agent-sync watch
Watching: ~/.config/agent-sync/config.toml
Press Ctrl+C to stop...
[15:30] Config changed, recompiling...
[15:30] ✓ All tools updated
```

---

## 16. Appendices

### Appendix A: Complete Example Configuration

```toml
# ~/.config/agent-sync/config.toml
# Complete example with all supported features

[settings]
version = "1.0"
default_targets = ["cursor", "opencode", "codex"]

[env]
GITHUB_TOKEN = "${GITHUB_PERSONAL_ACCESS_TOKEN}"
OPENAI_KEY = "${OPENAI_API_KEY}"
ASANA_TOKEN = "${ASANA_ACCESS_TOKEN}"
NEO4J_PASSWORD = "3i2lemma"

# STDIO server with environment variables
[mcp.servers.github]
command = "docker"
args = [
    "run", "-i", "--rm",
    "-e", "GITHUB_PERSONAL_ACCESS_TOKEN",
    "ghcr.io/github/github-mcp-server"
]
env.GITHUB_PERSONAL_ACCESS_TOKEN = "{GITHUB_TOKEN}"
enabled = true
targets = ["all"]

# Simple STDIO server
[mcp.servers.fetch]
command = "/Users/jfb/.local/bin/uvx"
args = ["mcp-server-fetch"]
enabled = true
targets = ["all"]

# STDIO server with complex environment
[mcp.servers.memento]
command = "npx"
args = ["-y", "@gannonh/memento-mcp"]
enabled = true
targets = ["all"]

[mcp.servers.memento.env]
MEMORY_STORAGE_TYPE = "neo4j"
NEO4J_URI = "bolt://127.0.0.1:7687"
NEO4J_USERNAME = "neo4j"
NEO4J_PASSWORD = "{NEO4J_PASSWORD}"
NEO4J_DATABASE = "neo4j"
OPENAI_API_KEY = "{OPENAI_KEY}"

# HTTP server (Codex only)
[mcp.servers.figma]
url = "https://mcp.figma.com/mcp"
enabled = true
targets = ["codex"]

# HTTP server with authentication
[mcp.servers.remote-api]
url = "https://api.example.com/mcp"
bearer_token = "{OPENAI_KEY}"
enabled = true
targets = ["codex"]

# Cursor-specific features
[mcp.servers.context7]
command = "npx"
args = ["-y", "@upstash/context7-mcp@latest"]
enabled = true
targets = ["cursor"]
disabled = false
autoApprove = ["search", "lookup"]

# Codex-specific timeouts
[mcp.servers.slow-server]
command = "node"
args = ["/path/to/slow-server.js"]
enabled = true
targets = ["codex"]
startup_timeout_sec = 60
tool_timeout_sec = 120

# Disabled server (not compiled)
[mcp.servers.experimental]
command = "experimental-tool"
enabled = false
targets = ["all"]
```

### Appendix B: Example Outputs

#### B.1 Cursor Output

```json
{
  "mcpServers": {
    "github": {
      "command": "docker",
      "args": [
        "run", "-i", "--rm",
        "-e", "GITHUB_PERSONAL_ACCESS_TOKEN",
        "ghcr.io/github/github-mcp-server"
      ],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_expandedtoken123"
      }
    },
    "fetch": {
      "command": "/Users/jfb/.local/bin/uvx",
      "args": ["mcp-server-fetch"]
    },
    "memento": {
      "command": "npx",
      "args": ["-y", "@gannonh/memento-mcp"],
      "env": {
        "MEMORY_STORAGE_TYPE": "neo4j",
        "NEO4J_URI": "bolt://127.0.0.1:7687",
        "NEO4J_USERNAME": "neo4j",
        "NEO4J_PASSWORD": "3i2lemma",
        "NEO4J_DATABASE": "neo4j",
        "OPENAI_API_KEY": "sk-expandedkey456"
      }
    },
    "context7": {
      "command": "npx",
      "args": ["-y", "@upstash/context7-mcp@latest"],
      "disabled": false,
      "autoApprove": ["search", "lookup"]
    }
  }
}
```

#### B.2 opencode.ai Output

```json
{
  "mcp": {
    "github": {
      "type": "local",
      "command": [
        "docker", "run", "-i", "--rm",
        "-e", "GITHUB_PERSONAL_ACCESS_TOKEN",
        "ghcr.io/github/github-mcp-server"
      ],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_expandedtoken123"
      },
      "enabled": true
    },
    "fetch": {
      "type": "local",
      "command": ["/Users/jfb/.local/bin/uvx", "mcp-server-fetch"],
      "enabled": true
    },
    "memento": {
      "type": "local",
      "command": ["npx", "-y", "@gannonh/memento-mcp"],
      "env": {
        "MEMORY_STORAGE_TYPE": "neo4j",
        "NEO4J_URI": "bolt://127.0.0.1:7687",
        "NEO4J_USERNAME": "neo4j",
        "NEO4J_PASSWORD": "3i2lemma",
        "NEO4J_DATABASE": "neo4j",
        "OPENAI_API_KEY": "sk-expandedkey456"
      },
      "enabled": true
    }
  }
}
```

#### B.3 Codex Output

```toml
[mcp_servers.github]
command = "docker"
args = [
    "run", "-i", "--rm",
    "-e", "GITHUB_PERSONAL_ACCESS_TOKEN",
    "ghcr.io/github/github-mcp-server"
]

[mcp_servers.github.env]
GITHUB_PERSONAL_ACCESS_TOKEN = "ghp_expandedtoken123"

[mcp_servers.fetch]
command = "/Users/jfb/.local/bin/uvx"
args = ["mcp-server-fetch"]

[mcp_servers.memento]
command = "npx"
args = ["-y", "@gannonh/memento-mcp"]

[mcp_servers.memento.env]
MEMORY_STORAGE_TYPE = "neo4j"
NEO4J_URI = "bolt://127.0.0.1:7687"
NEO4J_USERNAME = "neo4j"
NEO4J_PASSWORD = "3i2lemma"
NEO4J_DATABASE = "neo4j"
OPENAI_API_KEY = "sk-expandedkey456"

[mcp_servers.figma]
url = "https://mcp.figma.com/mcp"

[mcp_servers.remote-api]
url = "https://api.example.com/mcp"
bearer_token = "sk-expandedkey456"

[mcp_servers.slow-server]
command = "node"
args = ["/path/to/slow-server.js"]
startup_timeout_sec = 60
tool_timeout_sec = 120
```

### Appendix C: Environment Variable Resolution Algorithm

```python
def expand_environment_variables(value: str, env_section: dict, shell_env: dict) -> str:
    """
    Expand environment variable references in a string.

    Supports:
    - ${VAR} for shell environment variables
    - {VAR} for variables defined in [env] section

    Args:
        value: String potentially containing variable references
        env_section: Dictionary from [env] section of config
        shell_env: Shell environment variables (os.environ)

    Returns:
        String with all variables expanded

    Raises:
        ValueError: If circular reference detected
    """
    import re

    result = value
    max_depth = 10
    depth = 0

    while depth < max_depth:
        changed = False

        # Expand ${VAR} from shell environment
        shell_pattern = re.compile(r'\$\{([^}]+)\}')
        for match in shell_pattern.finditer(result):
            var_name = match.group(1)
            if var_name in shell_env:
                replacement = shell_env[var_name]
                result = result.replace(match.group(0), replacement)
                changed = True
            else:
                # Undefined variable: warn and replace with empty string
                warn(f"Undefined environment variable: {var_name}")
                result = result.replace(match.group(0), "")
                changed = True

        # Expand {VAR} from [env] section
        env_pattern = re.compile(r'\{([^}]+)\}')
        for match in env_pattern.finditer(result):
            var_name = match.group(1)
            if var_name in env_section:
                replacement = env_section[var_name]
                # Recursively expand the replacement
                replacement = expand_environment_variables(replacement, env_section, shell_env)
                result = result.replace(match.group(0), replacement)
                changed = True
            else:
                # Undefined variable: warn and replace with empty string
                warn(f"Undefined variable in [env] section: {var_name}")
                result = result.replace(match.group(0), "")
                changed = True

        if not changed:
            # No more expansions needed
            break

        depth += 1

    if depth >= max_depth:
        raise ValueError(f"Maximum expansion depth exceeded (circular reference?): {value}")

    return result


def expand_all_env_vars(config: dict) -> dict:
    """
    Expand all environment variables in configuration.

    Args:
        config: Parsed TOML configuration

    Returns:
        Configuration with all variables expanded
    """
    import os

    env_section = config.get("env", {})
    shell_env = dict(os.environ)

    # First, expand all values in [env] section itself
    expanded_env = {}
    for key, value in env_section.items():
        expanded_env[key] = expand_environment_variables(value, env_section, shell_env)

    # Now expand variables in server configurations
    expanded_servers = {}
    servers = config.get("mcp", {}).get("servers", {})

    for server_name, server_config in servers.items():
        expanded_server = {}

        for key, value in server_config.items():
            if isinstance(value, str):
                expanded_server[key] = expand_environment_variables(value, expanded_env, shell_env)
            elif isinstance(value, list):
                expanded_server[key] = [
                    expand_environment_variables(item, expanded_env, shell_env) if isinstance(item, str) else item
                    for item in value
                ]
            elif isinstance(value, dict):
                expanded_server[key] = {
                    k: expand_environment_variables(v, expanded_env, shell_env) if isinstance(v, str) else v
                    for k, v in value.items()
                }
            else:
                expanded_server[key] = value

        expanded_servers[server_name] = expanded_server

    # Return config with expanded values
    result = config.copy()
    result["mcp"] = {"servers": expanded_servers}
    return result
```

### Appendix D: File Operation Safety Protocol

```python
def safe_write_file(path: Path, content: str, backup: bool = True) -> None:
    """
    Safely write file with atomicity and backup.

    Protocol:
    1. Create backup of existing file (if exists and backup=True)
    2. Write content to temporary file
    3. Validate written content
    4. Atomically rename temp file to target path
    5. Update state tracker

    Args:
        path: Target file path
        content: File content to write
        backup: Whether to create backup (default True)

    Raises:
        OSError: If write fails
        ValueError: If validation fails
    """
    import tempfile
    import shutil
    from pathlib import Path

    # Step 1: Create backup if file exists
    if backup and path.exists():
        backup_path = path.with_suffix(path.suffix + ".backup")
        try:
            shutil.copy2(path, backup_path)
            print(f"Backed up existing config to {backup_path}")
        except OSError as e:
            raise OSError(f"Cannot create backup of {path}: {e}")

    # Step 2: Create parent directories if needed
    path.parent.mkdir(parents=True, exist_ok=True)

    # Step 3: Write to temporary file in same directory
    # (same directory ensures atomic rename works)
    temp_fd, temp_path = tempfile.mkstemp(
        dir=path.parent,
        prefix=f".{path.name}.",
        suffix=".tmp"
    )
    temp_path = Path(temp_path)

    try:
        # Write content
        with open(temp_fd, 'w', encoding='utf-8') as f:
            f.write(content)

        # Step 4: Validate written content
        # (format-specific validation)
        if path.suffix == '.json':
            import json
            with open(temp_path, 'r', encoding='utf-8') as f:
                json.load(f)  # Raises if invalid
        elif path.suffix == '.toml':
            import tomllib
            with open(temp_path, 'rb') as f:
                tomllib.load(f)  # Raises if invalid

        # Step 5: Set file permissions (0600 for configs)
        temp_path.chmod(0o600)

        # Step 6: Atomic rename
        temp_path.rename(path)

        print(f"Generated config: {path}")

    except Exception as e:
        # Clean up temp file on failure
        if temp_path.exists():
            temp_path.unlink()
        raise
```

### Appendix E: Glossary

**Agent**: A system prompt or mode configuration that modifies AI assistant behavior

**Atomicity**: Property that an operation either completes fully or has no effect

**Backup**: Copy of existing file created before modification

**CLI**: Command-Line Interface

**Compilation**: Process of transforming unified config to tool-specific formats

**Dry-run**: Execution mode that shows what would be done without making changes

**Environment Variable**: Named value from shell environment (e.g., `$PATH`)

**Expansion**: Replacing variable references with their values

**HTTP Server**: MCP server accessed via HTTP/HTTPS URL

**Idempotency**: Property that repeated execution produces same result

**Invariant**: Property that must always be true

**Lock File**: File indicating another process is running

**MCP**: Model Context Protocol - standard for connecting AI tools to external resources

**Prompt**: Instructions that guide AI assistant behavior

**Redaction**: Hiding sensitive values in logs/output

**Rollback**: Restoring previous configuration

**Schema**: Formal specification of data structure

**State Tracker**: Record of files generated by tool

**STDIO Server**: MCP server launched as subprocess

**Target**: Tool for which configuration is generated

**TOML**: Tom's Obvious, Minimal Language - configuration file format

**Unified Configuration**: Single TOML file containing all MCP server definitions

**Validation**: Checking configuration correctness without execution

---

## Document Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-10-12 | Initial specification |

---

**End of Requirements Specification**
