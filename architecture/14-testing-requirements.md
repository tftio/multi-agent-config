# 14. Testing Requirements

## 14.1 Unit Test Coverage

Minimum 80% code coverage for:
- Configuration parsing
- Environment variable expansion
- Schema validation
- Format transformation
- File operations

## 14.2 Integration Test Scenarios

**INT-1**: End-to-end compile with all tools

**INT-2**: Dry-run produces correct output without writing

**INT-3**: Diff shows accurate changes

**INT-4**: Backup and rollback procedure

**INT-5**: Concurrent execution handling

## 14.3 Validation Test Cases

Test matrix of valid/invalid configurations:

| Test Case | Config | Expected Result |
|-----------|--------|-----------------|
| Valid-1 | Complete config with all fields | Pass |
| Valid-2 | Minimal config (required fields only) | Pass |
| Valid-3 | Mixed STDIO and HTTP servers | Pass |
| Invalid-1 | Missing [mcp.servers] | Fail: Missing required section |
| Invalid-2 | Server with no command or url | Fail: Missing required field |
| Invalid-3 | Both command and url | Fail: Mutually exclusive |
| Invalid-4 | Invalid TOML syntax | Fail: Parse error |
| Invalid-5 | Circular env references | Fail: Circular reference |

## 14.4 Edge Case Test Matrix

Each edge case from Section 9 MUST have corresponding test:

- EC-1: Empty configuration → generates minimal output
- EC-6: Empty env var value → expands to empty string
- EC-10: Symlink output path → follows symlink correctly
- EC-23: Non-ASCII characters → preserved correctly
- (etc.)

## 14.5 Failure Mode Test Matrix

Each failure mode from Section 8 MUST have corresponding test:

- FM-1: Config not found → correct error message and exit code
- FM-8: Undefined env var → warning but continues
- FM-11: Output directory missing → creates directory
- FM-14: Backup fails → aborts without writing
- (etc.)
