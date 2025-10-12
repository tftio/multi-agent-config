# 3. System Overview

## 3.1 Components

1. **Configuration Parser**: Reads and validates unified TOML
2. **Environment Expander**: Resolves environment variable references
3. **Schema Validator**: Validates against required fields and types
4. **Target Filter**: Selects servers for specific tools
5. **Format Transformer**: Converts unified format to tool-specific schemas
6. **File Writer**: Safely writes configurations with backups
7. **State Tracker**: Records generated files for updates
8. **Diff Generator**: Compares current vs new configurations
9. **CLI Interface**: Provides user commands

## 3.2 Data Flow

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

## 3.3 Execution Modes

1. **init**: Create template configuration
2. **validate**: Check configuration correctness without writing
3. **compile**: Generate and write all tool configurations
4. **diff**: Show what would change without writing
5. **watch** (future): Monitor config file and auto-compile on changes
