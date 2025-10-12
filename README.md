# multi-agent-config

[![CI](https://github.com/jfb/multi-agent-config/workflows/CI/badge.svg)](https://github.com/jfb/multi-agent-config/actions/workflows/ci.yml)
[![Release](https://github.com/jfb/multi-agent-config/workflows/Release/badge.svg)](https://github.com/jfb/multi-agent-config/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**Unified Configuration Manager for AI Coding Tools**

Manage MCP (Model Context Protocol) server configurations for multiple AI coding assistants from a single TOML file. Eliminate configuration duplication and keep your AI tools synchronized.

## Features

- **Single Source of Truth**: Define all MCP servers in one TOML configuration
- **Multi-Tool Support**: Generate configs for Cursor, opencode.ai, Codex, and Claude Code
- **Environment Variables**: Expand `${SHELL_VAR}` and `{CONFIG_VAR}` references
- **Atomic Operations**: Safe file writes with automatic backups
- **Diff Preview**: See changes before applying
- **State Tracking**: SHA-256 hashing tracks generated files
- **Circular Reference Detection**: Prevents infinite variable expansion loops
- **Cross-Platform**: Works on macOS, Linux, and Windows

## Supported Tools

| Tool | Format | Server Types | Config Path |
|------|--------|--------------|-------------|
| **Cursor** | JSON | STDIO only | `~/.config/Cursor/User/globalStorage/.../mcp.json` |
| **opencode.ai** | JSON | STDIO + HTTP | `~/.config/opencode/mcp.json` |
| **Codex** | TOML | STDIO + HTTP | `~/.config/codex/mcp_config.toml` |
| **Claude Code** | JSON | STDIO + HTTP | `~/.config/claude/mcp.json` |

## Installation

### From crates.io

```bash
cargo install multi-agent-config
```

### From Source

```bash
git clone https://github.com/jfb/multi-agent-config.git
cd multi-agent-config
cargo install --path .
```

### From GitHub Releases

Download pre-built binaries from [Releases](https://github.com/jfb/multi-agent-config/releases).

## Quick Start

```bash
# Initialize configuration with template
multi-agent-config init

# Edit the configuration
# File location: ~/.config/multi-agent-config/config.toml

# Validate your configuration
multi-agent-config validate

# Preview changes
multi-agent-config diff

# Apply configuration to all tools
multi-agent-config compile

# Compile for specific tools only
multi-agent-config compile --tool cursor --tool codex
```

## Configuration Format

### Unified TOML Configuration

Location: `~/.config/multi-agent-config/config.toml`

```toml
[settings]
version = "1.0"
default_targets = ["cursor", "opencode", "codex"]

# Environment variables
[env]
GITHUB_TOKEN = "${GITHUB_PERSONAL_ACCESS_TOKEN}"
API_BASE = "https://api.example.com"

# STDIO MCP Server
[mcp.servers.github-mcp]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-github"]
enabled = true
targets = ["all"]

[mcp.servers.github-mcp.env]
GITHUB_PERSONAL_ACCESS_TOKEN = "{GITHUB_TOKEN}"

# HTTP MCP Server (opencode, codex, claude-code only)
[mcp.servers.remote-api]
url = "{API_BASE}/mcp"
bearer_token = "{GITHUB_TOKEN}"
targets = ["opencode", "codex", "claude-code"]

# Tool-specific server
[mcp.servers.cursor-only]
command = "node"
args = ["server.js"]
targets = ["cursor"]
disabled = false           # Cursor-specific
autoApprove = ["read"]     # Cursor-specific
```

### Variable Expansion

- `${VAR}` - Expands from shell environment
- `{VAR}` - Expands from `[env]` section
- Nested expansion supported (up to 10 levels)
- Circular references detected and rejected

### Target Filtering

- `targets = ["all"]` - Include for all tools
- `targets = ["cursor", "codex"]` - Include for specific tools only
- `enabled = false` - Disable server globally

## Commands

### `init`

Create template configuration:

```bash
multi-agent-config init

# Overwrite existing config
multi-agent-config init --force
```

### `validate`

Check configuration validity:

```bash
multi-agent-config validate

# Verbose output
multi-agent-config validate --verbose
```

### `compile`

Generate tool-specific configurations:

```bash
# Compile for all tools
multi-agent-config compile

# Compile for specific tools
multi-agent-config compile --tool cursor

# Dry run (show what would be done)
multi-agent-config compile --dry-run

# Verbose output
multi-agent-config compile --verbose
```

### `diff`

Preview changes without writing:

```bash
# Show diff for all tools
multi-agent-config diff

# Show diff for specific tool
multi-agent-config diff --tool cursor
```

### Standard Commands

```bash
# Show version
multi-agent-config version

# Show license information
multi-agent-config license

# Health check
multi-agent-config doctor

# Generate shell completions
multi-agent-config completions bash > /usr/local/etc/bash_completion.d/multi-agent-config
```

## Development

### Prerequisites

- Rust 1.85.0 or later
- [just](https://github.com/casey/just) - Task runner
- [peter-hook](https://crates.io/crates/peter-hook) - Git hooks manager
- [versioneer](https://crates.io/crates/versioneer) - Version management

### Development Workflow

```bash
# Quick development check
just dev

# Run tests
just test

# Full CI pipeline
just ci
```

### Testing

```bash
# Run all tests
cargo test --all --verbose

# Run with coverage (Linux only)
cargo tarpaulin --all --out Html --engine llvm --timeout 300
```

### Quality Tools

- **clippy**: Aggressive linting with pedantic and nursery checks
- **rustfmt**: Nightly formatter for consistent code style
- **cargo-audit**: Security vulnerability scanning
- **cargo-deny**: License and dependency compliance

### Git Hooks

Hooks automatically run on commit/push:

- **Pre-commit**: Format check, clippy
- **Pre-push**: Tests, security audit, version validation

```bash
# Install hooks
just install-hooks
```

## Architecture

### Module Structure

- `config`: TOML parsing and validation
- `expand`: Variable expansion with circular detection
- `transform`: Tool-specific format transformers
- `file_ops`: Atomic writes, backups, state tracking, diffs
- `cli`: Command implementations and output formatting

### Data Flow

```
Unified TOML → Parse → Validate → Expand Variables →
Transform per Tool → Backup → Atomic Write → Track State
```

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## Support

- Issues: [GitHub Issues](https://github.com/jfb/multi-agent-config/issues)
- Email: jfb@workhelix.com
