# 5. Output Specifications

## 5.1 Output Files Overview

| Tool | Format | Default Path |
|------|--------|--------------|
| Claude Code | JSON or CLI | `~/.config/claude-code/mcp.json` (TBD) or CLI commands |
| Cursor | JSON | `.cursor/mcp.json` (project-level) |
| opencode.ai | JSON | `~/.config/opencode/opencode.json` |
| OpenAI Codex | TOML | `~/.codex/config.toml` |

## 5.2 Claude Code Output

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

## 5.3 Cursor Output

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

## 5.4 opencode.ai Output

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

## 5.5 OpenAI Codex Output

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

## 5.6 Backup Files

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

## 5.7 State Tracking File

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
