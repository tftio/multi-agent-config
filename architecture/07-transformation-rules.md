# 7. Transformation Rules

## 7.1 Environment Variable Expansion

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

## 7.2 Target Filtering

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

## 7.3 Unified TOML → Cursor JSON

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

## 7.4 Unified TOML → opencode.ai JSON

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

## 7.5 Unified TOML → Codex TOML

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

## 7.6 Merge Strategy for Existing Configs

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
