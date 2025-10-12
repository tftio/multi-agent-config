# 10. Security Requirements

## 10.1 Credential Handling

**SEC-1**: Environment variables containing credentials MUST be expanded only in memory, never logged

**SEC-2**: Error messages MUST NOT include environment variable values

**SEC-3**: Dry-run and diff output MUST redact credential values

**Implementation**:
```
If field name matches (API_KEY|TOKEN|SECRET|PASSWORD|CREDENTIAL):
    Display as: <redacted>
```

## 10.2 File Permissions

**SEC-4**: Configuration files MUST be created with mode 0600 (user read/write only)

**SEC-5**: Backup files MUST preserve original file permissions

**SEC-6**: State files MUST be created with mode 0600

## 10.3 Path Traversal Prevention

**SEC-7**: Tool MUST reject config paths containing `..` (parent directory)

**SEC-8**: Output paths MUST be validated before write:
- Must be within expected directories
- Cannot write to system directories (/etc, /usr, etc.)

## 10.4 Code Injection Prevention

**SEC-9**: Environment variable expansion MUST NOT execute shell commands

**SEC-10**: Server `command` fields are NOT validated for safety (user responsibility)

**SEC-11**: Tool MUST NOT evaluate user input as code (no eval/exec)

## 10.5 Audit Logging

**SEC-12**: Tool SHOULD log:
- Configuration file read
- Files written
- Backups created
- Errors encountered

**SEC-13**: Logs MUST NOT contain credential values
