---
description: Read to point kb-mcp at your markdown: collections.ron, where kb-mcp finds it, cache_dir and index file names, and KB_MCP_CONFIG and KB_MCP_CACHE_DIR
---

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
| `cache_dir` | String | No | Cache directory for index files, relative to the config file (default: the platform cache directory, such as `~/.cache/kb-mcp` on Linux). `KB_MCP_CACHE_DIR` overrides it |
| `index_names` | `Path` or `Name` | No | How index files in `cache_dir` are named (default: `Path`). See [Index File Names](#index-file-names) |
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

Collection paths and `cache_dir` resolve relative to the config file's parent directory.

The `KB_MCP_CACHE_DIR` environment variable overrides `cache_dir`, for a launcher that shares one config but keeps its own index, such as a plugin.

## Index File Names

Each collection's index is a `.mv2` file plus a `.hashes` sidecar in `cache_dir`.

- `index_names: Path` (default): `<hash>-<name>.mv2`, where the hash is of the collection's absolute path. Safe in a cache folder shared by many configs, but when the collections move to a new absolute path, kb-mcp builds a new index and the old one is left behind.
- `index_names: Name`: `<name>.mv2`. The same collection keeps the same index wherever the config moves, and the next start syncs only the documents that changed. Use it when the config moves on every update, such as a plugin installed to a new folder per version, with `cache_dir` (or `KB_MCP_CACHE_DIR`) set to a folder that stays.

Only use `Name` with a `cache_dir` dedicated to this one config. Another config with a collection of the same name would share the file, and the two would keep rebuilding it. Two sessions running different versions of the same config also take turns rebuilding it while both are open, which is cheap for small collections.

Switching `index_names` builds a new index under the new name; the old files are not removed.

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
