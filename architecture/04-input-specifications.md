# 4. Input Specifications

## 4.1 Primary Input: Unified Configuration File

**Location**: `~/.config/agent-sync/config.toml` (default) or user-specified path

**Format**: TOML (Tom's Obvious, Minimal Language)

**Encoding**: UTF-8

**Line Endings**: LF (Unix) or CRLF (Windows) - tool MUST handle both

## 4.2 Unified Configuration Schema

### 4.2.1 Top-Level Structure

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

### 4.2.2 Settings Section

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

### 4.2.3 Environment Variables Section

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

### 4.2.4 MCP Server Section

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

## 4.3 Command-Line Interface

### 4.3.1 General Syntax

```
<tool-name> [global-options] <command> [command-options]
```

**Tool Name**: Implementation-defined (e.g., `agent-sync`, `ai-sync`, `mcp-sync`)

### 4.3.2 Global Options

```
--config <path>     Path to config.toml (default: ~/.config/agent-sync/config.toml)
--help              Show help message and exit
--version           Show version and exit
```

### 4.3.3 Commands

#### 4.3.3.1 `init`

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

#### 4.3.3.2 `validate`

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

#### 4.3.3.3 `compile`

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

#### 4.3.3.4 `diff`

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

## 4.4 Secondary Inputs

### 4.4.1 Environment Variables

**Source**: Shell environment at time of execution

**Access**: Via `${VAR_NAME}` syntax in configuration

**Security**:
- Sensitive values (tokens, passwords) SHOULD be in environment, not config file
- Tool MUST NOT log or display environment variable values
- Undefined environment variables SHOULD produce warning but expand to empty string

### 4.4.2 Existing Tool Configurations

**Purpose**: For backup and merge operations

**Behavior**:
- Tool MUST read existing config files before writing
- Tool MUST create backups with `.backup` extension
- Backups MUST preserve original timestamps
