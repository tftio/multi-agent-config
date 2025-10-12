# 16. Appendices

## Appendix A: Complete Example Configuration

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

## Appendix B: Example Outputs

### B.1 Cursor Output

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

### B.2 opencode.ai Output

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

### B.3 Codex Output

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

## Appendix C: Environment Variable Resolution Algorithm

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

## Appendix D: File Operation Safety Protocol

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

## Appendix E: Glossary

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
