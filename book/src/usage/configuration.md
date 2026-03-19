# Configuration

kb-mcp is configured via a `collections.ron` file that defines what markdown
directories to index.

## Config File

```ron
(
    // Optional: override cache directory (default: ~/.cache/kb-mcp)
    // cache_dir: "~/.cache/kb-mcp",
    collections: [
        (
            name: "docs",
            path: "docs",
            description: "Project documentation",
            writable: false,
            sections: [
                (prefix: "guides", description: "How-to guides"),
                (prefix: "reference", description: "API reference"),
            ],
        ),
        (
            name: "notes",
            path: "notes",
            description: "Working notes",
            writable: true,
            sections: [],
        ),
    ],
)
```

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cache_dir` | String | No | Cache directory for index files (default: `~/.cache/kb-mcp`) |
| `collections` | List | Yes | One or more collection definitions |

### Collection Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes | Unique identifier for the collection |
| `path` | String | Yes | Directory path (relative to config file) |
| `description` | String | Yes | Human-readable description |
| `writable` | Bool | No | Allow `kb_write` to create files (default: `false`) |
| `sections` | List | No | Section definitions for this collection |

### Section Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `prefix` | String | Yes | Directory prefix that identifies this section |
| `description` | String | Yes | Human-readable description |

## Resolution Order

kb-mcp searches for configuration in this order:

1. `--config <path>` CLI flag (explicit)
2. `KB_MCP_CONFIG` environment variable
3. `./collections.ron` (current working directory)
4. `~/.config/kb-mcp/collections.ron` (user default)

Collection paths resolve relative to the config file's parent directory.

## Cross-Project Use

Install kb-mcp globally, then point other projects at a specific config:

```json
{
  "mcpServers": {
    "kb": {
      "command": "kb-mcp",
      "env": {
        "KB_MCP_CONFIG": "/path/to/project/collections.ron"
      },
      "args": []
    }
  }
}
```
