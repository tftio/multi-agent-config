# 11. Operational Requirements

## 11.1 Dry-Run Mode

**OP-1**: Dry-run MUST perform all validation steps

**OP-2**: Dry-run MUST NOT modify any files (including backups)

**OP-3**: Dry-run output MUST show what would be generated

**OP-4**: Dry-run exit code MUST match what actual run would return

## 11.2 Validation

**OP-5**: Validation MUST check:
- TOML syntax
- Required fields
- Field types
- Circular references
- Valid tool names
- Executable existence (warning only)

**OP-6**: Validation SHOULD check:
- Recommended field values
- Common mistakes
- Performance implications (too many servers)

## 11.3 Diff Generation

**OP-7**: Diff MUST use unified diff format

**OP-8**: Diff MUST show:
- File path
- Tool name
- Whether file is new or modified

**OP-9**: Diff MUST handle:
- Non-existent files (show as new)
- Binary files (show size change)
- Large files (truncate if > 10000 lines)

## 11.4 State Tracking

**OP-10**: State file MUST include:
- Version
- Timestamp of last compile
- List of generated files with paths and hashes

**OP-11**: State file MUST be updated atomically (write to temp, rename)

**OP-12**: State file errors MUST NOT prevent compilation (warn and continue)

## 11.5 Rollback Capability

**OP-13**: Backup files enable manual rollback:
```bash
cp .cursor/mcp.json.backup .cursor/mcp.json
```

**OP-14**: Tool SHOULD provide `rollback` command in future versions

## 11.6 Lock Files

**OP-15**: Tool MAY use lock files to prevent concurrent execution

**OP-16**: Lock files MUST include:
- Process ID
- Start timestamp
- Host name

**OP-17**: Stale locks (process not running) MUST be cleaned up automatically

## 11.7 Idempotency

**OP-18**: Running compile twice with same input MUST produce identical output

**OP-19**: Repeated execution MUST NOT corrupt state

**OP-20**: Backup files MAY be overwritten on repeated execution
