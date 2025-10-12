# 12. Configuration Schema Reference

## 12.1 Input Schema (TOML)

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

## 12.2 Cursor Output Schema (JSON)

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

## 12.3 opencode.ai Output Schema (JSON)

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

## 12.4 Codex Output Schema (TOML)

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

## 12.5 State File Schema (JSON)

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
