# About Config
# 关于配置

The server reads `config.toml` from the same directory as the executable.
If the file does not exist, it will be auto-created with default content.

## Format

```toml
[routes]
disabled = []
```

| Field | Type | Description |
|-------|------|-------------|
| `routes.disabled` | `string[]` | List of route names to disable |