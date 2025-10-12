# 1. Executive Summary

## 1.1 Purpose

This document specifies the requirements for a unified configuration management tool that compiles a single TOML configuration into tool-specific formats for multiple AI coding assistants.

## 1.2 Target Tools

The system MUST support the following AI coding tools:

1. **Claude Code** - Anthropic's CLI coding assistant
2. **Cursor** - AI-powered code editor
3. **opencode.ai** - Terminal-based AI coding agent
4. **OpenAI Codex** - OpenAI's CLI coding assistant

## 1.3 Scope

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

## 1.4 High-Level Architecture

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
