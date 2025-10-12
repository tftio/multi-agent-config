# 13. Error Handling Specifications

## 13.1 Error Message Format

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

## 13.2 Exit Codes

| Code | Meaning | Examples |
|------|---------|----------|
| 0 | Success | All operations completed successfully |
| 1 | Validation error | Invalid configuration, missing required fields |
| 2 | File system error | Cannot read/write files, permissions |
| 3 | Partial failure | Some tools succeeded, some failed |
| 4 | Lock error | Another instance running |
| 5-99 | Reserved | Future use |

## 13.3 Logging

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

## 13.4 User-Facing vs Debug Messages

**User Messages** (stdout):
- Brief, actionable
- Suitable for end users
- No implementation details

**Debug Messages** (stderr or log file):
- Detailed technical information
- Stack traces on exceptions
- Intermediate values
- Enabled with `--verbose` or `--debug` flag
