# 15. Future Extensibility

## 15.1 Plugin Architecture (Future)

**Design Considerations**:
- Allow custom format transformers
- Plugin registration mechanism
- Plugin API version compatibility

**Example**:
```python
class FormatTransformer:
    def transform(self, servers: dict, env: dict) -> str:
        # Return tool-specific configuration
        pass
```

## 15.2 Schema Versioning

**Migration Strategy**:
- Version field in config enables breaking changes
- Tool detects old versions and migrates automatically
- Migration logged and user-confirmed

**Example**:
```
Detected configuration version 1.0
Migrating to version 2.0...
- Renamed field: 'disabled' → 'enabled' (inverted)
- Added required field: 'schema' (default: 'stdio')
Migration complete. Backup saved to: config.toml.v1.backup
```

## 15.3 Adding New Tools

**Process**:
1. Research tool's configuration format
2. Add tool name to valid targets
3. Implement format transformer
4. Add output path mapping
5. Update documentation
6. Add test cases

**Interface**:
```python
def transform_for_new_tool(servers: dict, env: dict) -> Output:
    # Implementation
    pass
```

## 15.4 Prompt Management (Future)

**Planned Features**:
- Unified prompt definitions in TOML
- Generate tool-specific prompt files (CLAUDE.md, .cursorrules, AGENTS.md)
- Merge strategies for existing prompts
- Template variables in prompts

**Example Config**:
```toml
[prompts.coding-standards]
files = ["~/.config/agent-sync/prompts/coding-standards.md"]
targets = ["all"]
merge_strategy = "append"
variables = { LANGUAGE = "Python" }
```

## 15.5 Watch Mode (Future)

**Behavior**:
- Monitor config file for changes
- Auto-compile on save
- Debounce rapid changes (wait 500ms)
- Notification on success/failure

**Command**:
```bash
agent-sync watch
Watching: ~/.config/agent-sync/config.toml
Press Ctrl+C to stop...
[15:30] Config changed, recompiling...
[15:30] ✓ All tools updated
```
