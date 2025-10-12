# 6. Invariants

These properties MUST hold true at all times during and after tool execution:

## 6.1 Configuration Consistency

**INV-1**: For any MCP server with `targets = ["all"]`, the server configuration MUST be identical across all tool outputs (modulo format differences)

**Verification**: After compilation, deserialize all tool configs and verify server definitions match

## 6.2 Atomicity

**INV-2**: File write operations MUST be atomic - either the entire file is written or no changes occur

**Implementation**: Write to temporary file, then atomic rename

## 6.3 Backup Preservation

**INV-3**: If a backup file is created, it MUST contain an exact copy of the original file at the time of backup

**Verification**: Compare byte-for-byte before and after backup creation

## 6.4 Environment Variable Security

**INV-4**: Environment variable values MUST NEVER be stored in plaintext in any log, error message, or state file visible to other users

**Implementation**: Redact environment variable values in all output except generated config files

## 6.5 Generated File Markers

**INV-5**: All generated configuration files MUST be valid in their respective formats (parseable by target tool)

**Verification**: After generation, parse with format-specific parser (JSON.parse, TOML parser)

## 6.6 State Consistency

**INV-6**: The state tracker MUST accurately reflect all files generated in the most recent successful compile operation

**Verification**: State file updated only after all writes succeed

## 6.7 Idempotency

**INV-7**: Running compile twice with the same input and environment MUST produce identical outputs

**Verification**: Hash generated files before and after second compile

## 6.8 Rollback Safety

**INV-8**: If compilation fails partway through, all backup files MUST remain intact and the system MUST be in a recoverable state

**Implementation**: Only delete backups after successful completion of all writes

## 6.9 Path Handling

**INV-9**: All file paths MUST be correctly handled on both Unix (/) and Windows (\\) systems

**Implementation**: Use platform-agnostic path manipulation libraries

## 6.10 Character Encoding

**INV-10**: All file operations MUST use UTF-8 encoding consistently

**Verification**: Files parseable with UTF-8 decoder
