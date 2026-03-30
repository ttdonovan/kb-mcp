# CLI

The `kb` binary provides CLI access to all MCP tools. Every MCP tool has a
CLI equivalent.

## Commands

```sh
# List all sections
kb list-sections

# Search
kb search --query "rate limits"
kb search --query "bevy" --collection skills
kb search --query "agents" --scope runtimes
kb search --query "MCP" --max-results 5

# Get full document
kb get-document --path "concepts/mcp-server-pattern.md"

# Token-efficient briefing
kb context --path "concepts/mcp-server-pattern.md"

# Write a note (writable collection only)
kb write --collection notes --title "My Note" --body "Content" --tags "tag1,tag2"

# Rebuild index
kb reindex
```

## Global Options

| Flag | Description |
|------|-------------|
| `--config <path>` | Path to `collections.ron` config file |

## Output

All commands output JSON to stdout. Errors go to stderr.
